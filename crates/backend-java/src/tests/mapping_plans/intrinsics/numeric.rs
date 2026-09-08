use super::*;
#[test]
fn wrapping_integer_arithmetic_all_variants() {
    for operand in [integer(), wide()] {
        verify(
            c::JavaWrappingIntegerArithmetic,
            c::wrapping_integer_arithmetic::JavaWrappingIntegerArithmeticInput::Neg {
                operand: operand.clone(),
                result: operand.ty.clone(),
            },
            R::Direct,
        );
        verify(
            c::JavaWrappingIntegerArithmetic,
            c::wrapping_integer_arithmetic::JavaWrappingIntegerArithmeticInput::Add {
                left: operand.clone(),
                right: operand.clone(),
                result: operand.ty.clone(),
            },
            R::Direct,
        );
        verify(
            c::JavaWrappingIntegerArithmetic,
            c::wrapping_integer_arithmetic::JavaWrappingIntegerArithmeticInput::Subtract {
                left: operand.clone(),
                right: operand.clone(),
                result: operand.ty.clone(),
            },
            R::Direct,
        );
        verify(
            c::JavaWrappingIntegerArithmetic,
            c::wrapping_integer_arithmetic::JavaWrappingIntegerArithmeticInput::Multiply {
                left: operand.clone(),
                right: operand.clone(),
                result: operand.ty.clone(),
            },
            R::Direct,
        );
    }
}
#[test]
fn integer_bitwise_all_variants() {
    for operand in [integer(), wide()] {
        verify(
            c::JavaIntegerBitwise,
            c::integer_bitwise::JavaIntegerBitwiseInput::Not {
                operand: operand.clone(),
                result: operand.ty.clone(),
            },
            R::Direct,
        );
        verify(
            c::JavaIntegerBitwise,
            c::integer_bitwise::JavaIntegerBitwiseInput::And {
                left: operand.clone(),
                right: operand.clone(),
                result: operand.ty.clone(),
            },
            R::Direct,
        );
        verify(
            c::JavaIntegerBitwise,
            c::integer_bitwise::JavaIntegerBitwiseInput::Or {
                left: operand.clone(),
                right: operand.clone(),
                result: operand.ty.clone(),
            },
            R::Direct,
        );
        verify(
            c::JavaIntegerBitwise,
            c::integer_bitwise::JavaIntegerBitwiseInput::Xor {
                left: operand.clone(),
                right: operand.clone(),
                result: operand.ty.clone(),
            },
            R::Direct,
        );
    }
}
#[test]
fn floating_point_arithmetic_all_variants() {
    verify(
        c::JavaFloatingPointArithmetic,
        c::floating_point_arithmetic::JavaFloatingPointArithmeticInput::Neg {
            operand: float(),
            result: JavaType::primitive(JavaPrimitive::Double),
        },
        R::Direct,
    );
    verify(
        c::JavaFloatingPointArithmetic,
        c::floating_point_arithmetic::JavaFloatingPointArithmeticInput::Add {
            left: float(),
            right: float(),
            result: JavaType::primitive(JavaPrimitive::Double),
        },
        R::Direct,
    );
    verify(
        c::JavaFloatingPointArithmetic,
        c::floating_point_arithmetic::JavaFloatingPointArithmeticInput::Subtract {
            left: float(),
            right: float(),
            result: JavaType::primitive(JavaPrimitive::Double),
        },
        R::Direct,
    );
    verify(
        c::JavaFloatingPointArithmetic,
        c::floating_point_arithmetic::JavaFloatingPointArithmeticInput::Multiply {
            left: float(),
            right: float(),
            result: JavaType::primitive(JavaPrimitive::Double),
        },
        R::Direct,
    );
    verify(
        c::JavaFloatingPointArithmetic,
        c::floating_point_arithmetic::JavaFloatingPointArithmeticInput::Divide {
            left: float(),
            right: float(),
            result: JavaType::primitive(JavaPrimitive::Double),
        },
        R::Direct,
    );
    verify(
        c::JavaFloatingPointArithmetic,
        c::floating_point_arithmetic::JavaFloatingPointArithmeticInput::Remainder {
            left: float(),
            right: float(),
            result: JavaType::primitive(JavaPrimitive::Double),
        },
        R::Direct,
    );
}
#[test]
fn checked_integer_arithmetic_all_variants() {
    for operand in [integer(), wide()] {
        verify(
            c::JavaCheckedIntegerArithmetic,
            c::checked_integer_arithmetic::JavaCheckedIntegerArithmeticInput::Neg {
                operand: operand.clone(),
                result: operand.ty.clone(),
            },
            R::RuntimeHelper,
        );
        verify(
            c::JavaCheckedIntegerArithmetic,
            c::checked_integer_arithmetic::JavaCheckedIntegerArithmeticInput::Add {
                left: operand.clone(),
                right: operand.clone(),
                result: operand.ty.clone(),
            },
            R::RuntimeHelper,
        );
        verify(
            c::JavaCheckedIntegerArithmetic,
            c::checked_integer_arithmetic::JavaCheckedIntegerArithmeticInput::Subtract {
                left: operand.clone(),
                right: operand.clone(),
                result: operand.ty.clone(),
            },
            R::RuntimeHelper,
        );
        verify(
            c::JavaCheckedIntegerArithmetic,
            c::checked_integer_arithmetic::JavaCheckedIntegerArithmeticInput::Multiply {
                left: operand.clone(),
                right: operand.clone(),
                result: operand.ty.clone(),
            },
            R::RuntimeHelper,
        );
        verify(
            c::JavaCheckedIntegerArithmetic,
            c::checked_integer_arithmetic::JavaCheckedIntegerArithmeticInput::Divide {
                left: operand.clone(),
                right: operand.clone(),
                result: operand.ty.clone(),
            },
            R::RuntimeHelper,
        );
        verify(
            c::JavaCheckedIntegerArithmetic,
            c::checked_integer_arithmetic::JavaCheckedIntegerArithmeticInput::Remainder {
                left: operand.clone(),
                right: operand.clone(),
                result: operand.ty.clone(),
            },
            R::RuntimeHelper,
        );
    }
}
#[test]
fn checked_integer_shifts_all_variants() {
    for operand in [integer(), wide()] {
        verify(
            c::JavaCheckedIntegerShifts,
            c::checked_integer_shifts::JavaCheckedIntegerShiftsInput::Left {
                value: operand.clone(),
                distance: integer(),
                result: operand.ty.clone(),
            },
            R::RuntimeHelper,
        );
        verify(
            c::JavaCheckedIntegerShifts,
            c::checked_integer_shifts::JavaCheckedIntegerShiftsInput::Right {
                value: operand.clone(),
                distance: integer(),
                result: operand.ty.clone(),
            },
            R::RuntimeHelper,
        );
    }
}
#[test]
fn floating_point_inspection_all_variants() {
    verify(
        c::JavaFloatingPointInspection,
        c::floating_point_inspection::JavaFloatingPointInspectionInput::Truncate {
            operand: float(),
            result: JavaType::primitive(JavaPrimitive::Double),
        },
        R::RuntimeHelper,
    );
    verify(
        c::JavaFloatingPointInspection,
        c::floating_point_inspection::JavaFloatingPointInspectionInput::IsNan {
            operand: float(),
            result: boolean(),
        },
        R::Direct,
    );
    verify(
        c::JavaFloatingPointInspection,
        c::floating_point_inspection::JavaFloatingPointInspectionInput::IsNegativeZero {
            operand: float(),
            result: boolean(),
        },
        R::RuntimeHelper,
    );
    verify(
        c::JavaFloatingPointInspection,
        c::floating_point_inspection::JavaFloatingPointInspectionInput::Absolute {
            operand: float(),
            result: JavaType::primitive(JavaPrimitive::Double),
        },
        R::RuntimeHelper,
    );
}
#[test]
fn integer_conversions_all_variants() {
    verify(
        c::JavaIntegerConversions,
        c::integer_conversions::JavaIntegerConversionsInput::WidenI32ToI64 {
            operand: integer(),
            result: long(),
        },
        R::Direct,
    );
    verify(
        c::JavaIntegerConversions,
        c::integer_conversions::JavaIntegerConversionsInput::NarrowI64ToI32Checked {
            operand: wide(),
            result: JavaType::primitive(JavaPrimitive::Int),
        },
        R::RuntimeHelper,
    );
}
#[test]
fn equality_all_variants() {
    verify(
        c::JavaEquality,
        c::equality::JavaEqualityInput::Equal {
            left: integer(),
            right: integer(),
            result: boolean(),
        },
        R::RuntimeHelper,
    );
    verify(
        c::JavaEquality,
        c::equality::JavaEqualityInput::NotEqual {
            left: integer(),
            right: integer(),
            result: boolean(),
        },
        R::RuntimeHelper,
    );
}
#[test]
fn ordering_all_variants() {
    for (operand, expected) in [
        (integer(), R::Direct),
        (wide(), R::Direct),
        (float(), R::Direct),
        (text(), R::RuntimeHelper),
        (
            value(JavaType::known(JavaKnownType::RuntimeScalar)),
            R::RuntimeHelper,
        ),
    ] {
        verify(
            c::JavaOrdering,
            c::ordering::JavaOrderingInput::Less {
                left: operand.clone(),
                right: operand.clone(),
                result: boolean(),
            },
            expected,
        );
        verify(
            c::JavaOrdering,
            c::ordering::JavaOrderingInput::LessEqual {
                left: operand.clone(),
                right: operand.clone(),
                result: boolean(),
            },
            expected,
        );
        verify(
            c::JavaOrdering,
            c::ordering::JavaOrderingInput::Greater {
                left: operand.clone(),
                right: operand.clone(),
                result: boolean(),
            },
            expected,
        );
        verify(
            c::JavaOrdering,
            c::ordering::JavaOrderingInput::GreaterEqual {
                left: operand.clone(),
                right: operand.clone(),
                result: boolean(),
            },
            expected,
        );
    }
}
