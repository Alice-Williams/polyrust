use portable_backend_java::JavaBackend;
use portable_build::{
    Expected, Invocation, ModuleBuilder, Operation, Type, TypedValue, Value, Visibility,
};
use portable_check::v0::CheckedProgram;
use portable_codegen::{Backend, BackendOptions, OutputContents};

#[path = "../src/tests/capability_fixtures.rs"]
mod fixtures;

fn main() {
    let output = std::path::PathBuf::from(std::env::args_os().nth(1).expect("output path"));
    for (name, program) in [
        ("operations", fixtures::capability_coverage_fixture()),
        ("methods", fixtures::portable_method_invocation_fixture()),
    ] {
        let manifest = JavaBackend
            .generate(&program, &BackendOptions::default())
            .expect("capability fixture generates");
        for file in manifest.files() {
            let path = output.join(name).join(file.path());
            std::fs::create_dir_all(path.parent().expect("parent")).expect("create parent");
            match file.contents() {
                OutputContents::Text(text) => std::fs::write(path, text),
                OutputContents::Bytes(bytes) => std::fs::write(path, bytes),
            }
            .expect("write generated capability fixture");
        }
    }
}
