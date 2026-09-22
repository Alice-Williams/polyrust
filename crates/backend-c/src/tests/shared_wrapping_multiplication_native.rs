//! Both original target owners compile independently against multiplication truth.
use super::*;
use std::{fs, path::PathBuf, process::Command};

#[test]
fn wrapping_multiplication_native_values_faults_headers_and_ubsan() {
    let root =
        PathBuf::from(std::env::var_os("TEST_TMPDIR").unwrap()).join("c-wrapping-multiplication");
    fs::create_dir_all(&root).unwrap();
    for (name, variant) in [
        ("valid", Variant::Valid),
        ("boundary-literals", Variant::BoundaryLiterals),
        ("wrong-operand", Variant::WrongOperand),
        ("wrong-result", Variant::WrongResult),
        ("wrong-operation", Variant::WrongOperation),
    ] {
        let directory = root.join(name);
        fs::create_dir_all(&directory).unwrap();
        for owner in chain(Operation::Multiply, variant) {
            for file in render_certified_package(&CStructuralRenderer, owner.package())
                .unwrap()
                .files()
            {
                let OutputContents::Text(text) = file.contents() else {
                    panic!("text")
                };
                fs::write(directory.join(file.path()), text).unwrap();
            }
        }
    }
    let runfiles = PathBuf::from(std::env::var_os("TEST_SRCDIR").unwrap())
        .join(std::env::var_os("TEST_WORKSPACE").unwrap());
    let output = Command::new("python3")
        .arg(runfiles.join("crates/backend-c/test/wrapping_addition_native.py"))
        .arg(&root)
        .arg(runfiles.join("tools/c/zig_native_oracle"))
        .arg(runfiles.join("experiments/rustc-frontend/test"))
        .arg("multiplication")
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    eprintln!("{}", String::from_utf8_lossy(&output.stdout));
}
