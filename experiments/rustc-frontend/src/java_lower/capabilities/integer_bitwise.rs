//! Ordinary Java operators on exact int/long representations.
use super::{BitwiseInput, BitwiseOperands, BitwiseOperator, IntegerBitwise, Mapping};
use crate::java_lower::{Reader, Result, TypePlan, Value};
use portable_backend_java::ast::{
    JavaBinaryOperator, JavaExpr, JavaExprKind, JavaPrecedence, JavaUnaryOperator,
};

#[derive(Clone, Copy)]
pub(crate) struct JavaIntegerBitwise;

impl Mapping for JavaIntegerBitwise {
    type Capability = IntegerBitwise;
    type Context<'tcx> = Reader<'tcx>;
    type Output = Value;

    fn lower<'tcx>(&self, reader: &mut Reader<'tcx>, input: BitwiseInput<'tcx>) -> Result<Value> {
        let (plan, precedence, kind) = match input.operands() {
            BitwiseOperands::Complement(operand) => {
                let operand = reader.expr(operand)?;
                (
                    operand.plan().clone(),
                    JavaPrecedence::Unary,
                    JavaExprKind::Unary {
                        operator: JavaUnaryOperator::BitNot,
                        operand: Box::new(operand.into_expression()),
                    },
                )
            }
            BitwiseOperands::Binary(operator, left, right) => {
                let left = reader.expr(left)?;
                let left = reader.materialize(left)?;
                let right = reader.expr(right)?;
                let right = reader.materialize(right)?;
                if left.plan() != right.plan() {
                    return Err("integer bitwise operand representations differ".into());
                }
                let (operator, precedence) = match operator {
                    BitwiseOperator::And => (JavaBinaryOperator::BitAnd, JavaPrecedence::BitAnd),
                    BitwiseOperator::Or => (JavaBinaryOperator::BitOr, JavaPrecedence::BitOr),
                    BitwiseOperator::Xor => (JavaBinaryOperator::BitXor, JavaPrecedence::BitXor),
                };
                (
                    left.plan().clone(),
                    precedence,
                    JavaExprKind::Binary {
                        operator,
                        left: Box::new(left.into_expression()),
                        right: Box::new(right.into_expression()),
                    },
                )
            }
        };
        if !matches!(plan, TypePlan::I32 | TypePlan::I64) {
            return Err("integer bitwise requires an exact integer representation".into());
        }
        let result = Value::new(
            plan.clone(),
            JavaExpr {
                ty: plan.java_type(),
                precedence,
                kind,
            },
        )?;
        #[cfg(bitwise_ast_probe)]
        super::bitwise_ast::check(reader, &input, &result.clone().into_expression());
        Ok(result)
    }
}
