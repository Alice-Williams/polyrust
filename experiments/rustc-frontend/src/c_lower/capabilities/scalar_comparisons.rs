//! Exact scalar comparison admission; references and overloaded operators reject.
use super::{ComparisonInput, Mapping, ScalarComparisons};
use crate::c_lower::{Reader, Result, c};
use portable_backend_c::ast::{CBinaryOperator, CScalarType, CValue};
use rustc_hir as hir;
use rustc_middle::ty;

#[derive(Clone, Copy)]
pub(crate) struct CScalarComparisons;

impl Mapping for CScalarComparisons {
    type Capability = ScalarComparisons;
    type Context<'tcx> = Reader<'tcx>;
    type Output = CValue;

    fn lower<'tcx>(
        &self,
        reader: &mut Reader<'tcx>,
        input: ComparisonInput<'tcx>,
    ) -> Result<CValue> {
        let expression = input.0;
        let hir::ExprKind::Binary(operator, left, right) = expression.kind else {
            return Err("comparison capability received an unsupported source shape".into());
        };
        if reader
            .checked
            .type_dependent_def_id(expression.hir_id)
            .is_some()
        {
            return Err("overloaded operators are not implemented".into());
        }
        if !reader.checked.expr_adjustments(expression).is_empty() {
            return Err("comparison compiler adjustment is not implemented".into());
        }
        for operand in [left, right] {
            if !matches!(
                reader.checked.expr_ty_adjusted(operand).kind(),
                ty::Int(ty::IntTy::I32 | ty::IntTy::I64) | ty::Bool | ty::Float(ty::FloatTy::F64)
            ) {
                return Err("only scalar comparisons are implemented".into());
            }
        }
        let operator = match operator.node {
            hir::BinOpKind::Eq => CBinaryOperator::Equal,
            hir::BinOpKind::Ne => CBinaryOperator::NotEqual,
            hir::BinOpKind::Lt => CBinaryOperator::Less,
            hir::BinOpKind::Le => CBinaryOperator::LessEqual,
            hir::BinOpKind::Gt => CBinaryOperator::Greater,
            hir::BinOpKind::Ge => CBinaryOperator::GreaterEqual,
            _ => return Err("only comparison binary operators are implemented".into()),
        };
        // Calls on either side materialize into the source-ordered prelude.
        #[cfg(binary64_ast_probe)]
        let start = reader.prelude.len();
        let left = reader.expr(left)?;
        let right = reader.expr(right)?;
        let result = c(reader.expressions().binary(operator, left, right))?;
        let result = c(reader
            .expressions()
            .numeric_conversion(CScalarType::Bool, result))?;
        #[cfg(binary64_ast_probe)]
        super::binary64_ast::comparison(reader, expression, &result, start);
        Ok(result)
    }
}
