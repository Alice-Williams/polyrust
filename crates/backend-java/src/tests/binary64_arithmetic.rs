//! Four exact Double operators, original dependencies and native equivalence.
use super::*;
use std::{fs, path::PathBuf, process::Command};

pub(super) const OPERATORS: [JavaBinaryOperator; 4] = [
    JavaBinaryOperator::Add,
    JavaBinaryOperator::Subtract,
    JavaBinaryOperator::Multiply,
    JavaBinaryOperator::Divide,
];
#[path = "binary64_arithmetic_fixture.rs"]
mod fixture;

#[test]
fn exact_arithmetic_original_imports_and_source_bounds() {
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
    assert_eq!(owners[1].functions().count(), 6);
    assert_eq!(owners[2].functions().count(), 6);
}

#[test]
fn native_arithmetic_values_grouping_and_evaluation_faults() {
    let root = PathBuf::from(std::env::var_os("TEST_TMPDIR").unwrap()).join("java-arithmetic");
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
        .arg(runfiles.join("crates/backend-java/test/binary64_arithmetic.py"))
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

#[path = "binary64_arithmetic_contracts.rs"]
mod contracts;
