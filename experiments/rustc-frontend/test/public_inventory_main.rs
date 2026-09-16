#![feature(rustc_private)]
#![forbid(unsafe_code)]

extern crate rustc_driver;
extern crate rustc_hir;
extern crate rustc_interface;
extern crate rustc_middle;
extern crate rustc_span;

#[path = "../src/inputs.rs"]
mod inputs;
#[path = "../src/source_origin/mod.rs"]
mod source_origin;

use portable_codegen::{RustExportTarget, RustSourceNode};
use rustc_driver::{Callbacks, Compilation};
use rustc_interface::interface;
use rustc_middle::ty::TyCtxt;
use source_origin::public_api::{DeclarationKind, Inventory};

struct Probe {
    inputs: inputs::DeclaredInputs,
    result: Option<Result<(), String>>,
}

fn check(tcx: TyCtxt<'_>) -> Result<(), String> {
    let mut cache = source_origin::Cache::default();
    let inventory = Inventory::read(tcx, &mut cache)?;
    let repeated = Inventory::read(tcx, &mut cache)?;
    assert_eq!(inventory.declarations(), repeated.declarations());
    assert!(std::sync::Arc::ptr_eq(
        inventory.exports(),
        repeated.exports()
    ));
    let mut previous = None;
    for (identity, declaration) in inventory.declarations() {
        assert!(previous.is_none_or(|previous| previous < *identity));
        previous = Some(*identity);
        let definition = declaration.definition();
        assert_eq!(
            *identity,
            source_origin::identity(tcx, definition.to_def_id())
        );
        let origin = source_origin::read(
            tcx,
            &mut cache,
            definition.to_def_id(),
            RustSourceNode::Declaration,
            tcx.def_span(definition),
        )?;
        assert!(origin.externally_reachable);
        assert_eq!(origin.declaration, *identity);
        assert!(std::sync::Arc::ptr_eq(
            &origin.crate_exports,
            inventory.exports()
        ));
        let path = tcx.def_path_str(definition.to_def_id());
        println!(
            "DECL {:?} {path} {:016x}",
            declaration.kind(),
            identity.definition_path_hash
        );
        for doc in &origin.documentation {
            println!("DOC {}", doc.trim());
        }
    }
    for bindings in inventory.exports().modules.values() {
        for (name, target) in bindings {
            if let RustExportTarget::Declaration(identity) = target {
                assert!(inventory.declarations().contains_key(identity));
                println!("BIND {} {:016x}", name.name, identity.definition_path_hash);
            }
        }
    }
    println!("MODULES {}", inventory.exports().modules.len());
    match inventory.function_roots() {
        Ok(functions) => {
            assert!(
                inventory
                    .declarations()
                    .values()
                    .all(|d| d.kind() == DeclarationKind::Function)
            );
            assert_eq!(functions.len(), inventory.declarations().len());
            println!("FUNCTIONS_ONLY {}", functions.len());
        }
        Err(error) => {
            assert!(
                inventory
                    .declarations()
                    .values()
                    .any(|d| d.kind() == DeclarationKind::Constant)
            );
            assert!(error.contains("constant exports"));
            println!("FUNCTIONS_ONLY_REJECTED");
        }
    }
    Ok(())
}

impl Callbacks for Probe {
    fn after_analysis<'tcx>(&mut self, _: &interface::Compiler, tcx: TyCtxt<'tcx>) -> Compilation {
        tcx.sess.dcx().abort_if_errors();
        self.result = Some(self.inputs.verify(tcx).and_then(|()| check(tcx)));
        Compilation::Stop
    }
}

fn main() -> std::process::ExitCode {
    #[cfg(public_inventory_private)]
    let _ = forged;
    let args: Vec<_> = std::env::args().collect();
    assert!(args.len() >= 3);
    let mut probe = Probe {
        inputs: inputs::DeclaredInputs::new(&args[2], &args[3..]).unwrap(),
        result: None,
    };
    let compiler_args = vec![
        "rustc".into(),
        probe.inputs.root().into(),
        "--sysroot".into(),
        args[1].clone(),
        "--crate-type=lib".into(),
        "--crate-name=poly_public_inventory_probe".into(),
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
    match probe
        .result
        .expect("compiler-backed inventory probe did not execute")
    {
        Ok(()) => std::process::ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("{error}");
            std::process::ExitCode::FAILURE
        }
    }
}

#[cfg(public_inventory_private)]
fn forged(exports: std::sync::Arc<portable_codegen::RustCrateExports>) -> Inventory {
    Inventory {
        exports,
        declarations: std::collections::BTreeMap::new(),
    }
}
