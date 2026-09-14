//! No-replace publication contracts, independently cached from rustc internals.
#[path = "../src/metadata_output.rs"]
mod metadata_output;
#[path = "../src/metadata_stage.rs"]
mod metadata_stage;
use metadata_output::MetadataOutput;
use std::{
    fs,
    path::PathBuf,
    sync::atomic::{AtomicUsize, Ordering},
};

static NEXT: AtomicUsize = AtomicUsize::new(0);

fn directory() -> PathBuf {
    let path = PathBuf::from(std::env::var_os("TEST_TMPDIR").expect("Bazel TEST_TMPDIR")).join(
        format!("metadata-output-{}", NEXT.fetch_add(1, Ordering::Relaxed)),
    );
    fs::create_dir(&path).unwrap();
    path
}

#[test]
fn successful_publication_exposes_only_the_complete_file() {
    let work = directory();
    let destination = work.join("liboutput.rmeta");
    let output = MetadataOutput::prepare(&destination).unwrap();
    assert!(!destination.exists());
    fs::write(output.staged_path(), b"compiler output fixture").unwrap();
    output.publish().unwrap();
    assert_eq!(fs::read(destination).unwrap(), b"compiler output fixture");
    assert_eq!(fs::read_dir(work).unwrap().count(), 1);
}

#[test]
fn existing_file_directory_and_publish_race_are_never_replaced() {
    let work = directory();
    let file = work.join("libexisting.rmeta");
    fs::write(&file, b"preserve").unwrap();
    assert!(MetadataOutput::prepare(&file).is_err());
    assert!(MetadataOutput::prepare(&work).is_err());
    let raced = work.join("libraced.rmeta");
    let output = MetadataOutput::prepare(&raced).unwrap();
    fs::write(output.staged_path(), b"new").unwrap();
    fs::write(&raced, b"racing owner").unwrap();
    assert!(output.publish().is_err());
    assert_eq!(fs::read(raced).unwrap(), b"racing owner");
    assert_eq!(fs::read(file).unwrap(), b"preserve");
    assert_eq!(fs::read_dir(work).unwrap().count(), 2);
}

#[test]
fn missing_or_empty_compiler_output_never_publishes() {
    let work = directory();
    for empty in [false, true] {
        let destination = work.join("liboutput.rmeta");
        let output = MetadataOutput::prepare(&destination).unwrap();
        if empty {
            fs::write(output.staged_path(), b"").unwrap();
        }
        assert!(output.publish().is_err());
        assert!(!destination.exists());
        assert_eq!(fs::read_dir(&work).unwrap().count(), 0);
    }
}

#[cfg(unix)]
#[test]
fn existing_output_symlinks_and_staged_symlinks_reject() {
    let work = directory();
    let destination = work.join("liboutput.rmeta");
    let missing = work.join("missing");
    std::os::unix::fs::symlink(&missing, &destination).unwrap();
    assert!(MetadataOutput::prepare(&destination).is_err());
    assert!(
        destination
            .symlink_metadata()
            .unwrap()
            .file_type()
            .is_symlink()
    );
    let target = work.join("input");
    fs::write(&target, b"keep").unwrap();
    let output = MetadataOutput::prepare(&work.join("libother.rmeta")).unwrap();
    std::os::unix::fs::symlink(&target, output.staged_path()).unwrap();
    assert!(output.publish().is_err());
    assert_eq!(fs::read(target).unwrap(), b"keep");
}

#[test]
fn cleanup_does_not_touch_preexisting_staging_names() {
    let work = directory();
    let existing = work.join(format!(".polyrust-metadata-{}-0", std::process::id()));
    fs::create_dir(&existing).unwrap();
    fs::write(existing.join("sentinel"), b"keep").unwrap();
    drop(MetadataOutput::prepare(&work.join("liboutput.rmeta")).unwrap());
    assert_eq!(fs::read(existing.join("sentinel")).unwrap(), b"keep");
    assert_eq!(fs::read_dir(work).unwrap().count(), 1);
}
