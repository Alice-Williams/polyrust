//! Map checked scalar values to typed Java literals without token interpretation.
use super::{LiteralInput, LiteralValues, Mapping};
use crate::java_lower::{Reader, Result, TypePlan, Value};
use crate::source_capabilities::LiteralValue;
use portable_backend_java::ast::{JavaExpr, JavaLiteral};

#[derive(Clone, Copy)]
pub(crate) struct JavaLiteralValues;

impl Mapping for JavaLiteralValues {
    type Capability = LiteralValues;
    type Context<'tcx> = Reader<'tcx>;
    type Output = Value;

    fn lower<'tcx>(&self, _: &mut Reader<'tcx>, input: LiteralInput<'tcx>) -> Result<Value> {
        let (plan, literal) = match input.value() {
            LiteralValue::I32(value) => (TypePlan::I32, JavaLiteral::I32(value)),
            LiteralValue::I64(value) => (TypePlan::I64, JavaLiteral::I64(value)),
            LiteralValue::Bool(value) => (TypePlan::Bool, JavaLiteral::Boolean(value)),
        };
        Value::new(plan.clone(), JavaExpr::literal(plan.java_type(), literal))
    }
}
