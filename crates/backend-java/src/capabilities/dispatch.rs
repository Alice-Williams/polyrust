//! Exhaustive dispatch from verified CoreIR intrinsics to capability mappings.

use portable_build::*;
use portable_core_ir::{
    CoreBinaryIntrinsic, CoreIntrinsicExpr, CoreTernaryIntrinsic, CoreUnaryIntrinsic,
    CoreVariadicIntrinsic,
};

use super::support::JavaIntrinsicMappingInput;
use crate::ast::{JavaExpr, JavaType};
pub(crate) enum JavaIntrinsicFamily {
    BooleanLogic(JavaIntrinsicMappingInput<BooleanLogic>),
    Equality(JavaIntrinsicMappingInput<Equality>),
    Ordering(JavaIntrinsicMappingInput<Ordering>),
    CheckedIntegerArithmetic(JavaIntrinsicMappingInput<CheckedIntegerArithmetic>),
    WrappingIntegerArithmetic(JavaIntrinsicMappingInput<WrappingIntegerArithmetic>),
    FloatingPointArithmetic(JavaIntrinsicMappingInput<FloatingPointArithmetic>),
    StringConcatenation(JavaIntrinsicMappingInput<StringConcatenation>),
    IntegerBitwise(JavaIntrinsicMappingInput<IntegerBitwise>),
    CheckedIntegerShifts(JavaIntrinsicMappingInput<CheckedIntegerShifts>),
    FloatingPointInspection(JavaIntrinsicMappingInput<FloatingPointInspection>),
    StringInspection(JavaIntrinsicMappingInput<StringInspection>),
    StringTransformation(JavaIntrinsicMappingInput<StringTransformation>),
    BytesOperations(JavaIntrinsicMappingInput<BytesOperations>),
    ListOperations(JavaIntrinsicMappingInput<ListOperations>),
    OptionOperations(JavaIntrinsicMappingInput<OptionOperations>),
    ResultOperations(JavaIntrinsicMappingInput<ResultOperations>),
    IntegerConversions(JavaIntrinsicMappingInput<IntegerConversions>),
    Utf8Conversions(JavaIntrinsicMappingInput<Utf8Conversions>),
}

