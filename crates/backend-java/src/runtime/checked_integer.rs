//! Typed runtime construction: checked integer.
use super::declaration_builders::{generic, identifier, parameter};
use super::member_builders::static_method;

use super::call_builders::{
    JavaRuntimeFailure, known_call, known_field, known_method_call, runtime_fail, runtime_ok,
};
use super::expression_builders::{binary, cast, int_literal, local, long_literal, unary};
use crate::ast::{
    JavaBinaryOperator, JavaBlock, JavaExpr, JavaKnownType, JavaLocalFinality, JavaMember,
    JavaPrimitive, JavaStmt, JavaType, JavaUnaryOperator,
};
use crate::dialect::{JavaKnownCallable, JavaKnownField, JavaKnownMethod, JavaRuntimeCallable};

pub(super) fn checked_integer_method(value: JavaRuntimeCallable) -> JavaMember {
    let int = JavaType::primitive(JavaPrimitive::Int);
    let long = JavaType::primitive(JavaPrimitive::Long);
    let bigint = JavaType::known(JavaKnownType::BigInteger);
    let wide = matches!(
        value,
        JavaRuntimeCallable::CheckedNegI64
            | JavaRuntimeCallable::CheckedAddI64
            | JavaRuntimeCallable::CheckedSubI64
            | JavaRuntimeCallable::CheckedMulI64
            | JavaRuntimeCallable::CheckedDivI64
            | JavaRuntimeCallable::CheckedRemI64
            | JavaRuntimeCallable::CheckedShiftLeftI64
            | JavaRuntimeCallable::CheckedShiftRightI64
    );
    let operand = if wide { long.clone() } else { int.clone() };
    let boxed = if wide {
        JavaType::Boxed(JavaPrimitive::Long)
    } else {
        JavaType::Boxed(JavaPrimitive::Int)
    };
    let result = generic(JavaKnownType::RuntimeResult, vec![boxed]);

    if value == JavaRuntimeCallable::NarrowI64ToI32 {
        let narrow_result = generic(
            JavaKnownType::RuntimeResult,
            vec![JavaType::Boxed(JavaPrimitive::Int)],
        );
        return static_method(
            vec![],
            narrow_result.clone(),
            value.name(),
            vec![parameter(long.clone(), "value")],
            checked_bounds_statements(
                local(long, "value"),
                narrow_result,
                JavaKnownField::IntegerMinValue,
                JavaKnownField::IntegerMaxValue,
                int,
                JavaRuntimeFailure::NarrowingOutOfRange,
            ),
        );
    }

    if matches!(
        value,
        JavaRuntimeCallable::CheckedShiftLeftI32
            | JavaRuntimeCallable::CheckedShiftLeftI64
            | JavaRuntimeCallable::CheckedShiftRightI32
            | JavaRuntimeCallable::CheckedShiftRightI64
    ) {
        let amount = operand.clone();
        let right = local(amount.clone(), "right");
        let invalid = binary(
            JavaBinaryOperator::LogicalOr,
            binary(
                JavaBinaryOperator::Less,
                right.clone(),
                if wide {
                    long_literal(0)
                } else {
                    int_literal(0)
                },
                JavaType::primitive(JavaPrimitive::Boolean),
            ),
            binary(
                JavaBinaryOperator::GreaterEqual,
                right.clone(),
                if wide {
                    long_literal(64)
                } else {
                    int_literal(32)
                },
                JavaType::primitive(JavaPrimitive::Boolean),
            ),
            JavaType::primitive(JavaPrimitive::Boolean),
        );
        let shift_amount = if wide {
            cast(int.clone(), right)
        } else {
            right
        };
        let shifted = binary(
            if matches!(
                value,
                JavaRuntimeCallable::CheckedShiftLeftI32 | JavaRuntimeCallable::CheckedShiftLeftI64
            ) {
                JavaBinaryOperator::ShiftLeft
            } else {
                JavaBinaryOperator::ShiftRight
            },
            local(operand.clone(), "left"),
            shift_amount,
            operand.clone(),
        );
        return static_method(
            vec![],
            result.clone(),
            value.name(),
            vec![
                parameter(operand.clone(), "left"),
                parameter(amount, "right"),
            ],
            vec![
                JavaStmt::If {
                    condition: invalid,
                    then_block: JavaBlock::new(vec![JavaStmt::Return(Some(runtime_fail(
                        result.clone(),
                        JavaRuntimeFailure::InvalidShift,
                    )))]),
                    else_block: None,
                },
                JavaStmt::Return(Some(runtime_ok(result, shifted))),
            ],
        );
    }

    if matches!(
        value,
        JavaRuntimeCallable::CheckedDivI32
            | JavaRuntimeCallable::CheckedDivI64
            | JavaRuntimeCallable::CheckedRemI32
            | JavaRuntimeCallable::CheckedRemI64
    ) {
        let left = local(operand.clone(), "left");
        let right = local(operand.clone(), "right");
        let zero = if wide {
            long_literal(0)
        } else {
            int_literal(0)
        };
        let minus_one = if wide {
            long_literal(-1)
        } else {
            int_literal(-1)
        };
        let minimum = known_field(if wide {
            JavaKnownField::LongMinValue
        } else {
            JavaKnownField::IntegerMinValue
        });
        let division = matches!(
            value,
            JavaRuntimeCallable::CheckedDivI32 | JavaRuntimeCallable::CheckedDivI64
        );
        let mut statements = vec![JavaStmt::If {
            condition: binary(
                JavaBinaryOperator::Equal,
                right.clone(),
                zero,
                JavaType::primitive(JavaPrimitive::Boolean),
            ),
            then_block: JavaBlock::new(vec![JavaStmt::Return(Some(runtime_fail(
                result.clone(),
                if division {
                    JavaRuntimeFailure::DivisionByZero
                } else {
                    JavaRuntimeFailure::RemainderByZero
                },
            )))]),
            else_block: None,
        }];
        statements.push(JavaStmt::If {
            condition: binary(
                JavaBinaryOperator::LogicalAnd,
                binary(
                    JavaBinaryOperator::Equal,
                    left.clone(),
                    minimum,
                    JavaType::primitive(JavaPrimitive::Boolean),
                ),
                binary(
                    JavaBinaryOperator::Equal,
                    right.clone(),
                    minus_one,
                    JavaType::primitive(JavaPrimitive::Boolean),
                ),
                JavaType::primitive(JavaPrimitive::Boolean),
            ),
            then_block: JavaBlock::new(vec![JavaStmt::Return(Some(runtime_fail(
                result.clone(),
                JavaRuntimeFailure::CheckedOverflow,
            )))]),
            else_block: None,
        });
        statements.push(JavaStmt::Return(Some(runtime_ok(
            result.clone(),
            binary(
                if division {
                    JavaBinaryOperator::Divide
                } else {
                    JavaBinaryOperator::Remainder
                },
                left,
                right,
                operand.clone(),
            ),
        ))));
        return static_method(
            vec![],
            result,
            value.name(),
            vec![
                parameter(operand.clone(), "left"),
                parameter(operand, "right"),
            ],
            statements,
        );
    }

    let unary_operation = matches!(
        value,
        JavaRuntimeCallable::CheckedNegI32 | JavaRuntimeCallable::CheckedNegI64
    );
    let result_expression = if wide {
        let left = known_call(
            JavaKnownCallable::BigIntegerValueOf,
            vec![local(long.clone(), "value")],
        );
        if unary_operation {
            known_method_call(
                JavaKnownMethod::BigIntegerNegate,
                left,
                vec![],
                bigint.clone(),
            )
        } else {
            let right = known_call(
                JavaKnownCallable::BigIntegerValueOf,
                vec![local(long.clone(), "right")],
            );
            known_method_call(
                match value {
                    JavaRuntimeCallable::CheckedAddI64 => JavaKnownMethod::BigIntegerAdd,
                    JavaRuntimeCallable::CheckedSubI64 => JavaKnownMethod::BigIntegerSubtract,
                    JavaRuntimeCallable::CheckedMulI64 => JavaKnownMethod::BigIntegerMultiply,
                    _ => unreachable!(),
                },
                known_call(
                    JavaKnownCallable::BigIntegerValueOf,
                    vec![local(long.clone(), "left")],
                ),
                vec![right],
                bigint.clone(),
            )
        }
    } else {
        let long_left = cast(
            long.clone(),
            local(int.clone(), if unary_operation { "value" } else { "left" }),
        );
        if unary_operation {
            unary(JavaUnaryOperator::Negate, long_left, long.clone())
        } else {
            binary(
                match value {
                    JavaRuntimeCallable::CheckedAddI32 => JavaBinaryOperator::Add,
                    JavaRuntimeCallable::CheckedSubI32 => JavaBinaryOperator::Subtract,
                    JavaRuntimeCallable::CheckedMulI32 => JavaBinaryOperator::Multiply,
                    _ => unreachable!(),
                },
                long_left,
                cast(long.clone(), local(int.clone(), "right")),
                long.clone(),
            )
        }
    };
    let statements = if wide {
        checked_bigint_statements(result_expression, result.clone(), long.clone())
    } else {
        checked_bounds_statements(
            result_expression,
            result.clone(),
            JavaKnownField::IntegerMinValue,
            JavaKnownField::IntegerMaxValue,
            int.clone(),
            JavaRuntimeFailure::CheckedOverflow,
        )
    };
    let parameters = if unary_operation {
        vec![parameter(operand, "value")]
    } else {
        vec![
            parameter(operand.clone(), "left"),
            parameter(operand, "right"),
        ]
    };
    static_method(vec![], result, value.name(), parameters, statements)
}

