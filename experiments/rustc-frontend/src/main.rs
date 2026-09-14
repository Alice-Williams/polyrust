#![feature(rustc_private)]
#![forbid(unsafe_code)]

extern crate rustc_abi;
extern crate rustc_ast;
extern crate rustc_data_structures;
extern crate rustc_driver;
extern crate rustc_hir;
extern crate rustc_interface;
extern crate rustc_metadata;
extern crate rustc_middle;
extern crate rustc_session;
extern crate rustc_span;

mod api_manifest;
mod c_graph;
pub mod c_lower;
mod compiler_dependencies;
#[cfg(any(
    c_graph_wrong_owner,
    c_graph_wrong_declaration,
    c_graph_wrong_signature
))]
#[path = "../test/c_foreign_mutations.rs"]
mod foreign_mutations;
mod metadata_cli;
mod metadata_dependencies;
mod metadata_stage;
mod source_admission;
mod source_capabilities;
mod source_check;
mod source_origin;
use portable_rustc_configuration as configuration;
mod extract;
mod inputs;
mod output;

use rustc_driver::{Callbacks, Compilation};
use rustc_interface::interface;
use rustc_middle::ty::TyCtxt;
use std::path::PathBuf;

struct Adapter {
    inputs: inputs::DeclaredInputs,
    mode: configuration::Mode,
    result: Option<Result<extract::Program, String>>,
}

impl Callbacks for Adapter {
    fn after_analysis<'tcx>(
        &mut self,
        _compiler: &interface::Compiler,
        tcx: TyCtxt<'tcx>,
    ) -> Compilation {
        tcx.sess.dcx().abort_if_errors();
        self.result = Some(
            self.inputs
                .verify(tcx)
                .and_then(|()| extract::program(tcx, self.mode)),
        );
        Compilation::Stop
    }
}

fn main() -> std::process::ExitCode {
    // The supported launcher clears this variable. Direct binary callers must
    // not inherit the adapter build's unstable-compiler exemption either.
    if std::env::var_os("RUSTC_BOOTSTRAP").is_some() {
        eprintln!("RUSTC_BOOTSTRAP is not permitted for input compilation; use the launcher");
        return std::process::ExitCode::from(2);
    }
    let args: Vec<String> = std::env::args().collect();
    if args.get(2).is_some_and(|argument| argument == "--bundle") {
        let result = args
            .get(3)
            .ok_or_else(|| "bundle output path is required".to_owned())
            .and_then(|path| {
                c_graph::lower(&args[1], &args[4..])
                    .and_then(|graph| output::publish_bundle(std::path::Path::new(path), &graph))
            });
        return match result {
            Ok(()) => std::process::ExitCode::SUCCESS,
            Err(error) => {
                eprintln!("C crate bundle: {error}");
                std::process::ExitCode::from(2)
            }
        };
    }
    if args
        .get(2)
        .is_some_and(|argument| argument == "--check-crates")
    {
        return match c_graph::check(&args[1], &args[3..]) {
            Ok(count) => {
                println!("certified {count} C crates; no output published");
                std::process::ExitCode::SUCCESS
            }
            Err(error) => {
                eprintln!("C crate graph: {error}");
                std::process::ExitCode::from(2)
            }
        };
    }
    if args.len() < 4 {
        eprintln!(
            "usage: rustc_frontend SYSROOT INPUT.rs OUTPUT [--package [--crate-name NAME --crate-key KEY]] [--input PATH]..."
        );
        std::process::exit(2);
    }
    let configuration = match configuration::Configuration::parse(&args[4..]) {
        Ok(configuration) => configuration,
        Err(error) => {
            eprintln!("compiler configuration: {error}");
            return std::process::ExitCode::from(2);
        }
    };
    let inputs = match inputs::DeclaredInputs::new(&args[2], configuration.declared_inputs()) {
        Ok(inputs) => inputs,
        Err(error) => {
            eprintln!("compiler inputs: {error}");
            return std::process::ExitCode::from(2);
        }
    };
    let mut adapter = Adapter {
        result: None,
        inputs,
        mode: configuration.mode(),
    };
    let compiler_args = configuration.compiler_arguments(adapter.inputs.root(), &args[1]);
    let status = rustc_driver::catch_with_exit_code(|| {
        rustc_driver::run_compiler(&compiler_args, &mut adapter);
    });
    if status != std::process::ExitCode::SUCCESS {
        return status;
    }
    let program = match adapter.result {
        Some(Ok(program)) => program,
        Some(Err(error)) => {
            eprintln!("unsupported Rust: {error}");
            std::process::exit(2);
        }
        None => {
            eprintln!("compiler analysis did not produce an admitted program");
            std::process::exit(2);
        }
    };
    let output = PathBuf::from(&args[3]);
    // Analysis, admission, certification and rendering precede opening output.
    if let Err(error) = output::publish(&output, &program) {
        eprintln!("cannot write {}: {error}", output.display());
        std::process::exit(2);
    }
    std::process::ExitCode::SUCCESS
}
