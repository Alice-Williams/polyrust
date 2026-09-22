//! Real certified fields/readers; no hand-written producer implementation.
use super::*;
use std::{
    fs,
    path::{Path, PathBuf},
    process::Command,
};

fn python(runfiles: &Path, arguments: &[&Path]) -> std::process::Output {
    let output = Command::new("python3")
        .arg(runfiles.join("crates/backend-java/test/character_constants.py"))
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

fn write(root: &Path, values: &[i32]) {
    for api in packages(values) {
        text(&api);
        let output = render_certified_package(&JavaStructuralRenderer, api.package()).unwrap();
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
fn character_constants_native_corpus_aliases_and_inlining_sensitive_faults() {
    let root =
        PathBuf::from(std::env::var_os("TEST_TMPDIR").unwrap()).join("java-character-constants");
    let runfiles = PathBuf::from(std::env::var_os("TEST_SRCDIR").unwrap())
        .join(std::env::var_os("TEST_WORKSPACE").unwrap());
    let oracle = runfiles.join("experiments/rustc-frontend/test");
    let output = python(&runfiles, &[Path::new("inputs"), &oracle]);
    let values: Vec<i32> = String::from_utf8(output.stdout)
        .unwrap()
        .lines()
        .map(|line| line.parse().unwrap())
        .collect();
    assert_eq!(values.len(), 4_133);
    for (index, batch) in values.chunks(64).enumerate() {
        write(&root.join(index.to_string()), batch);
    }
    let output = python(&runfiles, &[Path::new("controls"), &oracle]);
    let controls: Vec<i32> = String::from_utf8(output.stdout)
        .unwrap()
        .lines()
        .map(|line| line.parse().unwrap())
        .collect();
    assert_eq!(controls.len(), 19);
    write(&root.join("controls"), &controls);
    let output = python(
        &runfiles,
        &[
            &root,
            &super::super::source_constants_native::tool("javac"),
            &super::super::source_constants_native::tool("java"),
            &oracle,
        ],
    );
    eprintln!("{}", String::from_utf8_lossy(&output.stdout));
}
