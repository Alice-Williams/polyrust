use std::path::PathBuf;

use portable_backend_java::JavaBackend;
use portable_codegen::{Backend, BackendOptions, OutputContents};

fn main() {
    let mut arguments = std::env::args_os().skip(1);
    let input = PathBuf::from(arguments.next().expect("input path"));
    let output = PathBuf::from(arguments.next().expect("output path"));
    assert!(arguments.next().is_none(), "unexpected argument");
    let document = portable_ir::v0::from_json(&std::fs::read(input).expect("read fixture"))
        .expect("parse fixture");
    let checked = portable_check::v0::check_program(document).expect("check fixture");
    let manifest = JavaBackend
        .generate(&checked, &BackendOptions::default())
        .expect("generate fixture");
    for file in manifest.files() {
        let path = output.join(file.path());
        std::fs::create_dir_all(path.parent().expect("parent")).expect("create parent");
        match file.contents() {
            OutputContents::Text(text) => std::fs::write(path, text),
            OutputContents::Bytes(bytes) => std::fs::write(path, bytes),
        }
        .expect("write output");
    }
}
