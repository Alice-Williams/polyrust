//! Isolated Java adapter using full rustc analysis and the existing certificate.
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

#[cfg(java_graph)]
mod compiler_dependencies;
mod inputs;
#[cfg(java_graph)]
mod java_graph;
mod java_lower;
#[cfg(java_graph)]
mod metadata_cli;
#[cfg(java_graph)]
mod metadata_dependencies;
#[cfg(java_graph)]
mod metadata_stage;
mod source_admission;
mod source_capabilities;
#[cfg(java_graph)]
mod source_check;
mod source_origin;

#[cfg(java_ast_probe)]
#[path = "../test/java_dependency_assertions.rs"]
mod dependency_assertions;

use portable_backend_java::dialect::{JavaDialect, JavaStructuralRenderer};
use portable_codegen::{
    OutputContents, RenderReadyPackage, TargetLinker, certify_resolved_package,
    render_certified_package, verify_unresolved_package,
};
use portable_rustc_configuration as configuration;
use rustc_driver::{Callbacks, Compilation};
use rustc_interface::interface;
use rustc_middle::ty::TyCtxt;

struct Adapter {
    inputs: inputs::DeclaredInputs,
    public: bool,
    result: Option<Result<RenderReadyPackage<JavaDialect>, String>>,
}

impl Callbacks for Adapter {
    fn after_analysis<'tcx>(&mut self, _: &interface::Compiler, tcx: TyCtxt<'tcx>) -> Compilation {
        tcx.sess.dcx().abort_if_errors();
        self.result = Some(self.inputs.verify(tcx).and_then(|()| {
            let selection = if self.public {
                java_lower::Selection::PublicApi
            } else {
                let roots: Vec<_> = tcx
                    .hir_body_owners()
                    .filter(|id| {
                        tcx.def_kind(*id) == rustc_hir::def::DefKind::Fn
                            && tcx.item_name(id.to_def_id()).as_str() == "score"
                    })
                    .collect();
                let [root] = roots.as_slice() else {
                    return Err("expected exactly one score entry".into());
                };
                java_lower::Selection::Entry(*root)
            };
            let package = java_lower::lower(tcx, selection, None)?;
            let verified = verify_unresolved_package(&JavaDialect, package)
                .map_err(|error| format!("Java verification: {error:?}"))?;
            let linked = TargetLinker::new(JavaDialect)
                .link_ast(&verified)
                .map_err(|error| format!("Java linking: {error:?}"))?;
            let certificate = certify_resolved_package(&JavaDialect, linked)
                .map_err(|error| format!("Java certification: {error:?}"))?;
            #[cfg(java_ast_probe)]
            if self.public {
                dependency_assertions::check(tcx, &certificate);
            }
            Ok(certificate)
        }));
        Compilation::Stop
    }
}

fn main() -> std::process::ExitCode {
    if std::env::var_os("RUSTC_BOOTSTRAP").is_some() {
        eprintln!("RUSTC_BOOTSTRAP is not permitted for input compilation");
        return std::process::ExitCode::from(2);
    }
    let args: Vec<_> = std::env::args().collect();
    match run(&args) {
        Ok(()) => std::process::ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("{error}");
            std::process::ExitCode::from(1)
        }
    }
}

fn run(args: &[String]) -> Result<(), String> {
    #[cfg(java_graph)]
    if args.get(2).is_some_and(|value| value == "--bundle") {
        let destination = args.get(3).ok_or("expected Java bundle destination")?;
        let count = java_graph::publish(&args[1], std::path::Path::new(destination), &args[4..])?;
        println!("published {count} Java crates");
        return Ok(());
    }
    #[cfg(java_graph)]
    if args.get(2).is_some_and(|value| value == "--check-crates") {
        let count = java_graph::check(&args[1], &args[3..])?;
        println!("certified {count} Java crates; no output published");
        return Ok(());
    }
    if args.len() < 4 {
        return Err("expected SYSROOT INPUT OUTPUT [--package] [--input PATH]...".into());
    }
    let output = std::path::Path::new(&args[3]);
    if output.file_name() != Some(std::ffi::OsStr::new("Generated.java")) {
        return Err("Java output filename must be Generated.java".into());
    }
    let configuration = configuration::Configuration::parse(&args[4..])?;
    let mut adapter = Adapter {
        inputs: inputs::DeclaredInputs::new(&args[2], configuration.declared_inputs())?,
        public: configuration.mode() == configuration::Mode::PublicPackage,
        result: None,
    };
    let compiler_args = configuration.compiler_arguments(adapter.inputs.root(), &args[1]);
    let status = rustc_driver::catch_with_exit_code(|| {
        rustc_driver::run_compiler(&compiler_args, &mut adapter)
    });
    if status != std::process::ExitCode::SUCCESS {
        return Err("Rust source analysis failed".into());
    }
    let certificate = adapter
        .result
        .ok_or("compiler analysis callback did not execute")?
        .map_err(|error| format!("unsupported Rust: {error}"))?;
    let rendered = render_certified_package(&JavaStructuralRenderer, &certificate)
        .map_err(|error| format!("Java rendering: {error:?}"))?;
    for _ in 0..2 {
        let again = render_certified_package(&JavaStructuralRenderer, &certificate)
            .map_err(|error| format!("Java rendering: {error:?}"))?;
        if again != rendered {
            return Err("nondeterministic Java rendering".into());
        }
    }
    let [file] = rendered.files() else {
        return Err("expected one Java compilation unit".into());
    };
    let OutputContents::Text(text) = file.contents() else {
        return Err("expected textual Java output".into());
    };
    std::fs::write(output, text).map_err(|error| error.to_string())
}
