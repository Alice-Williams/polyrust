// Independent target-level native proof. rustc dependency joins are E04C.
use super::*;
use std::{
    collections::BTreeMap,
    path::{Path, PathBuf},
    process::Command,
};

fn tool(name: &str) -> PathBuf {
    let runfiles = std::env::var_os("RUNFILES_DIR")
        .or_else(|| std::env::var_os("TEST_SRCDIR"))
        .expect("authoritative Bazel runfiles");
    std::fs::read_dir(runfiles)
        .unwrap()
        .filter_map(Result::ok)
        .find_map(|entry| {
            entry
                .file_name()
                .to_string_lossy()
                .contains("remotejdk21")
                .then(|| entry.path().join("bin").join(name))
                .filter(|path| path.is_file())
        })
        .expect("pinned Java21 tool")
}

fn compile(source: &Path, classes: &Path) -> std::process::Output {
    Command::new(tool("javac"))
        .args([
            "--release",
            "21",
            "-encoding",
            "UTF-8",
            "-Xlint:all",
            "-Werror",
            "-implicit:none",
            "-sourcepath",
            "",
            "-cp",
        ])
        .arg(classes)
        .arg("-d")
        .arg(classes)
        .arg(source)
        .output()
        .unwrap()
}

fn generated_consumer(
    first: &JavaDependencyApi,
    second: &JavaDependencyApi,
    record: &JavaDependencyApi,
    unused: &JavaDependencyApi,
    reverse: bool,
) -> portable_codegen::RenderReadyPackage<JavaDialect> {
    let mut functions = vec![
        first.function(f::id(11, 10)).unwrap(),
        first.function(f::id(11, 11)).unwrap(),
        second.function(f::id(12, 12)).unwrap(),
        second.function(f::id(12, 10)).unwrap(),
        record.function(f::id(7, 5)).unwrap(),
        unused.function(f::id(6, 10)).unwrap(),
    ];
    if reverse {
        functions.reverse();
    }
    let mut scope = JavaDependencyScope::new();
    let mut imports = BTreeMap::new();
    for function in functions {
        let (next, callable) = scope.import(function.clone()).unwrap();
        scope = next;
        imports.insert(function.declaration(), callable);
    }
    let mut output = Vec::new();
    for (hash, dependency) in [
        (10, f::id(11, 10)),
        (11, f::id(11, 11)),
        (12, f::id(12, 12)),
        (20, f::id(12, 10)),
        (21, f::id(7, 5)),
    ] {
        let callable = &imports[&dependency];
        let parameters = callable
            .signature()
            .parameters
            .iter()
            .enumerate()
            .map(|(index, ty)| JavaParameter {
                ty: ty.clone(),
                name: f::name(&format!("p{index}")),
                final_parameter: true,
            })
            .collect::<Vec<_>>();
        let arguments = parameters
            .iter()
            .map(|parameter| JavaExpr::local(parameter.ty.clone(), parameter.name.clone()))
            .collect();
        output.push(f::Function {
            hash,
            public: true,
            name: f::name(&format!("fn{hash:016x}")),
            parameters,
            result: callable.signature().result.clone(),
            body: JavaBlock::new(vec![JavaStmt::Return(Some(call(callable, arguments)))]),
        });
    }
    f::certify(f::package_with_dependencies(13, output, scope.finish()))
}

fn publish_and_compile(
    package: &portable_codegen::RenderReadyPackage<JavaDialect>,
    root: &Path,
    classes: &Path,
) -> String {
    let rendered = render_certified_package(&JavaStructuralRenderer, package).unwrap();
    assert_eq!(rendered.files().len(), 1);
    let file = &rendered.files()[0];
    let OutputContents::Text(text) = file.contents() else {
        panic!("Java text");
    };
    let path = root.join(file.path());
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    assert!(!path.exists(), "each owning file is published once");
    std::fs::write(&path, text).unwrap();
    let output = compile(&path, classes);
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    text.clone()
}

