//! Canonical conditional transfer into one root-scope binding.
use super::super::super::{
    LinearError as Error, Result,
    exits::{self, Outcome},
    source::{binding, local},
};
use rustc_hir::{self as hir, HirId, def::DefKind, def_id::LocalDefId};
use rustc_middle::ty::TyCtxt;

pub(in crate::owned_linear::multiple) struct Shape<'tcx> {
    pub root: &'tcx hir::Block<'tcx>,
    pub branch: &'tcx hir::Expr<'tcx>,
    pub condition: &'tcx hir::Expr<'tcx>,
    pub operands: [&'tcx hir::Expr<'tcx>; 2],
    pub binding: HirId,
    pub parameter: HirId,
    pub parameter_index: usize,
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
        .filter_map(|(i, ty)| (*ty == tcx.types.bool).then_some(i))
        .collect();
    let [parameter_index] = booleans.as_slice() else {
        return Err(Error::Signature);
    };
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
    let (statements, end) = exits::parts(root, exits::Mode::Selection(Outcome::False))?;
    let exits::End::Exit(exit) = end else {
        return Err(Error::BodyShape);
    };
    let hir::StmtKind::Let(declaration) = statements.last().ok_or(Error::BodyShape)?.kind else {
        return Err(Error::BodyShape);
    };
    if declaration.els.is_some() {
        return Err(Error::BodyShape);
    }
    let destination = binding(declaration.pat)?;
    let branch = declaration.init.ok_or(Error::BodyShape)?;
    let hir::ExprKind::If(condition, yes, Some(no)) = branch.kind else {
        return Err(Error::BodyShape);
    };
    let operands = [no, yes].map(|arm| {
        let hir::ExprKind::Block(block, None) = arm.kind else {
            return Err(Error::BodyShape);
        };
        if !block.stmts.is_empty() {
            return Err(Error::BodyShape);
        }
        block.expr.ok_or(Error::BodyShape)
    });
    let [no, yes] = operands;
    let operands = [no?, yes?];
    let checked = tcx.typeck(owner);
    if !checked.expr_adjustments(branch).is_empty()
        || !checked.expr_adjustments(condition).is_empty()
        || operands
            .iter()
            .any(|e| !checked.expr_adjustments(e).is_empty())
        || local(checked, condition)? != parameter
        || checked.expr_ty(condition) != tcx.types.bool
        || local(checked, operands[0])? == local(checked, operands[1])?
    {
        return Err(Error::SourceIdentity);
    }
    let hir::ExprKind::Unary(hir::UnOp::Deref, operand) = exit.value().kind else {
        return Err(Error::Read);
    };
    if local(checked, operand)? != destination {
        return Err(Error::Read);
    }
    Ok(Shape {
        root,
        branch,
        condition,
        operands,
        binding: destination,
        parameter,
        parameter_index: *parameter_index,
    })
}

pub(in crate::owned_linear::multiple) fn operand<'tcx>(
    tcx: TyCtxt<'tcx>,
    owner: LocalDefId,
    expression: &'tcx hir::Expr<'tcx>,
    id: HirId,
    outcome: Outcome,
) -> Result<&'tcx hir::Expr<'tcx>> {
    let shape = read(tcx, owner)?;
    if !std::ptr::eq(expression, shape.branch) || id != shape.binding {
        return Err(Error::SourceIdentity);
    }
    Ok(shape.operands[outcome.value() as usize])
}
