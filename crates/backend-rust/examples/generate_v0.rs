#![forbid(unsafe_code)]

use std::path::Path;

use portable_backend_rust::RustBackend;
use portable_codegen::{Backend, BackendOptions, OutputContents};

fn main() {
    let mut arguments = std::env::args().skip(1);
    let input = arguments.next().expect("expected input .poly.json path");
    let output = arguments.next().expect("expected output directory path");
    assert!(arguments.next().is_none(), "unexpected extra argument");
    let document = portable_ir::v0::from_json(
        &std::fs::read(&input).unwrap_or_else(|error| panic!("cannot read {input}: {error}")),
    )
    .unwrap_or_else(|error| panic!("cannot parse {input}: {error}"));
    let checked = portable_check::v0::check_program(document)
        .unwrap_or_else(|diagnostics| panic!("fixture did not check: {diagnostics:?}"));
    let manifest = RustBackend
        .generate(&checked, &BackendOptions::default())
        .unwrap_or_else(|error| panic!("Rust generation failed: {error:?}"));
    for file in manifest.files() {
        let path = Path::new(&output).join(file.path());
        std::fs::create_dir_all(path.parent().expect("generated file has a parent"))
            .unwrap_or_else(|error| panic!("cannot create output parent: {error}"));
        match file.contents() {
            OutputContents::Text(text) => std::fs::write(&path, text),
            OutputContents::Bytes(bytes) => std::fs::write(&path, bytes),
        }
        .unwrap_or_else(|error| panic!("cannot write {}: {error}", path.display()));
    }
}
