//! The documentation passes through the production certificate and renderer.
use super::fixture;
use std::{path::PathBuf, process::Command};

fn tool(name: &str) -> PathBuf {
    let runfiles = std::env::var_os("RUNFILES_DIR")
        .or_else(|| std::env::var_os("TEST_SRCDIR"))
        .unwrap();
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

#[test]
fn production_documentation_compiles_runs_and_cannot_inject_members() {
    let Some(temp) = std::env::var_os("TEST_TMPDIR") else {
        return;
    };
    let root = PathBuf::from(temp).join("java-source-documentation");
    let classes = root.join("classes");
    std::fs::create_dir_all(&classes).unwrap();
    let generated = root.join("Generated.java");
    std::fs::write(&generated, fixture::text(fixture::package())).unwrap();
    let consumer = root.join("Consumer.java");
    std::fs::write(&consumer, r#"
import org.polyrust.generated.r0000000000000007.Generated;
public final class Consumer {
    public static void main(String[] args) {
        if (Generated.class.getDeclaredFields().length != 0) throw new AssertionError("injected fields");
        if (Generated.class.getDeclaredMethods().length != 1) throw new AssertionError("injected methods");
        for (int value : new int[] {Integer.MIN_VALUE, -7, 0, 13, Integer.MAX_VALUE}) {
            if (Generated.inspect(value, true) != value) throw new AssertionError("value");
            if (Generated.inspect(value, false) != -1) throw new AssertionError("flag");
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
    // Bypass encoding only in this oracle control: the raw terminator adds a
    // valid field inside the facade, which must compile and fail reflection.
    let injected = fixture::text(fixture::package()).replacen(
        "record first",
        "*/ public static int injected; /*",
        1,
    );
    std::fs::write(&generated, injected).unwrap();
    let control = root.join("control-classes");
    std::fs::create_dir_all(&control).unwrap();
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
        .arg(&control)
        .arg(&generated)
        .arg(&consumer)
        .output()
        .unwrap();
    assert!(
        compiled.status.success(),
        "raw control did not compile: {}",
        String::from_utf8_lossy(&compiled.stderr)
    );
    let executed = Command::new(tool("java"))
        .arg("-cp")
        .arg(&control)
        .arg("Consumer")
        .output()
        .unwrap();
    assert!(
        !executed.status.success()
            && String::from_utf8_lossy(&executed.stderr).contains("injected fields"),
        "raw control escaped detection: {}",
        String::from_utf8_lossy(&executed.stderr)
    );
}
