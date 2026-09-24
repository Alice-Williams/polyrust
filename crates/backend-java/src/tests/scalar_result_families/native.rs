use super::fixture::{Fixture, certify};
use crate::dialect::JavaScalarResultFamily;
use crate::tests::{
    budget_oracle,
    source_constants_native::{compile, tool},
};
use portable_codegen::{OutputContents, render_certified_package};
use std::{
    path::PathBuf,
    process::{Command, Output},
};

fn success(output: Output) {
    assert!(
        output.status.success() && output.stderr.is_empty(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
}
#[test]
fn scalar_result_families_native_values_null_policy_closed_subtypes_and_capacity() {
    let root = PathBuf::from(std::env::var_os("TEST_TMPDIR").expect("Bazel temp"))
        .join("java-scalar-result-families");
    let classes = root.join("classes");
    std::fs::create_dir_all(&classes).unwrap();
    let fixture = Fixture::new();
    let types = fixture.family.types;
    let family =
        JavaScalarResultFamily::from_certificate(certify(fixture.finish()), types).unwrap();
    let budgets = budget_oracle::collect(family.package().ast());
    assert_eq!(budgets.len(), 7);
    let rendered =
        render_certified_package(&crate::render::JavaRenderer, family.package()).unwrap();
    for file in rendered.files() {
        let OutputContents::Text(text) = file.contents() else {
            panic!("text")
        };
        let path = root.join(file.path());
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(&path, text).unwrap();
        success(compile(&path, &classes));
    }
    let directory = classes.join("org/polyrust/generated/r0000000000000007");
    assert_eq!(std::fs::read_dir(&directory).unwrap().count(), 7);
    budget_oracle::verify_directory(&directory, &budgets);
    let consumer = root.join("Consumer.java");
    std::fs::write(&consumer, r#"
import org.polyrust.generated.r0000000000000007.ResultFixture;
public final class Consumer {
    public static void main(String[] args) {
        int count = 0;
        for (int value : new int[] {Integer.MIN_VALUE, -257, -1, 0, 1, 17, 257, Integer.MAX_VALUE}) {
            for (boolean success : new boolean[] {false, true}) {
                for (int fallback : new int[] {Integer.MIN_VALUE, -1, 0, 17, Integer.MAX_VALUE}) {
                    ResultFixture.Outcome result = ResultFixture.create(success, value);
                    if ((result instanceof ResultFixture.Success) != success
                        || (result instanceof ResultFixture.Error) == success
                        || ResultFixture.copy(result) != result
                        || ResultFixture.observe(result, fallback) != (success ? value : fallback)
                        || ResultFixture.entry(success, value, fallback) != (success ? value : fallback)
                        || ResultFixture.directProbe(value) != value) {
                        throw new AssertionError("result");
                    }
                    count++;
                }
            }
        }
        try { ResultFixture.observe(null, 17); throw new AssertionError("null observe accepted"); }
        catch (NullPointerException expected) { }
        try { ResultFixture.copy(null); throw new AssertionError("null copy accepted"); }
        catch (NullPointerException expected) { }
        System.out.println(count + " local result observations");
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
            "80 local result observations"
        );
        success(output);
    }
    for (label, body, diagnostic) in [
        (
            "Rogue",
            "final class Rogue implements ResultFixture.Outcome {}",
            "not allowed to extend sealed class",
        ),
        (
            "Hidden",
            "final class Hidden { int value = new ResultFixture.Success(1).value; }",
            "private access",
        ),
        (
            "WrongFamily",
            "final class WrongFamily { int value = ResultFixture.observe(new ResultFixture.OtherError(), 0); }",
            "incompatible types",
        ),
        (
            "ErrorPayload",
            "final class ErrorPayload { int value = new ResultFixture.Error().value(); }",
            "cannot find symbol",
        ),
        (
            "Mutable",
            "final class Mutable { void mutate(ResultFixture.Success value) { value.value = 1; } }",
            "private access",
        ),
    ] {
        let path = root.join(format!("{label}.java"));
        std::fs::write(
            &path,
            format!("import org.polyrust.generated.r0000000000000007.ResultFixture; {body}"),
        )
        .unwrap();
        let output = compile(&path, &classes);
        assert!(
            !output.status.success()
                && String::from_utf8_lossy(&output.stderr).contains(diagnostic),
            "{label}: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }
}
