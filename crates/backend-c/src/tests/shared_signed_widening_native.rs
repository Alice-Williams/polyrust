//! Separately compiled actual target owners and external native consumers.
use super::*;
use std::{fs, path::PathBuf, process::Command};

#[test]
fn signed_widening_native_values_faults_headers_and_ubsan() {
    let root = PathBuf::from(std::env::var_os("TEST_TMPDIR").unwrap()).join("c-signed-widening");
    fs::create_dir_all(&root).unwrap();
    for owner in chain(fixture::Body::Widen) {
        for file in render_certified_package(&CStructuralRenderer, owner.package())
            .unwrap()
            .files()
        {
            let OutputContents::Text(text) = file.contents() else {
                panic!("text")
            };
            fs::write(root.join(file.path()), text).unwrap();
        }
    }
    let runfiles = PathBuf::from(std::env::var_os("TEST_SRCDIR").unwrap())
        .join(std::env::var_os("TEST_WORKSPACE").unwrap());
    let output = Command::new("python3")
        .arg(runfiles.join("crates/backend-c/test/signed_widening.py"))
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
