//! Independent Java21 oracle, finality failures and compiling semantic mutants.
use super::source_constant_fixture as c;
use crate::dialect::JavaStructuralRenderer;
use portable_codegen::{OutputContents, render_certified_package};
use std::{
    path::{Path, PathBuf},
    process::{Command, Output},
};

fn tool(name: &str) -> PathBuf {
    let root = std::env::var_os("RUNFILES_DIR")
        .or_else(|| std::env::var_os("TEST_SRCDIR"))
        .expect("authoritative Bazel runfiles");
    std::fs::read_dir(root)
        .unwrap()
        .filter_map(Result::ok)
        .find_map(|entry| {
            entry
                .file_name()
                .to_string_lossy()
                .contains("remotejdk21")
                .then(|| entry.path().join("bin").join(name))
                .filter(|p| p.is_file())
        })
        .expect("pinned Java21")
}
fn compile(source: &Path, classes: &Path) -> Output {
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
fn success(output: Output) {
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
}
fn run(classes: &Path) -> Output {
    Command::new(tool("java"))
        .arg("-cp")
        .arg(classes)
        .arg("Consumer")
        .output()
        .unwrap()
}
// Expectations are handwritten independently of the AST and renderer.
const EXPECTED: [&str; 8] = [
    "false",
    "true",
    "-2147483648",
    "2147483647",
    "-9223372036854775808L",
    "9223372036854775807L",
    "9007199254740993L",
    "62",
];
const OWNER: &str = "org.polyrust.generated.r000000000000035c.Generated";

fn oracle(mixed: bool) -> String {
    let mut checks = String::new();
    for (i, expected) in EXPECTED.iter().enumerate() {
        checks.push_str(&format!(
            "if ({OWNER}.constant{i} != {expected}) throw new AssertionError(\"constant{i}\");\n"
        ));
        if mixed {
            checks.push_str(&format!(
                "if ({OWNER}.read{i}() != {expected}) throw new AssertionError(\"read{i}\");\n"
            ));
        }
    }
    format!(
        "public final class Consumer {{ public static void main(String[] args) {{ {checks} }} }}"
    )
}
#[test]
fn constant_producers_compile_and_obey_independent_truth_and_finality() {
    let temp = PathBuf::from(std::env::var_os("TEST_TMPDIR").expect("Bazel test directory"));
    for mixed in [false, true] {
        let api = c::api(mixed);
        let output = render_certified_package(&JavaStructuralRenderer, api.package()).unwrap();
        assert_eq!(output.files().len(), 1);
        let file = &output.files()[0];
        let OutputContents::Text(text) = file.contents() else {
            panic!("Java source")
        };
        let root = temp.join(format!("java-constant-producer-{mixed}"));
        let source = root.join(file.path());
        let classes = root.join("classes");
        std::fs::create_dir_all(source.parent().unwrap()).unwrap();
        std::fs::create_dir_all(&classes).unwrap();
        std::fs::write(&source, text).unwrap();
        success(compile(&source, &classes));
        let consumer = root.join("Consumer.java");
        std::fs::write(&consumer, oracle(mixed)).unwrap();
        // The defining unit has already compiled; javac cannot discover its source.
        success(compile(&consumer, &classes));
        success(run(&classes));
        for (i, expected) in EXPECTED.iter().enumerate() {
            let assignment = root.join("Invalid.java");
            std::fs::write(&assignment, format!(
                "final class Invalid {{ static void write() {{ {OWNER}.constant{i} = {expected}; }} }}"
            )).unwrap();
            let result = compile(&assignment, &classes);
            let errors = String::from_utf8_lossy(&result.stderr);
            assert!(
                !result.status.success() && errors.contains("final variable"),
                "{errors}"
            );
        }
        // Constant-variable reads may be inlined by javac. Recompile the
        // independent consumer against each mutant, not just its producer.
        for (i, expected) in EXPECTED.iter().enumerate() {
            let wrong = if i == 0 {
                "true"
            } else if i == 1 {
                "false"
            } else if matches!(i, 4..=6) {
                "0L"
            } else {
                "0"
            };
            let old = format!("constant{i} = {expected};");
            assert!(text.contains(&old), "missing literal declaration {old}");
            let mutant = text.replacen(&old, &format!("constant{i} = {wrong};"), 1);
            let mutant_root = root.join(format!("mutant-{i}"));
            let mutant_classes = mutant_root.join("classes");
            std::fs::create_dir_all(&mutant_classes).unwrap();
            let mutant_source = mutant_root.join("Generated.java");
            std::fs::write(&mutant_source, mutant).unwrap();
            success(compile(&mutant_source, &mutant_classes));
            success(compile(&consumer, &mutant_classes));
            let result = run(&mutant_classes);
            assert!(
                !result.status.success()
                    && String::from_utf8_lossy(&result.stderr).contains("AssertionError")
            );
        }
    }
}
