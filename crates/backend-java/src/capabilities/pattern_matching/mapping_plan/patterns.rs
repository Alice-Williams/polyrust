//! Exact roots and bindings introduced by each pattern variant.
use super::super::*;
use crate::ast::{JavaCallableRef, JavaValueRef};
pub(super) fn verify(input: &JavaPatternInput, output: &JavaLoweredPattern) -> bool {
    if output.condition.ty != JavaType::primitive(JavaPrimitive::Boolean) {
        return false;
    }
    match input {
        JavaPatternInput::Wildcard => output.bindings.is_empty() && output.condition == bool_literal(true),
        JavaPatternInput::Bool { value, .. } => output.bindings.is_empty() && matches!(&output.condition.kind,
            JavaExprKind::Binary { operator: JavaBinaryOperator::Equal, right, .. } if right.as_ref() == &bool_literal(*value)),
        JavaPatternInput::EnumVariant { variant_type, variant_name, bindings, .. } => {
            matches!(&output.condition.kind, JavaExprKind::InstanceOf { target, binding: Some(name), .. }
                if target == variant_type && name == &identifier(variant_name))
                && bindings.len() == output.bindings.len() && bindings.iter().zip(&output.bindings).all(|(i, a)| {
                    let JavaStmt::Local { finality: JavaLocalFinality::Final, ty, name, value: Some(value) } = a else { return false; };
                    ty == &i.binding_type && name == &identifier(&i.binding_name) && value.ty == i.field_type
                        && matches!(&value.kind, JavaExprKind::Call { callable: JavaCallableRef::Member {
                            owner, name, origin: JavaMemberOrigin::GeneratedField(field), .. }, receiver: Some(receiver), arguments }
                            if owner == variant_type && name == &identifier(&i.field_name) && *field == i.field && arguments.is_empty()
                                && receiver.ty == *variant_type && receiver.kind == JavaExprKind::Value(JavaValueRef::Local(identifier(variant_name))))
                })
        }
        JavaPatternInput::None { .. } => output.bindings.is_empty() && negated_call(&output.condition, JavaRuntimeCallable::OptionIsSome),
        JavaPatternInput::Some { binding_name, binding_type, .. } =>
            tagged(output, binding_name, binding_type, JavaRuntimeCallable::OptionIsSome, JavaRuntimeCallable::OptionValue, false),
        JavaPatternInput::Ok { binding_name, binding_type, .. } =>
            tagged(output, binding_name, binding_type, JavaRuntimeCallable::ValueResultIsOk, JavaRuntimeCallable::ValueResultValue, false),
        JavaPatternInput::Err { binding_name, binding_type, .. } =>
            tagged(output, binding_name, binding_type, JavaRuntimeCallable::ValueResultIsOk, JavaRuntimeCallable::ValueResultError, true),
    }
}
fn is_call(value: &JavaExpr, expected: JavaRuntimeCallable) -> bool {
    matches!(&value.kind, JavaExprKind::Call {
        callable: JavaCallableRef::Runtime { callable, .. }, receiver: None, arguments,
    } if *callable == expected && arguments.len() == 1)
}
fn negated_call(value: &JavaExpr, expected: JavaRuntimeCallable) -> bool {
    matches!(&value.kind, JavaExprKind::Unary { operator: JavaUnaryOperator::Not, operand }
        if operand.ty == JavaType::primitive(JavaPrimitive::Boolean) && is_call(operand, expected))
}
fn tagged(
    output: &JavaLoweredPattern,
    name: &str,
    ty: &JavaType,
    test: JavaRuntimeCallable,
    get: JavaRuntimeCallable,
    negated: bool,
) -> bool {
    let condition = if negated {
        negated_call(&output.condition, test)
    } else {
        is_call(&output.condition, test)
    };
    condition
        && matches!(output.bindings.as_slice(), [
        JavaStmt::Local { finality: JavaLocalFinality::Final, ty: actual_ty, name: actual_name, value: Some(value) },
    ] if actual_ty == ty && actual_name == &identifier(name) && &value.ty == ty && is_call(value, get))
}
