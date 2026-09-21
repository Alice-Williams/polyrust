//! Exact typed fmod call, original producer closure, native and resource evidence.
use super::*;
use crate::dialect::{CKnownCall, CSystemLibrary};
use std::{collections::BTreeSet, fs, path::PathBuf, process::Command};

#[path = "shared_binary64_remainder_fixture.rs"]
mod fixture;

fn chain() -> Vec<CDependencyApi> {
    let leaf = api(&f::fixture(601, &[CScalarType::F64; 2], &[], &[None; 2]));
    let middle = api(&fixture::build(
        602,
        &leaf.functions().cloned().collect::<Vec<_>>(),
        fixture::Body::Remainder,
    ));
    let root = api(&fixture::build(
        603,
        &middle.functions().cloned().collect::<Vec<_>>(),
        fixture::Body::Forward,
    ));
    vec![leaf, middle, root]
}

#[test]
fn remainder_headers_libraries_and_byte_bounds_are_certificate_derived() {
    for (index, owner) in chain().iter().enumerate() {
        let libraries = if index == 0 {
            BTreeSet::new()
        } else {
            BTreeSet::from([CSystemLibrary::Math])
        };
        assert_eq!(owner.system_libraries(), &libraries);
        assert_eq!(
            crate::dialect::c_system_libraries(owner.package()).unwrap(),
            libraries
        );
        for function in owner.functions() {
            assert_eq!(function.package_identity().system_libraries(), &libraries);
        }
        let mut bytes = 0_u64;
        for file in render_certified_package(&CStructuralRenderer, owner.package())
            .unwrap()
            .files()
        {
            let OutputContents::Text(text) = file.contents() else {
                panic!("text")
            };
            assert_eq!(
                text.contains("#include <math.h>"),
                index == 1 && file.path().ends_with(".c")
            );
            assert!(!text.contains("runtime"));
            bytes += text.len() as u64;
        }
        assert!(bytes <= crate::dialect::c_output_byte_bound(owner.package()).unwrap());
    }
}

fn native(script: &str, directory: &str) {
    let path = PathBuf::from(std::env::var_os("TEST_TMPDIR").unwrap()).join(directory);
    fs::create_dir_all(&path).unwrap();
    let mut bounds = String::new();
    for owner in chain() {
        for function in owner.functions() {
            bounds.push_str(&format!(
                "{} {}\n",
                function.symbol().as_str(),
                function.stack_bound_bytes()
            ));
        }
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
    fs::write(path.join("stack_bounds.txt"), bounds).unwrap();
    let runfiles = PathBuf::from(std::env::var_os("TEST_SRCDIR").unwrap())
        .join(std::env::var_os("TEST_WORKSPACE").unwrap());
    let output = Command::new("python3")
        .arg(runfiles.join("crates/backend-c/test").join(script))
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

#[test]
fn remainder_native_values_and_original_operand_traces() {
    native("remainder_native.py", "c-remainder-native");
}
#[test]
fn remainder_native_stack_guard_and_watermark() {
    native("remainder_stack.py", "c-remainder-stack");
}

#[path = "shared_remainder_contracts.rs"]
mod contracts;
#[path = "shared_remainder_resources.rs"]
mod resources;
