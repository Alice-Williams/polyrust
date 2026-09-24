//! Handwritten foreign observer, never accepted as generation input.
use super::fixture::{Fault, checked};
use portable_backend_java::dialect::JavaStructuralRenderer;
use portable_codegen::{OutputContents, render_certified_package};
use std::{
    fs,
    path::{Path, PathBuf},
    process::{Command, Output},
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
        .expect("Bazel pinned Java21")
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
        output.status.success() && output.stderr.is_empty(),
        "{output:?}"
    );
}

const DRIVER: &str = r#"import org.polyrust.generated.ErrorFixture;
public final class Consumer {
    public static void main(String[] args) {
        ErrorFixture.Error[] expected = {
            ErrorFixture.Error.EMPTY, ErrorFixture.Error.INVALID_DIGIT,
            ErrorFixture.Error.POS_OVERFLOW, ErrorFixture.Error.NEG_OVERFLOW,
            ErrorFixture.Error.ZERO, ErrorFixture.Error.NOT_A_POWER_OF_TWO
        };
        ErrorFixture.Outcome[] actual = {
            ErrorFixture.error0(), ErrorFixture.error1(), ErrorFixture.error2(),
            ErrorFixture.error3(), ErrorFixture.error4(), ErrorFixture.error5()
        };
        for (int i = 0; i < 6; i++) {
            if (actual[i] != expected[i] || ErrorFixture.forward(actual[i]) != expected[i]) System.exit(17);
            for (int j = i + 1; j < 6; j++) if (actual[i] == actual[j]) System.exit(17);
        }
        int count = 6;
        for (int value = -4096; value <= 4096; value++) {
            check(value); count++;
        }
        for (int value : new int[] {Integer.MIN_VALUE, Integer.MIN_VALUE + 1, Integer.MAX_VALUE - 1, Integer.MAX_VALUE}) {
            check(value); count++;
        }
        try { ErrorFixture.forward(null); throw new AssertionError("null admitted"); }
        catch (NullPointerException expectedNull) { }
        if (count != 8203) System.exit(19);
        System.out.println("8203 six-state local family observations");
    }
    private static void check(int value) {
        ErrorFixture.Outcome output = ErrorFixture.success(value);
        if (!(output instanceof ErrorFixture.Success success) || success.value() != value) System.exit(18);
        if (ErrorFixture.forward(output) != output) System.exit(18);
    }
}
"#;

pub fn prove() {
    let root = PathBuf::from(std::env::var_os("TEST_TMPDIR").unwrap()).join("error-family");
    for (label, fault, expected) in [
        ("valid", Fault::None, 0),
        ("wrong-kind", Fault::WrongKind, 17),
        ("wrong-payload", Fault::ZeroSuccess, 18),
    ] {
        let directory = root.join(label);
        let classes = directory.join("classes");
        fs::create_dir_all(&classes).unwrap();
        let family = checked(fault);
        let output = render_certified_package(&JavaStructuralRenderer, family.package()).unwrap();
        for file in output.files() {
            let OutputContents::Text(text) = file.contents() else {
                panic!("text")
            };
            let path = directory.join(file.path());
            fs::create_dir_all(path.parent().unwrap()).unwrap();
            fs::write(&path, text).unwrap();
            success(compile(&path, &classes));
            if expected == 0 {
                let exported =
                    PathBuf::from(std::env::var_os("TEST_UNDECLARED_OUTPUTS_DIR").unwrap())
                        .join("six-state-example")
                        .join(file.path());
                fs::create_dir_all(exported.parent().unwrap()).unwrap();
                fs::write(exported, text).unwrap();
            }
        }
        let driver = directory.join("Consumer.java");
        fs::write(&driver, DRIVER).unwrap();
        success(compile(&driver, &classes));
        for options in [vec![], vec!["-Xint"]] {
            let result = Command::new(tool("java"))
                .args(options)
                .arg("-cp")
                .arg(&classes)
                .arg("Consumer")
                .output()
                .unwrap();
            assert_eq!(result.status.code(), Some(expected), "{label}: {result:?}");
            assert!(result.stderr.is_empty(), "{result:?}");
            if expected == 0 {
                assert_eq!(result.stdout, b"8203 six-state local family observations\n");
            } else {
                assert!(result.stdout.is_empty());
            }
        }
        if expected == 0 {
            for (name, body, message) in [
                (
                    "Rogue",
                    "final class Rogue implements ErrorFixture.Outcome {}",
                    "not allowed to extend sealed class",
                ),
                (
                    "NewError",
                    "final class NewError { Object error = new ErrorFixture.Error(); }",
                    "enum classes may not be instantiated",
                ),
                (
                    "PrivateField",
                    "final class PrivateField { int value = new ErrorFixture.Success(1).value; }",
                    "private access",
                ),
            ] {
                let path = directory.join(format!("{name}.java"));
                fs::write(
                    &path,
                    format!("import org.polyrust.generated.ErrorFixture; {body}"),
                )
                .unwrap();
                let result = compile(&path, &classes);
                assert!(
                    !result.status.success()
                        && String::from_utf8_lossy(&result.stderr).contains(message),
                    "{result:?}"
                );
            }
        }
    }
}
