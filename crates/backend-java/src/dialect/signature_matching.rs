//! Java dialect: signature matching.

use crate::ast::{JavaKnownType, JavaMethodSignature, JavaPrimitive, JavaType, JavaTypeName};
use std::collections::BTreeMap;

pub(super) fn signature_matches(
    pattern: &JavaMethodSignature,
    actual: &JavaMethodSignature,
) -> bool {
    if pattern.parameters.len() != actual.parameters.len()
        || pattern.checked_exceptions != actual.checked_exceptions
        || pattern.nullable_result != actual.nullable_result
        || pattern.pure != actual.pure
    {
        return false;
    }
    let mut bindings = BTreeMap::<String, JavaType>::new();
    let receiver_matches = match (&pattern.receiver, &actual.receiver) {
        (None, None) => true,
        (Some(pattern), Some(actual)) => type_pattern_matches(
            pattern,
            actual,
            JavaSignaturePosition::Receiver,
            &mut bindings,
        ),
        _ => false,
    };
    receiver_matches
        && pattern
            .parameters
            .iter()
            .zip(&actual.parameters)
            .all(|(pattern, actual)| {
                type_pattern_matches(
                    pattern,
                    actual,
                    JavaSignaturePosition::Parameter,
                    &mut bindings,
                )
            })
        && type_pattern_matches(
            &pattern.result,
            &actual.result,
            JavaSignaturePosition::Result,
            &mut bindings,
        )
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum JavaSignaturePosition {
    Receiver,
    Parameter,
    Result,
    TypeArgument,
}

pub(super) fn type_pattern_matches(
    pattern: &JavaType,
    actual: &JavaType,
    position: JavaSignaturePosition,
    bindings: &mut BTreeMap<String, JavaType>,
) -> bool {
    match pattern {
        JavaType::Reference(JavaTypeName::Known(JavaKnownType::Object)) => match position {
            JavaSignaturePosition::Receiver => matches!(
                actual,
                JavaType::Boxed(_)
                    | JavaType::Reference(_)
                    | JavaType::Array { .. }
                    | JavaType::Generic { .. }
                    | JavaType::TypeVariable(_)
            ),
            JavaSignaturePosition::Parameter => !matches!(
                actual,
                JavaType::Primitive(JavaPrimitive::Void)
                    | JavaType::Boxed(JavaPrimitive::Void)
                    | JavaType::Wildcard { .. }
            ),
            JavaSignaturePosition::Result | JavaSignaturePosition::TypeArgument => {
                pattern == actual
            }
        },
        JavaType::Boxed(pattern) => match position {
            JavaSignaturePosition::Parameter => matches!(
                actual,
                JavaType::Boxed(actual) | JavaType::Primitive(actual) if pattern == actual
            ),
            _ => matches!(actual, JavaType::Boxed(actual) if pattern == actual),
        },
        JavaType::Primitive(pattern) => match position {
            JavaSignaturePosition::Parameter => matches!(
                actual,
                JavaType::Primitive(actual) | JavaType::Boxed(actual) if pattern == actual
            ),
            _ => matches!(actual, JavaType::Primitive(actual) if pattern == actual),
        },
        JavaType::TypeVariable(name) => match bindings.get(name.as_str()) {
            Some(bound) => match position {
                JavaSignaturePosition::Parameter => invocation_types_match(bound, actual),
                JavaSignaturePosition::Result => type_variable_result_matches(bound, actual),
                JavaSignaturePosition::Receiver | JavaSignaturePosition::TypeArgument => {
                    bound == actual
                }
            },
            None => {
                let bound = if position == JavaSignaturePosition::Parameter {
                    actual.clone().boxed()
                } else {
                    actual.clone()
                };
                bindings.insert(name.as_str().to_owned(), bound);
                true
            }
        },
        JavaType::Array {
            component,
            ownership,
        } => matches!(
            actual,
            JavaType::Array { component: actual_component, ownership: actual_ownership }
                if ownership == actual_ownership
                    && type_pattern_matches(
                        component,
                        actual_component,
                        JavaSignaturePosition::TypeArgument,
                        bindings,
                    )
        ),
        JavaType::Generic { raw, arguments } => matches!(
            actual,
            JavaType::Generic { raw: actual_raw, arguments: actual_arguments }
                if (raw == actual_raw
                    || (matches!(
                        position,
                        JavaSignaturePosition::Receiver | JavaSignaturePosition::Parameter
                    ) && matches!(
                        (raw, actual_raw),
                        (
                            JavaTypeName::Known(JavaKnownType::List),
                            JavaTypeName::Known(JavaKnownType::ArrayList)
                        )
                    )))
                    && arguments.len() == actual_arguments.len()
                    && arguments.iter().zip(actual_arguments).all(|(pattern, actual)|
                        type_pattern_matches(
                            pattern,
                            actual,
                            JavaSignaturePosition::TypeArgument,
                            bindings,
                        ))
        ),
        _ => pattern == actual,
    }
}

fn type_variable_result_matches(bound: &JavaType, actual: &JavaType) -> bool {
    bound == actual
        || matches!(
            (bound, actual),
            (JavaType::Boxed(bound), JavaType::Primitive(actual)) if bound == actual
        )
        || matches!(
            (bound, actual),
            (
                JavaType::Wildcard { bound: None },
                JavaType::Reference(JavaTypeName::Known(JavaKnownType::Object))
            )
        )
        || matches!(
            (bound, actual),
            (
                JavaType::Wildcard {
                    bound: Some((crate::ast::JavaWildcardBound::Extends, upper)),
                },
                actual,
            ) if upper.as_ref() == actual
        )
        || matches!(
            (bound, actual),
            (
                JavaType::Wildcard {
                    bound: Some((crate::ast::JavaWildcardBound::Super, _)),
                },
                JavaType::Reference(JavaTypeName::Known(JavaKnownType::Object)),
            )
        )
}

pub(super) fn invocation_types_match(left: &JavaType, right: &JavaType) -> bool {
    left == right
        || matches!(
            (left, right),
            (
                JavaType::Wildcard { bound: None },
                JavaType::Reference(JavaTypeName::Known(JavaKnownType::Object))
            )
        )
        || matches!(
            (left, right),
            (JavaType::Primitive(left), JavaType::Boxed(right))
                | (JavaType::Boxed(left), JavaType::Primitive(right))
                if left == right
        )
}