fn checked_bounds_statements(
    candidate: JavaExpr,
    result: JavaType,
    minimum: JavaKnownField,
    maximum: JavaKnownField,
    output: JavaType,
    failure: JavaRuntimeFailure,
) -> Vec<JavaStmt> {
    let candidate_type = candidate.ty.clone();
    vec![
        JavaStmt::Local {
            finality: JavaLocalFinality::Final,
            ty: candidate_type.clone(),
            name: identifier("candidate"),
            value: Some(candidate),
        },
        JavaStmt::If {
            condition: binary(
                JavaBinaryOperator::LogicalOr,
                binary(
                    JavaBinaryOperator::Less,
                    local(candidate_type.clone(), "candidate"),
                    cast(candidate_type.clone(), known_field(minimum)),
                    JavaType::primitive(JavaPrimitive::Boolean),
                ),
                binary(
                    JavaBinaryOperator::Greater,
                    local(candidate_type.clone(), "candidate"),
                    cast(candidate_type.clone(), known_field(maximum)),
                    JavaType::primitive(JavaPrimitive::Boolean),
                ),
                JavaType::primitive(JavaPrimitive::Boolean),
            ),
            then_block: JavaBlock::new(vec![JavaStmt::Return(Some(runtime_fail(
                result.clone(),
                failure,
            )))]),
            else_block: None,
        },
        JavaStmt::Return(Some(runtime_ok(
            result,
            cast(output, local(candidate_type, "candidate")),
        ))),
    ]
}

