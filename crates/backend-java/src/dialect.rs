use std::collections::{BTreeMap, BTreeSet};

use portable_codegen::{
    AstViolation, CallablePattern, DependencyPolicy, FailureBehavior, FileItemRoots,
    GeneratedOrigin, GeneratedSymbolId, KnownCallableSpec, KnownConstructorSpec, KnownFieldSpec,
    KnownMethodSpec, KnownTypeSpec, LinkedFile, LinkedTargetPackage, LinkerDialect,
    PackageEcosystem, ResolvedReference, ResolvedReferenceMap, RuntimeCallableSpec,
    RuntimeHelperSpec, SymbolCatalogue, SymbolOrigin, SynthesisReason, TargetAstContext,
    TargetAstPackage, TargetCallableSignature, TargetDialect, TargetEffect, TargetExprId,
    TargetExpressionNode, TargetFile, TargetStatementNode, TargetSymbolRef, TargetTypeRef,
    TypeParameterSpec, TypePattern, TypedAstDialect, verify_linked_package, verify_target_ast,
};
use portable_diagnostics::{Diagnostic, DiagnosticCode, SourceRef};

use crate::ast::*;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
pub struct JavaDialect;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum JavaRuntimeType {
    Structural,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum JavaConstructedType {
    Array,
    Generic,
    Wildcard,
    TypeVariable,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum JavaInvocationKind {
    Static,
    Instance,
    Constructor,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum JavaSyntheticOrigin {
    Package,
    Test,
    Runtime,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum JavaNamespace {
    Type,
    Value,
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct JavaNameKey(String);

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum JavaPreludeSymbol {
    JavaLang,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum JavaStandardLibrary {
    Jdk21,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum JavaExternalPackage {
    None,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum JavaPackageFeature {
    None,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum JavaImportKind {
    Type(JavaKnownType),
}

impl JavaImportKind {
    pub const fn qualified_name(self) -> &'static str {
        match self {
            Self::Type(value) => value.qualified_name(),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum JavaQualifiedName {
    Type(JavaKnownType),
    Callable(JavaKnownCallable),
    RuntimeCallable(JavaRuntimeCallable),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum JavaGeneratedContainer {
    PublicApi,
}

impl JavaGeneratedContainer {
    pub const fn text(self) -> &'static str {
        match self {
            Self::PublicApi => "Generated",
        }
    }
}

impl JavaQualifiedName {
    pub const fn text(self) -> &'static str {
        match self {
            Self::Type(value) => value.qualified_name(),
            Self::Callable(value) => value.qualified_name(),
            Self::RuntimeCallable(value) => value.qualified_name(),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum JavaMemberName {
    MinValue,
    MaxValue,
    Utf8,
    Report,
    Equals,
    DeepEquals,
    RequireNonNull,
    DoubleToRawLongBits,
    LongBitsToDouble,
    IsNaN,
    Floor,
    Ceil,
    CopyOf,
    Of,
    Length,
    IsEmpty,
    Contains,
    StartsWith,
    EndsWith,
    Substring,
    Replace,
    ValueOf,
    ToUnsignedInt,
    Wrap,
    IsHighSurrogate,
    IsLowSurrogate,
    CharCount,
    CompareTo,
    IntValue,
    LongValue,
    Signum,
    Negate,
    Add,
    Subtract,
    Multiply,
    Divide,
    Remainder,
    ShiftLeft,
    ShiftRight,
    IndexOf,
    CodePointCount,
    OffsetByCodePoints,
    CharAt,
    CodePointAt,
    CodePointBefore,
    GetBytes,
    Size,
    Get,
    SubList,
    AddAll,
    NewDecoder,
    OnMalformedInput,
    OnUnmappableCharacter,
    Decode,
    ToString,
}

impl JavaMemberName {
    pub const fn text(self) -> &'static str {
        match self {
            Self::MinValue => "MIN_VALUE",
            Self::MaxValue => "MAX_VALUE",
            Self::Utf8 => "UTF_8",
            Self::Report => "REPORT",
            Self::Equals => "equals",
            Self::DeepEquals => "deepEquals",
            Self::RequireNonNull => "requireNonNull",
            Self::DoubleToRawLongBits => "doubleToRawLongBits",
            Self::LongBitsToDouble => "longBitsToDouble",
            Self::IsNaN => "isNaN",
            Self::Floor => "floor",
            Self::Ceil => "ceil",
            Self::CopyOf => "copyOf",
            Self::Of => "of",
            Self::Length => "length",
            Self::IsEmpty => "isEmpty",
            Self::Contains => "contains",
            Self::StartsWith => "startsWith",
            Self::EndsWith => "endsWith",
            Self::Substring => "substring",
            Self::Replace => "replace",
            Self::ValueOf => "valueOf",
            Self::ToUnsignedInt => "toUnsignedInt",
            Self::Wrap => "wrap",
            Self::IsHighSurrogate => "isHighSurrogate",
            Self::IsLowSurrogate => "isLowSurrogate",
            Self::CharCount => "charCount",
            Self::CompareTo => "compareTo",
            Self::IntValue => "intValue",
            Self::LongValue => "longValue",
            Self::Signum => "signum",
            Self::Negate => "negate",
            Self::Add => "add",
            Self::Subtract => "subtract",
            Self::Multiply => "multiply",
            Self::Divide => "divide",
            Self::Remainder => "remainder",
            Self::ShiftLeft => "shiftLeft",
            Self::ShiftRight => "shiftRight",
            Self::IndexOf => "indexOf",
            Self::CodePointCount => "codePointCount",
            Self::OffsetByCodePoints => "offsetByCodePoints",
            Self::CharAt => "charAt",
            Self::CodePointAt => "codePointAt",
            Self::CodePointBefore => "codePointBefore",
            Self::GetBytes => "getBytes",
            Self::Size => "size",
            Self::Get => "get",
            Self::SubList => "subList",
            Self::AddAll => "addAll",
            Self::NewDecoder => "newDecoder",
            Self::OnMalformedInput => "onMalformedInput",
            Self::OnUnmappableCharacter => "onUnmappableCharacter",
            Self::Decode => "decode",
            Self::ToString => "toString",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum JavaKnownField {
    IntegerMinValue,
    IntegerMaxValue,
    LongMinValue,
    LongMaxValue,
    StandardCharsetsUtf8,
    CodingErrorReport,
}

impl JavaKnownField {
    pub fn owner(self) -> JavaKnownType {
        match self {
            Self::IntegerMinValue | Self::IntegerMaxValue => JavaKnownType::Integer,
            Self::LongMinValue | Self::LongMaxValue => JavaKnownType::Long,
            Self::StandardCharsetsUtf8 => JavaKnownType::StandardCharsets,
            Self::CodingErrorReport => JavaKnownType::CodingErrorAction,
        }
    }

    pub fn member(self) -> JavaMemberName {
        match self {
            Self::IntegerMinValue | Self::LongMinValue => JavaMemberName::MinValue,
            Self::IntegerMaxValue | Self::LongMaxValue => JavaMemberName::MaxValue,
            Self::StandardCharsetsUtf8 => JavaMemberName::Utf8,
            Self::CodingErrorReport => JavaMemberName::Report,
        }
    }

    pub fn ty(self) -> JavaType {
        match self {
            Self::IntegerMinValue | Self::IntegerMaxValue => {
                JavaType::primitive(JavaPrimitive::Int)
            }
            Self::LongMinValue | Self::LongMaxValue => JavaType::primitive(JavaPrimitive::Long),
            Self::StandardCharsetsUtf8 => JavaType::known(JavaKnownType::Charset),
            Self::CodingErrorReport => JavaType::known(JavaKnownType::CodingErrorAction),
        }
    }
}

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

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum JavaKnownConstructor {
    AssertionErrorString,
    IllegalArgumentExceptionString,
    IllegalStateExceptionString,
    ArrayList,
    ArrayListFromList,
    RuntimeError,
    RuntimeResult,
    RuntimeOption,
    RuntimeValueResult,
    RuntimeBytes,
    RuntimeScalar,
    RuntimeUnit,
}

impl JavaKnownConstructor {
    pub const ALL: [Self; 12] = [
        Self::AssertionErrorString,
        Self::IllegalArgumentExceptionString,
        Self::IllegalStateExceptionString,
        Self::ArrayList,
        Self::RuntimeError,
        Self::RuntimeResult,
        Self::RuntimeOption,
        Self::RuntimeValueResult,
        Self::RuntimeBytes,
        Self::RuntimeScalar,
        Self::RuntimeUnit,
        Self::ArrayListFromList,
    ];

    pub fn owner(self) -> JavaKnownType {
        match self {
            Self::AssertionErrorString => JavaKnownType::AssertionError,
            Self::IllegalArgumentExceptionString => JavaKnownType::IllegalArgumentException,
            Self::IllegalStateExceptionString => JavaKnownType::IllegalStateException,
            Self::ArrayList | Self::ArrayListFromList => JavaKnownType::ArrayList,
            Self::RuntimeError => JavaKnownType::RuntimeError,
            Self::RuntimeResult => JavaKnownType::RuntimeResult,
            Self::RuntimeOption => JavaKnownType::RuntimeOption,
            Self::RuntimeValueResult => JavaKnownType::RuntimeValueResult,
            Self::RuntimeBytes => JavaKnownType::RuntimeBytes,
            Self::RuntimeScalar => JavaKnownType::RuntimeScalar,
            Self::RuntimeUnit => JavaKnownType::RuntimeUnit,
        }
    }

    pub fn signature(self) -> (JavaType, Vec<JavaType>) {
        let t = JavaType::TypeVariable(JavaIdentifier::from_portable("T"));
        let e = JavaType::TypeVariable(JavaIdentifier::from_portable("E"));
        let owner = match self {
            Self::ArrayList | Self::ArrayListFromList => {
                JavaType::generic(JavaKnownType::ArrayList, vec![t.clone()])
            }
            Self::RuntimeResult => JavaType::generic(JavaKnownType::RuntimeResult, vec![t.clone()]),
            Self::RuntimeOption => JavaType::generic(JavaKnownType::RuntimeOption, vec![t.clone()]),
            Self::RuntimeValueResult => JavaType::generic(
                JavaKnownType::RuntimeValueResult,
                vec![t.clone(), e.clone()],
            ),
            _ => JavaType::known(self.owner()),
        };
        let string = JavaType::known(JavaKnownType::String);
        let boolean = JavaType::primitive(JavaPrimitive::Boolean);
        let parameters = match self {
            Self::AssertionErrorString
            | Self::IllegalArgumentExceptionString
            | Self::IllegalStateExceptionString => vec![string],
            Self::ArrayList => vec![],
            Self::ArrayListFromList => {
                vec![JavaType::generic(JavaKnownType::List, vec![t.clone()])]
            }
            Self::RuntimeError => vec![string.clone(), string],
            Self::RuntimeResult => {
                vec![
                    boolean,
                    t.clone(),
                    JavaType::known(JavaKnownType::RuntimeError),
                ]
            }
            Self::RuntimeOption => vec![boolean, t.clone()],
            Self::RuntimeValueResult => vec![boolean, t, e],
            Self::RuntimeBytes => vec![JavaType::Array {
                component: Box::new(JavaType::primitive(JavaPrimitive::Byte)),
                ownership: JavaArrayOwnership::DefensiveCopyBoundary,
            }],
            Self::RuntimeScalar => vec![JavaType::primitive(JavaPrimitive::Int)],
            Self::RuntimeUnit => vec![],
        };
        (owner, parameters)
    }

    pub fn accepts(self, owner: &JavaType, parameters: &[JavaType]) -> bool {
        let (pattern_owner, pattern_parameters) = self.signature();
        if pattern_parameters.len() != parameters.len() {
            return false;
        }
        let mut bindings = BTreeMap::new();
        type_pattern_matches(&pattern_owner, owner, &mut bindings)
            && pattern_parameters
                .iter()
                .zip(parameters)
                .all(|(pattern, actual)| type_pattern_matches(pattern, actual, &mut bindings))
    }
}

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

fn signature_matches(pattern: &JavaMethodSignature, actual: &JavaMethodSignature) -> bool {
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
        (Some(pattern), Some(actual)) => type_pattern_matches(pattern, actual, &mut bindings),
        _ => false,
    };
    receiver_matches
        && pattern
            .parameters
            .iter()
            .zip(&actual.parameters)
            .all(|(pattern, actual)| type_pattern_matches(pattern, actual, &mut bindings))
        && type_pattern_matches(&pattern.result, &actual.result, &mut bindings)
}

fn type_pattern_matches(
    pattern: &JavaType,
    actual: &JavaType,
    bindings: &mut BTreeMap<String, JavaType>,
) -> bool {
    match pattern {
        JavaType::Reference(JavaTypeName::Known(JavaKnownType::Object)) => !matches!(
            actual,
            JavaType::Primitive(JavaPrimitive::Void) | JavaType::Boxed(JavaPrimitive::Void)
        ),
        JavaType::Boxed(pattern) => matches!(
            actual,
            JavaType::Boxed(actual) | JavaType::Primitive(actual) if pattern == actual
        ),
        JavaType::Primitive(pattern) => matches!(
            actual,
            JavaType::Primitive(actual) | JavaType::Boxed(actual) if pattern == actual
        ),
        JavaType::TypeVariable(name) => match bindings.get(name.as_str()) {
            Some(bound) => invocation_types_match(bound, actual),
            None => {
                bindings.insert(name.as_str().to_owned(), actual.clone());
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
                    && type_pattern_matches(component, actual_component, bindings)
        ),
        JavaType::Generic { raw, arguments } => matches!(
            actual,
            JavaType::Generic { raw: actual_raw, arguments: actual_arguments }
                if (raw == actual_raw
                    || matches!(
                        (raw, actual_raw),
                        (
                            JavaTypeName::Known(JavaKnownType::List),
                            JavaTypeName::Known(JavaKnownType::ArrayList)
                        )
                    ))
                    && arguments.len() == actual_arguments.len()
                    && arguments.iter().zip(actual_arguments).all(|(pattern, actual)|
                        type_pattern_matches(pattern, actual, bindings))
        ),
        _ => pattern == actual,
    }
}

fn invocation_types_match(left: &JavaType, right: &JavaType) -> bool {
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

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum JavaRuntimeHelper {
    Core,
    TaggedValues,
    CheckedIntegers,
    FloatBits,
    Unicode,
    Bytes,
    ImmutableLists,
    StringOperations,
    Interfaces,
}

impl JavaRuntimeHelper {
    pub const ALL: [Self; 9] = [
        Self::Core,
        Self::TaggedValues,
        Self::CheckedIntegers,
        Self::FloatBits,
        Self::Unicode,
        Self::Bytes,
        Self::ImmutableLists,
        Self::StringOperations,
        Self::Interfaces,
    ];

    pub const fn name(self) -> &'static str {
        match self {
            Self::Core => "java.runtime.core",
            Self::TaggedValues => "java.runtime.tagged-values",
            Self::CheckedIntegers => "java.runtime.checked-integers",
            Self::FloatBits => "java.runtime.float-bits",
            Self::Unicode => "java.runtime.unicode",
            Self::Bytes => "java.runtime.bytes",
            Self::ImmutableLists => "java.runtime.immutable-lists",
            Self::StringOperations => "java.runtime.string-operations",
            Self::Interfaces => "java.runtime.interfaces",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum JavaHelperCapability {
    Failures,
    TaggedValues,
    CheckedArithmetic,
    ExactFloatBits,
    UnicodeScalars,
    ImmutableBytes,
    ImmutableLists,
    StringOperations,
    InterfaceDispatch,
}

impl JavaHelperCapability {
    pub const fn name(self) -> &'static str {
        match self {
            Self::Failures => "failures",
            Self::TaggedValues => "tagged_values",
            Self::CheckedArithmetic => "checked_arithmetic",
            Self::ExactFloatBits => "exact_float_bits",
            Self::UnicodeScalars => "unicode_scalars",
            Self::ImmutableBytes => "immutable_bytes",
            Self::ImmutableLists => "immutable_lists",
            Self::StringOperations => "string_operations",
            Self::InterfaceDispatch => "interface_dispatch",
        }
    }
}

fn signature(
    receiver: Option<JavaType>,
    parameters: Vec<JavaType>,
    result: JavaType,
) -> JavaMethodSignature {
    JavaMethodSignature {
        receiver,
        parameters,
        result,
        checked_exceptions: vec![],
        nullable_result: false,
        pure: true,
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum JavaRuntimeCallable {
    Ok,
    Fail,
    DeepEqual,
    SemanticEqual,
    ValidatePublicValue,
    RequireScalarString,
    CompareScalarStrings,
    OptionNone,
    OptionSome,
    OptionIsSome,
    OptionValue,
    ValueResultOk,
    ValueResultErr,
    ValueResultIsOk,
    ValueResultValue,
    ValueResultError,
    CheckedNegI32,
    CheckedNegI64,
    CheckedAddI32,
    CheckedAddI64,
    CheckedSubI32,
    CheckedSubI64,
    CheckedMulI32,
    CheckedMulI64,
    CheckedDivI32,
    CheckedDivI64,
    CheckedRemI32,
    CheckedRemI64,
    CheckedShiftLeftI32,
    CheckedShiftLeftI64,
    CheckedShiftRightI32,
    CheckedShiftRightI64,
    NarrowI64ToI32,
    FloatTrunc,
    FloatIsNegativeZero,
    FloatAbs,
    ScalarLength,
    StringIndexOfLiteral,
    StringSliceScalars,
    StringReplaceAll,
    StringReplaceMany,
    StringTruncateUtf8Bytes,
    StringTrimStart,
    StringTrimEnd,
    StringToUtf8,
    StringFromUtf8,
    BytesOf,
    BytesToList,
    BytesLength,
    BytesIsEmpty,
    BytesConcat,
    BytesReplaceAll,
    ListCopy,
    ListLength,
    ListIsEmpty,
    ListGet,
    ListAppend,
    ListConcat,
    ListContains,
    ListIndexOf,
}

impl JavaRuntimeCallable {
    pub const ALL: [Self; 60] = [
        Self::Ok,
        Self::Fail,
        Self::DeepEqual,
        Self::SemanticEqual,
        Self::ValidatePublicValue,
        Self::RequireScalarString,
        Self::CompareScalarStrings,
        Self::OptionNone,
        Self::OptionSome,
        Self::OptionIsSome,
        Self::OptionValue,
        Self::ValueResultOk,
        Self::ValueResultErr,
        Self::ValueResultIsOk,
        Self::ValueResultValue,
        Self::ValueResultError,
        Self::CheckedNegI32,
        Self::CheckedNegI64,
        Self::CheckedAddI32,
        Self::CheckedAddI64,
        Self::CheckedSubI32,
        Self::CheckedSubI64,
        Self::CheckedMulI32,
        Self::CheckedMulI64,
        Self::CheckedDivI32,
        Self::CheckedDivI64,
        Self::CheckedRemI32,
        Self::CheckedRemI64,
        Self::CheckedShiftLeftI32,
        Self::CheckedShiftLeftI64,
        Self::CheckedShiftRightI32,
        Self::CheckedShiftRightI64,
        Self::NarrowI64ToI32,
        Self::FloatTrunc,
        Self::FloatIsNegativeZero,
        Self::FloatAbs,
        Self::ScalarLength,
        Self::StringIndexOfLiteral,
        Self::StringSliceScalars,
        Self::StringReplaceAll,
        Self::StringReplaceMany,
        Self::StringTruncateUtf8Bytes,
        Self::StringTrimStart,
        Self::StringTrimEnd,
        Self::StringToUtf8,
        Self::StringFromUtf8,
        Self::BytesOf,
        Self::BytesToList,
        Self::BytesLength,
        Self::BytesIsEmpty,
        Self::BytesConcat,
        Self::BytesReplaceAll,
        Self::ListCopy,
        Self::ListLength,
        Self::ListIsEmpty,
        Self::ListGet,
        Self::ListAppend,
        Self::ListConcat,
        Self::ListContains,
        Self::ListIndexOf,
    ];

    pub const fn helper(self) -> JavaRuntimeHelper {
        match self {
            Self::Ok
            | Self::Fail
            | Self::DeepEqual
            | Self::SemanticEqual
            | Self::ValidatePublicValue
            | Self::RequireScalarString
            | Self::CompareScalarStrings => JavaRuntimeHelper::Core,
            Self::OptionNone
            | Self::OptionSome
            | Self::OptionIsSome
            | Self::OptionValue
            | Self::ValueResultOk
            | Self::ValueResultErr
            | Self::ValueResultIsOk
            | Self::ValueResultValue
            | Self::ValueResultError => JavaRuntimeHelper::TaggedValues,
            Self::CheckedNegI32
            | Self::CheckedNegI64
            | Self::CheckedAddI32
            | Self::CheckedAddI64
            | Self::CheckedSubI32
            | Self::CheckedSubI64
            | Self::CheckedMulI32
            | Self::CheckedMulI64
            | Self::CheckedDivI32
            | Self::CheckedDivI64
            | Self::CheckedRemI32
            | Self::CheckedRemI64
            | Self::CheckedShiftLeftI32
            | Self::CheckedShiftLeftI64
            | Self::CheckedShiftRightI32
            | Self::CheckedShiftRightI64
            | Self::NarrowI64ToI32 => JavaRuntimeHelper::CheckedIntegers,
            Self::FloatTrunc | Self::FloatIsNegativeZero | Self::FloatAbs => {
                JavaRuntimeHelper::FloatBits
            }
            Self::ScalarLength
            | Self::StringIndexOfLiteral
            | Self::StringSliceScalars
            | Self::StringToUtf8
            | Self::StringFromUtf8 => JavaRuntimeHelper::Unicode,
            Self::StringReplaceAll
            | Self::StringReplaceMany
            | Self::StringTruncateUtf8Bytes
            | Self::StringTrimStart
            | Self::StringTrimEnd => JavaRuntimeHelper::StringOperations,
            Self::BytesOf
            | Self::BytesToList
            | Self::BytesLength
            | Self::BytesIsEmpty
            | Self::BytesConcat
            | Self::BytesReplaceAll => JavaRuntimeHelper::Bytes,
            Self::ListCopy
            | Self::ListLength
            | Self::ListIsEmpty
            | Self::ListGet
            | Self::ListAppend
            | Self::ListConcat
            | Self::ListContains
            | Self::ListIndexOf => JavaRuntimeHelper::ImmutableLists,
        }
    }

    pub const fn name(self) -> &'static str {
        match self {
            Self::Ok => "ok",
            Self::Fail => "fail",
            Self::DeepEqual => "deepEqual",
            Self::SemanticEqual => "semanticEqual",
            Self::ValidatePublicValue => "validatePublicValue",
            Self::RequireScalarString => "requireScalarString",
            Self::CompareScalarStrings => "compareScalarStrings",
            Self::OptionNone => "optionNone",
            Self::OptionSome => "optionSome",
            Self::OptionIsSome => "optionIsSome",
            Self::OptionValue => "optionValue",
            Self::ValueResultOk => "valueResultOk",
            Self::ValueResultErr => "valueResultErr",
            Self::ValueResultIsOk => "valueResultIsOk",
            Self::ValueResultValue => "valueResultValue",
            Self::ValueResultError => "valueResultError",
            Self::CheckedNegI32 => "checkedNegI32",
            Self::CheckedNegI64 => "checkedNegI64",
            Self::CheckedAddI32 => "checkedAddI32",
            Self::CheckedAddI64 => "checkedAddI64",
            Self::CheckedSubI32 => "checkedSubI32",
            Self::CheckedSubI64 => "checkedSubI64",
            Self::CheckedMulI32 => "checkedMulI32",
            Self::CheckedMulI64 => "checkedMulI64",
            Self::CheckedDivI32 => "checkedDivI32",
            Self::CheckedDivI64 => "checkedDivI64",
            Self::CheckedRemI32 => "checkedRemI32",
            Self::CheckedRemI64 => "checkedRemI64",
            Self::CheckedShiftLeftI32 => "checkedShiftLeftI32",
            Self::CheckedShiftLeftI64 => "checkedShiftLeftI64",
            Self::CheckedShiftRightI32 => "checkedShiftRightI32",
            Self::CheckedShiftRightI64 => "checkedShiftRightI64",
            Self::NarrowI64ToI32 => "narrowI64ToI32",
            Self::FloatTrunc => "floatTrunc",
            Self::FloatIsNegativeZero => "floatIsNegativeZero",
            Self::FloatAbs => "floatAbs",
            Self::ScalarLength => "scalarLength",
            Self::StringIndexOfLiteral => "stringIndexOfLiteral",
            Self::StringSliceScalars => "stringSliceScalars",
            Self::StringReplaceAll => "stringReplaceAll",
            Self::StringReplaceMany => "stringReplaceMany",
            Self::StringTruncateUtf8Bytes => "stringTruncateUtf8Bytes",
            Self::StringTrimStart => "stringTrimStart",
            Self::StringTrimEnd => "stringTrimEnd",
            Self::StringToUtf8 => "stringToUtf8",
            Self::StringFromUtf8 => "stringFromUtf8",
            Self::BytesOf => "bytesOf",
            Self::BytesToList => "bytesToList",
            Self::BytesLength => "bytesLength",
            Self::BytesIsEmpty => "bytesIsEmpty",
            Self::BytesConcat => "bytesConcat",
            Self::BytesReplaceAll => "bytesReplaceAll",
            Self::ListCopy => "listCopy",
            Self::ListLength => "listLength",
            Self::ListIsEmpty => "listIsEmpty",
            Self::ListGet => "listGet",
            Self::ListAppend => "listAppend",
            Self::ListConcat => "listConcat",
            Self::ListContains => "listContains",
            Self::ListIndexOf => "listIndexOf",
        }
    }

    pub const fn qualified_name(self) -> &'static str {
        match self {
            Self::Ok => "org.polyrust.generated.Runtime.ok",
            Self::Fail => "org.polyrust.generated.Runtime.fail",
            Self::DeepEqual => "org.polyrust.generated.Runtime.deepEqual",
            Self::SemanticEqual => "org.polyrust.generated.Runtime.semanticEqual",
            Self::ValidatePublicValue => "org.polyrust.generated.Runtime.validatePublicValue",
            Self::RequireScalarString => "org.polyrust.generated.Runtime.requireScalarString",
            Self::CompareScalarStrings => "org.polyrust.generated.Runtime.compareScalarStrings",
            Self::OptionNone => "org.polyrust.generated.Runtime.optionNone",
            Self::OptionSome => "org.polyrust.generated.Runtime.optionSome",
            Self::OptionIsSome => "org.polyrust.generated.Runtime.optionIsSome",
            Self::OptionValue => "org.polyrust.generated.Runtime.optionValue",
            Self::ValueResultOk => "org.polyrust.generated.Runtime.valueResultOk",
            Self::ValueResultErr => "org.polyrust.generated.Runtime.valueResultErr",
            Self::ValueResultIsOk => "org.polyrust.generated.Runtime.valueResultIsOk",
            Self::ValueResultValue => "org.polyrust.generated.Runtime.valueResultValue",
            Self::ValueResultError => "org.polyrust.generated.Runtime.valueResultError",
            Self::CheckedNegI32 => "org.polyrust.generated.Runtime.checkedNegI32",
            Self::CheckedNegI64 => "org.polyrust.generated.Runtime.checkedNegI64",
            Self::CheckedAddI32 => "org.polyrust.generated.Runtime.checkedAddI32",
            Self::CheckedAddI64 => "org.polyrust.generated.Runtime.checkedAddI64",
            Self::CheckedSubI32 => "org.polyrust.generated.Runtime.checkedSubI32",
            Self::CheckedSubI64 => "org.polyrust.generated.Runtime.checkedSubI64",
            Self::CheckedMulI32 => "org.polyrust.generated.Runtime.checkedMulI32",
            Self::CheckedMulI64 => "org.polyrust.generated.Runtime.checkedMulI64",
            Self::CheckedDivI32 => "org.polyrust.generated.Runtime.checkedDivI32",
            Self::CheckedDivI64 => "org.polyrust.generated.Runtime.checkedDivI64",
            Self::CheckedRemI32 => "org.polyrust.generated.Runtime.checkedRemI32",
            Self::CheckedRemI64 => "org.polyrust.generated.Runtime.checkedRemI64",
            Self::CheckedShiftLeftI32 => "org.polyrust.generated.Runtime.checkedShiftLeftI32",
            Self::CheckedShiftLeftI64 => "org.polyrust.generated.Runtime.checkedShiftLeftI64",
            Self::CheckedShiftRightI32 => "org.polyrust.generated.Runtime.checkedShiftRightI32",
            Self::CheckedShiftRightI64 => "org.polyrust.generated.Runtime.checkedShiftRightI64",
            Self::NarrowI64ToI32 => "org.polyrust.generated.Runtime.narrowI64ToI32",
            Self::FloatTrunc => "org.polyrust.generated.Runtime.floatTrunc",
            Self::FloatIsNegativeZero => "org.polyrust.generated.Runtime.floatIsNegativeZero",
            Self::FloatAbs => "org.polyrust.generated.Runtime.floatAbs",
            Self::ScalarLength => "org.polyrust.generated.Runtime.scalarLength",
            Self::StringIndexOfLiteral => "org.polyrust.generated.Runtime.stringIndexOfLiteral",
            Self::StringSliceScalars => "org.polyrust.generated.Runtime.stringSliceScalars",
            Self::StringReplaceAll => "org.polyrust.generated.Runtime.stringReplaceAll",
            Self::StringReplaceMany => "org.polyrust.generated.Runtime.stringReplaceMany",
            Self::StringTruncateUtf8Bytes => {
                "org.polyrust.generated.Runtime.stringTruncateUtf8Bytes"
            }
            Self::StringTrimStart => "org.polyrust.generated.Runtime.stringTrimStart",
            Self::StringTrimEnd => "org.polyrust.generated.Runtime.stringTrimEnd",
            Self::StringToUtf8 => "org.polyrust.generated.Runtime.stringToUtf8",
            Self::StringFromUtf8 => "org.polyrust.generated.Runtime.stringFromUtf8",
            Self::BytesOf => "org.polyrust.generated.Runtime.bytesOf",
            Self::BytesToList => "org.polyrust.generated.Runtime.bytesToList",
            Self::BytesLength => "org.polyrust.generated.Runtime.bytesLength",
            Self::BytesIsEmpty => "org.polyrust.generated.Runtime.bytesIsEmpty",
            Self::BytesConcat => "org.polyrust.generated.Runtime.bytesConcat",
            Self::BytesReplaceAll => "org.polyrust.generated.Runtime.bytesReplaceAll",
            Self::ListCopy => "org.polyrust.generated.Runtime.listCopy",
            Self::ListLength => "org.polyrust.generated.Runtime.listLength",
            Self::ListIsEmpty => "org.polyrust.generated.Runtime.listIsEmpty",
            Self::ListGet => "org.polyrust.generated.Runtime.listGet",
            Self::ListAppend => "org.polyrust.generated.Runtime.listAppend",
            Self::ListConcat => "org.polyrust.generated.Runtime.listConcat",
            Self::ListContains => "org.polyrust.generated.Runtime.listContains",
            Self::ListIndexOf => "org.polyrust.generated.Runtime.listIndexOf",
        }
    }

    /// The single authoritative Java signature pattern for this generated
    /// runtime entry point. Type variables are unified by `signature_matches`.
    pub fn signature(self) -> JavaMethodSignature {
        let t = JavaType::TypeVariable(JavaIdentifier::from_portable("T"));
        let e = JavaType::TypeVariable(JavaIdentifier::from_portable("E"));
        let boolean = JavaType::primitive(JavaPrimitive::Boolean);
        let int = JavaType::primitive(JavaPrimitive::Int);
        let long = JavaType::primitive(JavaPrimitive::Long);
        let double = JavaType::primitive(JavaPrimitive::Double);
        let string = JavaType::known(JavaKnownType::String);
        let object = JavaType::known(JavaKnownType::Object);
        let bytes = JavaType::known(JavaKnownType::RuntimeBytes);
        let result_t = JavaType::generic(JavaKnownType::RuntimeResult, vec![t.clone()]);
        let result_int = JavaType::generic(
            JavaKnownType::RuntimeResult,
            vec![JavaType::Boxed(JavaPrimitive::Int)],
        );
        let result_long = JavaType::generic(
            JavaKnownType::RuntimeResult,
            vec![JavaType::Boxed(JavaPrimitive::Long)],
        );
        let result_string = JavaType::generic(JavaKnownType::RuntimeResult, vec![string.clone()]);
        let option_t = JavaType::generic(JavaKnownType::RuntimeOption, vec![t.clone()]);
        let option_long = JavaType::generic(
            JavaKnownType::RuntimeOption,
            vec![JavaType::Boxed(JavaPrimitive::Long)],
        );
        let value_result = JavaType::generic(
            JavaKnownType::RuntimeValueResult,
            vec![t.clone(), e.clone()],
        );
        let list_t = JavaType::generic(JavaKnownType::List, vec![t.clone()]);
        let integer_list = JavaType::generic(
            JavaKnownType::List,
            vec![JavaType::Boxed(JavaPrimitive::Int)],
        );
        match self {
            Self::Ok => signature(None, vec![t], result_t),
            Self::Fail => signature(None, vec![string.clone(), string], result_t),
            Self::DeepEqual | Self::SemanticEqual => {
                signature(None, vec![object.clone(), object], boolean)
            }
            Self::ValidatePublicValue => signature(None, vec![t.clone()], t),
            Self::RequireScalarString => signature(None, vec![string.clone()], string),
            Self::CompareScalarStrings => signature(None, vec![string.clone(), string], int),
            Self::OptionNone => signature(None, vec![], option_t),
            Self::OptionSome => signature(None, vec![t], option_t),
            Self::OptionIsSome => signature(None, vec![option_t], boolean),
            Self::OptionValue => signature(None, vec![option_t], t),
            Self::ValueResultOk => signature(None, vec![t], value_result),
            Self::ValueResultErr => signature(None, vec![e], value_result),
            Self::ValueResultIsOk => signature(None, vec![value_result], boolean),
            Self::ValueResultValue => signature(None, vec![value_result], t),
            Self::ValueResultError => signature(None, vec![value_result], e),
            Self::CheckedNegI32 => signature(None, vec![int.clone()], result_int),
            Self::CheckedNegI64 => signature(None, vec![long.clone()], result_long),
            Self::CheckedAddI32
            | Self::CheckedSubI32
            | Self::CheckedMulI32
            | Self::CheckedDivI32
            | Self::CheckedRemI32
            | Self::CheckedShiftLeftI32
            | Self::CheckedShiftRightI32 => signature(None, vec![int.clone(), int], result_int),
            Self::CheckedAddI64
            | Self::CheckedSubI64
            | Self::CheckedMulI64
            | Self::CheckedDivI64
            | Self::CheckedRemI64
            | Self::CheckedShiftLeftI64
            | Self::CheckedShiftRightI64 => signature(None, vec![long.clone(), long], result_long),
            Self::NarrowI64ToI32 => signature(None, vec![long], result_int),
            Self::FloatTrunc | Self::FloatAbs => signature(None, vec![double.clone()], double),
            Self::FloatIsNegativeZero => signature(None, vec![double], boolean),
            Self::ScalarLength => signature(None, vec![string], result_long),
            Self::StringIndexOfLiteral => {
                signature(None, vec![string.clone(), string], option_long)
            }
            Self::StringSliceScalars => {
                signature(None, vec![string.clone(), long.clone(), long], string)
            }
            Self::StringReplaceAll => signature(
                None,
                vec![string.clone(), string.clone(), string.clone()],
                string,
            ),
            Self::StringReplaceMany => signature(
                None,
                vec![
                    string.clone(),
                    JavaType::generic(JavaKnownType::List, vec![string.clone()]),
                ],
                string,
            ),
            Self::StringTruncateUtf8Bytes => signature(None, vec![string.clone(), double], string),
            Self::StringTrimStart | Self::StringTrimEnd => {
                signature(None, vec![string.clone(), string.clone()], string)
            }
            Self::StringToUtf8 => signature(None, vec![string], bytes),
            Self::StringFromUtf8 => signature(None, vec![bytes], result_string),
            Self::BytesOf => signature(None, vec![integer_list], bytes),
            Self::BytesToList => signature(None, vec![bytes], integer_list),
            Self::BytesLength => signature(None, vec![bytes], long),
            Self::BytesIsEmpty => signature(None, vec![bytes], boolean),
            Self::BytesConcat => signature(None, vec![bytes.clone(), bytes.clone()], bytes),
            Self::BytesReplaceAll => signature(
                None,
                vec![bytes.clone(), bytes.clone(), bytes.clone()],
                bytes,
            ),
            Self::ListCopy => signature(None, vec![list_t.clone()], list_t),
            Self::ListLength => signature(None, vec![list_t], long),
            Self::ListIsEmpty => signature(None, vec![list_t], boolean),
            Self::ListGet => signature(None, vec![list_t, long], result_t),
            Self::ListAppend => signature(None, vec![list_t.clone(), t], list_t),
            Self::ListConcat => signature(None, vec![list_t.clone(), list_t.clone()], list_t),
            Self::ListContains => signature(None, vec![list_t, t], boolean),
            Self::ListIndexOf => signature(None, vec![list_t, t], option_long),
        }
    }

    pub fn accepts(self, signature: &JavaMethodSignature) -> bool {
        signature_matches(&self.signature(), signature)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum JavaArenaExpression {
    KnownCall {
        callable: JavaKnownCallable,
        arguments: Vec<TargetExprId>,
    },
    KnownConstructor {
        constructor: JavaKnownConstructor,
        arguments: Vec<TargetExprId>,
    },
    KnownMethod {
        method: JavaKnownMethod,
        receiver: TargetExprId,
        arguments: Vec<TargetExprId>,
    },
}

impl TargetExpressionNode<JavaDialect> for JavaArenaExpression {
    fn child_expressions(&self) -> Vec<TargetExprId> {
        match self {
            Self::KnownCall { arguments, .. } | Self::KnownConstructor { arguments, .. } => {
                arguments.clone()
            }
            Self::KnownMethod {
                receiver,
                arguments,
                ..
            } => std::iter::once(*receiver)
                .chain(arguments.iter().copied())
                .collect(),
        }
    }

    fn verify(
        &self,
        _stored_type: &TargetTypeRef<JavaDialect>,
        _context: &TargetAstContext<'_, JavaDialect>,
    ) -> Vec<AstViolation> {
        vec![]
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct JavaArenaStatement(pub Vec<TargetExprId>);

impl TargetStatementNode<JavaDialect> for JavaArenaStatement {
    fn child_expressions(&self) -> Vec<TargetExprId> {
        self.0.clone()
    }
    fn verify(&self, _context: &TargetAstContext<'_, JavaDialect>) -> Vec<AstViolation> {
        vec![]
    }
}

impl TargetDialect for JavaDialect {
    type Unresolved = TargetAstPackage<Self>;
    type Resolved = LinkedTargetPackage<Self>;

    fn verify_unresolved(&self, ast: &Self::Unresolved) -> Result<(), Vec<Diagnostic>> {
        verify_target_ast(ast)
    }

    fn verify_resolved(&self, ast: &Self::Resolved) -> Result<(), Vec<Diagnostic>> {
        verify_linked_package(ast)
    }
}

impl TypedAstDialect for JavaDialect {
    type PrimitiveType = JavaPrimitive;
    type KnownType = JavaKnownType;
    type RuntimeType = JavaRuntimeType;
    type ConstructedType = JavaConstructedType;
    type KnownCallable = JavaKnownCallable;
    type RuntimeCallable = JavaRuntimeCallable;
    type InvocationKind = JavaInvocationKind;
    type Visibility = JavaVisibility;
    type DeclarationKind = JavaDeclarationKind;
    type SymbolOrigin = JavaSyntheticOrigin;
    type TemplateId = JavaTemplateId;
    type ModuleDeclaration = JavaPackage;
    type FilePlacement = JavaFilePlacement;
    type Expression = JavaArenaExpression;
    type Statement = JavaArenaStatement;
    type FileItem = JavaFileItem;

    fn known_callable_signature(
        &self,
        callable: &Self::KnownCallable,
    ) -> TargetCallableSignature<Self> {
        self.coarse_signature(&callable.signature())
    }

    fn runtime_callable_signature(
        &self,
        callable: &Self::RuntimeCallable,
    ) -> TargetCallableSignature<Self> {
        self.coarse_signature(&callable.signature())
    }

    fn verify_signature(&self, signature: &TargetCallableSignature<Self>) -> Vec<AstViolation> {
        if signature.invocation == JavaInvocationKind::Constructor && signature.receiver.is_some() {
            vec![AstViolation::new(
                DiagnosticCode::InvalidInvocation,
                "Java constructors cannot have a receiver",
            )]
        } else {
            vec![]
        }
    }

    fn verify_source_file(
        &self,
        file: &TargetFile<Self>,
        context: &TargetAstContext<'_, Self>,
    ) -> Vec<AstViolation> {
        let mut violations = Vec::new();
        violations.extend(verify_java_file_identity(
            file.role(),
            file.path().as_str(),
            file.module(),
            file.placement(),
        ));
        if file
            .items()
            .iter()
            .any(|item| matches!(item, JavaFileItem::RuntimeMembers { .. }))
        {
            violations.push(AstViolation::new(
                DiagnosticCode::InvalidStructure,
                "Java runtime member fragments must be injected by the typed helper linker",
            ));
        }
        let has_compile_fail_member = file.items().iter().any(|item| {
            matches!(item, JavaFileItem::Type { declaration, .. }
                if declaration.contains_compile_fail_member())
        });
        match (*file.placement(), has_compile_fail_member) {
            (JavaFilePlacement::NegativeTest, false) => violations.push(AstViolation::new(
                DiagnosticCode::InvalidStructure,
                "Java negative-test file must contain a typed compile-fail member",
            )),
            (JavaFilePlacement::NegativeTest, true) => {}
            (_, true) => violations.push(AstViolation::new(
                DiagnosticCode::InvalidStructure,
                "Java compile-fail members are confined to negative-test files",
            )),
            (_, false) => {}
        }
        if !file.path().as_str().ends_with(".java") {
            violations.push(AstViolation::new(
                DiagnosticCode::UnsafeOutputPath,
                "Java source path must end in .java",
            ));
        }
        let public_types = file
            .items()
            .iter()
            .filter_map(|item| match item {
                JavaFileItem::Type { declaration, .. }
                    if declaration.visibility == JavaVisibility::Public =>
                {
                    Some(declaration)
                }
                JavaFileItem::Type { .. } | JavaFileItem::RuntimeMembers { .. } => None,
            })
            .collect::<Vec<_>>();
        if public_types.len() > 1 {
            violations.push(AstViolation::new(
                DiagnosticCode::InvalidStructure,
                "Java compilation unit has more than one public top-level type",
            ));
        }
        if let [public_type] = public_types.as_slice() {
            let actual = file.path().as_str().rsplit('/').next().unwrap_or_default();
            let expected = format!("{}.java", public_type.name.as_str());
            if actual != expected {
                violations.push(AstViolation::new(
                    DiagnosticCode::InvalidStructure,
                    format!(
                        "Java public top-level type `{}` must be declared in `{expected}`",
                        public_type.name.as_str()
                    ),
                ));
            }
        }
        if file.template() != &JavaTemplateId::CompilationUnit {
            violations.push(AstViolation::new(
                DiagnosticCode::InvalidStructure,
                "Java source files must use the compilation-unit template",
            ));
        }
        violations.extend(verify_composed_java_file(
            file.placement(),
            file.items().iter().collect(),
            context,
        ));
        violations
    }
}

impl JavaDialect {
    pub fn coarse_type(&self, ty: &JavaType) -> TargetTypeRef<Self> {
        match ty {
            JavaType::Primitive(value) => TargetTypeRef::Primitive(*value),
            JavaType::Boxed(value) => TargetTypeRef::Known(match value {
                JavaPrimitive::Boolean => JavaKnownType::Boolean,
                JavaPrimitive::Byte => JavaKnownType::Byte,
                JavaPrimitive::Char => JavaKnownType::Character,
                JavaPrimitive::Int => JavaKnownType::Integer,
                JavaPrimitive::Long => JavaKnownType::Long,
                JavaPrimitive::Double => JavaKnownType::Double,
                JavaPrimitive::Void => JavaKnownType::Object,
            }),
            JavaType::Reference(JavaTypeName::Known(value)) => TargetTypeRef::Known(*value),
            JavaType::Reference(JavaTypeName::Generated(value)) => TargetTypeRef::Generated(*value),
            JavaType::Array { .. } => TargetTypeRef::Constructed(JavaConstructedType::Array),
            JavaType::Generic { .. } => TargetTypeRef::Constructed(JavaConstructedType::Generic),
            JavaType::Wildcard { .. } => TargetTypeRef::Constructed(JavaConstructedType::Wildcard),
            JavaType::TypeVariable(_) => {
                TargetTypeRef::Constructed(JavaConstructedType::TypeVariable)
            }
        }
    }

    pub(crate) fn coarse_signature(
        &self,
        value: &JavaMethodSignature,
    ) -> TargetCallableSignature<Self> {
        TargetCallableSignature {
            invocation: if value.receiver.is_some() {
                JavaInvocationKind::Instance
            } else {
                JavaInvocationKind::Static
            },
            receiver: value.receiver.as_ref().map(|value| self.coarse_type(value)),
            parameters: value
                .parameters
                .iter()
                .map(|value| self.coarse_type(value))
                .collect(),
            return_type: self.coarse_type(&value.result),
        }
    }
}

impl LinkerDialect for JavaDialect {
    type KnownField = JavaKnownField;
    type KnownConstructor = JavaKnownConstructor;
    type KnownMethod = JavaKnownMethod;
    type PreludeSymbol = JavaPreludeSymbol;
    type StandardLibrary = JavaStandardLibrary;
    type ExternalPackage = JavaExternalPackage;
    type PackageFeature = JavaPackageFeature;
    type HelperId = JavaRuntimeHelper;
    type HelperCapability = JavaHelperCapability;
    type Identifier = JavaIdentifier;
    type QualifiedName = JavaQualifiedName;
    type MemberName = JavaMemberName;
    type Namespace = JavaNamespace;
    type NameKey = JavaNameKey;
    type ImportKind = JavaImportKind;
    type ResolvedModule = JavaPackage;
    type ResolvedFileItem = ResolvedJavaFileItem;

    fn package_ecosystem(&self, package: &Self::ExternalPackage) -> PackageEcosystem {
        match package {
            JavaExternalPackage::None => PackageEcosystem::Maven,
        }
    }

    fn package_name(&self, package: &Self::ExternalPackage) -> &'static str {
        match package {
            JavaExternalPackage::None => "none",
        }
    }

    fn package_feature_name(&self, feature: &Self::PackageFeature) -> &'static str {
        match feature {
            JavaPackageFeature::None => "none",
        }
    }

    fn helper_name(&self, helper: &Self::HelperId) -> &'static str {
        helper.name()
    }

    fn helper_capability_name(&self, capability: &Self::HelperCapability) -> &'static str {
        capability.name()
    }

    fn symbol_catalogue(&self) -> SymbolCatalogue<Self> {
        java_symbol_catalogue()
    }

    fn identifier_from_candidate(
        &self,
        candidate: &str,
        _namespace: &Self::Namespace,
    ) -> Result<Self::Identifier, AstViolation> {
        Ok(JavaIdentifier::from_portable(candidate))
    }

    fn identifier_key(&self, identifier: &Self::Identifier) -> Self::NameKey {
        JavaNameKey(identifier.as_str().to_owned())
    }

    fn is_public(&self, visibility: &Self::Visibility) -> bool {
        *visibility == JavaVisibility::Public
    }

    fn type_namespace(&self, _kind: &Self::DeclarationKind) -> Self::Namespace {
        JavaNamespace::Type
    }
    fn type_namespace_from_known(&self, _known: &Self::KnownType) -> Self::Namespace {
        JavaNamespace::Type
    }
    fn callable_namespace(&self) -> Self::Namespace {
        JavaNamespace::Value
    }
    fn member_namespace(&self) -> Self::Namespace {
        JavaNamespace::Value
    }
    fn value_namespace(&self) -> Self::Namespace {
        JavaNamespace::Value
    }

    fn known_call_expression(
        &self,
        callable: Self::KnownCallable,
        _invocation: Self::InvocationKind,
        arguments: Vec<TargetExprId>,
    ) -> Self::Expression {
        JavaArenaExpression::KnownCall {
            callable,
            arguments,
        }
    }

    fn known_constructor_expression(
        &self,
        constructor: Self::KnownConstructor,
        arguments: Vec<TargetExprId>,
    ) -> Self::Expression {
        JavaArenaExpression::KnownConstructor {
            constructor,
            arguments,
        }
    }

    fn known_method_expression(
        &self,
        method: Self::KnownMethod,
        receiver: TargetExprId,
        arguments: Vec<TargetExprId>,
    ) -> Self::Expression {
        JavaArenaExpression::KnownMethod {
            method,
            receiver,
            arguments,
        }
    }

    fn expression_references(&self, value: &Self::Expression) -> Vec<TargetSymbolRef<Self>> {
        match value {
            JavaArenaExpression::KnownCall { callable, .. } => {
                vec![TargetSymbolRef::KnownCallable(*callable)]
            }
            JavaArenaExpression::KnownConstructor { constructor, .. } => {
                vec![TargetSymbolRef::KnownConstructor(*constructor)]
            }
            JavaArenaExpression::KnownMethod { method, .. } => {
                vec![TargetSymbolRef::KnownMethod(*method)]
            }
        }
    }

    fn statement_references(&self, _statement: &Self::Statement) -> Vec<TargetSymbolRef<Self>> {
        vec![]
    }

    fn file_item_roots(&self, item: &Self::FileItem) -> FileItemRoots<Self> {
        FileItemRoots {
            declarations: item.declared_symbols(),
            expressions: vec![],
            statements: vec![],
            symbols: item.symbols(),
        }
    }

    fn resolve_module(
        &self,
        module: &Self::ModuleDeclaration,
    ) -> Result<Self::ResolvedModule, AstViolation> {
        Ok(*module)
    }

    fn resolve_file_item(
        &self,
        package: &TargetAstPackage<Self>,
        item: &Self::FileItem,
        references: &ResolvedReferenceMap<Self>,
    ) -> Result<Self::ResolvedFileItem, AstViolation> {
        let locally_declared = item.declared_symbols().into_iter().collect::<BTreeSet<_>>();
        let mut names = BTreeMap::new();
        for symbol in item.symbols() {
            let Some(resolved) = references.get(&symbol) else {
                return Err(AstViolation::new(
                    DiagnosticCode::UnresolvedReference,
                    "Java item symbol has no resolver-owned spelling",
                ));
            };
            let is_public_container_member = match symbol {
                TargetSymbolRef::Generated(GeneratedSymbolId::Type(id)) => {
                    !matches!(
                        package.generated_type(id).map(|value| &value.origin),
                        Some(GeneratedOrigin::Synthesized(
                            SynthesisReason::PackageEntryPoint
                        ))
                    ) && !locally_declared.contains(&GeneratedSymbolId::Type(id))
                }
                TargetSymbolRef::Generated(GeneratedSymbolId::Callable(id)) => {
                    !locally_declared.contains(&GeneratedSymbolId::Callable(id))
                }
                TargetSymbolRef::Generated(GeneratedSymbolId::Value(id)) => {
                    !locally_declared.contains(&GeneratedSymbolId::Value(id))
                }
                _ => false,
            };
            let name = match resolved {
                ResolvedReference::Local(value) if is_public_container_member => {
                    JavaResolvedName::GeneratedMember {
                        owner: JavaGeneratedContainer::PublicApi,
                        member: value.clone(),
                    }
                }
                ResolvedReference::Local(value)
                | ResolvedReference::Imported { binding: value, .. } => {
                    JavaResolvedName::Local(value.clone())
                }
                ResolvedReference::Qualified(value) => JavaResolvedName::Qualified(*value),
                ResolvedReference::Member { owner, member } => JavaResolvedName::Member {
                    owner: *owner,
                    member: *member,
                },
            };
            names.insert(symbol, name);
        }
        Ok(ResolvedJavaFileItem {
            item: item.clone(),
            names,
        })
    }

    fn verify_resolved_file_item(&self, item: &Self::ResolvedFileItem) -> Vec<AstViolation> {
        let expected = item.item.symbols().into_iter().collect::<BTreeSet<_>>();
        let actual = item.names.keys().cloned().collect::<BTreeSet<_>>();
        if expected == actual {
            vec![]
        } else {
            vec![AstViolation::new(
                DiagnosticCode::UnresolvedReference,
                "resolved Java item does not contain the exact linker-derived spelling map",
            )]
        }
    }

    fn verify_resolved_file(
        &self,
        file: &LinkedFile<Self>,
        context: &TargetAstContext<'_, Self>,
    ) -> Vec<AstViolation> {
        let mut violations = verify_java_file_identity(
            file.role(),
            file.path().as_str(),
            file.module(),
            file.placement(),
        );
        violations.extend(verify_composed_java_file(
            file.placement(),
            file.items().iter().map(|item| &item.item).collect(),
            context,
        ));
        violations
    }
}

fn verify_java_file_identity(
    role: portable_codegen::SourceRole,
    path: &str,
    module: &JavaPackage,
    placement: &JavaFilePlacement,
) -> Vec<AstViolation> {
    const RUNTIME_PATH: &str = "src/main/java/org/polyrust/generated/Runtime.java";
    let is_runtime = *placement == JavaFilePlacement::Runtime;
    let mut violations = Vec::new();
    if is_runtime
        && (role != portable_codegen::SourceRole::Runtime
            || module != &JavaPackage::Generated
            || path != RUNTIME_PATH)
    {
        violations.push(AstViolation::new(
            DiagnosticCode::InvalidStructure,
            "Java runtime placement requires the generated Runtime.java source role and path",
        ));
    }
    if !is_runtime && role == portable_codegen::SourceRole::Runtime {
        violations.push(AstViolation::new(
            DiagnosticCode::InvalidStructure,
            "Java runtime source role requires runtime placement",
        ));
    }
    violations
}

fn verify_composed_java_file(
    placement: &JavaFilePlacement,
    items: Vec<&JavaFileItem>,
    context: &TargetAstContext<'_, JavaDialect>,
) -> Vec<AstViolation> {
    let runtime_fragments = items
        .iter()
        .filter(|item| matches!(item, JavaFileItem::RuntimeMembers { .. }))
        .count();
    if *placement != JavaFilePlacement::Runtime {
        return (runtime_fragments > 0)
            .then(|| {
                AstViolation::new(
                    DiagnosticCode::InvalidStructure,
                    "Java runtime member fragments are confined to the runtime file",
                )
            })
            .into_iter()
            .collect();
    }

    let shells = items
        .iter()
        .filter_map(|item| match *item {
            JavaFileItem::Type { .. } => Some(*item),
            JavaFileItem::RuntimeMembers { .. } => None,
        })
        .collect::<Vec<_>>();
    if shells.len() != 1 {
        return vec![AstViolation::new(
            DiagnosticCode::InvalidStructure,
            "Java runtime file must contain exactly one typed class shell",
        )];
    }

    let expected_shell = crate::runtime::shell_item();
    if shells[0] != &expected_shell {
        return vec![AstViolation::new(
            DiagnosticCode::InvalidStructure,
            "Java runtime file does not contain the exact registered Runtime class shell",
        )];
    }

    let JavaFileItem::Type { declaration, .. } = shells[0] else {
        unreachable!("runtime shell collection contains only type items")
    };
    let mut combined = declaration.clone();
    for item in items {
        if let JavaFileItem::RuntimeMembers { members, .. } = item {
            combined.members.extend(members.iter().cloned());
        }
    }
    let mut violations = combined.verify(context, true);
    if combined.contains_compile_fail_member() {
        violations.push(AstViolation::new(
            DiagnosticCode::InvalidStructure,
            "Java compile-fail members are confined to negative-test files",
        ));
    }
    violations
}

fn java_symbol_catalogue() -> SymbolCatalogue<JavaDialect> {
    SymbolCatalogue {
        types: JavaKnownType::ALL
            .into_iter()
            .map(known_type_spec)
            .collect(),
        callables: JavaKnownCallable::ALL
            .into_iter()
            .map(known_callable_spec)
            .collect(),
        runtime_callables: JavaRuntimeCallable::ALL
            .into_iter()
            .map(runtime_callable_spec)
            .collect(),
        fields: [
            JavaKnownField::IntegerMinValue,
            JavaKnownField::IntegerMaxValue,
            JavaKnownField::LongMinValue,
            JavaKnownField::LongMaxValue,
            JavaKnownField::StandardCharsetsUtf8,
            JavaKnownField::CodingErrorReport,
        ]
        .into_iter()
        .map(known_field_spec)
        .collect(),
        constructors: JavaKnownConstructor::ALL
            .into_iter()
            .map(known_constructor_spec)
            .collect(),
        methods: JavaKnownMethod::ALL
            .into_iter()
            .map(known_method_spec)
            .collect(),
        helpers: JavaRuntimeHelper::ALL
            .into_iter()
            .enumerate()
            .map(|(order, helper)| RuntimeHelperSpec {
                id: helper,
                capability: helper_capability(helper),
                order: u32::try_from(order).expect("Java helper inventory fits u32"),
                name: JavaIdentifier::from_portable(helper.name()),
                alias_stem: helper.name().replace('.', "_"),
                namespace: JavaNamespace::Value,
                items: crate::runtime::helper_items(helper),
                placement: JavaFilePlacement::Runtime,
                visibility: JavaVisibility::Private,
                source: symbol_source("helper", helper.name()),
            })
            .collect(),
    }
}

fn known_type_spec(value: JavaKnownType) -> KnownTypeSpec<JavaDialect> {
    let (origin, policy, qualified_name) = if value.implicit() {
        (
            SymbolOrigin::LanguagePrelude(JavaPreludeSymbol::JavaLang),
            DependencyPolicy::Implicit,
            None,
        )
    } else if value.runtime_nested() {
        (
            SymbolOrigin::Runtime(
                value
                    .runtime_helper()
                    .expect("runtime nested type owns helper"),
            ),
            DependencyPolicy::Qualified,
            Some(JavaQualifiedName::Type(value)),
        )
    } else {
        (
            SymbolOrigin::StandardLibrary(JavaStandardLibrary::Jdk21),
            DependencyPolicy::Import(JavaImportKind::Type(value)),
            Some(JavaQualifiedName::Type(value)),
        )
    };
    KnownTypeSpec {
        symbol: value,
        name: JavaIdentifier::from_portable(value.simple_name()),
        alias_stem: value.simple_name().to_owned(),
        qualified_name,
        origin,
        arity: match value {
            JavaKnownType::ArrayList
            | JavaKnownType::List
            | JavaKnownType::RuntimeResult
            | JavaKnownType::RuntimeOption => 1,
            JavaKnownType::Map | JavaKnownType::RuntimeValueResult => 2,
            _ => 0,
        },
        policy,
        dependency: None,
        source: symbol_source("type", value.qualified_name()),
    }
}

fn known_callable_spec(value: JavaKnownCallable) -> KnownCallableSpec<JavaDialect> {
    let signature = value.signature();
    let member = match value {
        JavaKnownCallable::ObjectsDeepEquals => JavaMemberName::DeepEquals,
        JavaKnownCallable::ObjectsRequireNonNull => JavaMemberName::RequireNonNull,
        JavaKnownCallable::DoubleToRawLongBits => JavaMemberName::DoubleToRawLongBits,
        JavaKnownCallable::DoubleFromLongBits => JavaMemberName::LongBitsToDouble,
        JavaKnownCallable::DoubleIsNaN => JavaMemberName::IsNaN,
        JavaKnownCallable::MathFloor => JavaMemberName::Floor,
        JavaKnownCallable::MathCeil => JavaMemberName::Ceil,
        JavaKnownCallable::ListCopyOf => JavaMemberName::CopyOf,
        JavaKnownCallable::ListOf => JavaMemberName::Of,
        JavaKnownCallable::BigIntegerValueOf => JavaMemberName::ValueOf,
        JavaKnownCallable::ByteToUnsignedInt => JavaMemberName::ToUnsignedInt,
        JavaKnownCallable::ByteBufferWrap => JavaMemberName::Wrap,
        JavaKnownCallable::CharacterIsHighSurrogate => JavaMemberName::IsHighSurrogate,
        JavaKnownCallable::CharacterIsLowSurrogate => JavaMemberName::IsLowSurrogate,
        JavaKnownCallable::CharacterCharCount => JavaMemberName::CharCount,
    };
    KnownCallableSpec {
        symbol: value,
        owner: Some(value.owner()),
        name: JavaIdentifier::from_portable(value.name()),
        alias_stem: value.name().to_owned(),
        qualified_name: Some(JavaQualifiedName::Callable(value)),
        origin: if value.owner().implicit() {
            SymbolOrigin::LanguagePrelude(JavaPreludeSymbol::JavaLang)
        } else {
            SymbolOrigin::StandardLibrary(JavaStandardLibrary::Jdk21)
        },
        signature: callable_pattern(&JavaDialect.coarse_signature(&signature)),
        visibility: JavaVisibility::Public,
        policy: DependencyPolicy::Member {
            owner: JavaQualifiedName::Type(value.owner()),
            member,
        },
        dependency: None,
        source: symbol_source("callable", value.qualified_name()),
    }
}

fn runtime_callable_spec(value: JavaRuntimeCallable) -> RuntimeCallableSpec<JavaDialect> {
    let signature = JavaDialect.coarse_signature(&value.signature());
    RuntimeCallableSpec {
        symbol: value,
        name: JavaIdentifier::from_portable(value.name()),
        alias_stem: value.name().to_owned(),
        qualified_name: Some(JavaQualifiedName::RuntimeCallable(value)),
        origin: SymbolOrigin::Runtime(value.helper()),
        signature: callable_pattern(&signature),
        policy: DependencyPolicy::Qualified,
        dependency: None,
        source: symbol_source("runtime-callable", value.name()),
    }
}

fn known_field_spec(value: JavaKnownField) -> KnownFieldSpec<JavaDialect> {
    KnownFieldSpec {
        symbol: value,
        owner: value.owner(),
        name: JavaIdentifier::from_portable(value.member().text()),
        origin: if value.owner().implicit() {
            SymbolOrigin::LanguagePrelude(JavaPreludeSymbol::JavaLang)
        } else {
            SymbolOrigin::StandardLibrary(JavaStandardLibrary::Jdk21)
        },
        ty: TypePattern::Exact(JavaDialect.coarse_type(&value.ty())),
        policy: DependencyPolicy::Member {
            owner: JavaQualifiedName::Type(value.owner()),
            member: value.member(),
        },
        dependency: None,
        source: symbol_source("field", value.member().text()),
    }
}

fn known_constructor_spec(value: JavaKnownConstructor) -> KnownConstructorSpec<JavaDialect> {
    let (owner, parameters) = value.signature();
    let signature = TargetCallableSignature {
        invocation: JavaInvocationKind::Constructor,
        receiver: None,
        parameters: parameters
            .iter()
            .map(|value| JavaDialect.coarse_type(value))
            .collect(),
        return_type: JavaDialect.coarse_type(&owner),
    };
    let owner_type = value.owner();
    KnownConstructorSpec {
        symbol: value,
        owner: owner_type,
        name: JavaIdentifier::from_portable(owner_type.simple_name()),
        alias_stem: owner_type.simple_name().to_owned(),
        qualified_name: Some(JavaQualifiedName::Type(owner_type)),
        origin: if owner_type.runtime_nested() {
            SymbolOrigin::Runtime(
                owner_type
                    .runtime_helper()
                    .expect("runtime constructor owns helper"),
            )
        } else if owner_type.implicit() {
            SymbolOrigin::LanguagePrelude(JavaPreludeSymbol::JavaLang)
        } else {
            SymbolOrigin::StandardLibrary(JavaStandardLibrary::Jdk21)
        },
        signature: callable_pattern(&signature),
        visibility: JavaVisibility::Public,
        policy: if owner_type.runtime_nested() {
            DependencyPolicy::Qualified
        } else if owner_type.implicit() {
            DependencyPolicy::Implicit
        } else {
            DependencyPolicy::Import(JavaImportKind::Type(owner_type))
        },
        dependency: None,
        source: symbol_source("constructor", owner_type.qualified_name()),
    }
}

fn known_method_spec(value: JavaKnownMethod) -> KnownMethodSpec<JavaDialect> {
    let signature = value.signature();
    KnownMethodSpec {
        symbol: value,
        owner: value.owner(),
        name: JavaIdentifier::from_portable(value.name().text()),
        origin: if value.owner().implicit() {
            SymbolOrigin::LanguagePrelude(JavaPreludeSymbol::JavaLang)
        } else {
            SymbolOrigin::StandardLibrary(JavaStandardLibrary::Jdk21)
        },
        signature: callable_pattern(&JavaDialect.coarse_signature(&signature)),
        visibility: JavaVisibility::Public,
        policy: DependencyPolicy::Member {
            owner: JavaQualifiedName::Type(value.owner()),
            member: value.name(),
        },
        dependency: None,
        source: symbol_source("method", value.name().text()),
    }
}

fn callable_pattern(
    signature: &TargetCallableSignature<JavaDialect>,
) -> CallablePattern<JavaDialect> {
    CallablePattern {
        invocation: signature.invocation,
        type_parameters: Vec::<TypeParameterSpec<JavaDialect>>::new(),
        receiver: signature.receiver.clone().map(TypePattern::Exact),
        parameters: signature
            .parameters
            .iter()
            .cloned()
            .map(TypePattern::Exact)
            .collect(),
        result: TypePattern::Exact(signature.return_type.clone()),
        failure: FailureBehavior::Infallible,
        effects: BTreeSet::<TargetEffect>::new(),
    }
}

fn helper_capability(helper: JavaRuntimeHelper) -> JavaHelperCapability {
    match helper {
        JavaRuntimeHelper::Core => JavaHelperCapability::Failures,
        JavaRuntimeHelper::TaggedValues => JavaHelperCapability::TaggedValues,
        JavaRuntimeHelper::CheckedIntegers => JavaHelperCapability::CheckedArithmetic,
        JavaRuntimeHelper::FloatBits => JavaHelperCapability::ExactFloatBits,
        JavaRuntimeHelper::Unicode => JavaHelperCapability::UnicodeScalars,
        JavaRuntimeHelper::Bytes => JavaHelperCapability::ImmutableBytes,
        JavaRuntimeHelper::ImmutableLists => JavaHelperCapability::ImmutableLists,
        JavaRuntimeHelper::StringOperations => JavaHelperCapability::StringOperations,
        JavaRuntimeHelper::Interfaces => JavaHelperCapability::InterfaceDispatch,
    }
}

fn symbol_source(category: &str, name: &str) -> SourceRef {
    SourceRef::logical(["java-catalogue", category, name])
}

#[cfg(test)]
mod tests {
    use super::*;

    fn source(label: &str) -> SourceRef {
        SourceRef::logical(["java-verifier-test", label])
    }

    fn declaration(heritage: JavaHeritage, members: Vec<JavaMember>) -> JavaTypeDeclaration {
        JavaTypeDeclaration {
            declared: None,
            kind: JavaDeclarationKind::FinalClass,
            visibility: JavaVisibility::Package,
            modifiers: vec![],
            name: JavaIdentifier::from_portable("Fixture"),
            type_parameters: vec![],
            record_components: vec![],
            heritage,
            permits: vec![],
            members,
        }
    }

    fn verify_file_at_path(
        path: &str,
        declaration: JavaTypeDeclaration,
        role: portable_codegen::SourceRole,
        placement: JavaFilePlacement,
        group_role: portable_codegen::FileGroupRole,
    ) -> Result<(), Vec<Diagnostic>> {
        let mut builder = portable_codegen::TargetAstBuilder::new(JavaDialect);
        let file = builder.file(TargetFile::new(
            portable_codegen::RelativeOutputPath::new(path).unwrap(),
            role,
            JavaPackage::Generated,
            placement,
            vec![JavaFileItem::Type {
                declared: vec![],
                declaration,
            }],
            JavaTemplateId::CompilationUnit,
            source("file"),
        ));
        builder.group(portable_codegen::TargetFileGroup::new(
            group_role,
            vec![portable_codegen::TargetFileMember::Source(file)],
            source("group"),
        ));
        portable_codegen::verify_target_ast(&builder.build())
    }

    fn verify_single_file(
        declaration: JavaTypeDeclaration,
        role: portable_codegen::SourceRole,
        placement: JavaFilePlacement,
        group_role: portable_codegen::FileGroupRole,
    ) -> Result<(), Vec<Diagnostic>> {
        verify_file_at_path("Fixture.java", declaration, role, placement, group_role)
    }

    #[test]
    fn closed_catalogues_have_unique_qualified_names() {
        let type_names = JavaKnownType::ALL
            .into_iter()
            .map(JavaKnownType::qualified_name)
            .collect::<BTreeSet<_>>();
        assert_eq!(type_names.len(), JavaKnownType::ALL.len());

        let callable_names = JavaKnownCallable::ALL
            .into_iter()
            .map(JavaKnownCallable::qualified_name)
            .collect::<BTreeSet<_>>();
        assert_eq!(callable_names.len(), JavaKnownCallable::ALL.len());

        let runtime_names = JavaRuntimeCallable::ALL
            .into_iter()
            .map(JavaRuntimeCallable::qualified_name)
            .collect::<BTreeSet<_>>();
        assert_eq!(runtime_names.len(), JavaRuntimeCallable::ALL.len());
    }

    #[test]
    fn every_callable_accepts_its_authoritative_signature_and_rejects_mutation() {
        for callable in JavaKnownCallable::ALL {
            let signature = callable.signature();
            assert!(callable.accepts(&signature), "{callable:?}");
            let mut invalid = signature;
            invalid.pure = !invalid.pure;
            assert!(!callable.accepts(&invalid), "{callable:?}");
        }
        for method in JavaKnownMethod::ALL {
            let signature = method.signature();
            assert!(method.accepts(&signature), "{method:?}");
            let mut invalid = signature;
            invalid.nullable_result = !invalid.nullable_result;
            assert!(!method.accepts(&invalid), "{method:?}");
        }
        for callable in JavaRuntimeCallable::ALL {
            let signature = callable.signature();
            assert!(callable.accepts(&signature), "{callable:?}");
            let mut invalid = signature;
            invalid.receiver = Some(JavaType::known(JavaKnownType::Object));
            assert!(!callable.accepts(&invalid), "{callable:?}");
        }
    }

    #[test]
    fn every_constructor_accepts_its_generic_signature_and_rejects_wrong_arity() {
        for constructor in JavaKnownConstructor::ALL {
            let (owner, parameters) = constructor.signature();
            assert!(constructor.accepts(&owner, &parameters), "{constructor:?}");
            let mut invalid = parameters;
            invalid.push(JavaType::known(JavaKnownType::Object));
            assert!(!constructor.accepts(&owner, &invalid), "{constructor:?}");
        }
    }

    #[test]
    fn catalogue_inventory_and_dependency_policies_are_exhaustive() {
        let catalogue = java_symbol_catalogue();
        assert_eq!(catalogue.types.len(), JavaKnownType::ALL.len());
        assert_eq!(catalogue.callables.len(), JavaKnownCallable::ALL.len());
        assert_eq!(
            catalogue.runtime_callables.len(),
            JavaRuntimeCallable::ALL.len()
        );
        assert_eq!(
            catalogue.constructors.len(),
            JavaKnownConstructor::ALL.len()
        );
        assert_eq!(catalogue.methods.len(), JavaKnownMethod::ALL.len());
        assert_eq!(catalogue.helpers.len(), JavaRuntimeHelper::ALL.len());

        for ty in JavaKnownType::ALL {
            let policy = known_type_spec(ty).policy;
            match (ty.implicit(), ty.runtime_nested(), policy) {
                (true, false, DependencyPolicy::Implicit)
                | (false, true, DependencyPolicy::Qualified)
                | (false, false, DependencyPolicy::Import(_)) => {}
                combination => panic!("invalid dependency policy: {combination:?}"),
            }
        }
        for helper in JavaRuntimeHelper::ALL {
            assert!(
                !crate::runtime::helper_items(helper).is_empty(),
                "{helper:?}"
            );
        }
    }

    #[test]
    fn negative_nodes_and_heritage_exceptions_are_confined_and_fail_closed() {
        let compile_fail = JavaMember::CompileFailField(JavaCompileFailField {
            modifiers: vec![JavaModifier::Final],
            expected_type: JavaType::generic(
                JavaKnownType::RuntimeOption,
                vec![JavaType::Boxed(JavaPrimitive::Int)],
            ),
            name: JavaIdentifier::from_portable("invalid"),
            initializer: JavaExpr::literal(
                JavaType::known(JavaKnownType::String),
                JavaLiteral::String("missing".to_owned()),
            ),
        });
        let diagnostics = verify_single_file(
            declaration(JavaHeritage::None, vec![compile_fail]),
            portable_codegen::SourceRole::PublicApi,
            JavaFilePlacement::Main,
            portable_codegen::FileGroupRole::PublicApi,
        )
        .unwrap_err();
        assert!(
            diagnostics
                .iter()
                .any(|diagnostic| diagnostic.code == DiagnosticCode::InvalidStructure)
        );

        let diagnostics = verify_single_file(
            declaration(JavaHeritage::None, vec![]),
            portable_codegen::SourceRole::NegativeTest,
            JavaFilePlacement::NegativeTest,
            portable_codegen::FileGroupRole::NegativeTests,
        )
        .unwrap_err();
        assert!(
            diagnostics
                .iter()
                .any(|diagnostic| diagnostic.code == DiagnosticCode::InvalidStructure)
        );

        let diagnostics = verify_single_file(
            declaration(
                JavaHeritage::Interfaces(vec![JavaType::known(JavaKnownType::String)]),
                vec![],
            ),
            portable_codegen::SourceRole::PublicApi,
            JavaFilePlacement::Main,
            portable_codegen::FileGroupRole::PublicApi,
        )
        .unwrap_err();
        assert!(
            diagnostics
                .iter()
                .any(|diagnostic| { diagnostic.code == DiagnosticCode::InterfaceNonconformance })
        );
    }

    #[test]
    fn linked_runtime_members_are_verified_in_the_combined_class() {
        let shell = JavaFileItem::Type {
            declared: vec![],
            declaration: JavaTypeDeclaration {
                declared: None,
                kind: JavaDeclarationKind::FinalClass,
                visibility: JavaVisibility::Public,
                modifiers: vec![],
                name: JavaIdentifier::from_portable("Runtime"),
                type_parameters: vec![],
                record_components: vec![],
                heritage: JavaHeritage::None,
                permits: vec![],
                members: vec![JavaMember::Constructor(JavaConstructor {
                    modifiers: vec![JavaModifier::Private],
                    name: JavaIdentifier::from_portable("Runtime"),
                    parameters: vec![],
                    body: JavaBlock::new(vec![]),
                })],
            },
        };
        let fragment = JavaFileItem::RuntimeMembers {
            helper: JavaRuntimeHelper::Interfaces,
            members: vec![JavaMember::Field(JavaField {
                declared: None,
                modifiers: vec![JavaModifier::Private, JavaModifier::Final],
                ty: JavaType::primitive(JavaPrimitive::Int),
                name: JavaIdentifier::from_portable("x"),
                initializer: None,
            })],
        };
        let mut builder = portable_codegen::TargetAstBuilder::new(JavaDialect);
        let file = builder.file(TargetFile::new(
            portable_codegen::RelativeOutputPath::new(
                "src/main/java/org/polyrust/generated/Runtime.java",
            )
            .unwrap(),
            portable_codegen::SourceRole::Runtime,
            JavaPackage::Generated,
            JavaFilePlacement::Runtime,
            vec![shell.clone(), fragment.clone()],
            JavaTemplateId::CompilationUnit,
            source("combined-runtime"),
        ));
        builder.group(portable_codegen::TargetFileGroup::new(
            portable_codegen::FileGroupRole::Runtime,
            vec![portable_codegen::TargetFileMember::Source(file)],
            source("combined-runtime-group"),
        ));
        let diagnostics = portable_codegen::verify_target_ast(&builder.build()).unwrap_err();
        assert!(diagnostics.iter().any(|diagnostic| {
            diagnostic.code == DiagnosticCode::InvalidControlFlow
                && diagnostic
                    .message
                    .contains("without assigning blank final field `x`")
        }));

        let mut builder = portable_codegen::TargetAstBuilder::new(JavaDialect);
        let file = builder.file(TargetFile::new(
            portable_codegen::RelativeOutputPath::new("Fixture.java").unwrap(),
            portable_codegen::SourceRole::PublicApi,
            JavaPackage::Generated,
            JavaFilePlacement::Main,
            vec![fragment],
            JavaTemplateId::CompilationUnit,
            source("misplaced-runtime-fragment"),
        ));
        builder.group(portable_codegen::TargetFileGroup::new(
            portable_codegen::FileGroupRole::PublicApi,
            vec![portable_codegen::TargetFileMember::Source(file)],
            source("misplaced-runtime-fragment-group"),
        ));
        let diagnostics = portable_codegen::verify_target_ast(&builder.build()).unwrap_err();
        assert!(diagnostics.iter().any(|diagnostic| {
            diagnostic.code == DiagnosticCode::InvalidStructure
                && diagnostic.message.contains("confined to the runtime file")
        }));

        let duplicate = |helper| JavaFileItem::RuntimeMembers {
            helper,
            members: vec![JavaMember::Field(JavaField {
                declared: None,
                modifiers: vec![JavaModifier::Private, JavaModifier::Static],
                ty: JavaType::primitive(JavaPrimitive::Int),
                name: JavaIdentifier::from_portable("duplicate"),
                initializer: None,
            })],
        };
        let mut builder = portable_codegen::TargetAstBuilder::new(JavaDialect);
        let file = builder.file(TargetFile::new(
            portable_codegen::RelativeOutputPath::new(
                "src/main/java/org/polyrust/generated/Runtime.java",
            )
            .unwrap(),
            portable_codegen::SourceRole::Runtime,
            JavaPackage::Generated,
            JavaFilePlacement::Runtime,
            vec![
                shell,
                duplicate(JavaRuntimeHelper::Core),
                duplicate(JavaRuntimeHelper::Interfaces),
            ],
            JavaTemplateId::CompilationUnit,
            source("duplicate-runtime-members"),
        ));
        builder.group(portable_codegen::TargetFileGroup::new(
            portable_codegen::FileGroupRole::Runtime,
            vec![portable_codegen::TargetFileMember::Source(file)],
            source("duplicate-runtime-members-group"),
        ));
        let diagnostics = portable_codegen::verify_target_ast(&builder.build()).unwrap_err();
        assert!(diagnostics.iter().any(|diagnostic| {
            diagnostic.code == DiagnosticCode::DuplicateDeclaration
                && diagnostic
                    .message
                    .contains("field conflicts with another field")
        }));
    }

    #[test]
    fn public_top_level_type_must_match_its_java_filename() {
        let mut wrong = declaration(JavaHeritage::None, vec![]);
        wrong.visibility = JavaVisibility::Public;
        wrong.name = JavaIdentifier::from_portable("Wrong");
        let diagnostics = verify_file_at_path(
            "Other.java",
            wrong,
            portable_codegen::SourceRole::PublicApi,
            JavaFilePlacement::Main,
            portable_codegen::FileGroupRole::PublicApi,
        )
        .unwrap_err();
        assert!(diagnostics.iter().any(|diagnostic| {
            diagnostic.code == DiagnosticCode::InvalidStructure
                && diagnostic
                    .message
                    .contains("must be declared in `Wrong.java`")
        }));
    }
}
