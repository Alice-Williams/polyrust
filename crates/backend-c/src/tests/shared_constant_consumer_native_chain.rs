//! A native diamond retains one constant producer and an actual callable frame.
use super::{
    CDialect,
    constant_consumer_fixture::{Usage, api, fixture, producer},
    owned_constant_dependency_tests::write_package,
    owned_constant_fixture::Shape,
    owned_constant_native_tests::{compile, options, successful},
    owned_constant_tests::linked,
};
use portable_codegen::certify_resolved_package;
use std::{fs, path::PathBuf, process::Command};

#[test]
fn independently_compiled_constant_diamond_executes_under_both_native_compilers() {
    let owner = producer(Shape::ConstantsOnly);
    let values: Vec<_> = owner.constants().cloned().collect();
    let middle = api(&fixture(91, &values, None, Usage::Read));
    let input = fixture(
        90,
        &values,
        Some(middle.functions().nth(7).unwrap().clone()),
        Usage::Read,
    );
    let package = linked(&input);
    let name = package.files()[1].items()[0].spelling.functions[&input.functions[8]]
        .as_str()
        .to_owned();
    let certificate = certify_resolved_package(&CDialect, package).unwrap();
    let zig = PathBuf::from(std::env::var_os("TEST_SRCDIR").unwrap())
        .join(std::env::var_os("TEST_WORKSPACE").unwrap())
        .join("tools/c/zig_native_oracle");
    for (label, compiler) in [("gcc", PathBuf::from("gcc-14")), ("zig", zig)] {
        for optimization in ["-O0", "-O2"] {
            let directory = PathBuf::from(std::env::var_os("TEST_TMPDIR").unwrap())
                .join("constant-diamond")
                .join(format!("{label}{optimization}"));
            fs::create_dir_all(&directory).unwrap();
            let mut sources = write_package(owner.package(), &directory);
            sources.extend(write_package(middle.package(), &directory));
            sources.extend(write_package(&certificate, &directory));
            let driver = directory.join("driver.c");
            fs::write(&driver, format!("#include \"polyrust_constant_reader_90.h\"\nint main(void) {{ return {name}() == 62 ? 0 : 1; }}\n")).unwrap();
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
                        .join("constant-diamond");
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
        }
    }
}
