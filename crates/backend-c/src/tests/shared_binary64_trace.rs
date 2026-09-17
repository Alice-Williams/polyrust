//! Instrumentation observes certified call order; all mutants preserve return bits.
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
#[test]
fn double_operand_calls_are_once_and_ordered_with_value_preserving_controls() {
    let root = PathBuf::from(std::env::var_os("TEST_TMPDIR").unwrap()).join("c-double-trace");
    let producer = f::api(94, &[CScalarType::F64]);
    let first = producer.functions().next().unwrap();
    let zig = PathBuf::from(std::env::var_os("TEST_SRCDIR").unwrap())
        .join(std::env::var_os("TEST_WORKSPACE").unwrap())
        .join("tools/c/zig_native_oracle");
    for (index, mode) in [f::Probe::Ordered, f::Probe::DropLeft, f::Probe::Reversed]
        .into_iter()
        .enumerate()
    {
        let directory = root.join(index.to_string());
        fs::create_dir_all(&directory).unwrap();
        let consumer = api(&f::double_probe(first.clone(), mode));
        for owner in [&producer, &consumer] {
            let output = render_certified_package(&CStructuralRenderer, owner.package()).unwrap();
            for file in output.files() {
                let OutputContents::Text(text) = file.contents() else {
                    panic!("source")
                };
                let mut text = text.clone();
                if file.path() == "polyrust_dep_94.c" {
                    let start = text.find(&format!("{}(", first.symbol().as_str())).unwrap();
                    let brace = start + text[start..].find('{').unwrap() + 1;
                    assert!(text[start..brace].contains("double poly_input"));
                    text.insert_str(brace, "\nuint64_t observed; memcpy(&observed, &poly_input, sizeof observed); fprintf(stderr, \"%\" PRIx64 \",\", observed);\n");
                    text = format!(
                        "#include <inttypes.h>\n#include <stdio.h>\n#include <string.h>\n{text}"
                    );
                }
                fs::write(directory.join(file.path()), text).unwrap();
            }
        }
        let name = consumer.functions().next().unwrap().symbol().as_str();
        fs::write(directory.join("main.c"), format!(
            "#include \"polyrust_dep_95.h\"\nint main(void) {{ return {name}(1.5) == 1.5 ? 0 : 1; }}\n"
        )).unwrap();
        for compiler in [PathBuf::from("gcc-14"), zig.clone()] {
            for optimization in ["-O0", "-O2"] {
                let mut objects = vec![];
                for file in ["polyrust_dep_94", "polyrust_dep_95", "main"] {
                    let object = directory.join(format!("{file}.o"));
                    run(Command::new(&compiler)
                        .args([
                            "-std=c17",
                            "-Wall",
                            "-Wextra",
                            "-Wpedantic",
                            "-Werror",
                            "-fno-fast-math",
                            "-ffp-contract=off",
                            optimization,
                            "-c",
                        ])
                        .arg(directory.join(format!("{file}.c")))
                        .arg("-o")
                        .arg(&object));
                    objects.push(object);
                }
                let binary = directory.join("trace");
                run(Command::new(&compiler).args(objects).arg("-o").arg(&binary));
                let output = Command::new(binary).output().unwrap();
                assert!(
                    output.status.success(),
                    "every mutant must preserve the value"
                );
                let trace = String::from_utf8(output.stderr).unwrap();
                assert_eq!(
                    trace,
                    ["3ff8000000000000,0,", "0,", "0,3ff8000000000000,"][index]
                );
                assert_eq!(trace == "3ff8000000000000,0,", index == 0);
            }
        }
    }
}
