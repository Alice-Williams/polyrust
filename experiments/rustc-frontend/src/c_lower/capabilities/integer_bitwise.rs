//! Exact-width bit operators; calls are materialized by expression lowering.
use super::{BitwiseInput, BitwiseOperands, BitwiseOperator, IntegerBitwise, Mapping};
use crate::c_lower::{Reader, Result, c};
use portable_backend_c::ast::{
    CBinaryOperator, CObjectTypeKind, CScalarType, CUnaryOperator, CValue,
};

#[derive(Clone, Copy)]
pub(crate) struct CIntegerBitwise;

impl Mapping for CIntegerBitwise {
    type Capability = IntegerBitwise;
    type Context<'tcx> = Reader<'tcx>;
    type Output = CValue;

    fn lower<'tcx>(&self, reader: &mut Reader<'tcx>, input: BitwiseInput<'tcx>) -> Result<CValue> {
        let result = match input.operands() {
            BitwiseOperands::Complement(operand) => {
                let operand = reader.expr(operand)?;
                c(reader.expressions().unary(CUnaryOperator::BitNot, operand))
            }
            BitwiseOperands::Binary(operator, left, right) => {
                let left = reader.expr(left)?;
                let right = reader.expr(right)?;
                let operator = match operator {
                    BitwiseOperator::And => CBinaryOperator::BitAnd,
                    BitwiseOperator::Or => CBinaryOperator::BitOr,
                    BitwiseOperator::Xor => CBinaryOperator::BitXor,
                };
                c(reader.expressions().binary(operator, left, right))
            }
        }?;
        // C promotes int32_t to int. Reattach the exact source-width spelling
        // through the existing identity conversion; never narrow a wide value.
        let result = if matches!(
            result.ty().kind(),
            CObjectTypeKind::Scalar(CScalarType::Int)
        ) {
            c(reader
                .expressions()
                .numeric_conversion(CScalarType::I32, result))?
        } else {
            result
        };
        #[cfg(bitwise_ast_probe)]
        super::bitwise_ast::check(reader, &input, &result);
        Ok(result)
    }
}
