#![forbid(unsafe_code)]

use std::{sync::Arc, time::Instant};

use portable_backend_go::GoV0Backend;
use portable_backend_python::PythonBackend;
use portable_backend_rust::RustBackend;
use portable_backend_typescript::TypeScriptBackend;
use portable_build::{ModuleBuilder, Type, Visibility};
use portable_codegen::{Backend, BackendOptions};

const DECLARATIONS: usize = 1_000;

fn main() {
    let started = Instant::now();
    let mut module = ModuleBuilder::new("benchmark_1000_declarations");
    for index in 0..DECLARATIONS {
        module.alias(
            format!("Identifier{index:04}"),
            Visibility::Public,
            vec![],
            Type::i64(),
        );
    }
    let program = module.finish().expect("benchmark fixture checks");
    let backends: [Arc<dyn Backend>; 4] = [
        Arc::new(RustBackend),
        Arc::new(TypeScriptBackend),
        Arc::new(PythonBackend),
        Arc::new(GoV0Backend),
    ];
    let mut files = 0;
    let mut bytes = 0;
    for backend in backends {
        let manifest = backend
            .generate(&program, &BackendOptions::default())
            .expect("benchmark backend generates");
        files += manifest.files().len();
        bytes += manifest
            .files()
            .iter()
            .map(|file| match file.contents() {
                portable_codegen::OutputContents::Text(text) => text.len(),
                portable_codegen::OutputContents::Bytes(contents) => contents.len(),
            })
            .sum::<usize>();
    }
    println!(
        "{{\"schema\":\"polyrust.benchmark.v0\",\"tool\":\"polyrust-v0.1\",\"declarations\":{DECLARATIONS},\"targets\":4,\"files\":{files},\"bytes\":{bytes},\"elapsed_ms\":{}}}",
        started.elapsed().as_millis()
    );
}
