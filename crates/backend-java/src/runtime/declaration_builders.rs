//! Typed runtime declaration names, types, parameters, and components.

use crate::ast::{
    JavaIdentifier, JavaKnownType, JavaParameter, JavaRecordComponent, JavaRecordComponentOrigin,
    JavaRuntimeMember, JavaType,
};

pub(super) fn identifier(value: &str) -> JavaIdentifier {
    JavaIdentifier::from_portable(value)
}
pub(super) fn type_variable(value: &str) -> JavaType {
    JavaType::TypeVariable(identifier(value))
}
pub(super) fn generic(raw: JavaKnownType, arguments: Vec<JavaType>) -> JavaType {
    JavaType::generic(raw, arguments)
}
pub(super) fn component(
    ty: JavaType,
    name: &str,
    runtime_member: JavaRuntimeMember,
) -> JavaRecordComponent {
    JavaRecordComponent {
        origin: JavaRecordComponentOrigin::Runtime(runtime_member),
        ty,
        name: identifier(name),
    }
}
pub(super) fn parameter(ty: JavaType, name: &str) -> JavaParameter {
    JavaParameter {
        ty,
        name: identifier(name),
        final_parameter: true,
    }
}
