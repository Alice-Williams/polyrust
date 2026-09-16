//! Pinned Java compilation and reflection prove the empty facade contains no helpers.
use super::fixture;
use portable_codegen::*;
use std::{fs, path::PathBuf, process::Command};

fn tool(name: &str) -> PathBuf {
    let runfiles = std::env::var_os("RUNFILES_DIR")
        .or_else(|| std::env::var_os("TEST_SRCDIR"))
        .expect("Bazel runfiles");
    fs::read_dir(runfiles)
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
        .expect("pinned Java 21")
}

#[test]
fn generated_empty_documented_facade_compiles_without_runtime_or_fake_members() {
    let ready = fixture::certify(fixture::empty());
    let output = render_certified_package(&crate::render::JavaRenderer, &ready).unwrap();
    assert_eq!(output.files().len(), 1);
    let OutputContents::Text(text) = output.files()[0].contents() else {
        panic!("Java text")
    };
    let directory = PathBuf::from(std::env::var_os("TEST_TMPDIR").expect("Bazel temp"))
        .join("java-source-package");
    let classes = directory.join("classes");
    fs::create_dir_all(&classes).unwrap();
    let generated = directory.join("Generated.java");
    fs::write(&generated, text).unwrap();
    let consumer = directory.join("Consumer.java");
    fs::write(&consumer, r#"
import org.polyrust.generated.r0000000000000007.Generated;
public final class Consumer {
    public static void main(String[] args) {
        if (Generated.class.getDeclaredFields().length != 0) throw new AssertionError("fields");
        if (Generated.class.getDeclaredMethods().length != 0) throw new AssertionError("methods");
        var constructors = Generated.class.getDeclaredConstructors();
        if (constructors.length != 1 || !java.lang.reflect.Modifier.isPrivate(constructors[0].getModifiers())) {
            throw new AssertionError("private constructor");
        }
    }
}
"#).unwrap();
    let compiled = Command::new(tool("javac"))
        .args([
            "--release",
            "21",
            "-encoding",
            "UTF-8",
            "-Xlint:all",
            "-Werror",
            "-d",
        ])
        .arg(&classes)
        .arg(&generated)
        .arg(&consumer)
        .output()
        .unwrap();
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
    let export =
        PathBuf::from(std::env::var_os("TEST_UNDECLARED_OUTPUTS_DIR").expect("Bazel outputs"))
            .join("source-package");
    fs::create_dir_all(&export).unwrap();
    fs::copy(generated, export.join("Generated.java")).unwrap();
    fs::copy(consumer, export.join("Consumer.java")).unwrap();
}
