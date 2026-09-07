//! Java dialect: known methods.

use super::member_names::JavaMemberName;
use super::signature_builder::signature;
use super::signature_matching::signature_matches;
use crate::ast::{
    JavaArrayOwnership, JavaIdentifier, JavaKnownType, JavaMethodSignature, JavaPrimitive, JavaType,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum JavaKnownMethod {
    ObjectEquals,
    StringLength,
    StringIsEmpty,
    StringContains,
    StringStartsWith,
    StringEndsWith,
    StringSubstringFrom,
    StringSubstringRange,
    StringReplace,
    StringIndexOfString,
    StringIndexOfCodePoint,
    StringCodePointCount,
    StringOffsetByCodePoints,
    StringCharAt,
    StringCodePointAt,
    StringCodePointBefore,
    StringGetBytes,
    ListSize,
    ListIsEmpty,
    ListGet,
    ListSubList,
    ArrayListAdd,
    ArrayListAddAll,
    BigIntegerCompareTo,
    BigIntegerIntValue,
    BigIntegerLongValue,
    BigIntegerSignum,
    BigIntegerNegate,
    BigIntegerAdd,
    BigIntegerSubtract,
    BigIntegerMultiply,
    BigIntegerDivide,
    BigIntegerRemainder,
    BigIntegerShiftLeft,
    BigIntegerShiftRight,
    CharsetNewDecoder,
    DecoderOnMalformedInput,
    DecoderOnUnmappableCharacter,
    DecoderDecode,
    CharBufferToString,
}

impl JavaKnownMethod {
    pub const ALL: [Self; 40] = [
        Self::ObjectEquals,
        Self::StringLength,
        Self::StringIsEmpty,
        Self::StringContains,
        Self::StringStartsWith,
        Self::StringEndsWith,
        Self::StringSubstringFrom,
        Self::StringSubstringRange,
        Self::StringReplace,
        Self::StringIndexOfString,
        Self::StringIndexOfCodePoint,
        Self::StringCodePointCount,
        Self::StringOffsetByCodePoints,
        Self::StringCharAt,
        Self::StringCodePointAt,
        Self::StringCodePointBefore,
        Self::StringGetBytes,
        Self::ListSize,
        Self::ListIsEmpty,
        Self::ListGet,
        Self::ListSubList,
        Self::ArrayListAdd,
        Self::ArrayListAddAll,
        Self::BigIntegerCompareTo,
        Self::BigIntegerIntValue,
        Self::BigIntegerLongValue,
        Self::BigIntegerSignum,
        Self::BigIntegerNegate,
        Self::BigIntegerAdd,
        Self::BigIntegerSubtract,
        Self::BigIntegerMultiply,
        Self::BigIntegerDivide,
        Self::BigIntegerRemainder,
        Self::BigIntegerShiftLeft,
        Self::BigIntegerShiftRight,
        Self::CharsetNewDecoder,
        Self::DecoderOnMalformedInput,
        Self::DecoderOnUnmappableCharacter,
        Self::DecoderDecode,
        Self::CharBufferToString,
    ];

    pub const fn owner(self) -> JavaKnownType {
        match self {
            Self::ObjectEquals => JavaKnownType::Object,
            Self::StringLength
            | Self::StringIsEmpty
            | Self::StringContains
            | Self::StringStartsWith
            | Self::StringEndsWith
            | Self::StringSubstringFrom
            | Self::StringSubstringRange
            | Self::StringReplace
            | Self::StringIndexOfString
            | Self::StringIndexOfCodePoint
            | Self::StringCodePointCount
            | Self::StringOffsetByCodePoints
            | Self::StringCharAt
            | Self::StringCodePointAt
            | Self::StringCodePointBefore
            | Self::StringGetBytes => JavaKnownType::String,
            Self::ListSize | Self::ListIsEmpty | Self::ListGet | Self::ListSubList => {
                JavaKnownType::List
            }
            Self::ArrayListAdd | Self::ArrayListAddAll => JavaKnownType::ArrayList,
            Self::BigIntegerCompareTo
            | Self::BigIntegerIntValue
            | Self::BigIntegerLongValue
            | Self::BigIntegerSignum
            | Self::BigIntegerNegate
            | Self::BigIntegerAdd
            | Self::BigIntegerSubtract
            | Self::BigIntegerMultiply
            | Self::BigIntegerDivide
            | Self::BigIntegerRemainder
            | Self::BigIntegerShiftLeft
            | Self::BigIntegerShiftRight => JavaKnownType::BigInteger,
            Self::CharsetNewDecoder => JavaKnownType::Charset,
            Self::DecoderOnMalformedInput
            | Self::DecoderOnUnmappableCharacter
            | Self::DecoderDecode => JavaKnownType::CharsetDecoder,
            Self::CharBufferToString => JavaKnownType::CharBuffer,
        }
    }

    pub const fn name(self) -> JavaMemberName {
        match self {
            Self::ObjectEquals => JavaMemberName::Equals,
            Self::StringLength => JavaMemberName::Length,
            Self::StringIsEmpty => JavaMemberName::IsEmpty,
            Self::StringContains => JavaMemberName::Contains,
            Self::StringStartsWith => JavaMemberName::StartsWith,
            Self::StringEndsWith => JavaMemberName::EndsWith,
            Self::StringSubstringFrom | Self::StringSubstringRange => JavaMemberName::Substring,
            Self::StringReplace => JavaMemberName::Replace,
            Self::StringIndexOfString | Self::StringIndexOfCodePoint => JavaMemberName::IndexOf,
            Self::StringCodePointCount => JavaMemberName::CodePointCount,
            Self::StringOffsetByCodePoints => JavaMemberName::OffsetByCodePoints,
            Self::StringCharAt => JavaMemberName::CharAt,
            Self::StringCodePointAt => JavaMemberName::CodePointAt,
            Self::StringCodePointBefore => JavaMemberName::CodePointBefore,
            Self::StringGetBytes => JavaMemberName::GetBytes,
            Self::ListSize => JavaMemberName::Size,
            Self::ListIsEmpty => JavaMemberName::IsEmpty,
            Self::ListGet => JavaMemberName::Get,
            Self::ListSubList => JavaMemberName::SubList,
            Self::ArrayListAdd => JavaMemberName::Add,
            Self::ArrayListAddAll => JavaMemberName::AddAll,
            Self::BigIntegerCompareTo => JavaMemberName::CompareTo,
            Self::BigIntegerIntValue => JavaMemberName::IntValue,
            Self::BigIntegerLongValue => JavaMemberName::LongValue,
            Self::BigIntegerSignum => JavaMemberName::Signum,
            Self::BigIntegerNegate => JavaMemberName::Negate,
            Self::BigIntegerAdd => JavaMemberName::Add,
            Self::BigIntegerSubtract => JavaMemberName::Subtract,
            Self::BigIntegerMultiply => JavaMemberName::Multiply,
            Self::BigIntegerDivide => JavaMemberName::Divide,
            Self::BigIntegerRemainder => JavaMemberName::Remainder,
            Self::BigIntegerShiftLeft => JavaMemberName::ShiftLeft,
            Self::BigIntegerShiftRight => JavaMemberName::ShiftRight,
            Self::CharsetNewDecoder => JavaMemberName::NewDecoder,
            Self::DecoderOnMalformedInput => JavaMemberName::OnMalformedInput,
            Self::DecoderOnUnmappableCharacter => JavaMemberName::OnUnmappableCharacter,
            Self::DecoderDecode => JavaMemberName::Decode,
            Self::CharBufferToString => JavaMemberName::ToString,
        }
    }

    pub fn signature(self) -> JavaMethodSignature {
        let object = JavaType::known(JavaKnownType::Object);
        let string = JavaType::known(JavaKnownType::String);
        let boolean = JavaType::primitive(JavaPrimitive::Boolean);
        let byte = JavaType::primitive(JavaPrimitive::Byte);
        let character = JavaType::primitive(JavaPrimitive::Char);
        let int = JavaType::primitive(JavaPrimitive::Int);
        let long = JavaType::primitive(JavaPrimitive::Long);
        let bigint = JavaType::known(JavaKnownType::BigInteger);
        let charset = JavaType::known(JavaKnownType::Charset);
        let decoder = JavaType::known(JavaKnownType::CharsetDecoder);
        let byte_buffer = JavaType::known(JavaKnownType::ByteBuffer);
        let char_buffer = JavaType::known(JavaKnownType::CharBuffer);
        let coding_action = JavaType::known(JavaKnownType::CodingErrorAction);
        let t = JavaType::TypeVariable(JavaIdentifier::from_portable("T"));
        let list_t = JavaType::generic(JavaKnownType::List, vec![t.clone()]);
        let array_list_t = JavaType::generic(JavaKnownType::ArrayList, vec![t.clone()]);
        let bytes = JavaType::Array {
            component: Box::new(byte),
            ownership: JavaArrayOwnership::InternalMutable,
        };
        match self {
            Self::ObjectEquals => signature(Some(object.clone()), vec![object], boolean),
            Self::StringLength => signature(Some(string), vec![], int),
            Self::StringIsEmpty => signature(Some(string), vec![], boolean),
            Self::StringContains | Self::StringStartsWith | Self::StringEndsWith => {
                signature(Some(string.clone()), vec![string], boolean)
            }
            Self::StringSubstringFrom => signature(Some(string.clone()), vec![int], string),
            Self::StringSubstringRange => {
                signature(Some(string.clone()), vec![int.clone(), int], string)
            }
            Self::StringReplace => signature(
                Some(string.clone()),
                vec![string.clone(), string.clone()],
                string,
            ),
            Self::StringIndexOfString => signature(
                Some(string),
                vec![JavaType::known(JavaKnownType::String)],
                int,
            ),
            Self::StringIndexOfCodePoint => signature(Some(string), vec![int.clone()], int),
            Self::StringCodePointCount | Self::StringOffsetByCodePoints => {
                signature(Some(string), vec![int.clone(), int.clone()], int)
            }
            Self::StringCharAt => signature(Some(string), vec![int], character),
            Self::StringCodePointAt | Self::StringCodePointBefore => {
                signature(Some(string), vec![int.clone()], int)
            }
            Self::StringGetBytes => signature(Some(string), vec![charset], bytes),
            Self::ListSize => signature(Some(list_t), vec![], int),
            Self::ListIsEmpty => signature(Some(list_t), vec![], boolean),
            Self::ListGet => signature(Some(list_t), vec![int], t),
            Self::ListSubList => signature(Some(list_t.clone()), vec![int.clone(), int], list_t),
            Self::ArrayListAdd => signature(Some(array_list_t), vec![t], boolean),
            Self::ArrayListAddAll => signature(Some(array_list_t), vec![list_t], boolean),
            Self::BigIntegerCompareTo => signature(Some(bigint.clone()), vec![bigint], int),
            Self::BigIntegerIntValue => signature(Some(bigint), vec![], int),
            Self::BigIntegerLongValue => signature(Some(bigint), vec![], long),
            Self::BigIntegerSignum => signature(Some(bigint), vec![], int),
            Self::BigIntegerNegate => signature(Some(bigint.clone()), vec![], bigint),
            Self::BigIntegerAdd
            | Self::BigIntegerSubtract
            | Self::BigIntegerMultiply
            | Self::BigIntegerDivide
            | Self::BigIntegerRemainder => {
                signature(Some(bigint.clone()), vec![bigint.clone()], bigint)
            }
            Self::BigIntegerShiftLeft | Self::BigIntegerShiftRight => {
                signature(Some(bigint.clone()), vec![int], bigint)
            }
            Self::CharsetNewDecoder => signature(Some(charset), vec![], decoder),
            Self::DecoderOnMalformedInput | Self::DecoderOnUnmappableCharacter => {
                JavaMethodSignature {
                    receiver: Some(decoder.clone()),
                    parameters: vec![coding_action],
                    result: decoder,
                    checked_exceptions: vec![],
                    nullable_result: false,
                    pure: false,
                }
            }
            Self::DecoderDecode => JavaMethodSignature {
                receiver: Some(decoder),
                parameters: vec![byte_buffer],
                result: char_buffer,
                checked_exceptions: vec![JavaKnownType::CharacterCodingException],
                nullable_result: false,
                pure: false,
            },
            Self::CharBufferToString => signature(Some(char_buffer), vec![], string),
        }
    }

    pub fn accepts(self, actual: &JavaMethodSignature) -> bool {
        signature_matches(&self.signature(), actual)
    }
}
