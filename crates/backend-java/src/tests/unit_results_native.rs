//! Separately compiled Java21 void ABI and negative assignment proof.
use super::*;
use std::{
    fs,
    path::{Path, PathBuf},
    process::Command,
};

fn tool(name: &str) -> PathBuf {
    let root = std::env::var_os("RUNFILES_DIR")
        .or_else(|| std::env::var_os("TEST_SRCDIR"))
        .unwrap();
    fs::read_dir(root)
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
        .expect("pinned Java21")
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
#[test]
fn unit_results_compile_separately_and_reflect_primitive_void() {
    let root = PathBuf::from(std::env::var_os("TEST_TMPDIR").unwrap()).join("java-unit-results");
    let classes = root.join("classes");
    fs::create_dir_all(&classes).unwrap();
    for owner in chain() {
        let output = render_certified_package(&JavaStructuralRenderer, owner.package()).unwrap();
        for file in output.files() {
            let OutputContents::Text(text) = file.contents() else {
                panic!("Java text");
            };
            let path = root.join(file.path());
            fs::create_dir_all(path.parent().unwrap()).unwrap();
            fs::write(&path, text).unwrap();
            let result = compile(&path, &classes);
            assert!(
                result.status.success(),
                "{}",
                String::from_utf8_lossy(&result.stderr)
            );
        }
    }
    let oracle = root.join("Consumer.java");
    fs::write(&oracle, r#"
public final class Consumer {
    public static void main(String[] args) throws ReflectiveOperationException {
        for (int owner : new int[] {71, 72, 73}) {
            Class<?> type = Class.forName(String.format("org.polyrust.generated.r%016x.Generated", owner));
            if (type.getDeclaredFields().length != 0 || type.getDeclaredClasses().length != 0)
                throw new AssertionError("unexpected unit storage/helper");
            if (type.getDeclaredMethods().length != 1)
                throw new AssertionError("unexpected method");
            if (type.getDeclaredMethod("operation", int.class).getReturnType() != (owner == 73 ? int.class : void.class))
                throw new AssertionError("result ABI");
        }
        for (int value : new int[] {Integer.MIN_VALUE, -1, 0, 1, Integer.MAX_VALUE}) {
            org.polyrust.generated.r0000000000000047.Generated.operation(value);
            org.polyrust.generated.r0000000000000048.Generated.operation(value);
            if (org.polyrust.generated.r0000000000000049.Generated.operation(value) != value)
                throw new AssertionError("mixed result");
        }
    }
}
"#).unwrap();
    let result = compile(&oracle, &classes);
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
    let result = Command::new(tool("java"))
        .arg("-cp")
        .arg(&classes)
        .arg("Consumer")
        .output()
        .unwrap();
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
    let invalid = root.join("Invalid.java");
    fs::write(&invalid, "final class Invalid { int value = org.polyrust.generated.r0000000000000047.Generated.operation(1); }").unwrap();
    let result = compile(&invalid, &classes);
    assert!(!result.status.success());
    assert!(String::from_utf8_lossy(&result.stderr).contains("void cannot be converted to int"));
}
