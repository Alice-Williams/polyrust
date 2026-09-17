//! Closed binary64 operators, recursive authority, native values and evaluation.
use super::*;
use crate::dialect::CSystemLibrary;
use std::{fs, path::PathBuf, process::Command};

pub(super) const OPERATORS: [CBinaryOperator; 4] = [
    CBinaryOperator::Add,
    CBinaryOperator::Subtract,
    CBinaryOperator::Multiply,
    CBinaryOperator::Divide,
];
#[path = "shared_binary64_arithmetic_fixture.rs"]
mod fixture;

#[test]
fn exact_binary64_arithmetic_and_original_imports_are_certified() {
    let leaf = api(&f::fixture(501, &[CScalarType::F64; 2], &[], &[None; 2]));
    let imports: Vec<_> = leaf.functions().cloned().collect();
    let middle = api(&fixture::build(502, &imports, fixture::Body::Arithmetic));
    let root = api(&fixture::build(
        503,
        &middle.functions().cloned().collect::<Vec<_>>(),
        fixture::Body::Forward,
    ));
    for owner in [&leaf, &middle, &root] {
        assert!(owner.system_libraries().is_empty());
        assert!(!owner.system_libraries().contains(&CSystemLibrary::Math));
        let rendered = render_certified_package(&CStructuralRenderer, owner.package()).unwrap();
        let mut bytes = 0_u64;
        for file in rendered.files() {
            let OutputContents::Text(text) = file.contents() else {
                panic!("text")
            };
            assert!(!text.contains("math.h") && !text.contains("runtime"));
            bytes += text.len() as u64;
        }
        assert!(bytes <= crate::dialect::c_output_byte_bound(owner.package()).unwrap());
    }
    assert_eq!(middle.functions().count(), 6);
    assert_eq!(root.functions().count(), 6);
}

#[test]
fn arithmetic_native_values_grouping_and_evaluation_faults() {
    let path = PathBuf::from(std::env::var_os("TEST_TMPDIR").unwrap()).join("c-arithmetic");
    fs::create_dir_all(&path).unwrap();
    let leaf = api(&f::fixture(501, &[CScalarType::F64; 2], &[], &[None; 2]));
    let middle = api(&fixture::build(
        502,
        &leaf.functions().cloned().collect::<Vec<_>>(),
        fixture::Body::Arithmetic,
    ));
    let root = api(&fixture::build(
        503,
        &middle.functions().cloned().collect::<Vec<_>>(),
        fixture::Body::Forward,
    ));
    for owner in [&leaf, &middle, &root] {
        for file in render_certified_package(&CStructuralRenderer, owner.package())
            .unwrap()
            .files()
        {
            let OutputContents::Text(text) = file.contents() else {
                panic!("text")
            };
            fs::write(path.join(file.path()), text).unwrap();
        }
    }
    let runfiles = PathBuf::from(std::env::var_os("TEST_SRCDIR").unwrap())
        .join(std::env::var_os("TEST_WORKSPACE").unwrap());
    let output = Command::new("python3")
        .arg(runfiles.join("crates/backend-c/test/binary64_arithmetic.py"))
        .arg(&path)
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

#[path = "shared_binary64_arithmetic_contracts.rs"]
mod contracts;
