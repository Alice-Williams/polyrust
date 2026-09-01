#![forbid(unsafe_code)]

use std::path::{Path, PathBuf};

use portable_codegen::OutputContents;

fn main() {
    let output = std::env::args_os()
        .nth(1)
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("generated/trim-newlines"));
    for (target, manifest) in polyrust_trim_newlines_example::manifests() {
        for file in manifest.files() {
            write_file(&output, target, file.path(), file.contents());
        }
    }
    println!(
        "generated trim-newlines for rust, typescript, javascript, python, go, java, and C++ in {}",
        output.display()
    );
}

fn write_file(root: &Path, target: &str, relative: &str, contents: &OutputContents) {
    let destination = root.join(target).join(relative);
    std::fs::create_dir_all(destination.parent().expect("output file has parent"))
        .unwrap_or_else(|error| panic!("cannot create {}: {error}", destination.display()));
    match contents {
        OutputContents::Text(text) => std::fs::write(&destination, text),
        OutputContents::Bytes(bytes) => std::fs::write(&destination, bytes),
    }
    .unwrap_or_else(|error| panic!("cannot write {}: {error}", destination.display()));
}
