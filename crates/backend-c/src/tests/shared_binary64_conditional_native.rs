//! Native conditional selection, signed-zero correction and call-count faults.
use super::*;
use std::{fs, path::PathBuf, process::Command};

#[derive(Clone, Copy, Debug)]
enum Fault {
    None,
    Swapped,
    MissingZero,
    WrongCondition,
    Dropped,
    Duplicated,
}
impl Fault {
    const ALL: [Self; 6] = [
        Self::None,
        Self::Swapped,
        Self::MissingZero,
        Self::WrongCondition,
        Self::Dropped,
        Self::Duplicated,
    ];
    fn wrong_value(self) -> bool {
        matches!(
            self,
            Self::Swapped | Self::MissingZero | Self::WrongCondition
        )
    }
    fn trace(self) -> &'static str {
        match self {
            Self::Dropped => "ABCDBCD",
            Self::Duplicated => "ABCDAABCD",
            _ => "ABCDABCD",
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
#[cfg(test)]
fn body(text: &str, member: &str) -> (usize, usize) {
    let start = text.find(&format!("{member}(")).unwrap();
    let brace = start + text[start..].find('{').unwrap() + 1;
    let end = brace + text[brace..].find('}').unwrap();
    (brace, end)
}
fn parameter(text: &str, member: &str) -> String {
    let start = text.find(&format!("{member}(")).unwrap() + member.len() + 1;
    let end = start + text[start..].find(')').unwrap();
    text[start..end]
        .split_whitespace()
        .last()
        .unwrap()
        .to_owned()
}
#[test]
fn exact_double_conditional_selection_and_native_faults() {
    let root =
        PathBuf::from(std::env::var_os("TEST_TMPDIR").unwrap()).join("c-binary64-conditional");
    let first = owner(98, &[]);
    let imports: Vec<_> = first.functions().cloned().collect();
    let second = owner(99, &imports);
    let producer: Vec<_> = first.functions().map(|f| f.symbol().as_str()).collect();
    let consumer: Vec<_> = second.functions().map(|f| f.symbol().as_str()).collect();
    let cases: Vec<_> = super::super::comparisons::cases()
        .into_iter()
        .map(|(bits, _)| bits)
        .collect();
    let zig = PathBuf::from(std::env::var_os("TEST_SRCDIR").unwrap())
        .join(std::env::var_os("TEST_WORKSPACE").unwrap())
        .join("tools/c/zig_native_oracle");
    for fault in Fault::ALL {
        let directory = root.join(format!("{fault:?}"));
        fs::create_dir_all(&directory).unwrap();
        for api in [&first, &second] {
            let output = render_certified_package(&CStructuralRenderer, api.package()).unwrap();
            for file in output.files() {
                let OutputContents::Text(original) = file.contents() else {
                    panic!("text")
                };
                let mut text = original.clone();
                if file.path() == "polyrust_dep_98.c" {
                    let replacement = match fault {
                        Fault::Swapped => Some((0, "\nreturn -poly_input;\n")),
                        Fault::MissingZero => {
                            Some((3, "\nreturn poly_input < 0.0 ? -poly_input : poly_input;\n"))
                        }
                        Fault::WrongCondition => Some((
                            2,
                            "\nreturn poly_input >= 0.0 ? poly_input : -poly_input;\n",
                        )),
                        _ => None,
                    };
                    if let Some((index, replacement)) = replacement {
                        let (start, end) = body(&text, producer[index]);
                        let input = parameter(&text, producer[index]);
                        text.replace_range(start..end, &replacement.replace("poly_input", &input));
                    }
                    for (member, marker) in producer.iter().zip(['A', 'B', 'C', 'D']) {
                        let (start, _) = body(&text, member);
                        text.insert_str(start, &format!("\n(void)fputc('{marker}', stderr);\n"));
                    }
                    text = format!("#include <stdio.h>\n{text}");
                }
                if file.path() == "polyrust_dep_99.c" {
                    let (start, end) = body(&text, consumer[0]);
                    let input = parameter(&text, consumer[0]);
                    match fault {
                        Fault::Dropped => {
                            text.replace_range(start..end, &format!("\nreturn {input};\n"))
                        }
                        Fault::Duplicated => {
                            text.insert_str(start, &format!("\n(void){}({input});\n", producer[0]))
                        }
                        _ => {}
                    }
                }
                fs::write(directory.join(file.path()), text).unwrap();
            }
        }
        let mut source = String::from(
            r#"#include "polyrust_dep_98.h"
#include "polyrust_dep_99.h"
#include <stdint.h>
#include <string.h>
#include <fenv.h>
static int equal_bits(double value, uint64_t expected) {
    uint64_t actual; memcpy(&actual, &value, sizeof actual);
    if ((expected & UINT64_C(0x7fffffffffffffff)) > UINT64_C(0x7ff0000000000000))
        return (actual & UINT64_C(0x7fffffffffffffff)) > UINT64_C(0x7ff0000000000000);
    return actual == expected;
}
int main(void) {
    if (fegetround() != FE_TONEAREST) return 2;
"#,
        );
        for bits in &cases {
            source.push_str(&format!("{{ uint64_t raw=UINT64_C({bits}); double value; memcpy(&value,&raw,sizeof value);\n"));
            for (members, twice) in [(&producer, false), (&consumer, true)] {
                for (member, mode) in members.iter().zip(Selection::ALL) {
                    let expected = mode.expected(*bits);
                    let expected = if twice {
                        mode.expected(expected)
                    } else {
                        expected
                    };
                    source.push_str(&format!(
                        "if (!equal_bits({member}(value),UINT64_C({expected}))) return 1;\n"
                    ));
                }
            }
            source.push_str("}\n");
        }
        source.push_str("return 0; }\n");
        fs::write(directory.join("consumer.c"), source).unwrap();
        for compiler in [PathBuf::from("gcc-14"), zig.clone()] {
            for optimization in ["-O0", "-O2"] {
                let mut objects = vec![];
                for stem in ["polyrust_dep_98", "polyrust_dep_99", "consumer"] {
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
                            optimization,
                            "-c",
                        ])
                        .arg(directory.join(format!("{stem}.c")))
                        .arg("-o")
                        .arg(&object));
                    objects.push(object);
                }
                let executable = directory.join("consumer");
                run(Command::new(&compiler)
                    .args(objects)
                    .args(["-lm", "-o"])
                    .arg(&executable));
                let output = Command::new(executable).output().unwrap();
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
                        expected == "ABCDABCD".repeat(cases.len()),
                        matches!(fault, Fault::None)
                    );
                }
            }
        }
    }
}
