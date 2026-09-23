//! Compiler-only export classification; no concrete backend or renderer dependency.
#![feature(rustc_private)]
#![forbid(unsafe_code)]
extern crate rustc_driver;
extern crate rustc_hir;
extern crate rustc_interface;
extern crate rustc_middle;
extern crate rustc_span;

#[path = "../src/inputs.rs"]
mod inputs;
#[allow(dead_code, reason = "Metadata-only probes do not lower scalar values")]
#[path = "source_constant_facts_support.rs"]
mod source_capabilities;
#[path = "../src/source_origin/mod.rs"]
mod source_origin;

use portable_codegen::{RustDeclarationId, RustExportTarget};
use rustc_driver::{Callbacks, Compilation};
use rustc_interface::interface;
use rustc_middle::ty::TyCtxt;
use source_origin::public_api::Inventory;

fn spelling(id: RustDeclarationId) -> String {
    format!("{:016x}:{:016x}", id.crate_id, id.definition_path_hash)
}
fn check(tcx: TyCtxt<'_>) -> Result<(), String> {
    let mut cache = source_origin::Cache::default();
    let inventory = Inventory::read_with_constant_reexports(tcx, &mut cache)?;
    let repeated = Inventory::read_with_constant_reexports(tcx, &mut cache)?;
    assert_eq!(inventory.declarations(), repeated.declarations());
    assert_eq!(inventory.foreign_constants(), repeated.foreign_constants());
    assert!(std::sync::Arc::ptr_eq(
        inventory.exports(),
        repeated.exports()
    ));
    for (id, declaration) in inventory.declarations() {
        assert_eq!(
            *id,
            source_origin::identity(tcx, declaration.definition().to_def_id())
        );
        let origin = source_origin::read(
            tcx,
            &mut cache,
            declaration.definition().to_def_id(),
            portable_codegen::RustSourceNode::Declaration,
            tcx.def_span(declaration.definition()),
        )?;
        assert_eq!(origin.declaration, *id);
        assert!(std::sync::Arc::ptr_eq(
            &origin.crate_exports,
            inventory.exports()
        ));
        println!(
            "OWNED {:?} {} {}",
            declaration.kind(),
            tcx.def_path_str(declaration.definition().to_def_id()),
            spelling(*id)
        );
    }
    for (id, declaration) in inventory.foreign_constants() {
        assert!(!declaration.definition().is_local());
        assert_eq!(*id, source_origin::identity(tcx, declaration.definition()));
        assert!(!inventory.declarations().contains_key(id));
        println!(
            "FOREIGN {} {}",
            tcx.def_path_str(declaration.definition()),
            spelling(*id)
        );
    }
    for (module, bindings) in &inventory.exports().modules {
        for (name, target) in bindings {
            if let RustExportTarget::Declaration(id) = target {
                assert!(
                    inventory.declarations().contains_key(id)
                        || inventory.foreign_constants().contains_key(id)
                );
                println!("BIND {} {} {}", spelling(*module), name.name, spelling(*id));
            }
        }
    }
    println!("MODULES {}", inventory.exports().modules.len());
    if inventory.foreign_constants().is_empty() {
        assert!(Inventory::read(tcx, &mut cache).is_ok());
    } else {
        assert!(Inventory::read(tcx, &mut cache).is_err());
        assert!(inventory.function_roots().is_err());
        println!("PRODUCTION_REJECTED");
    }
    Ok(())
}
struct Probe {
    inputs: inputs::DeclaredInputs,
    result: Option<Result<(), String>>,
}
impl Callbacks for Probe {
    fn after_analysis<'tcx>(&mut self, _: &interface::Compiler, tcx: TyCtxt<'tcx>) -> Compilation {
        tcx.sess.dcx().abort_if_errors();
        self.result = Some(self.inputs.verify(tcx).and_then(|()| check(tcx)));
        Compilation::Stop
    }
}
fn main() -> std::process::ExitCode {
    #[cfg(foreign_export_private)]
    let _ = forged;
    let args: Vec<_> = std::env::args().collect();
    let mut probe = Probe {
        inputs: inputs::DeclaredInputs::new(&args[2], &[]).unwrap(),
        result: None,
    };
    let mut compiler_args = vec![
        "rustc".into(),
        probe.inputs.root().into(),
        "--sysroot".into(),
        args[1].clone(),
        "--crate-type=lib".into(),
        "--crate-name=export_root".into(),
        "--edition=2024".into(),
        "-Funsafe-code".into(),
        "-Awarnings".into(),
    ];
    compiler_args.extend_from_slice(&args[3..]);
    let status = rustc_driver::catch_with_exit_code(|| {
        rustc_driver::run_compiler(&compiler_args, &mut probe)
    });
    if status != std::process::ExitCode::SUCCESS {
        return status;
    }
    match probe.result.expect("compiler export probe did not execute") {
        Ok(()) => std::process::ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("{error}");
            std::process::ExitCode::FAILURE
        }
    }
}
#[cfg(foreign_export_private)]
fn forged(
    definition: rustc_hir::def_id::DefId,
) -> source_origin::public_api::ForeignConstantDeclaration {
    source_origin::public_api::ForeignConstantDeclaration { definition }
}
