//! Ordered source operands become exact, ordinary double operator nodes.
use super::{ArithmeticInput, FloatingArithmetic, FloatingArithmeticOperator, Mapping};
use crate::c_lower::{Reader, Result, c};
use portable_backend_c::ast::{CBinaryOperator, CObjectTypeKind, CScalarType, CValue};

#[derive(Clone, Copy)]
pub(crate) struct CFloatingArithmetic;
impl Mapping for CFloatingArithmetic {
    type Capability = FloatingArithmetic;
    type Context<'tcx> = Reader<'tcx>;
    type Output = CValue;
    fn lower<'tcx>(
        &self,
        reader: &mut Reader<'tcx>,
        input: ArithmeticInput<'tcx>,
    ) -> Result<CValue> {
        input.require_context(reader.tcx, reader.checked)?;
        #[cfg(arithmetic_ast_probe)]
        let start = reader.prelude.len();
        let left = reader.expr(input.left())?;
        let left = reader.materialize(left)?;
        #[cfg(arithmetic_ast_probe)]
        let middle = reader.prelude.len();
        let right = reader.expr(input.right())?;
        let right = reader.materialize(right)?;
        if left.ty().kind() != &CObjectTypeKind::Scalar(CScalarType::F64) || right.ty() != left.ty()
        {
            return Err("floating arithmetic requires exact F64 operands".into());
        }
        let operator = match input.operator() {
            FloatingArithmeticOperator::Add => CBinaryOperator::Add,
            FloatingArithmeticOperator::Subtract => CBinaryOperator::Subtract,
            FloatingArithmeticOperator::Multiply => CBinaryOperator::Multiply,
            FloatingArithmeticOperator::Divide => CBinaryOperator::Divide,
        };
        let result = c(reader.expressions().binary(operator, left, right))?;
        #[cfg(arithmetic_ast_probe)]
        super::arithmetic_ast::observe(reader, &input, &result, start, middle);
        Ok(result)
    }
}
