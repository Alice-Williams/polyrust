#![feature(rustc_private)]
#![forbid(unsafe_code)]
//! Isolated metadata build: no backend dependency and no target certificates.
extern crate rustc_driver;
extern crate rustc_interface;
extern crate rustc_middle;
extern crate rustc_session;
extern crate rustc_span;

mod compiler_dependencies;
mod inputs;
mod metadata_cli;
mod metadata_dependencies;
mod metadata_output;
mod metadata_stage;

use portable_rustc_configuration::graph::ResolvedInputs;
use rustc_driver::{Callbacks, Compilation};
use rustc_interface::interface;
use rustc_middle::ty::TyCtxt;

struct Analysis {
    inputs: inputs::DeclaredInputs,
    result: Option<Result<(), String>>,
}

impl Callbacks for Analysis {
    fn config(&mut self, config: &mut interface::Config) {
        compiler_dependencies::configure(config);
    }

    fn after_analysis<'tcx>(&mut self, _: &interface::Compiler, tcx: TyCtxt<'tcx>) -> Compilation {
        tcx.sess.dcx().abort_if_errors();
        self.result = Some(self.inputs.verify(tcx));
        if self.result.as_ref().is_some_and(Result::is_ok) {
            Compilation::Continue
        } else {
            Compilation::Stop
        }
    }
}

fn build(arguments: &[String]) -> Result<(), String> {
    let sysroot = arguments
        .get(1)
        .ok_or("metadata emitter requires its pinned sysroot")?;
    let graph = metadata_cli::parse(&arguments[2..])?;
    let description = graph
        .crates()
        .iter()
        .find(|item| item.key() == graph.root_key())
        .ok_or("metadata graph root missing")?;
    let resolved = ResolvedInputs::load(description)?;
    let inputs =
        inputs::DeclaredInputs::new(resolved.root(), &resolved.declared_input_arguments())?;
    if inputs.root() != resolved.root() {
        return Err("metadata root changed during input resolution".into());
    }
    let output =
        metadata_output::MetadataOutput::prepare(std::path::Path::new(description.metadata()))?;
    let staged = output
        .staged_path()
        .to_str()
        .ok_or("metadata staging path is not UTF-8")?;
    let dependencies = metadata_dependencies::MetadataDependencies::prepare(
        &graph,
        &resolved,
        output
            .staged_path()
            .parent()
            .ok_or("metadata stage has no parent")?,
    )?;
    let mut compiler_arguments = resolved.metadata_arguments(sysroot, staged)?;
    compiler_arguments.extend(dependencies.compiler_arguments(&graph)?);
    let mut analysis = Analysis {
        inputs,
        result: None,
    };
    let status = rustc_driver::catch_with_exit_code(|| {
        rustc_driver::run_compiler(&compiler_arguments, &mut analysis);
    });
    if status != std::process::ExitCode::SUCCESS {
        return Err("compiler rejected metadata input".into());
    }
    analysis
        .result
        .ok_or("metadata compiler analysis did not run")??;
    // Clean only our exact input snapshots before publishing/removing the stage.
    drop(dependencies);
    output.publish()
}

fn main() -> std::process::ExitCode {
    if std::env::var_os("RUSTC_BOOTSTRAP").is_some() {
        eprintln!("input compiler exemption is not permitted");
        return std::process::ExitCode::from(2);
    }
    match build(&std::env::args().collect::<Vec<_>>()) {
        Ok(()) => std::process::ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("metadata build: {error}");
            std::process::ExitCode::from(2)
        }
    }
}
