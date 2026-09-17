//! Exercise the production discovery walkers independently of target AST limits.
#![feature(rustc_private)]
#![forbid(unsafe_code)]
extern crate rustc_ast;
extern crate rustc_driver;
extern crate rustc_hir;
extern crate rustc_interface;
extern crate rustc_middle;
extern crate rustc_span;

#[path = "../src/c_lower/functions.rs"]
mod c_inventory;
#[path = "../src/java_lower/functions.rs"]
mod java_inventory;
// This focused probe uses discovery only, not the other exported mappings.
#[allow(dead_code, unused_imports)]
#[path = "../src/source_capabilities/mod.rs"]
mod source_capabilities;
#[allow(dead_code)]
#[path = "../src/source_origin/mod.rs"]
mod source_origin;

use rustc_driver::{Callbacks, Compilation};
use rustc_interface::interface;
use rustc_middle::ty::TyCtxt;
type Result<T> = std::result::Result<T, String>;

// Only the callable lookup is used by the production C discovery module.
// Constant registration and target certification are deliberately not run here.
struct ForeignLookup<'a> {
    function: &'a dyn Fn(
        rustc_hir::def_id::DefId,
    ) -> Result<portable_backend_c::dialect::CDependencyFunction>,
}
struct Probe {
    result: Option<Result<usize>>,
}

fn check(tcx: TyCtxt<'_>) -> Result<usize> {
    let mut cache = source_origin::Cache::default();
    let inventory =
        source_origin::public_api::Inventory::read_with_constant_reexports(tcx, &mut cache)?;
    let roots = inventory
        .declarations()
        .values()
        .filter(|declaration| {
            declaration.kind() == source_origin::public_api::DeclarationKind::Function
        })
        .map(|declaration| declaration.definition())
        .collect::<Vec<_>>();
    let c = c_inventory::inventory(tcx, &roots, None);
    let java = java_inventory::inventory(tcx, &roots);
    match (c, java) {
        (Ok(c), Ok(java)) => {
            assert_eq!(c.owned, java.local);
            assert!(c.foreign.is_empty() && java.foreign.is_empty());
            assert_eq!(c.constants, java.constants);
            let combined = inventory.constant_imports(tcx, &c.constants)?;
            assert_eq!(combined, inventory.constant_imports(tcx, &java.constants)?);
            let mut reversed = c.constants.clone();
            reversed.reverse();
            assert_eq!(combined, inventory.constant_imports(tcx, &reversed)?);
            Ok(combined.len())
        }
        (Err(c), Err(java)) => {
            assert_eq!(c, java);
            Err(c)
        }
        _ => panic!("C and Java discovery disagree"),
    }
}
impl Callbacks for Probe {
    fn after_analysis<'tcx>(&mut self, _: &interface::Compiler, tcx: TyCtxt<'tcx>) -> Compilation {
        tcx.sess.dcx().abort_if_errors();
        self.result = Some(check(tcx));
        Compilation::Stop
    }
}
fn main() -> std::process::ExitCode {
    let args: Vec<_> = std::env::args().collect();
    assert_eq!(args.len(), 5);
    let mut probe = Probe { result: None };
    let compiler_args = vec![
        "rustc".into(),
        args[2].clone(),
        "--sysroot".into(),
        args[1].clone(),
        "--crate-type=lib".into(),
        "--crate-name=inventory_root".into(),
        "--edition=2024".into(),
        "-Funsafe-code".into(),
        "-Awarnings".into(),
        "--extern".into(),
        format!("first={}", args[3]),
        "--extern".into(),
        format!("second={}", args[4]),
    ];
    let status = rustc_driver::catch_with_exit_code(|| {
        rustc_driver::run_compiler(&compiler_args, &mut probe)
    });
    if status != std::process::ExitCode::SUCCESS {
        return status;
    }
    match probe.result.expect("compiler discovery did not execute") {
        Ok(count) => {
            println!("CONSTANTS {count}");
            std::process::ExitCode::SUCCESS
        }
        Err(error) => {
            eprintln!("{error}");
            std::process::ExitCode::FAILURE
        }
    }
}
