//! Java AST: runtime members.

use super::expression_model::JavaMethodSignature;
use super::operator_signatures::invocation_types_match;
use super::types::{JavaArrayOwnership, JavaKnownType, JavaPrimitive, JavaType, JavaTypeName};

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum JavaRuntimeMember {
    SemanticEquals,
    DeepEquals,
    ScalarValue,
    ErrorCode,
    ErrorMessage,
    ResultOk,
    ResultValue,
    ResultError,
    OptionSome,
    OptionValue,
    ValueResultOk,
    ValueResultValue,
    ValueResultError,
    BytesValues,
}

impl JavaRuntimeMember {
    pub const ALL: [Self; 14] = [
        Self::SemanticEquals,
        Self::DeepEquals,
        Self::ScalarValue,
        Self::ErrorCode,
        Self::ErrorMessage,
        Self::ResultOk,
        Self::ResultValue,
        Self::ResultError,
        Self::OptionSome,
        Self::OptionValue,
        Self::ValueResultOk,
        Self::ValueResultValue,
        Self::ValueResultError,
        Self::BytesValues,
    ];

    pub const fn name(self) -> &'static str {
        match self {
            Self::SemanticEquals => "semanticEquals",
            Self::DeepEquals => "deepEquals",
            Self::ScalarValue | Self::ResultValue | Self::OptionValue | Self::ValueResultValue => {
                "value"
            }
            Self::ErrorCode => "code",
            Self::ErrorMessage => "message",
            Self::ResultOk | Self::ValueResultOk => "ok",
            Self::ResultError | Self::ValueResultError => "error",
            Self::OptionSome => "some",
            Self::BytesValues => "values",
        }
    }

    pub(super) fn accepts(self, signature: &JavaMethodSignature) -> bool {
        if !signature.checked_exceptions.is_empty() || signature.nullable_result || !signature.pure
        {
            return false;
        }
        let Some(receiver) = signature.receiver.as_ref() else {
            return false;
        };
        let boolean = JavaType::primitive(JavaPrimitive::Boolean);
        let int = JavaType::primitive(JavaPrimitive::Int);
        let string = JavaType::known(JavaKnownType::String);
        let object = JavaType::known(JavaKnownType::Object);
        let error = JavaType::known(JavaKnownType::RuntimeError);
        let no_arguments = signature.parameters.is_empty();
        match self {
            Self::SemanticEquals | Self::DeepEquals => {
                *receiver == JavaType::known(JavaKnownType::RuntimeSemanticValue)
                    && signature.parameters == [object]
                    && signature.result == boolean
            }
            Self::ScalarValue => {
                *receiver == JavaType::known(JavaKnownType::RuntimeScalar)
                    && no_arguments
                    && signature.result == int
            }
            Self::ErrorCode | Self::ErrorMessage => {
                *receiver == error && no_arguments && signature.result == string
            }
            Self::ResultOk => {
                runtime_generic_arguments(receiver, JavaKnownType::RuntimeResult, 1).is_some()
                    && no_arguments
                    && signature.result == boolean
            }
            Self::ResultValue => {
                runtime_generic_arguments(receiver, JavaKnownType::RuntimeResult, 1).is_some_and(
                    |arguments| {
                        no_arguments && invocation_types_match(&arguments[0], &signature.result)
                    },
                )
            }
            Self::ResultError => {
                runtime_generic_arguments(receiver, JavaKnownType::RuntimeResult, 1).is_some()
                    && no_arguments
                    && signature.result == error
            }
            Self::OptionSome => {
                runtime_generic_arguments(receiver, JavaKnownType::RuntimeOption, 1).is_some()
                    && no_arguments
                    && signature.result == boolean
            }
            Self::OptionValue => {
                runtime_generic_arguments(receiver, JavaKnownType::RuntimeOption, 1).is_some_and(
                    |arguments| {
                        no_arguments && invocation_types_match(&arguments[0], &signature.result)
                    },
                )
            }
            Self::ValueResultOk => {
                runtime_generic_arguments(receiver, JavaKnownType::RuntimeValueResult, 2).is_some()
                    && no_arguments
                    && signature.result == boolean
            }
            Self::ValueResultValue => {
                runtime_generic_arguments(receiver, JavaKnownType::RuntimeValueResult, 2)
                    .is_some_and(|arguments| {
                        no_arguments && invocation_types_match(&arguments[0], &signature.result)
                    })
            }
            Self::ValueResultError => {
                runtime_generic_arguments(receiver, JavaKnownType::RuntimeValueResult, 2)
                    .is_some_and(|arguments| {
                        no_arguments && invocation_types_match(&arguments[1], &signature.result)
                    })
            }
            Self::BytesValues => {
                *receiver == JavaType::known(JavaKnownType::RuntimeBytes)
                    && no_arguments
                    && signature.result
                        == JavaType::Array {
                            component: Box::new(JavaType::primitive(JavaPrimitive::Byte)),
                            ownership: JavaArrayOwnership::DefensiveCopyBoundary,
                        }
            }
        }
    }
}

fn runtime_generic_arguments(
    receiver: &JavaType,
    expected: JavaKnownType,
    arity: usize,
) -> Option<&[JavaType]> {
    match receiver {
        JavaType::Generic {
            raw: JavaTypeName::Known(actual),
            arguments,
        } if *actual == expected && arguments.len() == arity => Some(arguments),
        _ => None,
    }
}
