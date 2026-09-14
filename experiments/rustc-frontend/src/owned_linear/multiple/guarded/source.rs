//! Canonical one-condition grammar; no branch-local ownership operations.
use super::super::super::{
    LinearError as Error, Result,
    exits::{self, Outcome},
    source::{binding, local},
};
use rustc_hir::{self as hir, HirId, def::DefKind, def_id::LocalDefId};
use rustc_middle::ty::TyCtxt;

pub(in crate::owned_linear::multiple) struct Shape<'tcx> {
    pub branch: &'tcx hir::Expr<'tcx>,
    pub condition: &'tcx hir::Expr<'tcx>,
    pub parameter: HirId,
    pub parameter_index: usize,
    pub root: &'tcx hir::Block<'tcx>,
    pub arms: [&'tcx hir::Block<'tcx>; 2],
}
pub(in crate::owned_linear::multiple) fn read(
    tcx: TyCtxt<'_>,
    owner: LocalDefId,
) -> Result<Shape<'_>> {
    if tcx.def_kind(owner) != DefKind::Fn || tcx.generics_of(owner).count() != 0 {
        return Err(Error::Signature);
    }
    let signature = tcx.fn_sig(owner).instantiate_identity().skip_binder();
    let booleans: Vec<_> = signature
        .inputs()
        .iter()
        .enumerate()
        .filter(|(_, ty)| **ty == tcx.types.bool)
        .map(|(i, _)| i)
        .collect();
    let [parameter_index] = booleans.as_slice() else {
        return Err(Error::Signature);
    };
    if signature.inputs().len() < 2 {
        return Err(Error::Signature);
    }
    let body = tcx.hir_body_owned_by(owner);
    let parameter = binding(
        body.params
            .get(*parameter_index)
            .ok_or(Error::Signature)?
            .pat,
    )?;
    let hir::ExprKind::Block(root, None) = body.value.kind else {
        return Err(Error::BodyShape);
    };
    let branch = root.expr.ok_or(Error::BodyShape)?;
    let hir::ExprKind::If(condition, yes, Some(no)) = branch.kind else {
        return Err(Error::BodyShape);
    };
    let checked = tcx.typeck(owner);
    if local(checked, condition)? != parameter || checked.expr_ty(condition) != tcx.types.bool {
        return Err(Error::Argument);
    }
    let mut arms = Vec::new();
    for arm in [no, yes] {
        let hir::ExprKind::Block(block, None) = arm.kind else {
            return Err(Error::BodyShape);
        };
        let (prefix, end) = exits::parts(block, exits::Mode::Return)?;
        if !prefix.is_empty() || !matches!(end, exits::End::Exit(exits::Exit::Return { .. })) {
            return Err(Error::BodyShape);
        }
        arms.push(block);
    }
    Ok(Shape {
        branch,
        condition,
        parameter,
        parameter_index: *parameter_index,
        root,
        arms: [arms[0], arms[1]],
    })
}
impl Shape<'_> {
    pub(super) fn arm(&self, outcome: Outcome) -> HirId {
        self.arms[outcome.value() as usize].hir_id
    }
}
