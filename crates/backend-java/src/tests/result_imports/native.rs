//! Pinned javac compiles separate original-owner and importing compilation units.
use super::{fixture::text, methods};
use crate::{
    dialect::JavaDialect,
    tests::{
        budget_oracle,
        source_constants_native::{compile, tool},
    },
};
use portable_codegen::RenderReadyPackage;
use std::{
    path::{Path, PathBuf},
    process::{Command, Output},
};

fn success(output: Output) {
    assert!(
        output.status.success() && output.stderr.is_empty(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
}
fn emit(root: &Path, classes: &Path, certificate: &RenderReadyPackage<JavaDialect>) {
    let file = &certificate.ast().files()[0];
    let path = root.join(file.path().as_str());
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    std::fs::write(&path, text(certificate)).unwrap();
    success(compile(&path, classes));
    let namespace = file.module().name().replace('.', "/");
    budget_oracle::verify_directory(
        &classes.join(namespace),
        &budget_oracle::collect(certificate.ast()),
    );
}

#[test]
fn result_imports_native_construction_patterns_explicit_upcasts_and_nulls() {
    let root = PathBuf::from(std::env::var_os("TEST_TMPDIR").unwrap()).join("java-result-imports");
    let classes = root.join("classes");
    std::fs::create_dir_all(&classes).unwrap();
    let (owner, imported) = methods::certificates();
    emit(&root, &classes, owner.package());
    emit(&root, &classes, &imported);
    let relay = methods::relay(
        &crate::dialect::JavaDependencyApi::from_certificate(imported.clone()).unwrap(),
        10,
    );
    emit(&root, &classes, relay.package());
    assert!(!text(relay.package()).contains("record Success"));
    assert!(!text(&imported).contains("record Success"));
    let consumer = root.join("Consumer.java");
    std::fs::write(&consumer, r#"
import org.polyrust.generated.r000000000000000a.Generated;
import org.polyrust.generated.r0000000000000007.Generated.Outcome;
import org.polyrust.generated.r0000000000000007.Generated.Success;
import org.polyrust.generated.r0000000000000007.Generated.Error;
public final class Consumer {
    public static void main(String[] args) {
        int count = 0;
        for (int value : new int[] {Integer.MIN_VALUE, -257, -1, 0, 1, 17, 257, Integer.MAX_VALUE}) {
            for (int fallback : new int[] {Integer.MIN_VALUE, -1, 0, 17, Integer.MAX_VALUE}) {
                Outcome success = Generated.success(value);
                Outcome error = Generated.error();
                if (!(success instanceof Success payload) || payload.value() != value
                    || !(error instanceof Error)
                    || Generated.copy(success) != success || Generated.copy(error) != error
                    || Generated.observe(success, fallback) != value
                    || Generated.observe(error, fallback) != fallback
                    || Generated.directProbe(value) != value) {
                    throw new AssertionError("foreign result");
                }
                count++;
            }
        }
        try { Generated.copy(null); throw new AssertionError("null copy"); }
        catch (NullPointerException expected) {}
        try { Generated.observe(null, 17); throw new AssertionError("null observation"); }
        catch (NullPointerException expected) {}
        System.out.println(count + " original-family observations");
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
            "40 original-family observations"
        );
        success(output);
    }
    for (label, body, diagnostic) in [
        (
            "Rogue",
            "final class Rogue implements Producer.Outcome {}",
            "not allowed to extend sealed class",
        ),
        (
            "Hidden",
            "final class Hidden { int value = new Producer.Success(1).value; }",
            "private access",
        ),
        (
            "WrongFamily",
            "final class WrongFamily { int value = ConsumerApi.observe(new Producer.OtherError(), 0); }",
            "incompatible types",
        ),
        (
            "ErrorPayload",
            "final class ErrorPayload { int value = new Producer.Error().value(); }",
            "cannot find symbol",
        ),
        (
            "Mutable",
            "final class Mutable { void mutate(Producer.Success value) { value.value = 1; } }",
            "private access",
        ),
    ] {
        let path = root.join(format!("{label}.java"));
        let body = body
            .replace(
                "Producer.",
                "org.polyrust.generated.r0000000000000007.Generated.",
            )
            .replace(
                "ConsumerApi.",
                "org.polyrust.generated.r000000000000000a.Generated.",
            );
        std::fs::write(&path, body).unwrap();
        let output = compile(&path, &classes);
        assert!(
            !output.status.success()
                && String::from_utf8_lossy(&output.stderr).contains(diagnostic),
            "{label}: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }
}
