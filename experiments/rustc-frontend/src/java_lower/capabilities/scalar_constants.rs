//! Fold checked constants into ordinary Java primitive literal nodes.
use super::{ConstantInput, Mapping, ScalarConstants};
use crate::java_lower::{Reader, Result, TypePlan, Value};
use crate::source_capabilities::ScalarConstantValue;
use portable_backend_java::ast::{JavaExpr, JavaLiteral};

#[derive(Clone, Copy)]
pub(crate) struct JavaScalarConstants;

impl Mapping for JavaScalarConstants {
    type Capability = ScalarConstants;
    type Context<'tcx> = Reader<'tcx>;
    type Output = Value;

    fn lower<'tcx>(&self, _reader: &mut Reader<'tcx>, input: ConstantInput<'tcx>) -> Result<Value> {
        let (plan, literal) = match input.value() {
            ScalarConstantValue::I32(value) => (TypePlan::I32, JavaLiteral::I32(value)),
            ScalarConstantValue::I64(value) => (TypePlan::I64, JavaLiteral::I64(value)),
            ScalarConstantValue::Bool(value) => (TypePlan::Bool, JavaLiteral::Boolean(value)),
            ScalarConstantValue::F64(value) => (TypePlan::F64, JavaLiteral::F64(value)),
        };
        let value = Value::new(plan.clone(), JavaExpr::literal(plan.java_type(), literal))?;
        #[cfg(constant_ast_probe)]
        super::constant_ast::check(_reader, input, &value);
        Ok(value)
    }
}