#[test]
fn separately_compiled_dependency_owners_match_independent_java_consumers() {
    let Some(temp) = std::env::var_os("TEST_TMPDIR") else {
        return;
    };
    let root = PathBuf::from(temp).join("java-certified-dependency-consumers");
    let classes = root.join("classes");
    std::fs::create_dir_all(&classes).unwrap();
    let first = owner(11, 42);
    let second = owner(12, -99);
    let unused = owner(6, 777);
    let record =
        JavaDependencyApi::from_certificate(f::certify(f::record_package(|_| {}))).unwrap();
    let consumer = generated_consumer(&first, &second, &record, &unused, false);
    let output = render_certified_package(&JavaStructuralRenderer, &consumer).unwrap();
    for reverse in [true, false, true] {
        let repeated = generated_consumer(&first, &second, &record, &unused, reverse);
        assert_eq!(
            output,
            render_certified_package(&JavaStructuralRenderer, &repeated).unwrap()
        );
    }
    let api = JavaDependencyApi::from_certificate(consumer.clone()).unwrap();
    assert_eq!(
        api.dependencies()
            .map(|owner| owner.root().crate_id)
            .collect::<Vec<_>>(),
        [6, 7, 11, 12]
    );
    for owner in [&record, &first, &second] {
        let text = publish_and_compile(owner.package(), &root, &classes);
        assert!(!text.contains("import "));
    }
    let text = publish_and_compile(&consumer, &root, &classes);
    assert!(!text.contains("import "));
    assert!(!text.contains("r0000000000000006"));
    assert!(!text.contains("private record"));
    assert!(!text.contains("Function 13 documentation"));
    assert!(!text.contains("return 42") && !text.contains("return -99"));
    let oracle = root.join("Consumer.java");
    std::fs::write(&oracle, ORACLE).unwrap();
    let compiled = compile(&oracle, &classes);
    assert!(
        compiled.status.success(),
        "{}",
        String::from_utf8_lossy(&compiled.stderr)
    );
    let executed = Command::new(tool("java"))
        .arg("-cp")
        .arg(&classes)
        .arg("Consumer")
        .output()
        .unwrap();
    assert!(
        executed.status.success(),
        "{}",
        String::from_utf8_lossy(&executed.stderr)
    );
    assert_eq!(
        String::from_utf8_lossy(&executed.stdout).trim(),
        "8204 cases passed"
    );
    for (name, body) in [
        (
            "PrivateHelper",
            "int value = org.polyrust.generated.r000000000000000b.Generated.fn000000000000000d(0);",
        ),
        (
            "PrivateRecord",
            "Object value = new org.polyrust.generated.r0000000000000007.Generated.Cell(1, true);",
        ),
    ] {
        let path = root.join(format!("{name}.java"));
        std::fs::write(&path, format!("final class {name} {{ {body} }}")).unwrap();
        let compiled = compile(&path, &classes);
        let errors = String::from_utf8_lossy(&compiled.stderr);
        assert!(
            !compiled.status.success() && errors.contains("private access"),
            "{errors}"
        );
    }
}

// Handwritten expectations; no emitted body or lowering routine supplies them.
const ORACLE: &str = r#"
public final class Consumer {
    public static void main(String[] args) {
        if (org.polyrust.generated.r000000000000000d.Generated.fn000000000000000a() != 42) throw new AssertionError("first owner");
        if (org.polyrust.generated.r000000000000000d.Generated.fn0000000000000014() != -99) throw new AssertionError("second owner");
        final int[] edges = {Integer.MIN_VALUE, Integer.MIN_VALUE + 1, -65536, -128, -1, 0, 1, 127, 65535, 65536, Integer.MAX_VALUE - 1, Integer.MAX_VALUE};
        int state = 0x13579bdf;
        for (int i = 0; i < 8204; ++i) {
            state = 1664525 * state + 1013904223;
            final int value = i < edges.length ? edges[i] : state;
            for (boolean flag : new boolean[] {false, true}) {
                if (org.polyrust.generated.r000000000000000d.Generated.fn000000000000000b(value, flag, ~value, !flag) != value) throw new AssertionError("four parameters");
                if (org.polyrust.generated.r000000000000000d.Generated.fn000000000000000c(flag) != flag) throw new AssertionError("boolean");
                if (org.polyrust.generated.r000000000000000d.Generated.fn0000000000000015(value, flag) != (flag ? value : -1)) throw new AssertionError("private record behavior");
            }
        }
        System.out.println("8204 cases passed");
    }
}
"#;
