//! Compiler observations only: no conditional ownership admission boundary.
#![feature(rustc_private)]
#![forbid(unsafe_code)]
extern crate rustc_abi;
extern crate rustc_ast;
extern crate rustc_driver;
extern crate rustc_hir;
extern crate rustc_interface;
extern crate rustc_middle;
extern crate rustc_span;
mod events;
#[path = "../../src/inputs.rs"]
mod inputs;
mod observation;
mod source;
mod trace;
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
        self.inputs.verify(tcx).expect("declared inputs");
        let mut seen = Vec::new();
        let mut nominal = None;
        for owner in tcx
            .hir_body_owners()
            .filter(|id| tcx.def_kind(*id) == rustc_hir::def::DefKind::Fn)
        {
            let name = tcx.def_path_str(owner);
            seen.push(name.clone());
            let actual = observation::check(tcx, owner, &name);
            assert_eq!(
                actual,
                *nominal.get_or_insert(actual),
                "one canonical Pair type"
            );
        }
        seen.sort();
        assert_eq!(
            seen,
            [
                "early_first",
                "early_second",
                "initialize",
                "initialize_reversed",
                "partial_first",
                "partial_second"
            ]
        );
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
        "--crate-name=poly_conditional_owned".into(),
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
    println!("conditional ownership observation inventory passed");
    status
}
