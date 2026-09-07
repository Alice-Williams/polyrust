//! Separate compilation and execution of public consumers for totality regressions.

use portable_codegen::{OutputContents, OutputManifest};
use std::path::{Path, PathBuf};
use std::process::Command;

fn jdk_tool(name: &str) -> PathBuf {
    let runfiles = std::env::var_os("RUNFILES_DIR")
        .or_else(|| std::env::var_os("TEST_SRCDIR"))
        .map(PathBuf::from)
        .expect("totality compiler oracle requires Bazel runfiles");
    std::fs::read_dir(runfiles)
        .expect("read runfiles")
        .filter_map(Result::ok)
        .find_map(|entry| {
            entry
                .file_name()
                .to_string_lossy()
                .contains("remotejdk21")
                .then(|| entry.path().join("bin").join(name))
                .filter(|path| path.is_file())
        })
        .expect("hermetic Java 21 tool exists")
}

pub(super) struct CompiledPackage {
    root: PathBuf,
    classes: PathBuf,
}

impl CompiledPackage {
    pub(super) fn new(manifest: &OutputManifest, case: &str) -> Self {
        let root = PathBuf::from(std::env::var_os("TEST_TMPDIR").expect("Bazel test directory"))
            .join(case);
        let classes = root.join("generated-classes");
        std::fs::create_dir_all(&classes).expect("create compiler output");
        let sources = ["Generated.java", "Runtime.java"].map(|name| {
            let file = manifest
                .file(&format!("src/main/java/org/polyrust/generated/{name}"))
                .expect("generated main source");
            let OutputContents::Text(text) = file.contents() else {
                panic!("Java source is text")
            };
            let path = root.join(name);
            std::fs::write(&path, text).expect("write generated source to test scratch");
            path
        });
        let output = Command::new(jdk_tool("javac"))
            .args(["--release", "21", "-Xlint:all", "-Werror", "-d"])
            .arg(&classes)
            .args(sources)
            .output()
            .expect("compile generated package");
        assert!(
            output.status.success(),
            "generated package failed javac:\n{}",
            String::from_utf8_lossy(&output.stderr)
        );
        Self { root, classes }
    }

    pub(super) fn consumer(&self, source: &str) {
        let output = self.compile_consumer(source, "Consumer");
        assert!(
            output.status.success(),
            "separate consumer failed javac:\n{}",
            String::from_utf8_lossy(&output.stderr)
        );
        let classpath = format!(
            "{}:{}",
            self.classes.display(),
            self.root.join("consumer-classes").display()
        );
        let output = Command::new(jdk_tool("java"))
            .args(["-cp", &classpath, "org.polyrust.consumer.Consumer"])
            .output()
            .expect("run separate consumer");
        assert!(
            output.status.success(),
            "consumer failed:\n{}",
            String::from_utf8_lossy(&output.stderr)
        );
    }

    pub(super) fn rejects_consumer(&self, name: &str, source: &str, diagnostic: &str) {
        let output = self.compile_consumer(source, name);
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(
            !output.status.success() && stderr.contains(diagnostic),
            "expected {diagnostic:?}:\n{stderr}"
        );
    }

    pub(super) fn rejects_generated_member(&self, member: &str, diagnostic: &str) {
        let path = self.root.join("Generated.java");
        let original = std::fs::read_to_string(&path).expect("generated source");
        let end = original.rfind('}').expect("enclosing class terminator");
        let mutated = format!("{}{}\n}}", &original[..end], member);
        std::fs::write(&path, mutated).expect("write compile-negative generated fixture");
        let output = compile_against(&self.classes, &self.root.join("negative-classes"), &path);
        std::fs::write(&path, original).expect("restore generated test fixture");
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(
            !output.status.success() && stderr.contains(diagnostic),
            "expected {diagnostic:?}:\n{stderr}"
        );
    }

    fn compile_consumer(&self, source: &str, name: &str) -> std::process::Output {
        let path = self.root.join(format!("{name}.java"));
        std::fs::write(&path, source).expect("write consumer fixture");
        let output = self.root.join("consumer-classes");
        std::fs::create_dir_all(&output).expect("create consumer output");
        compile_against(&self.classes, &output, &path)
    }
}

fn compile_against(classes: &Path, output: &Path, source: &Path) -> std::process::Output {
    Command::new(jdk_tool("javac"))
        .args(["--release", "21", "-Xlint:all", "-Werror", "-cp"])
        .arg(classes)
        .arg("-d")
        .arg(output)
        .arg(source)
        .output()
        .expect("compile separate consumer")
}
