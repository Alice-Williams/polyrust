//! Mapping-owned structural output certificate.
use super::{JavaBooleanLogicInput, JavaBooleanLogicPlan};
use crate::ast::{
    JavaBinaryOperator, JavaExprKind, JavaIdentifier, JavaLocalFinality, JavaStmt, JavaType,
    JavaUnaryOperator, JavaValueRef,
};
use crate::capabilities::support::{java_input_plan, plans::JavaRepresentation as R};
use crate::lower::bool_literal;
java_input_plan!(JavaBooleanLogicInput, JavaBooleanLogicPlan);
fn representation(input: &JavaBooleanLogicInput) -> R {
    match input {
        JavaBooleanLogicInput::Not { .. } => R::Direct,
        JavaBooleanLogicInput::And { right, .. } | JavaBooleanLogicInput::Or { right, .. } => {
            if right.statements.is_empty() {
                R::Direct
            } else {
                R::StructuredControl
            }
        }
    }
}
fn verify(input: &JavaBooleanLogicInput, output: &JavaBooleanLogicPlan) -> bool {
    match input {
        JavaBooleanLogicInput::Not { operand, result } => {
            output.statements == operand.statements
                && &output.value.ty == result
                && matches!(
                    &output.value.kind,
                    JavaExprKind::Unary {
                        operator: JavaUnaryOperator::Not,
                        ..
                    }
                )
        }
        JavaBooleanLogicInput::And {
            left,
            right,
            result_name,
            result,
        } => verify_short_circuit(left, right, result_name, result, false, output),
        JavaBooleanLogicInput::Or {
            left,
            right,
            result_name,
            result,
        } => verify_short_circuit(left, right, result_name, result, true, output),
    }
}
fn verify_short_circuit(
    left: &JavaBooleanLogicPlan,
    right: &JavaBooleanLogicPlan,
    name: &JavaIdentifier,
    ty: &JavaType,
    is_or: bool,
    output: &JavaBooleanLogicPlan,
) -> bool {
    if &output.value.ty != ty {
        return false;
    }
    if right.statements.is_empty() {
        let expected = if is_or {
            JavaBinaryOperator::LogicalOr
        } else {
            JavaBinaryOperator::LogicalAnd
        };
        return output.statements == left.statements
            && matches!(&output.value.kind,
            JavaExprKind::Binary { operator, .. } if *operator == expected);
    }
    if output.value.kind != JavaExprKind::Value(JavaValueRef::Local(name.clone())) {
        return false;
    }
    let Some(owned) = output.statements.strip_prefix(left.statements.as_slice()) else {
        return false;
    };
    let [
        JavaStmt::Local {
            finality: JavaLocalFinality::Mutable,
            ty: local_ty,
            name: local_name,
            value: None,
        },
        JavaStmt::If {
            condition,
            then_block,
            else_block: Some(else_block),
        },
    ] = owned
    else {
        return false;
    };
    if local_ty != ty || local_name != name {
        return false;
    }
    if !is_or
        && (condition.ty != *ty
            || !matches!(
                &condition.kind,
                JavaExprKind::Unary {
                    operator: JavaUnaryOperator::Not,
                    ..
                }
            ))
    {
        return false;
    }
    let [JavaStmt::Assign { target, value }] = then_block.statements.as_slice() else {
        return false;
    };
    if target != &output.value || value != &bool_literal(is_or) {
        return false;
    }
    let Some([JavaStmt::Assign { target, .. }]) = else_block
        .statements
        .strip_prefix(right.statements.as_slice())
    else {
        return false;
    };
    target == &output.value
}
