//! Observation only: no owned-call capability or body admission yet.
#![feature(rustc_private)]
#![forbid(unsafe_code)]
extern crate rustc_driver;
extern crate rustc_hir;
extern crate rustc_interface;
extern crate rustc_middle;
extern crate rustc_span;
#[path = "../../src/inputs.rs"]
mod inputs;
use rustc_driver::{Callbacks, Compilation};
use rustc_interface::interface;
use rustc_middle::{mir, ty::TyCtxt};
struct Probe {
    inputs: inputs::DeclaredInputs,
    checked: bool,
}
impl Callbacks for Probe {
    fn after_analysis<'tcx>(&mut self, _: &interface::Compiler, tcx: TyCtxt<'tcx>) -> Compilation {
        tcx.sess.dcx().abort_if_errors();
        self.inputs.verify(tcx).expect("declared inputs");
        let mut seen = std::collections::BTreeSet::new();
        for owner in tcx
            .hir_body_owners()
            .filter(|id| tcx.def_kind(*id) == rustc_hir::def::DefKind::Fn)
        {
            let body = tcx.mir_drops_elaborated_and_const_checked(owner).borrow();
            assert_eq!(body.source.def_id(), owner.to_def_id());
            assert_eq!(
                body.phase,
                mir::MirPhase::Runtime(mir::RuntimePhase::PostCleanup)
            );
            let name = tcx.def_path_str(owner);
            println!(
                "FUNCTION {name} HIR {:?}",
                tcx.hir_body_owned_by(owner).value
            );
            for (block, data) in body.basic_blocks.iter_enumerated() {
                println!(
                    "{block:?}: {:?} {:?}",
                    data.statements,
                    data.terminator().kind
                );
            }
            assert!(seen.insert(name));
        }
        assert_eq!(
            seen,
            [
                "produce",
                "produce_return",
                "consume",
                "relay",
                "via_producer",
                "via_consumer",
                "via_relay",
                "aliased"
            ]
            .into_iter()
            .map(String::from)
            .collect()
        );
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
    let compiler_args = vec![
        "rustc".into(),
        probe.inputs.root().into(),
        "--sysroot".into(),
        args[1].clone(),
        "--crate-type=lib".into(),
        "--crate-name=poly_owned_calls".into(),
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
    println!("owned-call representation observation passed");
    status
}
