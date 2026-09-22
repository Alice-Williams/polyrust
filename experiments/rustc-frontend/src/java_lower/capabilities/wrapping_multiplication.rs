//! Ordered original operands become primitive modular multiplication, without helpers.
use super::{Mapping, MultiplicationInput, MultiplicationWidth, WrappingMultiplication};
use crate::java_lower::{Reader, Result, TypePlan, Value};
use portable_backend_java::ast::{JavaBinaryOperator, JavaExpr, JavaExprKind, JavaPrecedence};

#[derive(Clone, Copy)]
pub(crate) struct JavaWrappingMultiplication;
impl Mapping for JavaWrappingMultiplication {
    type Capability = WrappingMultiplication;
    type Context<'tcx> = Reader<'tcx>;
    type Output = Value;
    fn lower<'tcx>(
        &self,
        reader: &mut Reader<'tcx>,
        input: MultiplicationInput<'tcx>,
    ) -> Result<Value> {
        input.require_context(reader.tcx, reader.checked)?;
        #[cfg(multiplication_ast_probe)]
        let start = reader.prelude.len();
        let left = reader.expr(input.left())?;
        let left = reader.materialize(left)?;
        #[cfg(multiplication_ast_probe)]
        let middle = reader.prelude.len();
        let right = reader.expr(input.right())?;
        let right = reader.materialize(right)?;
        let plan = match input.width() {
            MultiplicationWidth::I32 => TypePlan::I32,
            MultiplicationWidth::I64 => TypePlan::I64,
        };
        if left.plan() != &plan || right.plan() != &plan {
            return Err(
                "wrapping multiplication Java representation differs from checked width".into(),
            );
        }
        let result = Value::new(
            plan.clone(),
            JavaExpr {
                ty: plan.java_type(),
                precedence: JavaPrecedence::Multiplicative,
                kind: JavaExprKind::Binary {
                    operator: JavaBinaryOperator::Multiply,
                    left: Box::new(left.into_expression()),
                    right: Box::new(right.into_expression()),
                },
            },
        )?;
        #[cfg(multiplication_ast_probe)]
        super::multiplication_ast::observe(
            reader,
            &input,
            &result.clone().into_expression(),
            start,
            middle,
        );
        Ok(result)
    }
}
