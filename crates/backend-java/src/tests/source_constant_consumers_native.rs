//! Independently compiled generated consumers, not only producer fields.
use super::{
    source_constant_consumer_fixture as f, source_constant_fixture as c,
    source_constants_native::{EXPECTED, compile, run, success},
    source_dependency_fixture as source,
};
use crate::dialect::{JavaDependencyApi, JavaDependencyScope, JavaStructuralRenderer};
use portable_codegen::{OutputContents, render_certified_package};
use std::path::Path;

fn publish(api: &JavaDependencyApi, root: &Path, classes: &Path) -> (std::path::PathBuf, String) {
    let output = render_certified_package(&JavaStructuralRenderer, api.package()).unwrap();
    assert_eq!(output.files().len(), 1);
    let file = &output.files()[0];
    let OutputContents::Text(text) = file.contents() else {
        panic!()
    };
    let path = root.join(file.path());
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    std::fs::write(&path, text).unwrap();
    success(compile(&path, classes));
    (path, text.clone())
}
#[test]
fn generated_constant_consumers_match_truth_and_reject_compiling_reference_mutants() {
    let temp =
        std::path::PathBuf::from(std::env::var_os("TEST_TMPDIR").expect("Bazel test directory"));
    for mixed in [false, true] {
        for used in [false, true] {
            let fixture = f::Fixture::new(mixed);
            let consumer = c::admit(fixture.draft(used)).unwrap();
            let root = temp.join(format!("java-constant-consumer-{mixed}-{used}"));
            let classes = root.join("classes");
            std::fs::create_dir_all(&classes).unwrap();
            publish(&fixture.owner, &root, &classes);
            let (path, text) = publish(&consumer, &root, &classes);
            let owner = "org.polyrust.generated.r0000000000000500.Generated";
            let mut checks = String::new();
            if used {
                for (i, expected) in EXPECTED.iter().enumerate() {
                    checks.push_str(&format!("if ({owner}.read{i}() != {expected}) throw new AssertionError(\"value{i}\");\n"));
                }
            } else if !mixed {
                checks.push_str(&format!(
                    "if ({owner}.fn000000000000000a() != 42) throw new AssertionError(\"unused\");"
                ));
            }
            if mixed {
                checks.push_str(&format!(
                    "if ({owner}.callBridge()) throw new AssertionError(\"call\");"
                ));
            }
            let oracle = root.join("Consumer.java");
            std::fs::write(&oracle, format!("public final class Consumer {{ public static void main(String[] args) {{ {checks} }} }}")).unwrap();
            success(compile(&oracle, &classes));
            success(run(&classes));
            for (i, expected) in EXPECTED.iter().enumerate() {
                let invalid = root.join("Invalid.java");
                std::fs::write(&invalid, format!("final class Invalid {{ static void write() {{ org.polyrust.generated.r000000000000035c.Generated.constant{i} = {expected}; }} }}")).unwrap();
                let result = compile(&invalid, &classes);
                assert!(
                    !result.status.success()
                        && String::from_utf8_lossy(&result.stderr).contains("final variable")
                );
            }
            if used {
                for (i, wrong) in [1, 0, 3, 2, 5, 6, 4, 2].into_iter().enumerate() {
                    let old = format!(".constant{i};");
                    assert!(text.contains(&old));
                    let mutant = text.replacen(&old, &format!(".constant{wrong};"), 1);
                    // The public call interface stays identical and every reference
                    // is still legal Java. Only independent truth detects this.
                    std::fs::write(&path, mutant).unwrap();
                    success(compile(&path, &classes));
                    success(compile(&oracle, &classes));
                    let result = run(&classes);
                    assert!(
                        !result.status.success()
                            && String::from_utf8_lossy(&result.stderr).contains("AssertionError")
                    );
                }
                // Export only the original certified source, never a mutant.
                std::fs::write(&path, text).unwrap();
                success(compile(&path, &classes));
                success(run(&classes));
            }
        }
    }
}
#[test]
fn generated_constant_diamond_executes_real_intermediate_call() {
    let fixture = f::Fixture::new(false);
    let bridge = c::admit(fixture.draft(true)).unwrap();
    let (scope, callable) =
        JavaDependencyScope::new().import(bridge.function(source::id(0x500, 27)).unwrap().clone());
    let (scope, value) = scope.import_constant(
        fixture
            .owner
            .constant(source::id(0x35c, 16))
            .unwrap()
            .clone(),
    );
    let root_api = c::admit(f::consumer(
        0x501,
        scope.finish(),
        &[value],
        Some(&callable),
    ))
    .unwrap();
    let root =
        std::path::PathBuf::from(std::env::var_os("TEST_TMPDIR").expect("Bazel test directory"))
            .join("java-constant-diamond");
    let classes = root.join("classes");
    std::fs::create_dir_all(&classes).unwrap();
    for api in [&fixture.owner, &bridge, &root_api] {
        publish(api, &root, &classes);
    }
    let oracle = root.join("Consumer.java");
    std::fs::write(
        &oracle,
        r#"public final class Consumer {
        public static void main(String[] args) {
            if (org.polyrust.generated.r0000000000000501.Generated.read0() != 9007199254740993L
                || org.polyrust.generated.r0000000000000501.Generated.callBridge() != 62)
                throw new AssertionError("diamond");
        }
    }"#,
    )
    .unwrap();
    success(compile(&oracle, &classes));
    success(run(&classes));
}
