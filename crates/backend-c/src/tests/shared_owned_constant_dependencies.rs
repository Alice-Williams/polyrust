//! Direct header and transitive external-symbol collisions include globals.
use super::{
    CDialect, CStructuralRenderer,
    bindings::CValueBinding,
    owned_constant_dependency_fixture::{consumer, middle, owner, selected},
    owned_constant_native_tests::{compile, options, successful},
    owned_constant_tests::linked,
    project_c_package,
};
use portable_codegen::{
    OutputContents, RenderReadyPackage, TargetLinker, certify_resolved_package,
    render_certified_package, verify_unresolved_package,
};
use std::{
    fs,
    path::{Path, PathBuf},
    process::Command,
};

#[test]
fn owned_constants_cannot_collide_with_unselected_direct_or_transitive_exports() {
    let owner = owner();
    let middle = middle(&owner);
    for name in ["other_export", "poly_other_export"] {
        for call in [false, true] {
            let direct = consumer(name, selected(&owner), call);
            let projected = project_c_package(direct.registry, direct.files).unwrap();
            let checked = verify_unresolved_package(&CDialect, projected).unwrap();
            let errors = TargetLinker::new(CDialect).link_ast(&checked).unwrap_err();
            assert!(
                errors
                    .iter()
                    .any(|error| error.message.contains("owned C binding collides"))
            );

            let transitive = consumer(name, selected(&middle), call);
            let errors = certify_resolved_package(&CDialect, linked(&transitive)).unwrap_err();
            assert!(errors.iter().any(|error| {
                error
                    .message
                    .contains("transitive dependency complete public symbols collide")
            }));
        }
    }
    for dependency in [selected(&owner), selected(&middle)] {
        let positive = consumer("own_constant", dependency, true);
        certify_resolved_package(&CDialect, linked(&positive)).unwrap();
    }
}

pub(super) fn write_package(
    package: &RenderReadyPackage<CDialect>,
    directory: &Path,
) -> Vec<PathBuf> {
    let rendered = render_certified_package(&CStructuralRenderer, package).unwrap();
    let mut sources = vec![];
    for file in rendered.files() {
        let OutputContents::Text(text) = file.contents() else {
            panic!("C text")
        };
        let path = directory.join(file.path());
        fs::write(&path, text).unwrap();
        if file.path().ends_with(".c") {
            sources.push(path);
        }
    }
    sources
}

#[derive(Clone, Copy, Debug)]
enum Distance {
    Direct,
    Transitive,
}

#[test]
fn native_compilers_confirm_global_function_conflicts_in_headers_and_at_link_time() {
    let owner = owner();
    let middle = middle(&owner);
    let zig = PathBuf::from(std::env::var_os("TEST_SRCDIR").unwrap())
        .join(std::env::var_os("TEST_WORKSPACE").unwrap())
        .join("tools/c/zig_native_oracle");
    for distance in [Distance::Direct, Distance::Transitive] {
        let dependency = match distance {
            Distance::Direct => selected(&owner),
            Distance::Transitive => selected(&middle),
        };
        let fixture = consumer("own_constant", dependency, true);
        let package = linked(&fixture);
        let header = package
            .files()
            .iter()
            .find(|file| file.module() == fixture.objects[0].file())
            .unwrap();
        let old = header.items()[0].spelling.values
            [&CValueBinding::Global(fixture.objects[0].clone())]
            .as_str()
            .to_owned();
        let entry = header.items()[0].spelling.functions[&fixture.functions[0]]
            .as_str()
            .to_owned();
        let certificate = certify_resolved_package(&CDialect, package).unwrap();
        for (label, compiler) in [("gcc", PathBuf::from("gcc-14")), ("zig", zig.clone())] {
            for optimization in ["-O0", "-O2"] {
                let directory = PathBuf::from(std::env::var_os("TEST_TMPDIR").unwrap())
                    .join("constant-export-collisions")
                    .join(format!("{distance:?}-{label}{optimization}"));
                fs::create_dir_all(&directory).unwrap();
                let mut sources = write_package(owner.package(), &directory);
                if matches!(distance, Distance::Transitive) {
                    sources.extend(write_package(middle.package(), &directory));
                }
                let mut objects = vec![];
                for (index, input) in sources.iter().enumerate() {
                    let object = directory.join(format!("dependency-{index}.o"));
                    compile(&compiler, optimization, input, &object, &directory);
                    objects.push(object);
                }
                let consumer_sources = write_package(&certificate, &directory);
                let [input] = consumer_sources.as_slice() else {
                    panic!("one implementation")
                };
                let consumer_object = directory.join("consumer.o");
                compile(&compiler, optimization, input, &consumer_object, &directory);
                let driver = directory.join("driver.c");
                fs::write(&driver, format!(
                    "#include \"polyrust_constant_consumer.h\"\nint main(void) {{ return {entry}() == 42 ? 0 : 1; }}\n"
                )).unwrap();
                let driver_object = directory.join("driver.o");
                compile(&compiler, optimization, &driver, &driver_object, &directory);
                objects.extend([consumer_object.clone(), driver_object]);
                let binary = directory.join("consumer");
                let mut command = Command::new(&compiler);
                options(&mut command, optimization);
                successful(command.args(&objects).arg("-o").arg(&binary));
                assert!(super::native_stack::run(&binary).success());

                // Mutate only a valid generated constant's spelling in BOTH files.
                // Independent C rejects the real namespace conflict; the typed
                // tests above reject it before any rendering/publication.
                for path in [
                    directory.join("polyrust_constant_consumer.h"),
                    input.clone(),
                ] {
                    let original = fs::read_to_string(&path).unwrap();
                    assert!(original.contains(&old));
                    fs::write(path, original.replace(&old, "poly_other_export")).unwrap();
                }
                let mut command = Command::new(&compiler);
                options(&mut command, optimization);
                let output = command
                    .arg("-I")
                    .arg(&directory)
                    .arg("-c")
                    .arg(input)
                    .arg("-o")
                    .arg(&consumer_object)
                    .output()
                    .unwrap();
                match distance {
                    Distance::Direct => {
                        assert!(!output.status.success());
                        assert!(
                            String::from_utf8_lossy(&output.stderr).contains("poly_other_export")
                        );
                    }
                    Distance::Transitive => {
                        assert!(
                            output.status.success(),
                            "{}",
                            String::from_utf8_lossy(&output.stderr)
                        );
                        let mut command = Command::new(&compiler);
                        options(&mut command, optimization);
                        let output = command
                            .args(&objects)
                            .arg("-o")
                            .arg(&binary)
                            .output()
                            .unwrap();
                        assert!(!output.status.success());
                        assert!(
                            String::from_utf8_lossy(&output.stderr).contains("poly_other_export")
                        );
                    }
                }
            }
        }
    }
}
