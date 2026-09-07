//! Java AST: completion.

use super::blocks::JavaBlock;
use super::expression_model::{JavaBinaryOperator, JavaLiteral, JavaUnaryOperator, JavaValueRef};
use super::expression_nodes::{JavaExpr, JavaExprKind};
use super::statement_model::{JavaPattern, JavaStmt};
use super::types::{JavaPrimitive, JavaType};

pub(super) fn block_guarantees_exit(block: &JavaBlock) -> bool {
    !block_can_complete_normally(block)
}

fn block_can_complete_normally(block: &JavaBlock) -> bool {
    let mut can_complete_normally = true;
    for statement in &block.statements {
        if !can_complete_normally {
            return false;
        }
        can_complete_normally = statement_can_complete_normally(statement);
    }
    can_complete_normally
}

pub(super) fn statement_can_complete_normally(statement: &JavaStmt) -> bool {
    match statement {
        JavaStmt::Return(_)
        | JavaStmt::Throw(_)
        | JavaStmt::ThrowAssertion(_)
        | JavaStmt::Break
        | JavaStmt::Continue => false,
        JavaStmt::If {
            then_block,
            else_block: Some(else_block),
            ..
        } => block_can_complete_normally(then_block) || block_can_complete_normally(else_block),
        JavaStmt::Switch { arms, .. }
            if arms
                .iter()
                .any(|arm| matches!(arm.pattern, JavaPattern::Default)) =>
        {
            arms.iter()
                .any(|arm| block_can_complete_normally(&arm.body))
        }
        JavaStmt::TryCatch { try_block, catches } => {
            block_can_complete_normally(try_block)
                || catches
                    .iter()
                    .any(|catch| block_can_complete_normally(&catch.body))
        }
        JavaStmt::While { condition, body } => match java_loop_condition(condition) {
            JavaLoopCondition::Always => block_has_reachable_break_for_current_loop(body),
            JavaLoopCondition::Never
            | JavaLoopCondition::Dynamic
            | JavaLoopCondition::UnsupportedConstantForm => true,
        },
        JavaStmt::Local { .. }
        | JavaStmt::Assign { .. }
        | JavaStmt::Expression(_)
        | JavaStmt::If {
            else_block: None, ..
        }
        | JavaStmt::ForEach { .. }
        | JavaStmt::Switch { .. } => true,
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum JavaLoopCondition {
    Always,
    Never,
    Dynamic,
    UnsupportedConstantForm,
}

pub(super) fn java_loop_condition(value: &JavaExpr) -> JavaLoopCondition {
    if let Some(value) = java_compile_time_boolean(value) {
        if value {
            JavaLoopCondition::Always
        } else {
            JavaLoopCondition::Never
        }
    } else if expression_has_runtime_dependency(value) {
        JavaLoopCondition::Dynamic
    } else {
        JavaLoopCondition::UnsupportedConstantForm
    }
}

fn java_compile_time_boolean(value: &JavaExpr) -> Option<bool> {
    match &value.kind {
        JavaExprKind::Literal(JavaLiteral::Boolean(value)) => Some(*value),
        JavaExprKind::Unary {
            operator: JavaUnaryOperator::Not,
            operand,
        } => java_compile_time_boolean(operand).map(|value| !value),
        JavaExprKind::Binary {
            operator: JavaBinaryOperator::LogicalAnd,
            left,
            right,
        } => Some(java_compile_time_boolean(left)? && java_compile_time_boolean(right)?),
        JavaExprKind::Binary {
            operator: JavaBinaryOperator::LogicalOr,
            left,
            right,
        } => Some(java_compile_time_boolean(left)? || java_compile_time_boolean(right)?),
        JavaExprKind::Binary {
            operator: JavaBinaryOperator::Equal,
            left,
            right,
        } => Some(java_compile_time_boolean(left)? == java_compile_time_boolean(right)?),
        JavaExprKind::Binary {
            operator: JavaBinaryOperator::NotEqual,
            left,
            right,
        } => Some(java_compile_time_boolean(left)? != java_compile_time_boolean(right)?),
        JavaExprKind::Conditional {
            condition,
            when_true,
            when_false,
        } => {
            let condition = java_compile_time_boolean(condition)?;
            let when_true = java_compile_time_boolean(when_true)?;
            let when_false = java_compile_time_boolean(when_false)?;
            Some(if condition { when_true } else { when_false })
        }
        JavaExprKind::Cast {
            target: JavaType::Primitive(JavaPrimitive::Boolean),
            value,
        } => java_compile_time_boolean(value),
        JavaExprKind::Literal(_)
        | JavaExprKind::Value(_)
        | JavaExprKind::Unary { .. }
        | JavaExprKind::Binary { .. }
        | JavaExprKind::Call { .. }
        | JavaExprKind::New { .. }
        | JavaExprKind::NewArray { .. }
        | JavaExprKind::ArrayIndex { .. }
        | JavaExprKind::Field { .. }
        | JavaExprKind::Cast { .. }
        | JavaExprKind::InterfaceCoercion { .. }
        | JavaExprKind::ArrayOwnershipTransition { .. }
        | JavaExprKind::InstanceOf { .. }
        | JavaExprKind::Lambda { .. } => None,
    }
}

fn expression_has_runtime_dependency(value: &JavaExpr) -> bool {
    match &value.kind {
        JavaExprKind::Literal(_) => false,
        JavaExprKind::Value(JavaValueRef::Local(_) | JavaValueRef::This) => true,
        JavaExprKind::Value(
            JavaValueRef::Generated(_)
            | JavaValueRef::EnumVariant { .. }
            | JavaValueRef::KnownField(_),
        ) => false,
        JavaExprKind::Unary { operand, .. } => expression_has_runtime_dependency(operand),
        JavaExprKind::Binary { left, right, .. } => {
            expression_has_runtime_dependency(left) || expression_has_runtime_dependency(right)
        }
        JavaExprKind::Conditional {
            condition,
            when_true,
            when_false,
        } => {
            expression_has_runtime_dependency(condition)
                || expression_has_runtime_dependency(when_true)
                || expression_has_runtime_dependency(when_false)
        }
        JavaExprKind::Cast { value, .. }
        | JavaExprKind::InterfaceCoercion { value, .. }
        | JavaExprKind::ArrayOwnershipTransition { value, .. } => {
            expression_has_runtime_dependency(value)
        }
        JavaExprKind::Call { .. }
        | JavaExprKind::New { .. }
        | JavaExprKind::NewArray { .. }
        | JavaExprKind::ArrayIndex { .. }
        | JavaExprKind::Field { .. }
        | JavaExprKind::InstanceOf { .. }
        | JavaExprKind::Lambda { .. } => true,
    }
}

fn block_has_reachable_break_for_current_loop(block: &JavaBlock) -> bool {
    for statement in &block.statements {
        if statement_has_reachable_break_for_current_loop(statement) {
            return true;
        }
        if !statement_can_complete_normally(statement) {
            return false;
        }
    }
    false
}

fn statement_has_reachable_break_for_current_loop(statement: &JavaStmt) -> bool {
    match statement {
        JavaStmt::Break => true,
        JavaStmt::If {
            then_block,
            else_block,
            ..
        } => {
            block_has_reachable_break_for_current_loop(then_block)
                || else_block
                    .as_ref()
                    .is_some_and(block_has_reachable_break_for_current_loop)
        }
        JavaStmt::TryCatch { try_block, catches } => {
            block_has_reachable_break_for_current_loop(try_block)
                || catches
                    .iter()
                    .any(|catch| block_has_reachable_break_for_current_loop(&catch.body))
        }
        JavaStmt::Local { .. }
        | JavaStmt::Assign { .. }
        | JavaStmt::Expression(_)
        | JavaStmt::Return(_)
        | JavaStmt::ForEach { .. }
        | JavaStmt::While { .. }
        | JavaStmt::Switch { .. }
        | JavaStmt::Throw(_)
        | JavaStmt::ThrowAssertion(_)
        | JavaStmt::Continue => false,
    }
}
