//! Native evidence for the certified target fixture, not a Rust translation claim.
use super::fixture::{Fixture, certify};
use portable_codegen::OutputContents;
use std::{
    path::{Path, PathBuf},
    process::Command,
};

fn tool(name: &str) -> PathBuf {
    let runfiles = std::env::var_os("RUNFILES_DIR")
        .or_else(|| std::env::var_os("TEST_SRCDIR"))
        .expect("Bazel runfiles");
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
        .expect("pinned Java 21 tool")
}

fn compile(source: &Path, classes: &Path) -> std::process::Output {
    Command::new(tool("javac"))
        .args(["--release", "21", "-Xlint:all", "-Werror", "-cp"])
        .arg(classes)
        .arg("-d")
        .arg(classes)
        .arg(source)
        .output()
        .unwrap()
}

#[test]
fn certified_private_source_record_compiles_executes_and_rejects_external_construction() {
    let Some(temp) = std::env::var_os("TEST_TMPDIR") else {
        return;
    };
    let root = PathBuf::from(temp).join("java-source-records");
    let classes = root.join("classes");
    std::fs::create_dir_all(&classes).unwrap();
    let rendered = certify(Fixture::new().finish());
    for file in rendered.files() {
        let OutputContents::Text(text) = file.contents() else {
            panic!("Java text");
        };
        assert!(text.contains("private record Cell(int value, boolean flag)"));
        let path = root.join(file.path());
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(&path, text).unwrap();
        let output = compile(&path, &classes);
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
    }
    let consumer = root.join("Consumer.java");
    std::fs::write(
        &consumer,
        r#"
import org.polyrust.generated.r0000000000000007.Fixture;
public final class Consumer {
    public static void main(String[] args) {
        for (int value : new int[] {Integer.MIN_VALUE, -7, 0, 13, Integer.MAX_VALUE}) {
            if (Fixture.inspect(value, true) != value) throw new AssertionError("value");
            if (Fixture.inspect(value, false) != -1) throw new AssertionError("flag");
        }
    }
}
"#,
    )
    .unwrap();
    let output = compile(&consumer, &classes);
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let output = Command::new(tool("java"))
        .arg("-cp")
        .arg(&classes)
        .arg("Consumer")
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let private = root.join("PrivateConsumer.java");
    std::fs::write(
        &private,
        r#"
import org.polyrust.generated.r0000000000000007.Fixture;
final class PrivateConsumer {
    final Object cell = new Fixture.Cell(1, true);
}
"#,
    )
    .unwrap();
    let output = compile(&private, &classes);
    assert!(
        !output.status.success()
            && String::from_utf8_lossy(&output.stderr).contains("private access"),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
}
