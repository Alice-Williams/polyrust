//! Compiler-evaluated scalar reads; no initializer interpreter or target syntax.
use super::{Capability, LiteralValue};
use rustc_hir::{
    self as hir,
    def::{DefKind, Res},
    def_id::DefId,
};
use rustc_middle::ty::{TyCtxt, TypeckResults};

pub(crate) struct ScalarConstants;

/// Construction retains compiler provenance and is confined to this module.
#[derive(Clone, Copy)]
pub(crate) struct ConstantInput<'tcx> {
    value: LiteralValue,
    _definition: DefId,
    _expression: &'tcx hir::Expr<'tcx>,
}

impl Capability for ScalarConstants {
    type Input<'tcx> = ConstantInput<'tcx>;
}

impl<'tcx> ConstantInput<'tcx> {
    pub(crate) fn is_constant(
        checked: &TypeckResults<'tcx>,
        expression: &'tcx hir::Expr<'tcx>,
        path: &hir::QPath<'tcx>,
    ) -> bool {
        matches!(
            checked.qpath_res(path, expression.hir_id),
            Res::Def(DefKind::Const { .. } | DefKind::AssocConst { .. }, _)
        )
    }

    pub(crate) fn read(
        tcx: TyCtxt<'tcx>,
        checked: &TypeckResults<'tcx>,
        expression: &'tcx hir::Expr<'tcx>,
    ) -> Result<Self, String> {
        let hir::ExprKind::Path(ref path) = expression.kind else {
            return Err("scalar constant input requires a resolved path".into());
        };
        let Res::Def(
            DefKind::Const {
                is_type_const: false,
            }
            | DefKind::AssocConst {
                is_type_const: false,
            },
            definition,
        ) = checked.qpath_res(path, expression.hir_id)
        else {
            return Err("scalar constant input requires a constant definition".into());
        };
        if !checked.expr_adjustments(expression).is_empty()
            || !checked.node_args(expression.hir_id).is_empty()
        {
            return Err(
                "scalar constants require nongeneric unadjusted module or inherent definitions"
                    .into(),
            );
        }
        let (ty, value) = super::constant_evaluation::evaluate(tcx, definition)?;
        if ty != checked.expr_ty(expression) {
            return Err("scalar constant definition and expression types disagree".into());
        }
        Ok(Self {
            value,
            _definition: definition,
            _expression: expression,
        })
    }

    pub(crate) fn value(self) -> LiteralValue {
        self.value
    }

    #[cfg(constant_ast_probe)]
    pub(crate) fn origin(self) -> (DefId, &'tcx hir::Expr<'tcx>) {
        (self._definition, self._expression)
    }
}
