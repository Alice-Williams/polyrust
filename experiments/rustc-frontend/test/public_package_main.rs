#![feature(rustc_private)]
#![forbid(unsafe_code)]
extern crate rustc_abi;
extern crate rustc_ast;
extern crate rustc_driver;
extern crate rustc_hir;
extern crate rustc_interface;
extern crate rustc_middle;
extern crate rustc_span;

#[path = "../src/api_manifest/mod.rs"]
mod api_manifest;
#[path = "../src/c_lower/mod.rs"]
pub mod c_lower;
#[path = "../src/inputs.rs"]
mod inputs;
#[path = "../src/source_admission.rs"]
mod source_admission;
#[path = "../src/source_capabilities/mod.rs"]
mod source_capabilities;
#[path = "../src/source_origin/mod.rs"]
mod source_origin;

use portable_backend_c::dialect::{CDialect, CStructuralRenderer, project_c_package};
use portable_codegen::{
    TargetLinker, certify_resolved_package, render_certified_package, verify_unresolved_package,
};
use rustc_driver::{Callbacks, Compilation};
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
        let mut previous = None;
        let mut previous_expected = None;
        for _ in 0..3 {
            let lowered = c_lower::lower(tcx, c_lower::Selection::PublicApi).unwrap();
            assert_eq!(
                lowered.exports.modules.keys().collect::<Vec<_>>(),
                lowered.exports.module_ancestries.keys().collect::<Vec<_>>()
            );
            let source_ids = tcx
                .hir_body_owners()
                .filter(|id| tcx.def_kind(*id) == rustc_hir::def::DefKind::Fn)
                .map(|id| {
                    let hash = tcx.def_path_hash(id.to_def_id());
                    portable_codegen::RustDeclarationId {
                        crate_id: hash.stable_crate_id().as_u64(),
                        definition_path_hash: hash.local_hash().as_u64(),
                    }
                })
                .collect::<std::collections::BTreeSet<_>>();
            assert_eq!(source_ids, lowered.functions.keys().copied().collect());
            let unresolved = project_c_package(lowered.registry, lowered.sources).unwrap();
            let checked = verify_unresolved_package(&CDialect, unresolved).unwrap();
            let linked = TargetLinker::new(CDialect).link_ast(&checked).unwrap();
            let certified = certify_resolved_package(&CDialect, linked).unwrap();
            let dependency =
                portable_backend_c::dialect::CDependencyApi::from_certificate(certified.clone())
                    .unwrap();
            assert_eq!(dependency.root(), lowered.exports.root);
            let public_ids = lowered
                .exports
                .modules
                .values()
                .flat_map(|bindings| bindings.values())
                .filter_map(|target| match target {
                    portable_codegen::RustExportTarget::Declaration(id) => Some(*id),
                    portable_codegen::RustExportTarget::Module(_) => None,
                })
                .collect::<std::collections::BTreeSet<_>>();
            assert_eq!(
                dependency
                    .functions()
                    .map(|function| function.declaration())
                    .collect::<std::collections::BTreeSet<_>>(),
                public_ids
            );
            for (id, function) in &lowered.functions {
                assert_eq!(dependency.function(*id).is_some(), public_ids.contains(id));
                if let Some(imported) = dependency.function(*id) {
                    assert_eq!(imported.function(), function);
                    assert!(imported.stack_bound_bytes() > 0);
                }
            }
            let manifest = api_manifest::ApiManifest::new(
                &certified,
                lowered.exports.clone(),
                &lowered.functions,
            )
            .unwrap();
            manifest.verify_owner(&dependency).unwrap();
            assert!(manifest.bundle_json().unwrap().len() <= manifest.bundle_bound().unwrap());
            api_manifest::contract::check(
                &certified,
                &manifest,
                &lowered.exports,
                &lowered.functions,
            );
            if let Some(expected) = &previous_expected {
                assert!(
                    api_manifest::ApiManifest::new(&certified, lowered.exports.clone(), expected)
                        .is_err(),
                    "foreign registry handles authenticated"
                );
            }
            previous_expected = Some(lowered.functions);
            let current = (
                manifest.canonical_json().unwrap(),
                manifest.bundle_json().unwrap(),
                render_certified_package(&CStructuralRenderer, &certified).unwrap(),
            );
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
    let mut probe = Probe {
        inputs: inputs::DeclaredInputs::new(&args[2], &args[3..]).unwrap(),
        checked: false,
    };
    let arguments = vec![
        "rustc".into(),
        probe.inputs.root().into(),
        "--sysroot".into(),
        args[1].clone(),
        "--crate-type=lib".into(),
        "--crate-name=poly_public_package_probe".into(),
        "--edition=2024".into(),
        "-Funsafe-code".into(),
        "-Cpanic=abort".into(),
    ];
    let status =
        rustc_driver::catch_with_exit_code(|| rustc_driver::run_compiler(&arguments, &mut probe));
    if status != std::process::ExitCode::SUCCESS {
        return status;
    }
    assert!(probe.checked);
    println!(
        "compiler identity, exact metadata reconstruction, foreign registry and deterministic package proof passed"
    );
    std::process::ExitCode::SUCCESS
}
