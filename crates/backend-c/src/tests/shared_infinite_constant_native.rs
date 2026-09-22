//! Independently compiled producers, alias headers and generated readers.
use super::*;
use std::{path::PathBuf, process::Command};

#[test]
fn infinity_constants_native_faults_headers_and_ubsan() {
    let root = PathBuf::from(std::env::var_os("TEST_TMPDIR").unwrap()).join("infinite-constants");
    std::fs::create_dir_all(&root).unwrap();
    let runfiles = PathBuf::from(std::env::var_os("TEST_SRCDIR").unwrap())
        .join(std::env::var_os("TEST_WORKSPACE").unwrap());
    let producer = api(&fixture(&[Binary64Sign::Positive, Binary64Sign::Negative]));
    let values: Vec<_> = producer.constants().cloned().collect();
    let facade = api(&constant_export_fixture::facade(921, &values));
    let mut aliases: Vec<_> = facade
        .foreign_constants()
        .map(|binding| binding.dependency().clone())
        .collect();
    aliases.sort_by_key(|value| value.declaration());
    assert_eq!(values, aliases);
    let reader = api(&constant_consumer_fixture::fixture(
        922,
        &aliases,
        None,
        constant_consumer_fixture::Usage::Read,
    ));
    for (index, owner) in [producer, facade, reader].iter().enumerate() {
        check_resources(owner, index == 0);
        super::super::owned_constant_dependency_tests::write_package(owner.package(), &root);
    }
    let output = Command::new("python3")
        .arg(runfiles.join("crates/backend-c/test/infinite_constants.py"))
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
