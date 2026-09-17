//! Native sign/category oracle and value-preserving call-count controls.
use super::*;
use std::{fs, path::PathBuf, process::Command};

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
#[test]
fn exact_double_negation_and_native_value_trace_mutants() {
    let root = PathBuf::from(std::env::var_os("TEST_TMPDIR").unwrap()).join("c-binary64-negation");
    let first = owner(96, &[]);
    let imports: Vec<_> = first.functions().cloned().collect();
    let second = owner(97, &imports);
    let producer: Vec<_> = first.functions().map(|f| f.symbol().as_str()).collect();
    let consumer: Vec<_> = second.functions().map(|f| f.symbol().as_str()).collect();
    let cases: Vec<_> = super::super::comparisons::cases()
        .into_iter()
        .map(|(bits, _)| bits)
        .collect();
    let zig = PathBuf::from(std::env::var_os("TEST_SRCDIR").unwrap())
        .join(std::env::var_os("TEST_WORKSPACE").unwrap())
        .join("tools/c/zig_native_oracle");
    for fault in 0..5 {
        let directory = root.join(format!("fault-{fault}"));
        fs::create_dir_all(&directory).unwrap();
        for api in [&first, &second] {
            let output = render_certified_package(&CStructuralRenderer, api.package()).unwrap();
            for file in output.files() {
                let OutputContents::Text(original) = file.contents() else {
                    panic!("text")
                };
                let mut text = original.clone();
                if file.path() == "polyrust_dep_96.c" {
                    if fault == 1 || fault == 2 {
                        let (start, end) = body(&text, producer[0]);
                        text.replace_range(
                            start..end,
                            if fault == 1 {
                                "\nreturn poly_input;\n"
                            } else {
                                "\nreturn 0.0 - poly_input;\n"
                            },
                        );
                    }
                    for (member, marker) in producer.iter().zip(['A', 'B']) {
                        let (start, _) = body(&text, member);
                        text.insert_str(start, &format!("\n(void)fputc('{marker}', stderr);\n"));
                    }
                    text = format!("#include <stdio.h>\n{text}");
                }
                if file.path() == "polyrust_dep_97.c" {
                    let (start, end) = body(&text, consumer[0]);
                    if fault == 3 {
                        // Double negation is identity, but skipping its call is observable.
                        text.replace_range(start..end, "\nreturn poly_input;\n");
                    } else if fault == 4 {
                        text.insert_str(start, &format!("\n(void){}(poly_input);\n", producer[0]));
                    }
                }
                fs::write(directory.join(file.path()), text).unwrap();
            }
        }
        let mut source = String::from(
            r#"#include "polyrust_dep_96.h"
#include "polyrust_dep_97.h"
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
            source.push_str(&format!("{{ uint64_t raw = UINT64_C({bits}); double value; memcpy(&value, &raw, sizeof value);\n"));
            for (name, flip) in [
                (producer[0], true),
                (producer[1], false),
                (consumer[0], false),
                (consumer[1], true),
            ] {
                let expected = bits ^ if flip { 1 << 63 } else { 0 };
                source.push_str(&format!(
                    "if (!equal_bits({name}(value), UINT64_C({expected}))) return 1;\n"
                ));
            }
            source.push_str("}\n");
        }
        source.push_str("return 0; }\n");
        fs::write(directory.join("consumer.c"), source).unwrap();
        for compiler in [PathBuf::from("gcc-14"), zig.clone()] {
            for optimization in ["-O0", "-O2"] {
                let mut objects = vec![];
                for stem in ["polyrust_dep_96", "polyrust_dep_97", "consumer"] {
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
                    Some(i32::from(fault == 1 || fault == 2))
                );
                if fault == 0 || fault >= 3 {
                    let expected = match fault {
                        3 => "ABB",
                        4 => "ABAAB",
                        _ => "ABAB",
                    }
                    .repeat(cases.len());
                    assert_eq!(String::from_utf8(output.stderr).unwrap(), expected);
                    assert_eq!(expected == "ABAB".repeat(cases.len()), fault == 0);
                }
            }
        }
    }
}
