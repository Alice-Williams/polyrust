//! Certified ordinary fields and readers, consumed by a separate Java21 client.
use super::*;
use std::{
    fs,
    path::{Path, PathBuf},
    process::Command,
};

fn python(runfiles: &Path, arguments: &[&Path]) -> std::process::Output {
    let output = Command::new("python3")
        .arg(runfiles.join("crates/backend-java/test/finite_constants.py"))
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
    let owner = api(bits, readers);
    let mut packages = vec![owner];
    if readers {
        packages.push(c::admit(consumer(&packages[0]).0).unwrap());
    }
    for package in packages {
        check_bounds(&package);
        let output = render_certified_package(&JavaStructuralRenderer, package.package()).unwrap();
        for file in output.files() {
            let path = root.join(file.path());
            fs::create_dir_all(path.parent().unwrap()).unwrap();
            let OutputContents::Text(text) = file.contents() else {
                panic!("source")
            };
            fs::write(path, text).unwrap();
        }
    }
}

#[test]
fn finite_constants_native_corpus_and_inlining_sensitive_faults() {
    let root =
        PathBuf::from(std::env::var_os("TEST_TMPDIR").unwrap()).join("java-finite-constants");
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
    for (index, batch) in values.chunks(256).enumerate() {
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
        &[
            &root,
            &crate::tests::source_constants_native::tool("javac"),
            &crate::tests::source_constants_native::tool("java"),
            &oracle,
        ],
    );
    eprintln!("{}", String::from_utf8_lossy(&output.stdout));
}
