//! Exact primitive Java negation; a source receiver is evaluated once.
use super::{Mapping, WrappingInput, WrappingNegation, WrappingWidth};
use crate::java_lower::{Reader, Result, TypePlan, Value};
use portable_backend_java::ast::{JavaExpr, JavaExprKind, JavaPrecedence, JavaUnaryOperator};

#[derive(Clone, Copy)]
pub(crate) struct JavaWrappingNegation;
impl Mapping for JavaWrappingNegation {
    type Capability = WrappingNegation;
    type Context<'tcx> = Reader<'tcx>;
    type Output = Value;

    fn lower<'tcx>(&self, reader: &mut Reader<'tcx>, input: WrappingInput<'tcx>) -> Result<Value> {
        input.require_context(reader.tcx, reader.checked)?;
        #[cfg(wrapping_ast_probe)]
        let start = reader.prelude.len();
        let operand = reader.expr(input.receiver())?;
        let operand = reader.materialize(operand)?;
        let plan = match input.width() {
            WrappingWidth::I32 => TypePlan::I32,
            WrappingWidth::I64 => TypePlan::I64,
        };
        if operand.plan() != &plan {
            return Err("wrapping negation Java representation differs from checked width".into());
        }
        let result = Value::new(
            plan.clone(),
            JavaExpr {
                ty: plan.java_type(),
                precedence: JavaPrecedence::Unary,
                kind: JavaExprKind::Unary {
                    operator: JavaUnaryOperator::Negate,
                    operand: Box::new(operand.into_expression()),
                },
            },
        )?;
        #[cfg(wrapping_ast_probe)]
        super::wrapping_ast::check(reader, &input, &result.clone().into_expression(), start);
        Ok(result)
    }
}
