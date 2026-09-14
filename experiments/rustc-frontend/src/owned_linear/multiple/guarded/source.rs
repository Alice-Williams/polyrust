//! Canonical one-condition grammar; no branch-local ownership operations.
use super::super::super::{
    LinearError as Error, Result,
    exits::{self, Outcome},
    source::{binding, local},
};
use rustc_hir::{self as hir, HirId, def::DefKind, def_id::LocalDefId};
use rustc_middle::ty::TyCtxt;

#[derive(Clone, Copy)]
pub(in crate::owned_linear::multiple) enum Grammar {
    IfElse,
    Early,
}

pub(in crate::owned_linear::multiple) struct Shape<'tcx> {
    pub grammar: Grammar,
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
    read_grammar(tcx, owner, Grammar::IfElse)
}
pub(in crate::owned_linear::multiple) fn read_early(
    tcx: TyCtxt<'_>,
    owner: LocalDefId,
) -> Result<Shape<'_>> {
    read_grammar(tcx, owner, Grammar::Early)
}
fn read_grammar(tcx: TyCtxt<'_>, owner: LocalDefId, grammar: Grammar) -> Result<Shape<'_>> {
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
    let (branch, condition, arms) = match grammar {
        Grammar::IfElse => {
            let branch = root.expr.ok_or(Error::BodyShape)?;
            let hir::ExprKind::If(condition, yes, Some(no)) = branch.kind else {
                return Err(Error::BodyShape);
            };
            let hir::ExprKind::Block(no, None) = no.kind else {
                return Err(Error::BodyShape);
            };
            let hir::ExprKind::Block(yes, None) = yes.kind else {
                return Err(Error::BodyShape);
            };
            require_return(no)?;
            require_return(yes)?;
            (branch, condition, [no, yes])
        }
        Grammar::Early => {
            let parts = exits::early::root(root)?;
            require_return(parts.arm)?;
            (parts.branch, parts.condition, [root, parts.arm])
        }
    };
    let checked = tcx.typeck(owner);
    if local(checked, condition)? != parameter || checked.expr_ty(condition) != tcx.types.bool {
        return Err(Error::Argument);
    }
    Ok(Shape {
        grammar,
        branch,
        condition,
        parameter,
        parameter_index: *parameter_index,
        root,
        arms,
    })
}
fn require_return(block: &hir::Block<'_>) -> Result<()> {
    let (prefix, end) = exits::parts(block, exits::Mode::Return)?;
    if !prefix.is_empty() || !matches!(end, exits::End::Exit(exits::Exit::Return { .. })) {
        return Err(Error::BodyShape);
    }
    Ok(())
}
impl Shape<'_> {
    pub(super) fn arm(&self, outcome: Outcome) -> HirId {
        self.arms[outcome.value() as usize].hir_id
    }
}
