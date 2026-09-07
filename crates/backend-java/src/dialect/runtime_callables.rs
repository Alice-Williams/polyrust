//! Java dialect: runtime callables.

use super::runtime_helpers::JavaRuntimeHelper;
use super::signature_builder::signature;
use super::signature_matching::signature_matches;
use crate::ast::{JavaIdentifier, JavaKnownType, JavaMethodSignature, JavaPrimitive, JavaType};

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
