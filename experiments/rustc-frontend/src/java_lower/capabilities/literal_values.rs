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

    fn lower<'tcx>(&self, reader: &mut Reader<'tcx>, input: LiteralInput<'tcx>) -> Result<Value> {
        input.require_context(reader.tcx, reader.checked)?;
        let (plan, literal) = match input.value() {
            LiteralValue::I32(value) => (TypePlan::I32, JavaLiteral::I32(value)),
            LiteralValue::I64(value) => (TypePlan::I64, JavaLiteral::I64(value)),
            LiteralValue::Bool(value) => (TypePlan::Bool, JavaLiteral::Boolean(value)),
            LiteralValue::Char(value) => (
                TypePlan::Char,
                JavaLiteral::I32(
                    i32::try_from(u32::from(value)).map_err(|_| "scalar exceeds Java Int")?,
                ),
            ),
            LiteralValue::F64(value) => (TypePlan::F64, JavaLiteral::F64(value)),
        };
        let value = Value::new(plan.clone(), JavaExpr::literal(plan.java_type(), literal))?;
        #[cfg(binary64_ast_probe)]
        super::binary64_ast::check(reader, input, &value);
        Ok(value)
    }
}
