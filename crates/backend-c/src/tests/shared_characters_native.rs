//! Compile the actual certified target files, never a handwritten substitute.
use super::*;
use std::{fs, path::PathBuf, process::Command};

#[test]
fn characters_native_full_domain_comparisons_faults_and_headers() {
    let root = PathBuf::from(std::env::var_os("TEST_TMPDIR").unwrap()).join("c-characters");
    fs::create_dir_all(&root).unwrap();
    for owner in chain() {
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
    let record = record();
    let symbol = crate::dialect::c_defined_functions(&record)
        .find(|function| function.linkage() == CLinkage::External)
        .unwrap();
    fs::write(root.join("record_symbol.txt"), symbol.name().as_str()).unwrap();
    for file in render_certified_package(&CStructuralRenderer, &record)
        .unwrap()
        .files()
    {
        let OutputContents::Text(text) = file.contents() else {
            panic!("text")
        };
        fs::write(root.join(file.path()), text).unwrap();
    }
    let runfiles = PathBuf::from(std::env::var_os("TEST_SRCDIR").unwrap())
        .join(std::env::var_os("TEST_WORKSPACE").unwrap());
    let output = Command::new("python3")
        .arg(runfiles.join("crates/backend-c/test/characters.py"))
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
