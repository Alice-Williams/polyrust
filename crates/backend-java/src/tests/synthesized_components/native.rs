//! Real Java declarations/accessors, independently compiled external consumers.
use super::fixture::{Fixture, certify};
use crate::ast::JavaVisibility;
use crate::tests::source_constants_native::tool;
use portable_codegen::{OutputContents, render_certified_package};
use std::{
    path::Path,
    process::{Command, Output},
};

fn compile(source: &Path, classes: &Path) -> Output {
    Command::new(tool("javac"))
        .args(["--release", "21", "-Xlint:all", "-Werror", "-cp"])
        .arg(classes)
        .arg("-d")
        .arg(classes)
        .arg(source)
        .output()
        .unwrap()
}
fn success(output: Output) {
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        output.stderr.is_empty(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
}
fn producer(root: &Path, classes: &Path, visibility: JavaVisibility) {
    std::fs::create_dir_all(classes).unwrap();
    let certificate = certify(Fixture::with_visibility(visibility).finish());
    let budgets = crate::tests::budget_oracle::collect(certificate.ast());
    assert_eq!(budgets.len(), 3);
    let output = render_certified_package(&crate::render::JavaRenderer, &certificate).unwrap();
    assert_eq!(output.files().len(), 1);
    for file in output.files() {
        let OutputContents::Text(text) = file.contents() else {
            panic!("text")
        };
        let path = root.join(file.path());
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(&path, text).unwrap();
        success(compile(&path, classes));
    }
    let directory = classes.join("org/polyrust/generated/r0000000000000007");
    assert_eq!(std::fs::read_dir(&directory).unwrap().count(), 3);
    crate::tests::budget_oracle::verify_directory(&directory, &budgets);
}
#[test]
fn synthesized_components_native_accessors_copies_and_private_boundaries() {
    let root = std::path::PathBuf::from(std::env::var_os("TEST_TMPDIR").expect("Bazel test temp"))
        .join("synthesized-components");
    let classes = root.join("classes");
    producer(&root, &classes, JavaVisibility::Public);
    let consumer = root.join("Consumer.java");
    std::fs::write(&consumer, r#"
import org.polyrust.generated.r0000000000000007.SynthesizedFixture;
public final class Consumer {
    public static void main(String[] args) {
        int count = 0;
        for (int value : new int[] {Integer.MIN_VALUE, -1, 0, 1, 17, Integer.MAX_VALUE}) {
            var created = SynthesizedFixture.create(value);
            var copied = SynthesizedFixture.copy(created);
            if (created != copied || created.value() != value
                || SynthesizedFixture.direct(copied) != value
                || SynthesizedFixture.accessor(copied) != value) throw new AssertionError("payload");
            count++;
        }
        System.out.println(count + " synthesized component observations");
    }
}
"#).unwrap();
    success(compile(&consumer, &classes));
    for options in [vec![], vec!["-Xint"]] {
        let output = Command::new(tool("java"))
            .args(options)
            .arg("-cp")
            .arg(&classes)
            .arg("Consumer")
            .output()
            .unwrap();
        assert_eq!(
            String::from_utf8_lossy(&output.stdout).trim(),
            "6 synthesized component observations"
        );
        success(output);
    }
    for (label, expression, diagnostic) in [
        (
            "PrivateField",
            "new SynthesizedFixture.Payload(1).value",
            "private access",
        ),
        (
            "WrongNominal",
            "SynthesizedFixture.direct(new SynthesizedFixture.Other(1))",
            "incompatible types",
        ),
        (
            "WrongConstructor",
            "new SynthesizedFixture.Payload(true).value()",
            "incompatible types",
        ),
    ] {
        let path = root.join(format!("{label}.java"));
        std::fs::write(&path, format!("import org.polyrust.generated.r0000000000000007.SynthesizedFixture; final class {label} {{ final int value = {expression}; }}")).unwrap();
        let output = compile(&path, &classes);
        assert!(
            !output.status.success()
                && String::from_utf8_lossy(&output.stderr).contains(diagnostic),
            "{label}: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }
    let private_root = root.join("private");
    let private_classes = private_root.join("classes");
    producer(&private_root, &private_classes, JavaVisibility::Private);
    let path = private_root.join("PrivateConstructor.java");
    std::fs::write(&path, "import org.polyrust.generated.r0000000000000007.SynthesizedFixture; final class PrivateConstructor { final Object value = new SynthesizedFixture.Payload(1); }").unwrap();
    let output = compile(&path, &private_classes);
    assert!(
        !output.status.success()
            && String::from_utf8_lossy(&output.stderr).contains("private access"),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
}
