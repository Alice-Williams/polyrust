//! Exact scalar literal interpretation, shared before target-specific lowering.
use super::Capability;
use portable_binary64::FiniteBinary64;
use rustc_ast::LitKind;
use rustc_hir as hir;
use rustc_middle::ty::{self, TyCtxt, TypeckResults};

pub(crate) struct LiteralValues;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum LiteralValue {
    I32(i32),
    I64(i64),
    Bool(bool),
    F64(FiniteBinary64),
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
        tcx: TyCtxt<'tcx>,
        checked: &TypeckResults<'tcx>,
        expression: &hir::Expr<'tcx>,
    ) -> Result<Self, String> {
        let hir::Node::Expr(canonical) = tcx.hir_node(expression.hir_id) else {
            return Err("literal requires a canonical HIR expression".into());
        };
        if !std::ptr::eq(canonical, expression)
            || !std::ptr::eq(checked, tcx.typeck(expression.hir_id.owner.def_id))
        {
            return Err("literal requires its original checked function and HIR node".into());
        }
        let expression = canonical;
        if !checked.expr_adjustments(expression).is_empty()
            || checked.type_dependent_def_id(expression.hir_id).is_some()
        {
            return Err("literal compiler adjustment or overload is not implemented".into());
        }
        let (literal, negative) = match expression.kind {
            hir::ExprKind::Lit(literal) => (literal, false),
            hir::ExprKind::Unary(hir::UnOp::Neg, operand) => {
                let hir::ExprKind::Lit(literal) = operand.kind else {
                    return Err("only negative scalar literals are implemented".into());
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
            (LitKind::Float(..), ty::Float(ty::FloatTy::F64)) => {
                let evaluated = tcx
                    .at(expression.span)
                    .lit_to_const(ty::LitToConstInput {
                        lit: literal.node,
                        ty: Some(checked.expr_ty(expression)),
                        neg: negative,
                    })
                    .ok_or("compiler could not evaluate f64 literal")?;
                let scalar = tcx
                    .valtree_to_const_val(evaluated)
                    .try_to_scalar_int()
                    .ok_or("compiler f64 literal is not scalar bits")?;
                if scalar.size().bytes() != 8 {
                    return Err("compiler f64 literal width differs".into());
                }
                LiteralValue::F64(
                    FiniteBinary64::from_bits(scalar.to_u64())
                        .map_err(|_| "nonfinite f64 literals are not implemented")?,
                )
            }
            _ => return Err("literal is not implemented".into()),
        };
        Ok(Self {
            value,
            _expression: expression,
        })
    }

    pub(crate) fn require_context(
        &self,
        tcx: TyCtxt<'tcx>,
        checked: &TypeckResults<'tcx>,
    ) -> Result<(), String> {
        if std::ptr::eq(checked, tcx.typeck(self._expression.hir_id.owner.def_id)) {
            Ok(())
        } else {
            Err("literal mapping requires its original checked function".into())
        }
    }

    pub(crate) fn value(self) -> LiteralValue {
        self.value
    }
}

#[cfg(binary64_ast_probe)]
#[path = "../../test/binary64_input_probe.rs"]
mod binary64_input_probe;
