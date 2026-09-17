//! Original compiler-owned evidence for four built-in binary64 operators.
use super::Capability;
use rustc_hir as hir;
use rustc_middle::ty::{self, TyCtxt, TypeckResults};

pub(crate) struct FloatingArithmetic;
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum FloatingArithmeticOperator {
    Add,
    Subtract,
    Multiply,
    Divide,
}
pub(crate) struct ArithmeticInput<'tcx> {
    source: &'tcx hir::Expr<'tcx>,
    operator: FloatingArithmeticOperator,
    left: &'tcx hir::Expr<'tcx>,
    right: &'tcx hir::Expr<'tcx>,
}
impl Capability for FloatingArithmetic {
    type Input<'tcx> = ArithmeticInput<'tcx>;
}
impl<'tcx> ArithmeticInput<'tcx> {
    pub(crate) fn read(
        tcx: TyCtxt<'tcx>,
        checked: &TypeckResults<'tcx>,
        expression: &hir::Expr<'tcx>,
    ) -> Result<Self, String> {
        let hir::Node::Expr(canonical) = tcx.hir_node(expression.hir_id) else {
            return Err("floating arithmetic requires a canonical HIR expression".into());
        };
        if !std::ptr::eq(canonical, expression)
            || !std::ptr::eq(checked, tcx.typeck(expression.hir_id.owner.def_id))
        {
            return Err(
                "floating arithmetic requires its original checked function and HIR node".into(),
            );
        }
        let hir::ExprKind::Binary(operator, left, right) = canonical.kind else {
            return Err("floating arithmetic requires a binary expression".into());
        };
        let operator = match operator.node {
            hir::BinOpKind::Add => FloatingArithmeticOperator::Add,
            hir::BinOpKind::Sub => FloatingArithmeticOperator::Subtract,
            hir::BinOpKind::Mul => FloatingArithmeticOperator::Multiply,
            hir::BinOpKind::Div => FloatingArithmeticOperator::Divide,
            _ => return Err("unsupported floating arithmetic operator".into()),
        };
        let ty = checked.expr_ty(canonical);
        if checked.type_dependent_def_id(canonical.hir_id).is_some()
            || !matches!(ty.kind(), ty::Float(ty::FloatTy::F64))
            || checked.expr_ty(left) != ty
            || checked.expr_ty(right) != ty
            || !checked.expr_adjustments(canonical).is_empty()
            || !checked.expr_adjustments(left).is_empty()
            || !checked.expr_adjustments(right).is_empty()
        {
            return Err(
                "floating arithmetic requires unadjusted built-in f64 operands and result".into(),
            );
        }
        Ok(Self {
            source: canonical,
            operator,
            left,
            right,
        })
    }
    pub(crate) fn require_context(
        &self,
        tcx: TyCtxt<'tcx>,
        checked: &TypeckResults<'tcx>,
    ) -> Result<(), String> {
        let current = Self::read(tcx, checked, self.source)?;
        if current.operator != self.operator
            || !std::ptr::eq(current.left, self.left)
            || !std::ptr::eq(current.right, self.right)
        {
            return Err("floating arithmetic operator or operand identity differs".into());
        }
        Ok(())
    }
    pub(crate) fn operator(&self) -> FloatingArithmeticOperator {
        self.operator
    }
    pub(crate) fn left(&self) -> &'tcx hir::Expr<'tcx> {
        self.left
    }
    pub(crate) fn right(&self) -> &'tcx hir::Expr<'tcx> {
        self.right
    }
}

#[cfg(arithmetic_ast_probe)]
#[path = "../../test/arithmetic_input_probe.rs"]
mod probe;