pub(crate) fn classify_intrinsic(
    value: CoreIntrinsicExpr<JavaExpr>,
    result: JavaType,
) -> JavaIntrinsicFamily {
    use CoreBinaryIntrinsic as B;
    use CoreUnaryIntrinsic as U;

    enum Family {
        BooleanLogic,
        Equality,
        Ordering,
        CheckedIntegerArithmetic,
        WrappingIntegerArithmetic,
        FloatingPointArithmetic,
        StringConcatenation,
        IntegerBitwise,
        CheckedIntegerShifts,
        FloatingPointInspection,
        StringInspection,
        StringTransformation,
        BytesOperations,
        ListOperations,
        OptionOperations,
        ResultOperations,
        IntegerConversions,
        Utf8Conversions,
    }

    let family = match &value {
        CoreIntrinsicExpr::Unary { operation, .. } => match operation {
            U::BoolNot => Family::BooleanLogic,
            U::IntNegChecked => Family::CheckedIntegerArithmetic,
            U::IntNegWrapping => Family::WrappingIntegerArithmetic,
            U::IntBitNot => Family::IntegerBitwise,
            U::FloatNeg => Family::FloatingPointArithmetic,
            U::FloatTrunc | U::FloatIsNaN | U::FloatIsNegativeZero | U::FloatAbs => {
                Family::FloatingPointInspection
            }
            U::StringScalarLength | U::StringUtf16Length | U::StringIsEmpty => {
                Family::StringInspection
            }
            U::BytesLength | U::BytesIsEmpty => Family::BytesOperations,
            U::ListLength | U::ListIsEmpty => Family::ListOperations,
            U::OptionIsSome | U::OptionIsNone => Family::OptionOperations,
            U::ResultIsOk | U::ResultIsErr => Family::ResultOperations,
            U::WidenI32ToI64 | U::NarrowI64ToI32Checked => Family::IntegerConversions,
            U::StringToUtf8 | U::StringFromUtf8Checked => Family::Utf8Conversions,
        },
        CoreIntrinsicExpr::Binary { operation, .. } => match operation {
            B::BoolAnd | B::BoolOr => Family::BooleanLogic,
            B::Equal | B::NotEqual => Family::Equality,
            B::Less | B::LessEqual | B::Greater | B::GreaterEqual => Family::Ordering,
            B::IntAddChecked
            | B::IntSubChecked
            | B::IntMulChecked
            | B::IntDivChecked
            | B::IntRemChecked => Family::CheckedIntegerArithmetic,
            B::IntAddWrapping | B::IntSubWrapping | B::IntMulWrapping => {
                Family::WrappingIntegerArithmetic
            }
            B::FloatAdd | B::FloatSub | B::FloatMul | B::FloatDiv | B::FloatRemTrunc => {
                Family::FloatingPointArithmetic
            }
            B::StringConcat => Family::StringConcatenation,
            B::IntBitAnd | B::IntBitOr | B::IntBitXor => Family::IntegerBitwise,
            B::IntShiftLeftChecked | B::IntShiftRightChecked => Family::CheckedIntegerShifts,
            B::StringIndexOfLiteral
            | B::StringContains
            | B::StringStartsWith
            | B::StringEndsWith => Family::StringInspection,
            B::StringStripPrefix
            | B::StringTruncateUtf8Bytes
            | B::StringTrimStart
            | B::StringTrimEnd => Family::StringTransformation,
            B::BytesConcat => Family::BytesOperations,
            B::ListGetChecked
            | B::ListAppend
            | B::ListConcat
            | B::ListContains
            | B::ListIndexOf => Family::ListOperations,
            B::OptionUnwrapOr => Family::OptionOperations,
        },
        CoreIntrinsicExpr::Ternary { operation, .. } => match operation {
            CoreTernaryIntrinsic::StringSliceScalars | CoreTernaryIntrinsic::StringReplaceAll => {
                Family::StringTransformation
            }
            CoreTernaryIntrinsic::BytesReplaceAll => Family::BytesOperations,
        },
        CoreIntrinsicExpr::Variadic { operation, .. } => match operation {
            CoreVariadicIntrinsic::StringReplaceMany => Family::StringTransformation,
        },
    };
    match family {
        Family::BooleanLogic => {
            JavaIntrinsicFamily::BooleanLogic(JavaIntrinsicMappingInput::new(value, result))
        }
        Family::Equality => {
            JavaIntrinsicFamily::Equality(JavaIntrinsicMappingInput::new(value, result))
        }
        Family::Ordering => {
            JavaIntrinsicFamily::Ordering(JavaIntrinsicMappingInput::new(value, result))
        }
        Family::CheckedIntegerArithmetic => JavaIntrinsicFamily::CheckedIntegerArithmetic(
            JavaIntrinsicMappingInput::new(value, result),
        ),
        Family::WrappingIntegerArithmetic => JavaIntrinsicFamily::WrappingIntegerArithmetic(
            JavaIntrinsicMappingInput::new(value, result),
        ),
        Family::FloatingPointArithmetic => JavaIntrinsicFamily::FloatingPointArithmetic(
            JavaIntrinsicMappingInput::new(value, result),
        ),
        Family::StringConcatenation => {
            JavaIntrinsicFamily::StringConcatenation(JavaIntrinsicMappingInput::new(value, result))
        }
        Family::IntegerBitwise => {
            JavaIntrinsicFamily::IntegerBitwise(JavaIntrinsicMappingInput::new(value, result))
        }
        Family::CheckedIntegerShifts => {
            JavaIntrinsicFamily::CheckedIntegerShifts(JavaIntrinsicMappingInput::new(value, result))
        }
        Family::FloatingPointInspection => JavaIntrinsicFamily::FloatingPointInspection(
            JavaIntrinsicMappingInput::new(value, result),
        ),
        Family::StringInspection => {
            JavaIntrinsicFamily::StringInspection(JavaIntrinsicMappingInput::new(value, result))
        }
        Family::StringTransformation => {
            JavaIntrinsicFamily::StringTransformation(JavaIntrinsicMappingInput::new(value, result))
        }
        Family::BytesOperations => {
            JavaIntrinsicFamily::BytesOperations(JavaIntrinsicMappingInput::new(value, result))
        }
        Family::ListOperations => {
            JavaIntrinsicFamily::ListOperations(JavaIntrinsicMappingInput::new(value, result))
        }
        Family::OptionOperations => {
            JavaIntrinsicFamily::OptionOperations(JavaIntrinsicMappingInput::new(value, result))
        }
        Family::ResultOperations => {
            JavaIntrinsicFamily::ResultOperations(JavaIntrinsicMappingInput::new(value, result))
        }
        Family::IntegerConversions => {
            JavaIntrinsicFamily::IntegerConversions(JavaIntrinsicMappingInput::new(value, result))
        }
        Family::Utf8Conversions => {
            JavaIntrinsicFamily::Utf8Conversions(JavaIntrinsicMappingInput::new(value, result))
        }
    }
}
