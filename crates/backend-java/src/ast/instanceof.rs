//! Reference type tests and ownership-preserving pattern bindings.

use super::JavaExpr;
use super::invocations::{
    java_instanceof_is_legal, java_type_is_reifiable, value_transfer_preserves_isolation,
};
use super::types::{JavaPrimitive, JavaType, JavaTypeUse, type_error};
use crate::dialect::JavaDialect;
use portable_codegen::{AstViolation, TargetAstContext};

pub(super) fn verify(
    result: &JavaType,
    value: &JavaExpr,
    target: &JavaType,
    binds: bool,
    context: &TargetAstContext<'_, JavaDialect>,
) -> Vec<AstViolation> {
    let mut violations = value.verify(context);
    violations.extend(target.verify(JavaTypeUse::Value));
    if *result != JavaType::Primitive(JavaPrimitive::Boolean) {
        violations.push(type_error("instanceof result must be boolean"));
    }
    if !java_type_is_reifiable(target) {
        violations.push(type_error("Java instanceof target must be reifiable"));
    }
    if !java_instanceof_is_legal(&value.ty, target, context) {
        violations.push(type_error(&format!(
            "Java instanceof is not legal from {:?} to {target:?}",
            value.ty,
        )));
    }
    if binds && !value_transfer_preserves_isolation(&value.ty, target) {
        violations.push(type_error(
            "Java pattern binding cannot erase or manufacture internal mutable references",
        ));
    }
    violations
}
