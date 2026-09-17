//! Separate javac invocations, handwritten truth, reflection, and inlining-aware mutants.
use super::{
    constant_exports_fixture as f, source_constant_consumer_fixture as consumer,
    source_constant_fixture as c, source_constants_native as native,
};
use crate::dialect::*;
use portable_codegen::{OutputContents, render_certified_package};
use std::path::{Path, PathBuf};

fn emit(api: &JavaDependencyApi, root: &Path) -> (PathBuf, String) {
    let output = render_certified_package(&JavaStructuralRenderer, api.package()).unwrap();
    assert_eq!(output.files().len(), 1);
    let file = &output.files()[0];
    let OutputContents::Text(text) = file.contents() else {
        panic!()
    };
    let path = root.join(file.path());
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    std::fs::write(&path, text).unwrap();
    if let Some(artifacts) = std::env::var_os("TEST_UNDECLARED_OUTPUTS_DIR") {
        let artifact = PathBuf::from(artifacts)
            .join("java-constant-exports")
            .join(file.path());
        std::fs::create_dir_all(artifact.parent().unwrap()).unwrap();
        std::fs::write(artifact, text).unwrap();
    }
    (path, text.clone())
}
#[test]
fn independent_alias_facades_compile_without_storage_and_keep_exact_producer_values() {
    let producer = c::api(false);
    let second = c::admit(f::Fixture::new(0x620, &[], true).finish()).unwrap();
    let mut values: Vec<_> = producer.constants().cloned().collect();
    values.push(second.constants().next().unwrap().clone());
    let middle = c::admit(f::Fixture::new(0x621, &values, false).finish()).unwrap();
    let aliases: Vec<_> = middle
        .foreign_constants()
        .filter(|v| v.module() == middle.root())
        .map(|v| v.dependency().clone())
        .collect();
    assert_eq!(aliases, values);
    let mixed = c::admit(f::Fixture::new(0x622, &aliases, true).finish()).unwrap();
    let mut selected: Vec<_> = mixed
        .foreign_constants()
        .filter(|v| v.module() == mixed.root())
        .map(|v| v.dependency().clone())
        .collect();
    selected.push(mixed.constants().next().unwrap().clone());
    let mut scope = JavaDependencyScope::new();
    let mut reads = Vec::new();
    for constant in selected {
        let (next, value) = scope.import_constant(constant);
        scope = next;
        reads.push(value);
    }
    let downstream = c::admit(consumer::consumer(0x623, scope.finish(), &reads, None)).unwrap();
    let root =
        PathBuf::from(std::env::var_os("TEST_TMPDIR").unwrap()).join("java-constant-exports");
    let classes = root.join("classes");
    std::fs::create_dir_all(&classes).unwrap();
    let mut sources = Vec::new();
    for (name, api) in [
        ("producer", &producer),
        ("second", &second),
        ("middle", &middle),
        ("mixed", &mixed),
        ("downstream", &downstream),
    ] {
        let (path, text) = emit(api, &root.join(name));
        native::success(native::compile(&path, &classes));
        assert!(!text.contains("Runtime") && !text.contains("import "));
        sources.push((path, text));
    }
    let mut checks = String::new();
    for (i, expected) in native::EXPECTED
        .iter()
        .chain(["42", "42"].iter())
        .enumerate()
    {
        checks.push_str(&format!("if (org.polyrust.generated.r0000000000000623.Generated.read{i}() != {expected}) throw new AssertionError(\"read{i}\");\n"));
    }
    checks.push_str(r#"
        Class<?> alias = Class.forName("org.polyrust.generated.r0000000000000621.Generated");
        if (alias.getDeclaredFields().length != 0 || alias.getDeclaredMethods().length != 0)
            throw new AssertionError("alias storage or wrapper");
        if (alias.getDeclaredConstructors().length != 1 ||
            !java.lang.reflect.Modifier.isPrivate(alias.getDeclaredConstructors()[0].getModifiers()))
            throw new AssertionError("alias constructor");
        Class<?> mixed = Class.forName("org.polyrust.generated.r0000000000000622.Generated");
        if (mixed.getDeclaredFields().length != 1 || mixed.getDeclaredMethods().length != 0)
            throw new AssertionError("mixed alias storage");
    "#);
    let oracle = root.join("Consumer.java");
    std::fs::write(&oracle, format!("public final class Consumer {{ public static void main(String[] args) throws Exception {{ {checks} }} }}")).unwrap();
    if let Some(artifacts) = std::env::var_os("TEST_UNDECLARED_OUTPUTS_DIR") {
        std::fs::copy(
            &oracle,
            PathBuf::from(artifacts).join("java-constant-exports/Consumer.java"),
        )
        .unwrap();
    }
    native::success(native::compile(&oracle, &classes));
    native::success(native::run(&classes));
    for (i, expected) in native::EXPECTED.iter().enumerate() {
        let wrong = if i == 0 {
            "true"
        } else if i == 1 {
            "false"
        } else if matches!(i, 4..=6) {
            "0L"
        } else {
            "0"
        };
        let old = format!("constant{i} = {expected};");
        assert!(sources[0].1.contains(&old));
        let mutant = sources[0]
            .1
            .replacen(&old, &format!("constant{i} = {wrong};"), 1);
        let mutant_root = root.join(format!("mutant-{i}"));
        let mutant_classes = mutant_root.join("classes");
        std::fs::create_dir_all(&mutant_classes).unwrap();
        let mutant_path = mutant_root.join("Generated.java");
        std::fs::write(&mutant_path, mutant).unwrap();
        native::success(native::compile(&mutant_path, &mutant_classes));
        // Recompile both the generated readers and oracle: javac may inline constants.
        for (path, _) in sources.iter().skip(1) {
            native::success(native::compile(path, &mutant_classes));
        }
        native::success(native::compile(&oracle, &mutant_classes));
        let output = native::run(&mutant_classes);
        assert!(
            !output.status.success()
                && String::from_utf8_lossy(&output.stderr)
                    .contains(&format!("AssertionError: read{i}"))
        );
    }
}
