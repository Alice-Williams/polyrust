//! Actual certified files are the sole source for native compilation and fault copies.
use super::*;
use std::{fs, path::PathBuf, process::Command};

#[test]
fn signed_widening_native_values_and_compiling_faults() {
    for materialized in [false, true] {
        let root = PathBuf::from(std::env::var_os("TEST_TMPDIR").unwrap())
            .join(format!("java-signed-widening-{materialized}"));
        for (index, owner) in chain(materialized).iter().enumerate() {
            for file in render_certified_package(&JavaStructuralRenderer, owner.package())
                .unwrap()
                .files()
            {
                let OutputContents::Text(text) = file.contents() else {
                    panic!("text")
                };
                let path = root.join(format!("owner{index}")).join(file.path());
                fs::create_dir_all(path.parent().unwrap()).unwrap();
                fs::write(path, text).unwrap();
            }
        }
        let runfiles = PathBuf::from(std::env::var_os("TEST_SRCDIR").unwrap())
            .join(std::env::var_os("TEST_WORKSPACE").unwrap());
        let output = Command::new("python3")
            .arg(runfiles.join("crates/backend-java/test/signed_widening.py"))
            .arg(root)
            .arg(crate::tests::source_constants_native::tool("javac"))
            .arg(crate::tests::source_constants_native::tool("java"))
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
}
