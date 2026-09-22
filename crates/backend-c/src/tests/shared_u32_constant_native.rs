//! Certified U32 objects, original alias facades and generated import readers.
use super::*;
use std::{
    fs,
    path::{Path, PathBuf},
    process::Command,
};

fn python(runfiles: &Path, arguments: &[&Path]) -> std::process::Output {
    let output = Command::new("python3")
        .arg(runfiles.join("crates/backend-c/test/u32_constants.py"))
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

fn write(root: &Path, values: &[u32]) {
    fs::create_dir_all(root).unwrap();
    let producer = api(&fixture(values));
    let constants: Vec<_> = producer.constants().cloned().collect();
    let facade = api(&constant_export_fixture::facade(921, &constants));
    let mut aliases: Vec<_> = facade
        .foreign_constants()
        .map(|binding| binding.dependency().clone())
        .collect();
    aliases.sort_by_key(|value| value.declaration());
    assert_eq!(constants, aliases);
    let reader = api(&constant_consumer_fixture::fixture(
        922,
        &aliases,
        None,
        constant_consumer_fixture::Usage::Read,
    ));
    for owner in [producer, facade, reader] {
        check_resources(&owner);
        super::super::owned_constant_dependency_tests::write_package(owner.package(), root);
    }
}

#[test]
fn u32_constants_native_corpus_imports_faults_headers_and_ubsan() {
    let root = PathBuf::from(std::env::var_os("TEST_TMPDIR").unwrap()).join("u32-constants");
    let runfiles = PathBuf::from(std::env::var_os("TEST_SRCDIR").unwrap())
        .join(std::env::var_os("TEST_WORKSPACE").unwrap());
    let oracle = runfiles.join("experiments/rustc-frontend/test");
    let output = python(&runfiles, &[Path::new("inputs"), &oracle]);
    let values: Vec<_> = String::from_utf8(output.stdout)
        .unwrap()
        .lines()
        .map(|line| u32::from_str_radix(line, 16).unwrap())
        .collect();
    assert_eq!(values.len(), 4133);
    // Keep certification graphs small without reducing values or imported readers.
    for (index, batch) in values.chunks(64).enumerate() {
        write(&root.join(index.to_string()), batch);
    }
    // Includes scalar-domain boundaries and noncharacters, never just ASCII.
    let controls = [
        0, 1, 0x7f, 0x80, 0xff, 0x100, 0x378, 0x7ff, 0x800, 0xd7ff, 0xe000, 0xfdd0, 0xfffe, 0xffff,
        0x10000, 0x1f980, 0xf0000, 0x10fffe, 0x10ffff,
    ];
    write(&root.join("controls"), &controls);
    let output = python(
        &runfiles,
        &[&root, &runfiles.join("tools/c/zig_native_oracle"), &oracle],
    );
    eprintln!("{}", String::from_utf8_lossy(&output.stdout));
}
