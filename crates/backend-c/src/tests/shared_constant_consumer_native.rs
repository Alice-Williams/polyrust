//! Independent truth and separate native object files for imported scalar reads.
use super::{
    CDialect,
    constant_consumer_fixture::{Usage, fixture, producer},
    owned_constant_dependency_tests::write_package,
    owned_constant_fixture::Shape,
    owned_constant_native_tests::{compile, options, successful},
    owned_constant_tests::linked,
};
use portable_codegen::certify_resolved_package;
use std::{fs, path::PathBuf, process::Command};

#[test]
fn constant_imports_compile_link_execute_and_reject_readonly_mutations() {
    let zig = PathBuf::from(std::env::var_os("TEST_SRCDIR").unwrap())
        .join(std::env::var_os("TEST_WORKSPACE").unwrap())
        .join("tools/c/zig_native_oracle");
    // Independent of the producer AST and its certificate values.
    let truth = [
        "0",
        "1",
        "INT32_MIN",
        "INT32_MAX",
        "INT64_MIN",
        "INT64_MAX",
        "INT64_C(9007199254740993)",
        "INT32_C(62)",
    ];
    for shape in [Shape::ConstantsOnly, Shape::Mixed] {
        let owner = producer(shape);
        let values: Vec<_> = owner.constants().cloned().collect();
        let call = owner.functions().next().cloned();
        for usage in [Usage::Read, Usage::Unused] {
            let input = fixture(90, &values, call.clone(), usage);
            let package = linked(&input);
            let names: Vec<_> = input
                .functions
                .iter()
                .map(|function| {
                    package.files()[1].items()[0].spelling.functions[function]
                        .as_str()
                        .to_owned()
                })
                .collect();
            let certificate = certify_resolved_package(&CDialect, package).unwrap();
            for (label, compiler) in [("gcc", PathBuf::from("gcc-14")), ("zig", zig.clone())] {
                for optimization in ["-O0", "-O2"] {
                    let directory = PathBuf::from(std::env::var_os("TEST_TMPDIR").unwrap())
                        .join("constant-imports")
                        .join(format!("{shape:?}-{usage:?}-{label}{optimization}"));
                    fs::create_dir_all(&directory).unwrap();
                    let mut sources = write_package(owner.package(), &directory);
                    let consumer_sources = write_package(&certificate, &directory);
                    sources.extend(consumer_sources.iter().cloned());
                    let checks = truth
                        .iter()
                        .enumerate()
                        .map(|(index, expected)| {
                            let expected = if usage == Usage::Difference && index >= 2 {
                                "0"
                            } else {
                                expected
                            };
                            format!(
                                "if ({}() != {expected}) return {};",
                                names[index],
                                index + 1
                            )
                        })
                        .collect::<Vec<_>>()
                        .join("\n");
                    let call_check = if call.is_some() {
                        format!("if ({}() != 0) return 9;", names[8])
                    } else {
                        String::new()
                    };
                    let driver = directory.join("driver.c");
                    fs::write(&driver, format!("#include <stdint.h>\n#include \"polyrust_constant_reader_90.h\"\nint main(void) {{\n{checks}\n{call_check}\nreturn 0;\n}}\n")).unwrap();
                    sources.push(driver);
                    let objects: Vec<_> = sources
                        .iter()
                        .enumerate()
                        .map(|(index, source)| {
                            let object = directory.join(format!("object-{index}.o"));
                            compile(&compiler, optimization, source, &object, &directory);
                            object
                        })
                        .collect();
                    let binary = directory.join("consumer");
                    let mut command = Command::new(&compiler);
                    options(&mut command, optimization);
                    successful(command.args(&objects).arg("-o").arg(&binary));
                    assert!(super::native_stack::run(&binary).success());
                    if label == "gcc" && optimization == "-O0" {
                        let export =
                            PathBuf::from(std::env::var_os("TEST_UNDECLARED_OUTPUTS_DIR").unwrap())
                                .join("constant-imports")
                                .join(format!("{shape:?}-{usage:?}"));
                        fs::create_dir_all(&export).unwrap();
                        for entry in fs::read_dir(&directory).unwrap() {
                            let path = entry.unwrap().path();
                            if matches!(
                                path.extension().and_then(|value| value.to_str()),
                                Some("c" | "h")
                            ) {
                                fs::copy(&path, export.join(path.file_name().unwrap())).unwrap();
                            }
                        }
                    }
                    for value in &values {
                        let readonly = directory.join("readonly.c");
                        fs::write(&readonly, format!("#include \"polyrust_constants.h\"\nint main(void) {{ {} = 0; return 0; }}\n", value.symbol().as_str())).unwrap();
                        let mut command = Command::new(&compiler);
                        options(&mut command, optimization);
                        let result = command
                            .arg("-I")
                            .arg(&directory)
                            .arg("-c")
                            .arg(&readonly)
                            .arg("-o")
                            .arg(directory.join("readonly.o"))
                            .output()
                            .unwrap();
                        assert!(!result.status.success());
                        let message = String::from_utf8_lossy(&result.stderr);
                        assert!(message.contains(value.symbol().as_str()));
                        assert!(
                            message.contains("read-only") || message.contains("const-qualified"),
                            "{message}"
                        );
                    }
                    if usage == Usage::Read {
                        let source = &consumer_sources[0];
                        let original = fs::read_to_string(source).unwrap();
                        assert!(original.contains("poly_computed_value"));
                        fs::write(
                            source,
                            original.replace("poly_computed_value", "poly_i32_max"),
                        )
                        .unwrap();
                        let consumer_index =
                            sources.iter().position(|path| path == source).unwrap();
                        compile(
                            &compiler,
                            optimization,
                            source,
                            &objects[consumer_index],
                            &directory,
                        );
                        let mut command = Command::new(&compiler);
                        options(&mut command, optimization);
                        successful(command.args(&objects).arg("-o").arg(&binary));
                        assert!(
                            !super::native_stack::run(&binary).success(),
                            "wrong imported symbol escaped truth oracle"
                        );
                    }
                }
            }
        }
    }
}
