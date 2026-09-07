//! Typed runtime construction: dispatch.

use super::bytes_operations::bytes_method;
use super::checked_integer::checked_integer_method;
use super::float::float_method;
use super::immutable_lists::{bytes_to_list_method, list_method};
use super::string_replace::{string_replace_all_method, string_replace_many_method};
use super::string_transform::{string_trim_method, string_truncate_method};
use super::unicode::unicode_method;
use crate::ast::JavaMember;
use crate::dialect::{JavaRuntimeCallable, JavaRuntimeHelper};

pub(super) fn runtime_methods(helper: JavaRuntimeHelper) -> Vec<JavaMember> {
    JavaRuntimeCallable::ALL
        .into_iter()
        .filter(|value| value.helper() == helper)
        .map(runtime_method)
        .collect()
}

pub(super) fn runtime_method(value: JavaRuntimeCallable) -> JavaMember {
    match value {
        JavaRuntimeCallable::CheckedNegI32
        | JavaRuntimeCallable::CheckedNegI64
        | JavaRuntimeCallable::CheckedAddI32
        | JavaRuntimeCallable::CheckedAddI64
        | JavaRuntimeCallable::CheckedSubI32
        | JavaRuntimeCallable::CheckedSubI64
        | JavaRuntimeCallable::CheckedMulI32
        | JavaRuntimeCallable::CheckedMulI64
        | JavaRuntimeCallable::CheckedDivI32
        | JavaRuntimeCallable::CheckedDivI64
        | JavaRuntimeCallable::CheckedRemI32
        | JavaRuntimeCallable::CheckedRemI64
        | JavaRuntimeCallable::CheckedShiftLeftI32
        | JavaRuntimeCallable::CheckedShiftLeftI64
        | JavaRuntimeCallable::CheckedShiftRightI32
        | JavaRuntimeCallable::CheckedShiftRightI64
        | JavaRuntimeCallable::NarrowI64ToI32 => checked_integer_method(value),
        JavaRuntimeCallable::FloatTrunc
        | JavaRuntimeCallable::FloatIsNegativeZero
        | JavaRuntimeCallable::FloatAbs => float_method(value),
        JavaRuntimeCallable::RequireScalarString
        | JavaRuntimeCallable::CompareScalarStrings
        | JavaRuntimeCallable::ScalarLength
        | JavaRuntimeCallable::StringIndexOfLiteral
        | JavaRuntimeCallable::StringSliceScalars
        | JavaRuntimeCallable::StringToUtf8
        | JavaRuntimeCallable::StringFromUtf8 => unicode_method(value),
        JavaRuntimeCallable::BytesToList => bytes_to_list_method(value),
        JavaRuntimeCallable::BytesLength
        | JavaRuntimeCallable::BytesIsEmpty
        | JavaRuntimeCallable::BytesConcat
        | JavaRuntimeCallable::BytesReplaceAll => bytes_method(value),
        JavaRuntimeCallable::ListCopy
        | JavaRuntimeCallable::ListLength
        | JavaRuntimeCallable::ListIsEmpty
        | JavaRuntimeCallable::ListGet
        | JavaRuntimeCallable::ListAppend
        | JavaRuntimeCallable::ListConcat
        | JavaRuntimeCallable::ListContains
        | JavaRuntimeCallable::ListIndexOf => list_method(value),
        JavaRuntimeCallable::StringReplaceAll
        | JavaRuntimeCallable::StringReplaceMany
        | JavaRuntimeCallable::StringTruncateUtf8Bytes
        | JavaRuntimeCallable::StringTrimStart
        | JavaRuntimeCallable::StringTrimEnd => string_method(value),
        JavaRuntimeCallable::Ok
        | JavaRuntimeCallable::Fail
        | JavaRuntimeCallable::DeepEqual
        | JavaRuntimeCallable::SemanticEqual
        | JavaRuntimeCallable::ValidatePublicValue
        | JavaRuntimeCallable::OptionNone
        | JavaRuntimeCallable::OptionSome
        | JavaRuntimeCallable::OptionIsSome
        | JavaRuntimeCallable::OptionValue
        | JavaRuntimeCallable::ValueResultOk
        | JavaRuntimeCallable::ValueResultErr
        | JavaRuntimeCallable::ValueResultIsOk
        | JavaRuntimeCallable::ValueResultValue
        | JavaRuntimeCallable::ValueResultError
        | JavaRuntimeCallable::BytesOf => {
            unreachable!("{value:?} has a dedicated typed declaration")
        }
    }
}

fn string_method(value: JavaRuntimeCallable) -> JavaMember {
    match value {
        JavaRuntimeCallable::StringReplaceAll => string_replace_all_method(value),
        JavaRuntimeCallable::StringReplaceMany => string_replace_many_method(value),
        JavaRuntimeCallable::StringTruncateUtf8Bytes => string_truncate_method(value),
        JavaRuntimeCallable::StringTrimStart | JavaRuntimeCallable::StringTrimEnd => {
            string_trim_method(value)
        }
        _ => unreachable!(),
    }
}
