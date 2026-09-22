//! Multiplication reuses integer fixture ownership, never addition's expected values.
use super::wrapping_integer::*;
use std::{fs, path::PathBuf, process::Command};

#[path = "wrapping_multiplication_contracts.rs"]
mod contracts;

fn multiply(width: JavaPrimitive, left: JavaExpr, right: JavaExpr) -> JavaExpr {
    fixture::binary(JavaBinaryOperator::Multiply, width, left, right)
}

fn chain() -> Vec<JavaDependencyApi> {
    fixture::chain_with_operator(JavaBinaryOperator::Multiply)
}

#[test]
fn wrapping_multiplication_original_dependencies_and_source_bounds() {
    for (index, owner) in chain().iter().enumerate() {
        assert_eq!(owner.functions().count(), 2);
        for function in owner.functions() {
            assert_eq!(function.call_height(), index + 1);
        }
        let output = render_certified_package(&JavaStructuralRenderer, owner.package()).unwrap();
        assert_eq!(output.files().len(), 1);
        let mut bytes = 0;
        for file in output.files() {
            let OutputContents::Text(text) = file.contents() else {
                panic!("text")
            };
            bytes += text.len() as u64;
            assert!(!text.contains("Runtime") && !text.contains("Math."));
            if index == 1 {
                assert!(text.contains("r0000000000000321"));
            }
        }
        assert!(bytes <= owner.source_byte_bound().unwrap());
    }
}

#[test]
fn wrapping_multiplication_native_values_and_faults() {
    let root = PathBuf::from(std::env::var_os("TEST_TMPDIR").unwrap()).join("java-wrapping-mul");
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
        .arg(runfiles.join("crates/backend-java/test/wrapping_addition.py"))
        .arg(root)
        .arg(crate::tests::source_constants_native::tool("javac"))
        .arg(crate::tests::source_constants_native::tool("java"))
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
