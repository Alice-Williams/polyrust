//! Native files come exclusively from actual certified target packages.
use super::*;
use std::{fs, path::PathBuf, process::Command};

#[test]
fn characters_native_full_domain_and_compiling_faults() {
    let root = PathBuf::from(std::env::var_os("TEST_TMPDIR").unwrap()).join("java-characters");
    for (index, owner) in chain().iter().enumerate() {
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
        .arg(runfiles.join("crates/backend-java/test/characters.py"))
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
