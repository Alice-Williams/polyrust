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
mod mapping_assertions;
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
    checked: bool,
}

impl Callbacks for Probe {
    fn after_analysis<'tcx>(&mut self, _: &interface::Compiler, tcx: TyCtxt<'tcx>) -> Compilation {
        tcx.sess.dcx().abort_if_errors();
        let roots: Vec<_> = tcx
            .hir_body_owners()
            .filter(|id| {
                tcx.def_kind(*id) == DefKind::Fn
                    && tcx.item_name(id.to_def_id()).as_str() == "score"
            })
            .collect();
        assert_eq!(roots.len(), 1);
        let (registry, source) = c_lower::check(tcx, roots[0]).expect("typed C mapping");
        mapping_assertions::check(&source);

        let package = portable_backend_c::dialect::project_c_package(registry, vec![source])
            .expect("compiler C tree shared projection");
        let verified = portable_codegen::verify_unresolved_package(
            &portable_backend_c::dialect::CDialect,
            package,
        )
        .expect("compiler C tree shared verification");
        let linked = portable_codegen::TargetLinker::new(portable_backend_c::dialect::CDialect)
            .link_ast(&verified)
            .expect("compiler C tree shared linking");
        portable_codegen::verify_linked_package(&linked).expect("compiler C tree post-link proof");
        let certified = portable_codegen::certify_resolved_package(
            &portable_backend_c::dialect::CDialect,
            linked,
        )
        .expect("compiler C tree resource certification");
        let rendered = portable_codegen::render_certified_package(
            &portable_backend_c::dialect::CStructuralRenderer,
            &certified,
        )
        .expect("compiler C tree certified output");
        assert_eq!(rendered.files().len(), 1);
        self.checked = true;
        Compilation::Stop
    }
}

fn main() -> std::process::ExitCode {
    assert!(std::env::var_os("RUSTC_BOOTSTRAP").is_none());
    let args: Vec<_> = std::env::args().collect();
    assert_eq!(args.len(), 3, "expected SYSROOT INPUT");
    let mut probe = Probe { checked: false };
    let compiler_args = vec![
        "rustc".into(),
        args[2].clone(),
        "--sysroot".into(),
        args[1].clone(),
        "--crate-type=lib".into(),
        "--crate-name=poly_provenance_probe".into(),
        "--edition=2024".into(),
        "-Funsafe-code".into(),
        "-Copt-level=0".into(),
        "-Cpanic=abort".into(),
    ];
    let status = rustc_driver::catch_with_exit_code(|| {
        rustc_driver::run_compiler(&compiler_args, &mut probe)
    });
    if status != std::process::ExitCode::SUCCESS {
        return status;
    }
    assert!(probe.checked, "compiler-backed assertions did not execute");
    println!("all eight source capability AST assertions passed");
    std::process::ExitCode::SUCCESS
}
