//! Native two-owner packages are checked against the independent modular oracle.
use super::*;
use std::{fs, path::PathBuf, process::Command};

#[test]
fn wrapping_addition_native_values_safe_mutants_headers_and_ubsan() {
    let root = PathBuf::from(std::env::var_os("TEST_TMPDIR").unwrap()).join("c-wrapping-addition");
    fs::create_dir_all(&root).unwrap();
    for (name, variant) in [
        ("valid", Variant::Valid),
        ("boundary-literals", Variant::BoundaryLiterals),
        ("wrong-operand", Variant::WrongOperand),
        ("wrong-result", Variant::WrongResult),
    ] {
        let directory = root.join(name);
        fs::create_dir_all(&directory).unwrap();
        for owner in chain(variant) {
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
