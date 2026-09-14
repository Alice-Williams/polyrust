//! Owned observations only: no compiler context survives an invocation.
use rustc_hir::{
    self as hir,
    def::{DefKind, Res},
    def_id::{CrateNum, DefId, LOCAL_CRATE},
    intravisit::{self, Visitor},
};
use rustc_middle::ty::{TyCtxt, TypeckResults};
use std::collections::BTreeSet;

pub(super) fn read(tcx: TyCtxt<'_>) -> Vec<String> {
    let mut records = BTreeSet::from([crate_record(tcx, "local", LOCAL_CRATE)]);
    for function in tcx
        .hir_body_owners()
        .filter(|id| tcx.def_kind(*id) == DefKind::Fn)
    {
        records.insert(function_record(tcx, "local", function.to_def_id()));
        let mut calls = Calls {
            tcx,
            checked: tcx.typeck(function),
            records: &mut records,
        };
        calls.visit_body(tcx.hir_body_owned_by(function));
    }
    records.into_iter().collect()
}

pub(super) fn loaded(tcx: TyCtxt<'_>) -> Vec<String> {
    let mut records: Vec<_> = tcx
        .crates(())
        .iter()
        .map(|krate| crate_record(tcx, "loaded", *krate))
        .collect();
    if let Some(krate) = super::preload::resolved(tcx) {
        records.push(crate_record(tcx, "alias", krate));
    }
    for krate in tcx.crates(()) {
        if let Some(path) = &tcx.used_crate_source(*krate).rmeta {
            records.push(format!(
                "metadata\t{}\t{}",
                tcx.crate_name(*krate),
                path.display()
            ));
        }
    }
    records
}

fn crate_record(tcx: TyCtxt<'_>, role: &str, krate: CrateNum) -> String {
    format!(
        "crate\t{role}\t{}\t{:016x}\t{:?}",
        tcx.crate_name(krate),
        tcx.stable_crate_id(krate).as_u64(),
        tcx.crate_hash(krate),
    )
}

fn function_record(tcx: TyCtxt<'_>, role: &str, function: DefId) -> String {
    let identity = tcx.def_path_hash(function);
    format!(
        "function\t{role}\t{}\t{:016x}\t{:016x}\t{:?}",
        tcx.item_name(function),
        identity.stable_crate_id().as_u64(),
        identity.local_hash().as_u64(),
        tcx.fn_sig(function).instantiate_identity(),
    )
}

struct Calls<'a, 'tcx> {
    tcx: TyCtxt<'tcx>,
    checked: &'tcx TypeckResults<'tcx>,
    records: &'a mut BTreeSet<String>,
}

impl<'tcx> Visitor<'tcx> for Calls<'_, 'tcx> {
    fn visit_expr(&mut self, expression: &'tcx hir::Expr<'tcx>) {
        if let hir::ExprKind::Call(callee, _) = expression.kind
            && let hir::ExprKind::Path(path) = &callee.kind
            && let Res::Def(DefKind::Fn, function) = self.checked.qpath_res(path, callee.hir_id)
            && !function.is_local()
        {
            self.records
                .insert(crate_record(self.tcx, "foreign", function.krate));
            self.records
                .insert(function_record(self.tcx, "foreign", function));
        }
        intravisit::walk_expr(self, expression);
    }
}
