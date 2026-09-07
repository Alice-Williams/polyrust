//! Java AST: exceptions.

use super::blocks::JavaBlock;
use super::expression_nodes::{JavaExpr, JavaExprKind};
use super::statement_model::JavaStmt;
use super::types::{JavaKnownType, JavaType, JavaTypeName};
use crate::dialect::JavaDialect;
use portable_codegen::TargetAstContext;
use std::collections::BTreeSet;

pub(super) fn expr_checked_exceptions(
    value: &JavaExpr,
    context: &TargetAstContext<'_, JavaDialect>,
) -> BTreeSet<JavaKnownType> {
    let mut exceptions = BTreeSet::new();
    match &value.kind {
        JavaExprKind::Literal(_) | JavaExprKind::Value(_) => {}
        JavaExprKind::Unary { operand, .. } => {
            exceptions.extend(expr_checked_exceptions(operand, context));
        }
        JavaExprKind::Binary { left, right, .. } => {
            exceptions.extend(expr_checked_exceptions(left, context));
            exceptions.extend(expr_checked_exceptions(right, context));
        }
        JavaExprKind::Conditional {
            condition,
            when_true,
            when_false,
        } => {
            exceptions.extend(expr_checked_exceptions(condition, context));
            exceptions.extend(expr_checked_exceptions(when_true, context));
            exceptions.extend(expr_checked_exceptions(when_false, context));
        }
        JavaExprKind::Call {
            callable,
            receiver,
            arguments,
        } => {
            if let Some(signature) = callable.signature(context) {
                exceptions.extend(signature.checked_exceptions);
            }
            if let Some(receiver) = receiver {
                exceptions.extend(expr_checked_exceptions(receiver, context));
            }
            for argument in arguments {
                exceptions.extend(expr_checked_exceptions(argument, context));
            }
        }
        JavaExprKind::New { arguments, .. } => {
            for argument in arguments {
                exceptions.extend(expr_checked_exceptions(argument, context));
            }
        }
        JavaExprKind::NewArray { length, .. } => {
            exceptions.extend(expr_checked_exceptions(length, context));
        }
        JavaExprKind::ArrayIndex { array, index } => {
            exceptions.extend(expr_checked_exceptions(array, context));
            exceptions.extend(expr_checked_exceptions(index, context));
        }
        JavaExprKind::Field { receiver, .. } => {
            exceptions.extend(expr_checked_exceptions(receiver, context));
        }
        JavaExprKind::Cast { value, .. }
        | JavaExprKind::InterfaceCoercion { value, .. }
        | JavaExprKind::ArrayOwnershipTransition { value, .. }
        | JavaExprKind::InstanceOf { value, .. } => {
            exceptions.extend(expr_checked_exceptions(value, context));
        }
        JavaExprKind::Lambda { body, .. } => {
            exceptions.extend(block_checked_exceptions(body, context));
        }
    }
    exceptions
}

pub(super) fn block_checked_exceptions(
    block: &JavaBlock,
    context: &TargetAstContext<'_, JavaDialect>,
) -> BTreeSet<JavaKnownType> {
    let mut exceptions = BTreeSet::new();
    for statement in &block.statements {
        match statement {
            JavaStmt::Local { value, .. } | JavaStmt::Return(value) => {
                if let Some(value) = value {
                    exceptions.extend(expr_checked_exceptions(value, context));
                }
            }
            JavaStmt::Assign { target, value } => {
                exceptions.extend(expr_checked_exceptions(target, context));
                exceptions.extend(expr_checked_exceptions(value, context));
            }
            JavaStmt::Expression(value)
            | JavaStmt::Throw(value)
            | JavaStmt::ThrowAssertion(value) => {
                exceptions.extend(expr_checked_exceptions(value, context));
                if let JavaStmt::Throw(_) = statement
                    && let Some(exception) = throwable_known_type(&value.ty)
                    && is_checked_exception(exception)
                {
                    exceptions.insert(exception);
                }
            }
            JavaStmt::If {
                condition,
                then_block,
                else_block,
            } => {
                exceptions.extend(expr_checked_exceptions(condition, context));
                exceptions.extend(block_checked_exceptions(then_block, context));
                if let Some(else_block) = else_block {
                    exceptions.extend(block_checked_exceptions(else_block, context));
                }
            }
            JavaStmt::ForEach { iterable, body, .. } => {
                exceptions.extend(expr_checked_exceptions(iterable, context));
                exceptions.extend(block_checked_exceptions(body, context));
            }
            JavaStmt::While { condition, body } => {
                exceptions.extend(expr_checked_exceptions(condition, context));
                exceptions.extend(block_checked_exceptions(body, context));
            }
            JavaStmt::Switch { value, arms } => {
                exceptions.extend(expr_checked_exceptions(value, context));
                for arm in arms {
                    exceptions.extend(block_checked_exceptions(&arm.body, context));
                }
            }
            JavaStmt::TryCatch { try_block, catches } => {
                let mut try_exceptions = block_checked_exceptions(try_block, context);
                for catch in catches {
                    if let Some(caught) = throwable_known_type(&catch.exception_type) {
                        try_exceptions.remove(&caught);
                    }
                    exceptions.extend(block_checked_exceptions(&catch.body, context));
                }
                exceptions.extend(try_exceptions);
            }
            JavaStmt::Break | JavaStmt::Continue => {}
        }
    }
    exceptions
}

pub(super) fn throwable_known_type(ty: &JavaType) -> Option<JavaKnownType> {
    match ty {
        JavaType::Reference(JavaTypeName::Known(value))
            if matches!(
                value,
                JavaKnownType::AssertionError
                    | JavaKnownType::IllegalArgumentException
                    | JavaKnownType::IllegalStateException
                    | JavaKnownType::RuntimeException
                    | JavaKnownType::CharacterCodingException
            ) =>
        {
            Some(*value)
        }
        _ => None,
    }
}

pub(super) fn is_checked_exception(value: JavaKnownType) -> bool {
    matches!(value, JavaKnownType::CharacterCodingException)
}

pub(super) fn admitted_throwable_is_supertype_of(
    supertype: JavaKnownType,
    subtype: JavaKnownType,
) -> bool {
    supertype == subtype
        || matches!(
            (supertype, subtype),
            (
                JavaKnownType::RuntimeException,
                JavaKnownType::IllegalArgumentException | JavaKnownType::IllegalStateException
            )
        )
}
