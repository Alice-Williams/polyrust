use std::{env, fs, path::PathBuf};

use portable_backend_c::CBackend;
use portable_codegen::{Backend, BackendOptions, OutputContents};

fn main() {
    let mut arguments = env::args_os().skip(1);
    let input = PathBuf::from(arguments.next().expect("input path"));
    let output = PathBuf::from(arguments.next().expect("output directory"));
    assert!(arguments.next().is_none(), "unexpected arguments");
    let document =
        portable_ir::v0::from_json(&fs::read(input).expect("read input")).expect("parse input");
    let checked = portable_check::v0::check_program(document).expect("check input");
    let manifest = CBackend
        .generate(&checked, &BackendOptions::default())
        .expect("generate C17 package");
    for file in manifest.files() {
        let path = output.join(file.path());
        fs::create_dir_all(path.parent().expect("generated file parent")).expect("create parent");
        match file.contents() {
            OutputContents::Text(text) => fs::write(path, text).expect("write text"),
            OutputContents::Bytes(bytes) => fs::write(path, bytes).expect("write bytes"),
        }
    }
}
