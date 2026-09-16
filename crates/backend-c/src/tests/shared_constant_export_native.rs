//! Independent C consumers include only the facade, not a producer header.
use super::{
    CDependencyApi,
    constant_consumer_fixture::producer,
    constant_export_fixture::{facade, mixed},
    constant_producer_tests::certify,
    owned_constant_dependency_tests::write_package,
    owned_constant_fixture::Shape,
    owned_constant_native_tests::{compile, options, successful},
};
use std::{fs, path::PathBuf, process::Command};

#[test]
fn alias_only_facades_compile_separately_and_expose_original_constant_values() {
    let zig = PathBuf::from(std::env::var_os("TEST_SRCDIR").unwrap())
        .join(std::env::var_os("TEST_WORKSPACE").unwrap())
        .join("tools/c/zig_native_oracle");
    // Independent of the producer AST and certificate values.
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
        assert_eq!(values.len(), truth.len());
        let second =
            CDependencyApi::from_certificate(certify(&mixed(96, &[], "second_producer_value")))
                .unwrap();
        let extra = second.constants().next().unwrap();
        let mut all_values = values.clone();
        all_values.push(extra.clone());
        let facade = CDependencyApi::from_certificate(certify(&facade(94, &all_values))).unwrap();
        assert_eq!(super::c_defined_constants(facade.package()).count(), 0);
        assert_eq!(super::c_defined_functions(facade.package()).count(), 0);
        let transit_values: Vec<_> = facade
            .foreign_constants()
            .map(|binding| binding.dependency().clone())
            .collect();
        let mixed_facade = CDependencyApi::from_certificate(certify(&mixed(
            95,
            &transit_values,
            "owned_facade_value",
        )))
        .unwrap();
        assert_eq!(
            super::c_defined_constants(mixed_facade.package()).count(),
            1
        );
        let owned = mixed_facade.constants().next().unwrap();
        let checks = values
            .iter()
            .zip(truth)
            .enumerate()
            .map(|(index, (value, expected))| {
                format!(
                    "if ({} != {expected}) return {};",
                    value.symbol().as_str(),
                    index + 1
                )
            })
            .collect::<Vec<_>>()
            .join("\n")
            + &format!(
                "\nif ({} != INT32_C(42)) return 9;",
                owned.symbol().as_str()
            );
        let checks = checks
            + &format!(
                "\nif ({} != INT32_C(42)) return 10;",
                extra.symbol().as_str()
            );
        for (label, compiler) in [("gcc", PathBuf::from("gcc-14")), ("zig", zig.clone())] {
            for optimization in ["-O0", "-O2"] {
                let directory = PathBuf::from(std::env::var_os("TEST_TMPDIR").unwrap())
                    .join("constant-exports")
                    .join(format!("{shape:?}-{label}{optimization}"));
                fs::create_dir_all(&directory).unwrap();
                let mut sources = write_package(owner.package(), &directory);
                sources.extend(write_package(second.package(), &directory));
                sources.extend(write_package(facade.package(), &directory));
                sources.extend(write_package(mixed_facade.package(), &directory));
                // Independently compile each public header, including repeated inclusion.
                for (index, header) in [
                    owner.public_header(),
                    second.public_header(),
                    facade.public_header(),
                    mixed_facade.public_header(),
                ]
                .iter()
                .enumerate()
                {
                    let input = directory.join(format!("header-{index}.c"));
                    fs::write(
                        &input,
                        format!(
                            "#include \"{0}\"\n#include \"{0}\"\n",
                            header.include_path()
                        ),
                    )
                    .unwrap();
                    compile(
                        &compiler,
                        optimization,
                        &input,
                        &directory.join(format!("header-{index}.o")),
                        &directory,
                    );
                }
                let driver = directory.join("consumer.c");
                fs::write(&driver, format!("#include \"polyrust_constant_facade_95.h\"\nint main(void) {{\n{checks}\nreturn 0;\n}}\n")).unwrap();
                sources.push(driver);
                let objects: Vec<_> = sources
                    .iter()
                    .enumerate()
                    .map(|(index, source)| {
                        let object = directory.join(format!("unit-{index}.o"));
                        compile(&compiler, optimization, source, &object, &directory);
                        object
                    })
                    .collect();
                let binary = directory.join("consumer");
                let mut link = Command::new(&compiler);
                options(&mut link, optimization);
                successful(link.args(&objects).arg("-o").arg(&binary));
                assert!(super::native_stack::run(&binary).success());
                if label == "gcc" && optimization == "-O0" {
                    let export =
                        PathBuf::from(std::env::var_os("TEST_UNDECLARED_OUTPUTS_DIR").unwrap())
                            .join("constant-exports")
                            .join(format!("{shape:?}"));
                    fs::create_dir_all(&export).unwrap();
                    for entry in fs::read_dir(&directory).unwrap() {
                        let entry = entry.unwrap();
                        if matches!(
                            entry.path().extension().and_then(|v| v.to_str()),
                            Some("c" | "h")
                        ) {
                            fs::copy(entry.path(), export.join(entry.file_name())).unwrap();
                        }
                    }
                }
            }
        }
    }
}
