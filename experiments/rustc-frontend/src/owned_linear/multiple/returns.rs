//! An explicit source return paired with its authenticated normal MIR exit.
use super::super::{LinearError as Error, Result, exits, flow};
use super::{MultipleOwnedBody, relations, source};
use rustc_hir::{self as hir, HirId, def_id::LocalDefId};
use rustc_middle::{mir, ty::TyCtxt};

pub(crate) struct ReturnEvidence<'tcx> {
    expression: &'tcx hir::Expr<'tcx>,
    value: &'tcx hir::Expr<'tcx>,
    scope: HirId,
    returning: mir::Location,
}
impl<'tcx> ReturnEvidence<'tcx> {
    pub(crate) fn expression(&self) -> &'tcx hir::Expr<'tcx> {
        self.expression
    }
    pub(crate) fn value(&self) -> &'tcx hir::Expr<'tcx> {
        self.value
    }
    pub(crate) fn scope(&self) -> HirId {
        self.scope
    }
    pub(crate) fn location(&self) -> mir::Location {
        self.returning
    }
}

pub(crate) struct ReturningOwnedBody<'tcx> {
    body: MultipleOwnedBody<'tcx>,
    exit: ReturnEvidence<'tcx>,
}
impl<'tcx> ReturningOwnedBody<'tcx> {
    pub(crate) fn read(tcx: TyCtxt<'tcx>, owner: LocalDefId) -> Result<Self> {
        let plan = source::read_return(tcx, owner)?;
        let body = tcx.mir_drops_elaborated_and_const_checked(owner).borrow();
        let matched = relations::validate(tcx, owner, &plan, &body)?;
        let exits::Exit::Return { expression, value } = plan.scopes.exit() else {
            return Err(Error::BodyShape);
        };
        let exit = certify(
            &plan,
            &body,
            Claims {
                expression,
                value,
                scope: plan.scopes.read_scope(),
                returning: matched.returning,
            },
        )?;
        Ok(Self {
            body: MultipleOwnedBody::from_matched(plan, matched)?,
            exit,
        })
    }
    pub(crate) fn body(&self) -> &MultipleOwnedBody<'tcx> {
        &self.body
    }
    pub(crate) fn exit(&self) -> &ReturnEvidence<'tcx> {
        &self.exit
    }
    pub(crate) fn into_parts(self) -> (MultipleOwnedBody<'tcx>, ReturnEvidence<'tcx>) {
        (self.body, self.exit)
    }
}

struct Claims<'tcx> {
    expression: &'tcx hir::Expr<'tcx>,
    value: &'tcx hir::Expr<'tcx>,
    scope: HirId,
    returning: mir::Location,
}
fn certify<'tcx>(
    plan: &source::Plan<'tcx>,
    body: &mir::Body<'tcx>,
    claims: Claims<'tcx>,
) -> Result<ReturnEvidence<'tcx>> {
    if !plan.scopes.exit().same(exits::Exit::Return {
        expression: claims.expression,
        value: claims.value,
    }) || plan.scopes.read_scope() != claims.scope
        || flow::trace(body)?.returning != claims.returning
    {
        return Err(Error::Scope);
    }
    Ok(ReturnEvidence {
        expression: claims.expression,
        value: claims.value,
        scope: claims.scope,
        returning: claims.returning,
    })
}

#[cfg(owned_return_proof)]
#[path = "../../../test/owned_returns/mutations.rs"]
pub(crate) mod mutations;
