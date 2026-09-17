//! Independent bit oracle; separate value, evaluation-count and linker controls.
use super::*;
use std::{fs, path::PathBuf, process::Command};

pub(super) fn expected(bits: u64) -> u64 {
    let exponent = ((bits >> 52) & 0x7ff) as i32 - 1023;
    if exponent < 0 {
        bits & (1 << 63)
    } else if exponent < 52 {
        bits & !((1_u64 << (52 - exponent)) - 1)
    } else {
        bits
    }
}
pub(super) fn cases() -> Vec<u64> {
    let mut bits: Vec<_> = super::super::comparisons::cases()
        .into_iter()
        .map(|(bits, _)| bits)
        .collect();
    for exponent in [0_u64, 1, 1022, 1023, 1024, 1074, 1075, 1076, 2046] {
        for fraction in [0, 1, (1 << 51) - 1, 1 << 51, (1 << 52) - 1] {
            for sign in [0, 1 << 63] {
                bits.push(sign | (exponent << 52) | fraction);
            }
        }
    }
    bits
}

#[derive(Clone, Copy, Debug)]
enum Fault {
    None,
    Floor,
    Ceil,
    ZeroSign,
    Dropped,
    Duplicated,
}
impl Fault {
    const ALL: [Self; 6] = [
        Self::None,
        Self::Floor,
        Self::Ceil,
        Self::ZeroSign,
        Self::Dropped,
        Self::Duplicated,
    ];
    fn wrong_value(self) -> bool {
        matches!(self, Self::Floor | Self::Ceil | Self::ZeroSign)
    }
    fn trace(self) -> &'static str {
        match self {
            Self::Dropped => "A",
            Self::Duplicated => "AAAAA",
            _ => "AAA",
        }
    }
}
fn run(command: &mut Command) {
    let output = command.output().unwrap();
    assert!(
        output.status.success(),
        "{command:?}: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}
fn body(text: &str, member: &str) -> (usize, usize, String) {
    let start = text.find(&format!("{member}(")).unwrap() + member.len() + 1;
    let end = start + text[start..].find(')').unwrap();
    let parameter = text[start..end]
        .split_whitespace()
        .last()
        .unwrap()
        .to_owned();
    let brace = end + text[end..].find('{').unwrap() + 1;
    (brace, brace + text[brace..].find('}').unwrap(), parameter)
}
fn link_options(api: &CDependencyApi) -> Vec<&'static str> {
    api.system_libraries()
        .iter()
        .map(|library| match library {
            CSystemLibrary::Math => "-lm",
        })
        .collect()
}

#[test]
fn exact_truncation_bits_calls_and_transitive_native_linking() {
    let root =
        PathBuf::from(std::env::var_os("TEST_TMPDIR").unwrap()).join("c-binary64-truncation");
    let owners = chain();
    let members: Vec<_> = owners
        .iter()
        .map(|api| api.functions().next().unwrap().symbol().as_str())
        .collect();
    let cases = cases();
    let zig = PathBuf::from(std::env::var_os("TEST_SRCDIR").unwrap())
        .join(std::env::var_os("TEST_WORKSPACE").unwrap())
        .join("tools/c/zig_native_oracle");
    for fault in Fault::ALL {
        let directory = root.join(format!("{fault:?}"));
        fs::create_dir_all(&directory).unwrap();
        for (index, api) in owners.iter().enumerate() {
            let rendered = render_certified_package(&CStructuralRenderer, api.package()).unwrap();
            for file in rendered.files() {
                let OutputContents::Text(original) = file.contents() else {
                    panic!("text")
                };
                let mut text = original.clone();
                if file.path().ends_with(".c") && index == 0 {
                    let (start, end, input) = body(&text, members[0]);
                    let replacement = match fault {
                        Fault::Floor => Some(format!("\nreturn floor({input});\n")),
                        Fault::Ceil => Some(format!("\nreturn ceil({input});\n")),
                        Fault::ZeroSign => Some(format!("\nreturn trunc({input}) + 0.0;\n")),
                        _ => None,
                    };
                    if let Some(replacement) = replacement {
                        text.replace_range(start..end, &replacement);
                    }
                    let (start, _, _) = body(&text, members[0]);
                    text.insert_str(start, "\n(void)fputc('A', stderr);\n");
                    text = format!("#include <stdio.h>\n{text}");
                }
                if file.path().ends_with(".c") && index == 1 {
                    let (start, end, input) = body(&text, members[1]);
                    match fault {
                        // Use the real primitive so the value remains right; only the call trace changes.
                        Fault::Dropped => {
                            text.replace_range(start..end, &format!("\nreturn trunc({input});\n"));
                            text = format!("#include <math.h>\n{text}");
                        }
                        Fault::Duplicated => {
                            text.insert_str(start, &format!("\n(void){}({input});\n", members[0]))
                        }
                        _ => {}
                    }
                }
                fs::write(directory.join(file.path()), text).unwrap();
            }
        }
        let mut source = String::from(
            r#"#include "polyrust_dep_101.h"
#include "polyrust_dep_102.h"
#include "polyrust_dep_103.h"
#include <stdint.h>
#include <string.h>
static int equal_bits(double value, uint64_t expected) {
    uint64_t actual; memcpy(&actual, &value, sizeof actual);
    if ((expected & UINT64_C(0x7fffffffffffffff)) > UINT64_C(0x7ff0000000000000))
        return (actual & UINT64_C(0x7fffffffffffffff)) > UINT64_C(0x7ff0000000000000);
    return actual == expected;
}
int main(void) {
"#,
        );
        for bits in &cases {
            source.push_str(&format!("{{ uint64_t raw=UINT64_C({bits}); double value; memcpy(&value,&raw,sizeof value);\n"));
            for member in &members {
                source.push_str(&format!(
                    "if (!equal_bits({member}(value),UINT64_C({}))) return 1;\n",
                    expected(*bits)
                ));
            }
            source.push_str("}\n");
        }
        source.push_str("return 0; }\n");
        fs::write(directory.join("consumer.c"), source).unwrap();
        for compiler in [PathBuf::from("gcc-14"), zig.clone()] {
            for optimization in ["-O0", "-O2"] {
                let mut objects = vec![];
                for stem in [
                    "polyrust_dep_101",
                    "polyrust_dep_102",
                    "polyrust_dep_103",
                    "consumer",
                ] {
                    let object = directory.join(format!("{stem}.o"));
                    run(Command::new(&compiler)
                        .args([
                            "-std=c17",
                            "-Wall",
                            "-Wextra",
                            "-Wpedantic",
                            "-Werror",
                            "-Wstrict-prototypes",
                            "-Wmissing-prototypes",
                            "-fno-fast-math",
                            "-ffp-contract=off",
                            "-fno-builtin",
                            optimization,
                            "-c",
                        ])
                        .arg(directory.join(format!("{stem}.c")))
                        .arg("-o")
                        .arg(&object));
                    objects.push(object);
                }
                let executable = directory.join("consumer");
                if compiler == std::path::Path::new("gcc-14") && matches!(fault, Fault::None) {
                    let omitted = Command::new(&compiler)
                        .args(&objects)
                        .arg("-o")
                        .arg(&executable)
                        .output()
                        .unwrap();
                    assert!(
                        !omitted.status.success(),
                        "omitting derived math linkage must fail"
                    );
                    assert!(String::from_utf8_lossy(&omitted.stderr).contains("trunc"));
                }
                run(Command::new(&compiler)
                    .args(objects)
                    .args(link_options(&owners[2]))
                    .arg("-o")
                    .arg(&executable));
                let output = Command::new(&executable).output().unwrap();
                assert_eq!(
                    output.status.code(),
                    Some(i32::from(fault.wrong_value())),
                    "{fault:?}"
                );
                if !fault.wrong_value() {
                    let expected = fault.trace().repeat(cases.len());
                    assert_eq!(
                        String::from_utf8(output.stderr).unwrap(),
                        expected,
                        "{fault:?}"
                    );
                    assert_eq!(
                        expected == "AAA".repeat(cases.len()),
                        matches!(fault, Fault::None)
                    );
                }
            }
        }
    }
}
