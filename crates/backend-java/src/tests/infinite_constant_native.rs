//! Separate native producers, alias facade and readers; mutation-sensitive bits.
use super::*;
use std::{fs, path::PathBuf, process::Command};

#[test]
fn infinity_constants_native_aliases_and_inlining_sensitive_faults() {
    let root =
        PathBuf::from(std::env::var_os("TEST_TMPDIR").unwrap()).join("java-infinite-constants");
    for api in packages() {
        text(&api);
        let output = render_certified_package(&JavaStructuralRenderer, api.package()).unwrap();
        for file in output.files() {
            let path = root.join(file.path());
            fs::create_dir_all(path.parent().unwrap()).unwrap();
            let OutputContents::Text(text) = file.contents() else {
                panic!("source")
            };
            fs::write(path, text).unwrap();
        }
    }
    let runfiles = PathBuf::from(std::env::var_os("TEST_SRCDIR").unwrap())
        .join(std::env::var_os("TEST_WORKSPACE").unwrap());
    let output = Command::new("python3")
        .arg(runfiles.join("crates/backend-java/test/infinite_constants.py"))
        .arg(root)
        .arg(super::super::source_constants_native::tool("javac"))
        .arg(super::super::source_constants_native::tool("java"))
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
