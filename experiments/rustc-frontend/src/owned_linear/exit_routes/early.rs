//! A false continuation stays in its real block; no synthetic else scope.
use super::{End, Exit, Mode, Outcome};
use crate::owned_linear::{LinearError as Error, Result};
use rustc_hir as hir;

pub(in crate::owned_linear) struct Parts<'tcx> {
    pub prefix: &'tcx [hir::Stmt<'tcx>],
    pub branch: &'tcx hir::Expr<'tcx>,
    pub condition: &'tcx hir::Expr<'tcx>,
    pub arm: &'tcx hir::Block<'tcx>,
    pub continuation: Exit<'tcx>,
}

pub(in crate::owned_linear) fn root<'tcx>(block: &'tcx hir::Block<'tcx>) -> Result<Parts<'tcx>> {
    let (statements, value) = match block.expr {
        Some(value) => (block.stmts, value),
        None => {
            let (last, before) = block.stmts.split_last().ok_or(Error::BodyShape)?;
            let hir::StmtKind::Semi(value) = last.kind else {
                return Err(Error::BodyShape);
            };
            if !matches!(value.kind, hir::ExprKind::Ret(Some(_))) {
                return Err(Error::BodyShape);
            }
            (before, value)
        }
    };
    let continuation = match value.kind {
        hir::ExprKind::Ret(Some(inner)) => Exit::Return {
            expression: value,
            value: inner,
        },
        _ => Exit::Tail(value),
    };
    let (last, prefix) = statements.split_last().ok_or(Error::BodyShape)?;
    let (hir::StmtKind::Expr(branch) | hir::StmtKind::Semi(branch)) = last.kind else {
        return Err(Error::BodyShape);
    };
    let hir::ExprKind::If(condition, arm, None) = branch.kind else {
        return Err(Error::BodyShape);
    };
    let hir::ExprKind::Block(arm, None) = arm.kind else {
        return Err(Error::BodyShape);
    };
    Ok(Parts {
        prefix,
        branch,
        condition,
        arm,
        continuation,
    })
}

pub(super) fn select<'tcx>(
    block: &'tcx hir::Block<'tcx>,
    outcome: Outcome,
) -> Result<(&'tcx [hir::Stmt<'tcx>], End<'tcx>)> {
    // The caller has already authenticated the root grammar. Its selected
    // early arm has no condition and must be a plain explicit-return block.
    match root(block) {
        Ok(parts) => Ok((
            parts.prefix,
            match outcome {
                Outcome::False => End::Exit(parts.continuation),
                Outcome::True => End::Nested(parts.arm),
            },
        )),
        Err(_) => super::parts(block, Mode::Return),
    }
}
