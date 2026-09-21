//! Typed remainder, authenticated dependencies and exact native equivalence.
use super::*;
use std::{fs, path::PathBuf, process::Command};

#[path = "binary64_remainder_fixture.rs"]
mod fixture;

#[test]
fn exact_remainder_original_imports_and_source_bounds() {
    let owners = fixture::chain();
    for owner in &owners {
        let output = render_certified_package(&JavaStructuralRenderer, owner.package()).unwrap();
        let mut bytes = 0_u64;
        for file in output.files() {
            let OutputContents::Text(text) = file.contents() else {
                panic!("text")
            };
            bytes += text.len() as u64;
            assert!(
                !text.contains("Runtime") && !text.contains("Math.") && !text.contains("strictfp")
            );
        }
        assert!(bytes <= owner.source_byte_bound().unwrap());
    }
    assert_eq!(owners[1].functions().count(), 1);
    assert_eq!(owners[2].functions().count(), 1);
}

#[test]
fn native_remainder_values_and_evaluation_faults() {
    let root = PathBuf::from(std::env::var_os("TEST_TMPDIR").unwrap()).join("java-remainder");
    for (index, owner) in fixture::chain().iter().enumerate() {
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
        .arg(runfiles.join("crates/backend-java/test/binary64_remainder.py"))
        .arg(&root)
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

#[path = "binary64_remainder_contracts.rs"]
mod contracts;
