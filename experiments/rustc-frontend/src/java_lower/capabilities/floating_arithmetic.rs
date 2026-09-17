//! Left-to-right materialization and exact structural Double arithmetic.
use super::{ArithmeticInput, FloatingArithmetic, FloatingArithmeticOperator, Mapping};
use crate::java_lower::{Reader, Result, TypePlan, Value};
use portable_backend_java::ast::{JavaBinaryOperator, JavaExpr, JavaExprKind, JavaPrecedence};

#[derive(Clone, Copy)]
pub(crate) struct JavaFloatingArithmetic;
impl Mapping for JavaFloatingArithmetic {
    type Capability = FloatingArithmetic;
    type Context<'tcx> = Reader<'tcx>;
    type Output = Value;
    fn lower<'tcx>(
        &self,
        reader: &mut Reader<'tcx>,
        input: ArithmeticInput<'tcx>,
    ) -> Result<Value> {
        input.require_context(reader.tcx, reader.checked)?;
        #[cfg(arithmetic_ast_probe)]
        let start = reader.prelude.len();
        let left = reader.expr(input.left())?;
        let left = reader.materialize(left)?;
        #[cfg(arithmetic_ast_probe)]
        let middle = reader.prelude.len();
        let right = reader.expr(input.right())?;
        let right = reader.materialize(right)?;
        if left.plan() != &TypePlan::F64 || right.plan() != left.plan() {
            return Err("floating arithmetic requires exact Double operands".into());
        }
        let (operator, precedence) = match input.operator() {
            FloatingArithmeticOperator::Add => (JavaBinaryOperator::Add, JavaPrecedence::Additive),
            FloatingArithmeticOperator::Subtract => {
                (JavaBinaryOperator::Subtract, JavaPrecedence::Additive)
            }
            FloatingArithmeticOperator::Multiply => {
                (JavaBinaryOperator::Multiply, JavaPrecedence::Multiplicative)
            }
            FloatingArithmeticOperator::Divide => {
                (JavaBinaryOperator::Divide, JavaPrecedence::Multiplicative)
            }
        };
        let result = Value::new(
            TypePlan::F64,
            JavaExpr {
                ty: TypePlan::F64.java_type(),
                precedence,
                kind: JavaExprKind::Binary {
                    operator,
                    left: Box::new(left.into_expression()),
                    right: Box::new(right.into_expression()),
                },
            },
        )?;
        #[cfg(arithmetic_ast_probe)]
        super::arithmetic_ast::observe(
            reader,
            &input,
            &result.clone().into_expression(),
            start,
            middle,
        );
        Ok(result)
    }
}
