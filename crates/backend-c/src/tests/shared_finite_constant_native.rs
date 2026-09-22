//! Real certified storage, alias facades and selected generated readers.
use super::*;
use std::{
    fs,
    path::{Path, PathBuf},
    process::Command,
};

fn python(runfiles: &Path, arguments: &[&Path]) -> std::process::Output {
    let output = Command::new("python3")
        .arg(runfiles.join("crates/backend-c/test/finite_constants.py"))
        .args(arguments)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    output
}

fn write(root: &Path, bits: &[u64], readers: bool) {
    fs::create_dir_all(root).unwrap();
    let producer = api(&fixture(bits));
    let mut owners = vec![producer];
    if readers {
        let values: Vec<_> = owners[0].constants().cloned().collect();
        let facade = api(&constant_export_fixture::facade(921, &values));
        let mut aliases: Vec<_> = facade
            .foreign_constants()
            .map(|binding| binding.dependency().clone())
            .collect();
        aliases.sort_by_key(|value| value.declaration());
        assert_eq!(values, aliases);
        owners.push(facade);
        owners.push(api(&constant_consumer_fixture::fixture(
            922,
            &aliases,
            None,
            constant_consumer_fixture::Usage::Read,
        )));
    }
    for owner in owners {
        check_resources(&owner);
        super::super::owned_constant_dependency_tests::write_package(owner.package(), root);
    }
}

#[test]
fn finite_constants_native_corpus_faults_headers_and_ubsan() {
    let root = PathBuf::from(std::env::var_os("TEST_TMPDIR").unwrap()).join("finite-constants");
    let runfiles = PathBuf::from(std::env::var_os("TEST_SRCDIR").unwrap())
        .join(std::env::var_os("TEST_WORKSPACE").unwrap());
    let oracle = runfiles.join("experiments/rustc-frontend/test");
    let output = python(&runfiles, &[Path::new("inputs"), &oracle]);
    let values: Vec<_> = String::from_utf8(output.stdout)
        .unwrap()
        .lines()
        .map(|line| u64::from_str_radix(line, 16).unwrap())
        .collect();
    assert_eq!(values.len(), 24_576);
    for (index, batch) in values.chunks(384).enumerate() {
        write(&root.join(index.to_string()), batch, false);
    }
    let controls = [
        0,
        1 << 63,
        1,
        0x0010_0000_0000_0000,
        0x3ff0_0000_0000_0001,
        0x3fb9_9999_9999_999a,
        0x7fef_ffff_ffff_ffff,
        0xffef_ffff_ffff_ffff,
    ];
    write(&root.join("controls"), &controls, true);
    let output = python(
        &runfiles,
        &[&root, &runfiles.join("tools/c/zig_native_oracle"), &oracle],
    );
    eprintln!("{}", String::from_utf8_lossy(&output.stdout));
}
