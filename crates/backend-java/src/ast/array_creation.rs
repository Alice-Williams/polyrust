//! Allocation-specific Java array validity, separate from array value types.

use super::invocations::java_type_is_reifiable;
use super::types::type_error;
use super::{JavaArrayOwnership, JavaExpr, JavaPrimitive, JavaType, JavaTypeUse};
use crate::dialect::JavaDialect;
use portable_codegen::{AstViolation, TargetAstContext};

pub(super) fn verify(
    result: &JavaType,
    component: &JavaType,
    length: &JavaExpr,
    context: &TargetAstContext<'_, JavaDialect>,
) -> Vec<AstViolation> {
    let mut errors = component.verify(JavaTypeUse::Value);
    errors.extend(length.verify(context));
    if !java_type_is_reifiable(component) {
        errors.push(type_error(
            "new-array component must be reifiable; generic array creation is invalid Java",
        ));
    }
    let expected = JavaType::Array {
        component: Box::new(component.clone()),
        ownership: JavaArrayOwnership::InternalMutable,
    };
    if *result != expected || length.ty != JavaType::Primitive(JavaPrimitive::Int) {
        errors.push(type_error(
            "new-array component, length, or result type mismatch",
        ));
    }
    errors
}
