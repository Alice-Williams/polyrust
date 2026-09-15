//! C Boolean bits promote to Int; restore the exact source Boolean result.
use super::{EagerBooleanInput, EagerBooleanOperator, EagerBooleans, Mapping};
use crate::c_lower::{Reader, Result, c};
use portable_backend_c::ast::{CBinaryOperator, CScalarType, CValue};

#[derive(Clone, Copy)]
pub(crate) struct CEagerBooleans;

impl Mapping for CEagerBooleans {
    type Capability = EagerBooleans;
    type Context<'tcx> = Reader<'tcx>;
    type Output = CValue;

    fn lower<'tcx>(
        &self,
        reader: &mut Reader<'tcx>,
        input: EagerBooleanInput<'tcx>,
    ) -> Result<CValue> {
        // Reader materializes every call. Never move right lowering into a
        // conditional: even a decisive left value must evaluate the right.
        let left = reader.expr(input.left())?;
        let right = reader.expr(input.right())?;
        let operator = match input.operator() {
            EagerBooleanOperator::And => CBinaryOperator::BitAnd,
            EagerBooleanOperator::Or => CBinaryOperator::BitOr,
            EagerBooleanOperator::Xor => CBinaryOperator::BitXor,
        };
        let operation = c(reader.expressions().binary(operator, left, right))?;
        let result = c(reader
            .expressions()
            .numeric_conversion(CScalarType::Bool, operation))?;
        #[cfg(eager_ast_probe)]
        super::eager_ast::check(&input, &result);
        Ok(result)
    }
}