fn checked_bigint_statements(
    candidate: JavaExpr,
    result: JavaType,
    output: JavaType,
) -> Vec<JavaStmt> {
    let bigint = JavaType::known(JavaKnownType::BigInteger);
    let minimum = known_call(
        JavaKnownCallable::BigIntegerValueOf,
        vec![known_field(JavaKnownField::LongMinValue)],
    );
    let maximum = known_call(
        JavaKnownCallable::BigIntegerValueOf,
        vec![known_field(JavaKnownField::LongMaxValue)],
    );
    vec![
        JavaStmt::Local {
            finality: JavaLocalFinality::Final,
            ty: bigint.clone(),
            name: identifier("candidate"),
            value: Some(candidate),
        },
        JavaStmt::If {
            condition: binary(
                JavaBinaryOperator::LogicalOr,
                binary(
                    JavaBinaryOperator::Less,
                    known_method_call(
                        JavaKnownMethod::BigIntegerCompareTo,
                        local(bigint.clone(), "candidate"),
                        vec![minimum],
                        JavaType::primitive(JavaPrimitive::Int),
                    ),
                    int_literal(0),
                    JavaType::primitive(JavaPrimitive::Boolean),
                ),
                binary(
                    JavaBinaryOperator::Greater,
                    known_method_call(
                        JavaKnownMethod::BigIntegerCompareTo,
                        local(bigint.clone(), "candidate"),
                        vec![maximum],
                        JavaType::primitive(JavaPrimitive::Int),
                    ),
                    int_literal(0),
                    JavaType::primitive(JavaPrimitive::Boolean),
                ),
                JavaType::primitive(JavaPrimitive::Boolean),
            ),
            then_block: JavaBlock::new(vec![JavaStmt::Return(Some(runtime_fail(
                result.clone(),
                JavaRuntimeFailure::CheckedOverflow,
            )))]),
            else_block: None,
        },
        JavaStmt::Return(Some(runtime_ok(
            result,
            known_method_call(
                JavaKnownMethod::BigIntegerLongValue,
                local(bigint, "candidate"),
                vec![],
                output,
            ),
        ))),
    ]
}
