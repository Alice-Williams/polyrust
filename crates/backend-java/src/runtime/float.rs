//! Typed float runtime construction.
use super::declaration_builders::parameter;
use super::member_builders::static_method;

use super::call_builders::{known_call, known_field};
use super::expression_builders::{binary, conditional, double_literal, local};
use crate::ast::{JavaBinaryOperator, JavaMember, JavaPrimitive, JavaStmt, JavaType};
use crate::dialect::{JavaKnownCallable, JavaKnownField, JavaRuntimeCallable};

pub(super) fn float_method(value: JavaRuntimeCallable) -> JavaMember {
    let double = JavaType::primitive(JavaPrimitive::Double);
    let boolean = JavaType::primitive(JavaPrimitive::Boolean);
    let operand = local(double.clone(), "value");
    let returned = match value {
        JavaRuntimeCallable::FloatTrunc => conditional(
            binary(
                JavaBinaryOperator::Greater,
                operand.clone(),
                double_literal(0),
                boolean.clone(),
            ),
            known_call(JavaKnownCallable::MathFloor, vec![operand.clone()]),
            known_call(JavaKnownCallable::MathCeil, vec![operand.clone()]),
            double.clone(),
        ),
        JavaRuntimeCallable::FloatIsNegativeZero => binary(
            JavaBinaryOperator::Equal,
            known_call(
                JavaKnownCallable::DoubleToRawLongBits,
                vec![operand.clone()],
            ),
            known_field(JavaKnownField::LongMinValue),
            boolean.clone(),
        ),
        JavaRuntimeCallable::FloatAbs => known_call(
            JavaKnownCallable::DoubleFromLongBits,
            vec![binary(
                JavaBinaryOperator::BitAnd,
                known_call(JavaKnownCallable::DoubleToRawLongBits, vec![operand]),
                known_field(JavaKnownField::LongMaxValue),
                JavaType::primitive(JavaPrimitive::Long),
            )],
        ),
        _ => unreachable!(),
    };
    static_method(
        vec![],
        returned.ty.clone(),
        value.name(),
        vec![parameter(double, "value")],
        vec![JavaStmt::Return(Some(returned))],
    )
}
