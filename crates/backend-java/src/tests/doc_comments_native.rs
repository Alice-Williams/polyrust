//! Lexical proof scaffold only; production attachment is the parent checkpoint.
use super::{JavaDocComment, corpus};
use std::{
    path::{Path, PathBuf},
    process::Command,
};

fn tool(name: &str) -> PathBuf {
    let root = std::env::var_os("RUNFILES_DIR")
        .or_else(|| std::env::var_os("TEST_SRCDIR"))
        .expect("Bazel runfiles");
    std::fs::read_dir(root)
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

fn scaffold(payloads: impl IntoIterator<Item = String>) -> String {
    let mut source = String::from("public final class DocFixture {\n");
    for payload in payloads {
        source.push_str("/**\n");
        for line in payload.split('\n') {
            source.push_str(" * ");
            source.push_str(line);
            source.push('\n');
        }
        source.push_str(" */\n");
    }
    source.push_str("public static void main(String[] args) {\nif (DocFixture.class.getDeclaredFields().length != 0) throw new AssertionError(\"injected tokens\");\n}\n}\n");
    source
}

fn compile(path: &Path, classes: &Path) {
    let result = Command::new(tool("javac"))
        .args([
            "--release",
            "21",
            "-encoding",
            "UTF-8",
            "-Xlint:all",
            "-Werror",
            "-d",
        ])
        .arg(classes)
        .arg(path)
        .output()
        .unwrap();
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
}

fn run(classes: &Path) -> std::process::Output {
    Command::new(tool("java"))
        .arg("-cp")
        .arg(classes)
        .arg("DocFixture")
        .output()
        .unwrap()
}

#[test]
fn encoded_corpus_is_non_executable_and_raw_controls_demonstrate_injection() {
    let Some(temp) = std::env::var_os("TEST_TMPDIR") else {
        return;
    };
    let root = PathBuf::from(temp).join("java-doc-comments");
    let classes = root.join("classes");
    std::fs::create_dir_all(&classes).unwrap();
    let path = root.join("DocFixture.java");
    let safe = scaffold(
        corpus()
            .iter()
            .map(|input| JavaDocComment::new(input).text().to_owned()),
    );
    std::fs::write(&path, safe).unwrap();
    compile(&path, &classes);
    let result = run(&classes);
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
    for raw in [
        "*/ static int injected; /*",
        r"\u002a\u002f static int injected; \u002f\u002a",
    ] {
        std::fs::write(&path, scaffold([raw.to_owned()])).unwrap();
        compile(&path, &classes);
        let result = run(&classes);
        assert!(
            !result.status.success()
                && String::from_utf8_lossy(&result.stderr).contains("injected tokens"),
            "{}",
            String::from_utf8_lossy(&result.stderr)
        );
    }
}
