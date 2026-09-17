//! Exact primitive Double unary minus, never zero subtraction or a helper.
use super::{FloatingInput, FloatingNegation, Mapping};
use crate::java_lower::{Reader, Result, TypePlan, Value};
use portable_backend_java::ast::{JavaExpr, JavaExprKind, JavaPrecedence, JavaUnaryOperator};
#[derive(Clone, Copy)]
pub(crate) struct JavaFloatingNegation;
impl Mapping for JavaFloatingNegation {
    type Capability = FloatingNegation;
    type Context<'tcx> = Reader<'tcx>;
    type Output = Value;
    fn lower<'tcx>(&self, reader: &mut Reader<'tcx>, input: FloatingInput<'tcx>) -> Result<Value> {
        input.require_context(reader.tcx, reader.checked)?;
        #[cfg(floating_ast_probe)]
        let start = reader.prelude.len();
        let operand = reader.expr(input.operand())?;
        let operand = reader.materialize(operand)?;
        if operand.plan() != &TypePlan::F64 {
            return Err("floating negation Java representation differs from checked f64".into());
        }
        let result = Value::new(
            TypePlan::F64,
            JavaExpr {
                ty: TypePlan::F64.java_type(),
                precedence: JavaPrecedence::Unary,
                kind: JavaExprKind::Unary {
                    operator: JavaUnaryOperator::Negate,
                    operand: Box::new(operand.into_expression()),
                },
            },
        )?;
        #[cfg(floating_ast_probe)]
        super::floating_ast::check(reader, &input, &result.clone().into_expression(), start);
        Ok(result)
    }
}
