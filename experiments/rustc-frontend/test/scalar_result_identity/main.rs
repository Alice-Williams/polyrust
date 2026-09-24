#![feature(rustc_private)]
#![forbid(unsafe_code)]
//! Backend-independent checked-signature observation, not source admission.
extern crate rustc_driver;
extern crate rustc_hir;
extern crate rustc_interface;
extern crate rustc_middle;
extern crate rustc_span;

#[path = "../../src/inputs.rs"]
mod inputs;
mod shape;

use rustc_driver::{Callbacks, Compilation};
use rustc_hir::def::DefKind;
use rustc_interface::interface;
use rustc_middle::ty::{self, TyCtxt};

fn observe(tcx: TyCtxt<'_>) -> Result<Vec<String>, String> {
    let mut records = Vec::new();
    for item in tcx.hir_crate_items(()).free_items() {
        let definition = item.owner_id.def_id;
        if tcx.def_kind(definition) != DefKind::Fn {
            continue;
        }
        if tcx.generics_of(definition).count() != 0 {
            return Err("generic result signatures are not observed".into());
        }
        let signature = tcx
            .try_normalize_erasing_regions(
                ty::TypingEnv::fully_monomorphized(),
                tcx.fn_sig(definition).instantiate_identity(),
            )
            .map_err(|_| "signature normalization failed")?;
        if !signature.bound_vars().is_empty() {
            return Err("bound result signatures are not observed".into());
        }
        let signature = signature.skip_binder();
        if signature.inputs().len() != 1 {
            return Err("identity observation requires one result parameter".into());
        }
        let input = shape::ResultShape::observe(tcx, signature.inputs()[0])?;
        let output = shape::ResultShape::observe(tcx, signature.output())?;
        if !input.same_instance(&output) {
            return Err("result instance changed across signature".into());
        }
        records.push(input.describe(tcx));
    }
    if records.is_empty() {
        return Err("no result signature observed".into());
    }
    Ok(records)
}

struct Probe {
    inputs: inputs::DeclaredInputs,
    result: Option<Result<Vec<String>, String>>,
}

impl Callbacks for Probe {
    fn after_analysis<'tcx>(&mut self, _: &interface::Compiler, tcx: TyCtxt<'tcx>) -> Compilation {
        tcx.sess.dcx().abort_if_errors();
        self.result = Some(self.inputs.verify(tcx).and_then(|()| observe(tcx)));
        Compilation::Stop
    }
}

fn main() -> std::process::ExitCode {
    let args: Vec<_> = std::env::args().collect();
    assert_eq!(args.len(), 3);
    let mut probe = Probe {
        inputs: inputs::DeclaredInputs::new(&args[2], &[]).unwrap(),
        result: None,
    };
    let compiler_args = vec![
        "rustc".into(),
        probe.inputs.root().into(),
        "--sysroot".into(),
        args[1].clone(),
        "--crate-type=lib".into(),
        "--crate-name=poly_result_identity_probe".into(),
        "--edition=2024".into(),
        "-Funsafe-code".into(),
        "-Cpanic=abort".into(),
    ];
    let status = rustc_driver::catch_with_exit_code(|| {
        rustc_driver::run_compiler(&compiler_args, &mut probe);
    });
    if status != std::process::ExitCode::SUCCESS {
        return status;
    }
    match probe.result.expect("compiler observation did not execute") {
        Ok(records) => {
            for record in records {
                println!("{record}");
            }
            std::process::ExitCode::SUCCESS
        }
        Err(error) => {
            eprintln!("{error}");
            std::process::ExitCode::FAILURE
        }
    }
}

#[cfg(scalar_result_forge)]
#[allow(dead_code)]
fn forged(tcx: TyCtxt<'_>, variant: rustc_hir::def_id::DefId) -> shape::ResultShape<'_> {
    shape::ResultShape {
        value: tcx.types.i32,
        ok: variant,
        err: variant,
        facts: shape::ResultShape::observe(tcx, tcx.types.i32)
            .unwrap()
            .facts(),
        error_bytes: 1,
        error_facts: shape::ResultShape::observe(tcx, tcx.types.i32)
            .unwrap()
            .error_facts(),
    }
}
