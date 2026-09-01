#![forbid(unsafe_code)]

use portable_backend_cpp::CppBackend;
use portable_codegen::{Backend, BackendOptions, OutputContents};
use std::path::Path;

fn main() {
    let mut arguments = std::env::args().skip(1);
    let input = arguments.next().expect("expected input path");
    let output = arguments.next().expect("expected output path");
    assert!(arguments.next().is_none(), "unexpected argument");
    let document = portable_ir::v0::from_json(&std::fs::read(input).expect("read input"))
        .expect("parse input");
    let checked = portable_check::v0::check_program(document).expect("check input");
    let manifest = CppBackend
        .generate(&checked, &BackendOptions::default())
        .expect("generate C++");
    for file in manifest.files() {
        let path = Path::new(&output).join(file.path());
        std::fs::create_dir_all(path.parent().expect("parent")).expect("create parent");
        match file.contents() {
            OutputContents::Text(text) => std::fs::write(path, text),
            OutputContents::Bytes(bytes) => std::fs::write(path, bytes),
        }
        .expect("write output");
    }
}
