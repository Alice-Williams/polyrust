//! Native independent controls for complete mixed-producer constant inventories.
use super::{
    CDialect,
    constant_producer_dependency_tests::{Distance, consumer, middle, producer},
    owned_constant_dependency_tests::write_package,
    owned_constant_native_tests::{compile, options, successful},
    owned_constant_tests::linked,
};
use portable_codegen::certify_resolved_package;
use std::{fs, path::PathBuf, process::Command};

#[test]
fn mixed_producer_constants_really_conflict_with_owned_functions() {
    let producer = producer();
    let middle = middle(&producer);
    let zig = PathBuf::from(std::env::var_os("TEST_SRCDIR").unwrap())
        .join(std::env::var_os("TEST_WORKSPACE").unwrap())
        .join("tools/c/zig_native_oracle");
    for distance in [Distance::Direct, Distance::Transitive] {
        let fixture = consumer(&producer, &middle, distance, "consumer_read", true);
        let package = linked(&fixture);
        let old = package
            .files()
            .iter()
            .flat_map(|file| file.items())
            .find_map(|unit| unit.spelling.functions.get(&fixture.functions[0]))
            .unwrap()
            .as_str()
            .to_owned();
        let certificate = certify_resolved_package(&CDialect, package).unwrap();
        for (label, compiler) in [("gcc", PathBuf::from("gcc-14")), ("zig", zig.clone())] {
            for optimization in ["-O0", "-O2"] {
                let directory = PathBuf::from(std::env::var_os("TEST_TMPDIR").unwrap())
                    .join("constant-producer-native")
                    .join(format!("{distance:?}-{label}{optimization}"));
                fs::create_dir_all(&directory).unwrap();
                let mut sources = write_package(producer.package(), &directory);
                if matches!(distance, Distance::Transitive) {
                    sources.extend(write_package(middle.package(), &directory));
                }
                let mut objects = vec![];
                for (index, source) in sources.iter().enumerate() {
                    let object = directory.join(format!("dependency-{index}.o"));
                    compile(&compiler, optimization, source, &object, &directory);
                    objects.push(object);
                }
                let consumer_sources = write_package(&certificate, &directory);
                let [source] = consumer_sources.as_slice() else {
                    panic!("one implementation")
                };
                let consumer_object = directory.join("consumer.o");
                compile(
                    &compiler,
                    optimization,
                    source,
                    &consumer_object,
                    &directory,
                );
                let expected = match distance {
                    Distance::Direct => 62,
                    Distance::Transitive => 42,
                };
                let driver = directory.join("driver.c");
                let original_driver = format!(
                    "#include \"polyrust_constant_consumer.h\"\nint main(void) {{ return {old}() == {expected} ? 0 : 1; }}\n"
                );
                fs::write(&driver, &original_driver).unwrap();
                let driver_object = directory.join("driver.o");
                compile(&compiler, optimization, &driver, &driver_object, &directory);
                objects.extend([consumer_object.clone(), driver_object.clone()]);
                let binary = directory.join("consumer");
                let mut link = Command::new(&compiler);
                options(&mut link, optimization);
                successful(link.args(&objects).arg("-o").arg(&binary));
                assert!(super::native_stack::run(&binary).success());

                if label == "gcc" && optimization == "-O0" {
                    let export =
                        PathBuf::from(std::env::var_os("TEST_UNDECLARED_OUTPUTS_DIR").unwrap())
                            .join("constant-producers")
                            .join(format!("{distance:?}"));
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

                for path in [
                    directory.join("polyrust_constant_consumer.h"),
                    source.clone(),
                ] {
                    let original = fs::read_to_string(&path).unwrap();
                    assert!(original.contains(&old));
                    fs::write(path, original.replace(&old, "poly_false_value")).unwrap();
                }
                let mut command = Command::new(&compiler);
                options(&mut command, optimization);
                let result = command
                    .arg("-I")
                    .arg(&directory)
                    .arg("-c")
                    .arg(source)
                    .arg("-o")
                    .arg(&consumer_object)
                    .output()
                    .unwrap();
                match distance {
                    Distance::Direct => {
                        assert!(!result.status.success());
                        assert!(
                            String::from_utf8_lossy(&result.stderr).contains("poly_false_value")
                        );
                    }
                    Distance::Transitive => {
                        assert!(
                            result.status.success(),
                            "{}",
                            String::from_utf8_lossy(&result.stderr)
                        );
                        // Update the driver too: do not mistake an old undefined
                        // function reference for proof of the symbol collision.
                        fs::write(&driver, original_driver.replace(&old, "poly_false_value"))
                            .unwrap();
                        compile(&compiler, optimization, &driver, &driver_object, &directory);
                        let mut command = Command::new(&compiler);
                        options(&mut command, optimization);
                        let result = command
                            .args(&objects)
                            .arg("-o")
                            .arg(&binary)
                            .output()
                            .unwrap();
                        assert!(!result.status.success());
                        let diagnostic = String::from_utf8_lossy(&result.stderr);
                        assert!(diagnostic.contains("poly_false_value"));
                        assert!(
                            diagnostic.contains("multiple definition")
                                || diagnostic.contains("duplicate symbol"),
                            "{diagnostic}"
                        );
                    }
                }
            }
        }
    }
}
