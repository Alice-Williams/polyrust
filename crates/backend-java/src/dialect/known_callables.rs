//! Java dialect: known callables.

use super::signature_builder::signature;
use super::signature_matching::{invocation_types_match, signature_matches};
use crate::ast::{
    JavaArrayOwnership, JavaIdentifier, JavaKnownType, JavaMethodSignature, JavaPrimitive,
    JavaType, JavaTypeName,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum JavaKnownCallable {
    ObjectsDeepEquals,
    ObjectsRequireNonNull,
    DoubleToRawLongBits,
    DoubleFromLongBits,
    DoubleIsNaN,
    MathFloor,
    MathCeil,
    ListCopyOf,
    ListOf,
    BigIntegerValueOf,
    ByteToUnsignedInt,
    ByteBufferWrap,
    CharacterIsHighSurrogate,
    CharacterIsLowSurrogate,
    CharacterCharCount,
}

impl JavaKnownCallable {
    pub const ALL: [Self; 15] = [
        Self::ObjectsDeepEquals,
        Self::ObjectsRequireNonNull,
        Self::DoubleToRawLongBits,
        Self::DoubleFromLongBits,
        Self::DoubleIsNaN,
        Self::MathFloor,
        Self::MathCeil,
        Self::ListCopyOf,
        Self::ListOf,
        Self::BigIntegerValueOf,
        Self::ByteToUnsignedInt,
        Self::ByteBufferWrap,
        Self::CharacterIsHighSurrogate,
        Self::CharacterIsLowSurrogate,
        Self::CharacterCharCount,
    ];

    pub const fn owner(self) -> JavaKnownType {
        match self {
            Self::ObjectsDeepEquals | Self::ObjectsRequireNonNull => JavaKnownType::Objects,
            Self::DoubleToRawLongBits | Self::DoubleFromLongBits | Self::DoubleIsNaN => {
                JavaKnownType::Double
            }
            Self::MathFloor | Self::MathCeil => JavaKnownType::Math,
            Self::ListCopyOf | Self::ListOf => JavaKnownType::List,
            Self::BigIntegerValueOf => JavaKnownType::BigInteger,
            Self::ByteToUnsignedInt => JavaKnownType::Byte,
            Self::ByteBufferWrap => JavaKnownType::ByteBuffer,
            Self::CharacterIsHighSurrogate
            | Self::CharacterIsLowSurrogate
            | Self::CharacterCharCount => JavaKnownType::Character,
        }
    }

    pub const fn name(self) -> &'static str {
        match self {
            Self::ObjectsDeepEquals => "deepEquals",
            Self::ObjectsRequireNonNull => "requireNonNull",
            Self::DoubleToRawLongBits => "doubleToRawLongBits",
            Self::DoubleFromLongBits => "longBitsToDouble",
            Self::DoubleIsNaN => "isNaN",
            Self::MathFloor => "floor",
            Self::MathCeil => "ceil",
            Self::ListCopyOf => "copyOf",
            Self::ListOf => "of",
            Self::BigIntegerValueOf => "valueOf",
            Self::ByteToUnsignedInt => "toUnsignedInt",
            Self::ByteBufferWrap => "wrap",
            Self::CharacterIsHighSurrogate => "isHighSurrogate",
            Self::CharacterIsLowSurrogate => "isLowSurrogate",
            Self::CharacterCharCount => "charCount",
        }
    }

    pub const fn qualified_name(self) -> &'static str {
        match self {
            Self::ObjectsDeepEquals => "java.util.Objects.deepEquals",
            Self::ObjectsRequireNonNull => "java.util.Objects.requireNonNull",
            Self::DoubleToRawLongBits => "java.lang.Double.doubleToRawLongBits",
            Self::DoubleFromLongBits => "java.lang.Double.longBitsToDouble",
            Self::DoubleIsNaN => "java.lang.Double.isNaN",
            Self::MathFloor => "java.lang.Math.floor",
            Self::MathCeil => "java.lang.Math.ceil",
            Self::ListCopyOf => "java.util.List.copyOf",
            Self::ListOf => "java.util.List.of",
            Self::BigIntegerValueOf => "java.math.BigInteger.valueOf",
            Self::ByteToUnsignedInt => "java.lang.Byte.toUnsignedInt",
            Self::ByteBufferWrap => "java.nio.ByteBuffer.wrap",
            Self::CharacterIsHighSurrogate => "java.lang.Character.isHighSurrogate",
            Self::CharacterIsLowSurrogate => "java.lang.Character.isLowSurrogate",
            Self::CharacterCharCount => "java.lang.Character.charCount",
        }
    }

    pub fn signature(self) -> JavaMethodSignature {
        let object = JavaType::known(JavaKnownType::Object);
        let boolean = JavaType::primitive(JavaPrimitive::Boolean);
        let long = JavaType::primitive(JavaPrimitive::Long);
        let double = JavaType::primitive(JavaPrimitive::Double);
        let t = JavaType::TypeVariable(JavaIdentifier::from_portable("T"));
        let list_t = JavaType::generic(JavaKnownType::List, vec![t.clone()]);
        match self {
            Self::ObjectsDeepEquals => signature(None, vec![object.clone(), object], boolean),
            Self::ObjectsRequireNonNull => signature(None, vec![t.clone()], t),
            Self::DoubleToRawLongBits => signature(None, vec![double], long),
            Self::DoubleFromLongBits => signature(None, vec![long], double),
            Self::DoubleIsNaN => signature(None, vec![double], boolean),
            Self::MathFloor | Self::MathCeil => signature(None, vec![double.clone()], double),
            Self::ListCopyOf => signature(None, vec![list_t.clone()], list_t),
            Self::ListOf => signature(None, vec![], list_t),
            Self::BigIntegerValueOf => {
                signature(None, vec![long], JavaType::known(JavaKnownType::BigInteger))
            }
            Self::ByteToUnsignedInt => signature(
                None,
                vec![JavaType::primitive(JavaPrimitive::Byte)],
                JavaType::primitive(JavaPrimitive::Int),
            ),
            Self::ByteBufferWrap => signature(
                None,
                vec![JavaType::Array {
                    component: Box::new(JavaType::primitive(JavaPrimitive::Byte)),
                    ownership: JavaArrayOwnership::DefensiveCopyBoundary,
                }],
                JavaType::known(JavaKnownType::ByteBuffer),
            ),
            Self::CharacterIsHighSurrogate | Self::CharacterIsLowSurrogate => signature(
                None,
                vec![JavaType::primitive(JavaPrimitive::Char)],
                boolean,
            ),
            Self::CharacterCharCount => signature(
                None,
                vec![JavaType::primitive(JavaPrimitive::Int)],
                JavaType::primitive(JavaPrimitive::Int),
            ),
        }
    }

    pub fn accepts(self, signature: &JavaMethodSignature) -> bool {
        if signature.receiver.is_some()
            || !signature.checked_exceptions.is_empty()
            || signature.nullable_result
            || !signature.pure
        {
            return false;
        }
        match self {
            Self::ListCopyOf => match (&signature.parameters[..], &signature.result) {
                (
                    [JavaType::Generic { raw, arguments }],
                    JavaType::Generic {
                        raw: JavaTypeName::Known(JavaKnownType::List),
                        arguments: result_arguments,
                    },
                ) if matches!(
                    raw,
                    JavaTypeName::Known(JavaKnownType::List | JavaKnownType::ArrayList)
                ) && arguments.len() == 1
                    && result_arguments.len() == 1 =>
                {
                    invocation_types_match(&arguments[0], &result_arguments[0])
                }
                _ => false,
            },
            Self::ListOf => match &signature.result {
                JavaType::Generic {
                    raw: JavaTypeName::Known(JavaKnownType::List),
                    arguments,
                } if arguments.len() == 1 => signature
                    .parameters
                    .iter()
                    .all(|parameter| invocation_types_match(parameter, &arguments[0])),
                _ => false,
            },
            _ => signature_matches(&self.signature(), signature),
        }
    }
}
