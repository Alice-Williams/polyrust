#![feature(rustc_private)]
#![forbid(unsafe_code)]

extern crate rustc_abi;
extern crate rustc_ast;
extern crate rustc_driver;
extern crate rustc_hir;
extern crate rustc_interface;
extern crate rustc_middle;
extern crate rustc_span;

#[path = "../src/c_lower/mod.rs"]
pub mod c_lower;
mod export_graph_assertions;
mod export_graph_mutations;
#[path = "../src/inputs.rs"]
mod inputs;
#[path = "../src/source_admission.rs"]
mod source_admission;
#[path = "../src/source_capabilities/mod.rs"]
mod source_capabilities;
#[path = "../src/source_origin/mod.rs"]
mod source_origin;

use rustc_driver::{Callbacks, Compilation};
use rustc_hir::def::DefKind;
use rustc_interface::interface;
use rustc_middle::ty::TyCtxt;

struct Probe {
    inputs: inputs::DeclaredInputs,
    checked: bool,
}

impl Callbacks for Probe {
    fn after_analysis<'tcx>(&mut self, _: &interface::Compiler, tcx: TyCtxt<'tcx>) -> Compilation {
        tcx.sess.dcx().abort_if_errors();
        self.inputs.verify(tcx).unwrap();
        let roots: Vec<_> = tcx
            .hir_body_owners()
            .filter(|id| {
                tcx.def_kind(*id) == DefKind::Fn
                    && tcx.item_name(id.to_def_id()).as_str() == "score"
            })
            .collect();
        assert_eq!(roots.len(), 1);
        let mut previous = None;
        for _ in 0..3 {
            let (registry, source) = c_lower::check(tcx, roots[0]).unwrap();
            let graph = export_graph_assertions::check(&source);
            use portable_backend_c::dialect::{CDialect, CStructuralRenderer, project_c_package};
            use portable_codegen::{
                TargetLinker, certify_resolved_package, render_certified_package,
                verify_unresolved_package,
            };
            let package = project_c_package(registry, vec![source]).unwrap();
            export_graph_mutations::check(&package);
            let verified = verify_unresolved_package(&CDialect, package).unwrap();
            let linked = TargetLinker::new(CDialect).link_ast(&verified).unwrap();
            let certified = certify_resolved_package(&CDialect, linked).unwrap();
            let output = render_certified_package(&CStructuralRenderer, &certified).unwrap();
            let current = (graph, output);
            if let Some(previous) = &previous {
                assert_eq!(previous, &current);
            }
            previous = Some(current);
        }
        self.checked = true;
        Compilation::Stop
    }
}

fn main() -> std::process::ExitCode {
    assert!(std::env::var_os("RUSTC_BOOTSTRAP").is_none());
    let args: Vec<_> = std::env::args().collect();
    assert!(args.len() >= 3);
    let mut probe = Probe {
        inputs: inputs::DeclaredInputs::new(&args[2], &args[3..]).unwrap(),
        checked: false,
    };
    let compiler_args = vec![
        "rustc".into(),
        probe.inputs.root().into(),
        "--sysroot".into(),
        args[1].clone(),
        "--crate-type=lib".into(),
        "--crate-name=poly_export_graph_probe".into(),
        "--edition=2024".into(),
        "-Funsafe-code".into(),
        "-Cpanic=abort".into(),
    ];
    let status = rustc_driver::catch_with_exit_code(|| {
        rustc_driver::run_compiler(&compiler_args, &mut probe)
    });
    if status != std::process::ExitCode::SUCCESS {
        return status;
    }
    assert!(probe.checked);
    println!("resolved export graph, alias cycles, namespaces and visibility passed");
    std::process::ExitCode::SUCCESS
}
