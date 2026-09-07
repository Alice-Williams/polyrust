//! Exhaustive semantic dispatch from verified CoreIR to exact capability inputs.

use portable_core_ir::{
    CoreBinaryIntrinsic as B, CoreIntrinsicExpr, CoreTernaryIntrinsic as T,
    CoreUnaryIntrinsic as U, CoreVariadicIntrinsic as V,
};

use super::{
    JavaBytesOperationsInput, JavaCheckedIntegerArithmeticInput, JavaCheckedIntegerShiftsInput,
    JavaEqualityInput, JavaFloatingPointArithmeticInput, JavaFloatingPointInspectionInput,
    JavaIntegerBitwiseInput, JavaIntegerConversionsInput, JavaListOperationsInput,
    JavaOptionOperationsInput, JavaOrderingInput, JavaResultOperationsInput,
    JavaStringConcatenationInput, JavaStringInspectionInput, JavaStringTransformationInput,
    JavaUtf8ConversionsInput, JavaWrappingIntegerArithmeticInput,
};
use crate::{
    ast::{JavaExpr, JavaType},
    lower::diagnostic,
};

pub(crate) enum JavaIntrinsicFamily {
    Equality(JavaEqualityInput),
    Ordering(JavaOrderingInput),
    CheckedIntegerArithmetic(JavaCheckedIntegerArithmeticInput),
    WrappingIntegerArithmetic(JavaWrappingIntegerArithmeticInput),
    FloatingPointArithmetic(JavaFloatingPointArithmeticInput),
    StringConcatenation(JavaStringConcatenationInput),
    IntegerBitwise(JavaIntegerBitwiseInput),
    CheckedIntegerShifts(JavaCheckedIntegerShiftsInput),
    FloatingPointInspection(JavaFloatingPointInspectionInput),
    StringInspection(JavaStringInspectionInput),
    StringTransformation(Box<JavaStringTransformationInput>),
    BytesOperations(JavaBytesOperationsInput),
    ListOperations(JavaListOperationsInput),
    OptionOperations(JavaOptionOperationsInput),
    ResultOperations(JavaResultOperationsInput),
    IntegerConversions(JavaIntegerConversionsInput),
    Utf8Conversions(JavaUtf8ConversionsInput),
}

