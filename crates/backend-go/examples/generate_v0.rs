#![forbid(unsafe_code)]
use portable_backend_go::GoV0Backend;
use portable_codegen::{Backend, BackendOptions, OutputContents};
use std::path::Path;
fn main() {
    let mut arguments = std::env::args().skip(1);
    let input = arguments.next().expect("input");
    let output = arguments.next().expect("output");
    assert!(arguments.next().is_none());
    let document = portable_ir::v0::from_json(&std::fs::read(input).expect("read")).expect("parse");
    let checked = portable_check::v0::check_program(document).expect("check");
    let manifest = GoV0Backend
        .generate(&checked, &BackendOptions::default())
        .expect("generate");
    for file in manifest.files() {
        let path = Path::new(&output).join(file.path());
        std::fs::create_dir_all(path.parent().expect("parent")).expect("mkdir");
        match file.contents() {
            OutputContents::Text(text) => std::fs::write(path, text),
            OutputContents::Bytes(bytes) => std::fs::write(path, bytes),
        }
        .expect("write");
    }
}
