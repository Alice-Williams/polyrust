//! Exact scalar literal interpretation, shared before target-specific lowering.
use super::Capability;
use rustc_ast::LitKind;
use rustc_hir as hir;
use rustc_middle::ty::{self, TypeckResults};

pub(crate) struct LiteralValues;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum LiteralValue {
    I32(i32),
    I64(i64),
    Bool(bool),
}

#[derive(Clone, Copy)]
pub(crate) struct LiteralInput<'tcx> {
    value: LiteralValue,
    _expression: &'tcx hir::Expr<'tcx>,
}

impl Capability for LiteralValues {
    type Input<'tcx> = LiteralInput<'tcx>;
}

impl<'tcx> LiteralInput<'tcx> {
    pub(crate) fn read(
        checked: &TypeckResults<'tcx>,
        expression: &'tcx hir::Expr<'tcx>,
    ) -> Result<Self, String> {
        if !checked.expr_adjustments(expression).is_empty()
            || checked.type_dependent_def_id(expression.hir_id).is_some()
        {
            return Err("literal compiler adjustment or overload is not implemented".into());
        }
        let (literal, negative) = match expression.kind {
            hir::ExprKind::Lit(literal) => (literal, false),
            hir::ExprKind::Unary(hir::UnOp::Neg, operand) => {
                let hir::ExprKind::Lit(literal) = operand.kind else {
                    return Err("only negative integer literals are implemented".into());
                };
                if !checked.expr_adjustments(operand).is_empty()
                    || checked.expr_ty(operand) != checked.expr_ty(expression)
                {
                    return Err("negative literal operand type or adjustment differs".into());
                }
                (literal, true)
            }
            _ => return Err("literal capability received an unsupported source shape".into()),
        };
        let value = match (literal.node, checked.expr_ty(expression).kind()) {
            (LitKind::Bool(value), ty::Bool) if !negative => LiteralValue::Bool(value),
            (LitKind::Int(magnitude, _), ty::Int(kind @ (ty::IntTy::I32 | ty::IntTy::I64))) => {
                let magnitude =
                    i128::try_from(magnitude.0).map_err(|_| "integer literal overflow")?;
                let value = if negative { -magnitude } else { magnitude };
                match kind {
                    ty::IntTy::I32 => {
                        LiteralValue::I32(i32::try_from(value).map_err(|_| "i32 literal overflow")?)
                    }
                    ty::IntTy::I64 => {
                        LiteralValue::I64(i64::try_from(value).map_err(|_| "i64 literal overflow")?)
                    }
                    _ => unreachable!("closed integer type match"),
                }
            }
            _ => return Err("literal is not implemented".into()),
        };
        Ok(Self {
            value,
            _expression: expression,
        })
    }

    pub(crate) fn value(self) -> LiteralValue {
        self.value
    }
}