pub(crate) fn classify_intrinsic(
    value: CoreIntrinsicExpr<JavaExpr>,
    result: JavaType,
) -> Result<JavaIntrinsicFamily, Vec<portable_diagnostics::Diagnostic>> {
    Ok(match value {
        CoreIntrinsicExpr::Unary { operation, operand } => match operation {
            U::BoolNot => {
                return Err(vec![diagnostic(
                    "boolean logic must be lowered with its evaluation plan",
                )]);
            }
            U::IntNegChecked => JavaIntrinsicFamily::CheckedIntegerArithmetic(
                JavaCheckedIntegerArithmeticInput::Neg { operand, result },
            ),
            U::IntNegWrapping => JavaIntrinsicFamily::WrappingIntegerArithmetic(
                JavaWrappingIntegerArithmeticInput::Neg { operand, result },
            ),
            U::IntBitNot => JavaIntrinsicFamily::IntegerBitwise(JavaIntegerBitwiseInput::Not {
                operand,
                result,
            }),
            U::FloatNeg => JavaIntrinsicFamily::FloatingPointArithmetic(
                JavaFloatingPointArithmeticInput::Neg { operand, result },
            ),
            U::FloatTrunc => JavaIntrinsicFamily::FloatingPointInspection(
                JavaFloatingPointInspectionInput::Truncate { operand, result },
            ),
            U::FloatIsNaN => JavaIntrinsicFamily::FloatingPointInspection(
                JavaFloatingPointInspectionInput::IsNan { operand, result },
            ),
            U::FloatIsNegativeZero => JavaIntrinsicFamily::FloatingPointInspection(
                JavaFloatingPointInspectionInput::IsNegativeZero { operand, result },
            ),
            U::FloatAbs => JavaIntrinsicFamily::FloatingPointInspection(
                JavaFloatingPointInspectionInput::Absolute { operand, result },
            ),
            U::StringScalarLength => {
                JavaIntrinsicFamily::StringInspection(JavaStringInspectionInput::ScalarLength {
                    source: operand,
                    result,
                })
            }
            U::StringUtf16Length => {
                JavaIntrinsicFamily::StringInspection(JavaStringInspectionInput::Utf16Length {
                    source: operand,
                    result,
                })
            }
            U::StringIsEmpty => {
                JavaIntrinsicFamily::StringInspection(JavaStringInspectionInput::IsEmpty {
                    source: operand,
                    result,
                })
            }
            U::BytesLength => {
                JavaIntrinsicFamily::BytesOperations(JavaBytesOperationsInput::Length {
                    bytes: operand,
                    result,
                })
            }
            U::BytesIsEmpty => {
                JavaIntrinsicFamily::BytesOperations(JavaBytesOperationsInput::IsEmpty {
                    bytes: operand,
                    result,
                })
            }
            U::ListLength => JavaIntrinsicFamily::ListOperations(JavaListOperationsInput::Length {
                list: operand,
                result,
            }),
            U::ListIsEmpty => {
                JavaIntrinsicFamily::ListOperations(JavaListOperationsInput::IsEmpty {
                    list: operand,
                    result,
                })
            }
            U::OptionIsSome => {
                JavaIntrinsicFamily::OptionOperations(JavaOptionOperationsInput::IsSome {
                    operand,
                    result,
                })
            }
            U::OptionIsNone => {
                JavaIntrinsicFamily::OptionOperations(JavaOptionOperationsInput::IsNone {
                    operand,
                    result,
                })
            }
            U::ResultIsOk => {
                JavaIntrinsicFamily::ResultOperations(JavaResultOperationsInput::IsOk {
                    operand,
                    result,
                })
            }
            U::ResultIsErr => {
                JavaIntrinsicFamily::ResultOperations(JavaResultOperationsInput::IsErr {
                    operand,
                    result,
                })
            }
            U::WidenI32ToI64 => JavaIntrinsicFamily::IntegerConversions(
                JavaIntegerConversionsInput::WidenI32ToI64 { operand, result },
            ),
            U::NarrowI64ToI32Checked => JavaIntrinsicFamily::IntegerConversions(
                JavaIntegerConversionsInput::NarrowI64ToI32Checked { operand, result },
            ),
            U::StringToUtf8 => {
                JavaIntrinsicFamily::Utf8Conversions(JavaUtf8ConversionsInput::Encode {
                    operand,
                    result,
                })
            }
            U::StringFromUtf8Checked => {
                JavaIntrinsicFamily::Utf8Conversions(JavaUtf8ConversionsInput::DecodeChecked {
                    operand,
                    result,
                })
            }
        },
        CoreIntrinsicExpr::Binary {
            operation,
            left,
            right,
        } => match operation {
            B::BoolAnd | B::BoolOr => {
                return Err(vec![diagnostic(
                    "boolean logic must be lowered with its evaluation plan",
                )]);
            }
            B::Equal => JavaIntrinsicFamily::Equality(JavaEqualityInput::Equal {
                left,
                right,
                result,
            }),
            B::NotEqual => JavaIntrinsicFamily::Equality(JavaEqualityInput::NotEqual {
                left,
                right,
                result,
            }),
            B::Less => JavaIntrinsicFamily::Ordering(JavaOrderingInput::Less {
                left,
                right,
                result,
            }),
            B::LessEqual => JavaIntrinsicFamily::Ordering(JavaOrderingInput::LessEqual {
                left,
                right,
                result,
            }),
            B::Greater => JavaIntrinsicFamily::Ordering(JavaOrderingInput::Greater {
                left,
                right,
                result,
            }),
            B::GreaterEqual => JavaIntrinsicFamily::Ordering(JavaOrderingInput::GreaterEqual {
                left,
                right,
                result,
            }),
            B::IntAddChecked => JavaIntrinsicFamily::CheckedIntegerArithmetic(
                JavaCheckedIntegerArithmeticInput::Add {
                    left,
                    right,
                    result,
                },
            ),
            B::IntSubChecked => JavaIntrinsicFamily::CheckedIntegerArithmetic(
                JavaCheckedIntegerArithmeticInput::Subtract {
                    left,
                    right,
                    result,
                },
            ),
            B::IntMulChecked => JavaIntrinsicFamily::CheckedIntegerArithmetic(
                JavaCheckedIntegerArithmeticInput::Multiply {
                    left,
                    right,
                    result,
                },
            ),
            B::IntDivChecked => JavaIntrinsicFamily::CheckedIntegerArithmetic(
                JavaCheckedIntegerArithmeticInput::Divide {
                    left,
                    right,
                    result,
                },
            ),
            B::IntRemChecked => JavaIntrinsicFamily::CheckedIntegerArithmetic(
                JavaCheckedIntegerArithmeticInput::Remainder {
                    left,
                    right,
                    result,
                },
            ),
            B::IntAddWrapping => JavaIntrinsicFamily::WrappingIntegerArithmetic(
                JavaWrappingIntegerArithmeticInput::Add {
                    left,
                    right,
                    result,
                },
            ),
            B::IntSubWrapping => JavaIntrinsicFamily::WrappingIntegerArithmetic(
                JavaWrappingIntegerArithmeticInput::Subtract {
                    left,
                    right,
                    result,
                },
            ),
            B::IntMulWrapping => JavaIntrinsicFamily::WrappingIntegerArithmetic(
                JavaWrappingIntegerArithmeticInput::Multiply {
                    left,
                    right,
                    result,
                },
            ),
            B::FloatAdd => JavaIntrinsicFamily::FloatingPointArithmetic(
                JavaFloatingPointArithmeticInput::Add {
                    left,
                    right,
                    result,
                },
            ),
            B::FloatSub => JavaIntrinsicFamily::FloatingPointArithmetic(
                JavaFloatingPointArithmeticInput::Subtract {
                    left,
                    right,
                    result,
                },
            ),
            B::FloatMul => JavaIntrinsicFamily::FloatingPointArithmetic(
                JavaFloatingPointArithmeticInput::Multiply {
                    left,
                    right,
                    result,
                },
            ),
            B::FloatDiv => JavaIntrinsicFamily::FloatingPointArithmetic(
                JavaFloatingPointArithmeticInput::Divide {
                    left,
                    right,
                    result,
                },
            ),
            B::FloatRemTrunc => JavaIntrinsicFamily::FloatingPointArithmetic(
                JavaFloatingPointArithmeticInput::Remainder {
                    left,
                    right,
                    result,
                },
            ),
            B::StringConcat => {
                JavaIntrinsicFamily::StringConcatenation(JavaStringConcatenationInput {
                    left,
                    right,
                    result,
                })
            }
            B::IntBitAnd => JavaIntrinsicFamily::IntegerBitwise(JavaIntegerBitwiseInput::And {
                left,
                right,
                result,
            }),
            B::IntBitOr => JavaIntrinsicFamily::IntegerBitwise(JavaIntegerBitwiseInput::Or {
                left,
                right,
                result,
            }),
            B::IntBitXor => JavaIntrinsicFamily::IntegerBitwise(JavaIntegerBitwiseInput::Xor {
                left,
                right,
                result,
            }),
            B::IntShiftLeftChecked => {
                JavaIntrinsicFamily::CheckedIntegerShifts(JavaCheckedIntegerShiftsInput::Left {
                    value: left,
                    distance: right,
                    result,
                })
            }
            B::IntShiftRightChecked => {
                JavaIntrinsicFamily::CheckedIntegerShifts(JavaCheckedIntegerShiftsInput::Right {
                    value: left,
                    distance: right,
                    result,
                })
            }
            B::StringIndexOfLiteral => {
                JavaIntrinsicFamily::StringInspection(JavaStringInspectionInput::IndexOfLiteral {
                    source: left,
                    needle: right,
                    result,
                })
            }
            B::StringContains => {
                JavaIntrinsicFamily::StringInspection(JavaStringInspectionInput::Contains {
                    source: left,
                    needle: right,
                    result,
                })
            }
            B::StringStartsWith => {
                JavaIntrinsicFamily::StringInspection(JavaStringInspectionInput::StartsWith {
                    source: left,
                    prefix: right,
                    result,
                })
            }
            B::StringEndsWith => {
                JavaIntrinsicFamily::StringInspection(JavaStringInspectionInput::EndsWith {
                    source: left,
                    suffix: right,
                    result,
                })
            }
            B::StringStripPrefix => JavaIntrinsicFamily::StringTransformation(Box::new(
                JavaStringTransformationInput::StripPrefix {
                    source: left,
                    prefix: right,
                    result,
                },
            )),
            B::StringTruncateUtf8Bytes => JavaIntrinsicFamily::StringTransformation(Box::new(
                JavaStringTransformationInput::TruncateUtf8Bytes {
                    source: left,
                    budget: right,
                    result,
                },
            )),
            B::StringTrimStart => JavaIntrinsicFamily::StringTransformation(Box::new(
                JavaStringTransformationInput::TrimStart {
                    source: left,
                    characters: right,
                    result,
                },
            )),
            B::StringTrimEnd => JavaIntrinsicFamily::StringTransformation(Box::new(
                JavaStringTransformationInput::TrimEnd {
                    source: left,
                    characters: right,
                    result,
                },
            )),
            B::BytesConcat => {
                JavaIntrinsicFamily::BytesOperations(JavaBytesOperationsInput::Concat {
                    left,
                    right,
                    result,
                })
            }
            B::ListGetChecked => {
                JavaIntrinsicFamily::ListOperations(JavaListOperationsInput::GetChecked {
                    list: left,
                    index: right,
                    result,
                })
            }
            B::ListAppend => JavaIntrinsicFamily::ListOperations(JavaListOperationsInput::Append {
                list: left,
                value: right,
                result,
            }),
            B::ListConcat => JavaIntrinsicFamily::ListOperations(JavaListOperationsInput::Concat {
                left,
                right,
                result,
            }),
            B::ListContains => {
                JavaIntrinsicFamily::ListOperations(JavaListOperationsInput::Contains {
                    list: left,
                    value: right,
                    result,
                })
            }
            B::ListIndexOf => {
                JavaIntrinsicFamily::ListOperations(JavaListOperationsInput::IndexOf {
                    list: left,
                    value: right,
                    result,
                })
            }
            B::OptionUnwrapOr => {
                JavaIntrinsicFamily::OptionOperations(JavaOptionOperationsInput::UnwrapOr {
                    option: left,
                    fallback: Box::new(right),
                    result,
                })
            }
        },
        CoreIntrinsicExpr::Ternary {
            operation,
            first,
            second,
            third,
        } => match operation {
            T::StringSliceScalars => JavaIntrinsicFamily::StringTransformation(Box::new(
                JavaStringTransformationInput::SliceScalars {
                    source: first,
                    start: second,
                    end: third,
                    result,
                },
            )),
            T::StringReplaceAll => JavaIntrinsicFamily::StringTransformation(Box::new(
                JavaStringTransformationInput::ReplaceAll {
                    source: first,
                    needle: second,
                    replacement: third,
                    result,
                },
            )),
            T::BytesReplaceAll => {
                JavaIntrinsicFamily::BytesOperations(JavaBytesOperationsInput::ReplaceAll {
                    source: first,
                    needle: second,
                    replacement: Box::new(third),
                    result,
                })
            }
        },
        CoreIntrinsicExpr::Variadic {
            operation: V::StringReplaceMany,
            arguments,
        } => {
            let mut arguments = arguments.into_iter();
            let source = arguments
                .next()
                .ok_or_else(|| vec![diagnostic("replace-many source missing")])?;
            JavaIntrinsicFamily::StringTransformation(Box::new(
                JavaStringTransformationInput::ReplaceMany {
                    source,
                    replacements: arguments.collect(),
                    result,
                },
            ))
        }
    })
}
