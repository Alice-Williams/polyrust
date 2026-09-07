//! Exhaustive structural expression visitation for package-context checks.

use super::{JavaBlock, JavaExpr, JavaExprKind, JavaMember, JavaStmt};

pub(super) fn member(value: &JavaMember, visit: &mut impl FnMut(&JavaExpr)) {
    match value {
        JavaMember::Field(field) => {
            if let Some(value) = &field.initializer {
                expression(value, visit);
            }
        }
        JavaMember::CompileFailField(field) => expression(&field.initializer, visit),
        JavaMember::Method(method) => {
            if let Some(body) = &method.body {
                block(body, visit);
            }
        }
        JavaMember::Constructor(constructor) => block(&constructor.body, visit),
        JavaMember::NestedType(declaration) => {
            for value in &declaration.members {
                member(value, visit);
            }
        }
        JavaMember::EnumConstant(_) => {}
    }
}

fn block(value: &JavaBlock, visit: &mut impl FnMut(&JavaExpr)) {
    for statement in &value.statements {
        match statement {
            JavaStmt::Local { value, .. } | JavaStmt::Return(value) => {
                if let Some(value) = value {
                    expression(value, visit);
                }
            }
            JavaStmt::Assign { target, value } => {
                expression(target, visit);
                expression(value, visit);
            }
            JavaStmt::Expression(value)
            | JavaStmt::Throw(value)
            | JavaStmt::ThrowAssertion(value) => expression(value, visit),
            JavaStmt::If {
                condition,
                then_block,
                else_block,
            } => {
                expression(condition, visit);
                block(then_block, visit);
                if let Some(body) = else_block {
                    block(body, visit);
                }
            }
            JavaStmt::ForEach { iterable, body, .. } => {
                expression(iterable, visit);
                block(body, visit);
            }
            JavaStmt::While { condition, body } => {
                expression(condition, visit);
                block(body, visit);
            }
            JavaStmt::Switch { value, arms } => {
                expression(value, visit);
                for arm in arms {
                    block(&arm.body, visit);
                }
            }
            JavaStmt::TryCatch { try_block, catches } => {
                block(try_block, visit);
                for catch in catches {
                    block(&catch.body, visit);
                }
            }
            JavaStmt::Break | JavaStmt::Continue => {}
        }
    }
}

fn expression(value: &JavaExpr, visit: &mut impl FnMut(&JavaExpr)) {
    visit(value);
    match &value.kind {
        JavaExprKind::Literal(_) | JavaExprKind::Value(_) => {}
        JavaExprKind::Unary { operand, .. } => expression(operand, visit),
        JavaExprKind::Binary { left, right, .. } => {
            expression(left, visit);
            expression(right, visit);
        }
        JavaExprKind::Conditional {
            condition,
            when_true,
            when_false,
        } => {
            expression(condition, visit);
            expression(when_true, visit);
            expression(when_false, visit);
        }
        JavaExprKind::Call {
            receiver,
            arguments,
            ..
        } => {
            if let Some(receiver) = receiver {
                expression(receiver, visit);
            }
            for argument in arguments {
                expression(argument, visit);
            }
        }
        JavaExprKind::New { arguments, .. } => {
            for argument in arguments {
                expression(argument, visit);
            }
        }
        JavaExprKind::NewArray { length, .. } => expression(length, visit),
        JavaExprKind::ArrayIndex { array, index } => {
            expression(array, visit);
            expression(index, visit);
        }
        JavaExprKind::Field { receiver, .. } => expression(receiver, visit),
        JavaExprKind::Cast { value, .. }
        | JavaExprKind::InterfaceCoercion { value, .. }
        | JavaExprKind::ArrayOwnershipTransition { value, .. }
        | JavaExprKind::InstanceOf { value, .. } => expression(value, visit),
        JavaExprKind::Lambda { body, .. } => block(body, visit),
    }
}
