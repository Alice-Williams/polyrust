use super::{Block, ConstantExpression, Declaration, Expression, Intrinsic, Module, Statement};

/// Returns whether a module contains an intrinsic accepted by `predicate`.
///
/// This is a semantic tree walk for target lowering and helper-root selection.
/// It deliberately does not inspect serialized or rendered text and does not
/// infer one intrinsic from a broader capability.
pub fn module_uses_intrinsic(
    module: &Module,
    predicate: impl Fn(Intrinsic) -> bool + Copy,
) -> bool {
    module
        .declarations
        .iter()
        .any(|declaration| match declaration {
            Declaration::Constant(declaration) => {
                constant_uses_intrinsic(&declaration.value, predicate)
            }
            Declaration::Implementation(declaration) => declaration
                .methods
                .iter()
                .any(|method| block_uses_intrinsic(&method.body, predicate)),
            Declaration::Function(declaration) => {
                block_uses_intrinsic(&declaration.body, predicate)
            }
            Declaration::Alias(_)
            | Declaration::Record(_)
            | Declaration::Enum(_)
            | Declaration::Contract(_)
            | Declaration::Test(_) => false,
        })
}

fn constant_uses_intrinsic(
    expression: &ConstantExpression,
    predicate: impl Fn(Intrinsic) -> bool + Copy,
) -> bool {
    match expression {
        ConstantExpression::Literal { .. }
        | ConstantExpression::Reference { .. }
        | ConstantExpression::None { .. } => false,
        ConstantExpression::Record { fields, .. } | ConstantExpression::Enum { fields, .. } => {
            fields
                .iter()
                .any(|field| constant_uses_intrinsic(&field.value, predicate))
        }
        ConstantExpression::Some { value, .. }
        | ConstantExpression::Ok { value, .. }
        | ConstantExpression::Err { value, .. } => constant_uses_intrinsic(value, predicate),
        ConstantExpression::List { elements, .. } => elements
            .iter()
            .any(|element| constant_uses_intrinsic(element, predicate)),
        ConstantExpression::Intrinsic {
            operation,
            arguments,
            ..
        } => {
            predicate(*operation)
                || arguments
                    .iter()
                    .any(|argument| constant_uses_intrinsic(argument, predicate))
        }
    }
}

fn block_uses_intrinsic(block: &Block, predicate: impl Fn(Intrinsic) -> bool + Copy) -> bool {
    block.statements.iter().any(|statement| match statement {
        Statement::Let { value, .. }
        | Statement::Return {
            value: Some(value), ..
        }
        | Statement::Expression { value, .. } => expression_uses_intrinsic(value, predicate),
        Statement::ForEach { iterable, body, .. } => {
            expression_uses_intrinsic(iterable, predicate) || block_uses_intrinsic(body, predicate)
        }
        Statement::Return { value: None, .. } => false,
    }) || block
        .result
        .as_deref()
        .is_some_and(|result| expression_uses_intrinsic(result, predicate))
}

fn expression_uses_intrinsic(
    expression: &Expression,
    predicate: impl Fn(Intrinsic) -> bool + Copy,
) -> bool {
    match expression {
        Expression::Literal { .. }
        | Expression::Local { .. }
        | Expression::Constant { .. }
        | Expression::SelfValue { .. }
        | Expression::ConstructNone { .. } => false,
        Expression::ConstructRecord { fields, .. } | Expression::ConstructEnum { fields, .. } => {
            fields
                .iter()
                .any(|field| expression_uses_intrinsic(&field.value, predicate))
        }
        Expression::ConstructSome { value, .. }
        | Expression::ConstructOk { value, .. }
        | Expression::ConstructErr { value, .. }
        | Expression::Field { base: value, .. } => expression_uses_intrinsic(value, predicate),
        Expression::ConstructList { elements, .. } => elements
            .iter()
            .any(|element| expression_uses_intrinsic(element, predicate)),
        Expression::Call { arguments, .. } => arguments
            .iter()
            .any(|argument| expression_uses_intrinsic(argument, predicate)),
        Expression::MethodCall {
            receiver,
            arguments,
            ..
        } => {
            expression_uses_intrinsic(receiver, predicate)
                || arguments
                    .iter()
                    .any(|argument| expression_uses_intrinsic(argument, predicate))
        }
        Expression::Intrinsic {
            operation,
            arguments,
            ..
        } => {
            predicate(*operation)
                || arguments
                    .iter()
                    .any(|argument| expression_uses_intrinsic(argument, predicate))
        }
        Expression::If {
            condition,
            then_block,
            else_block,
            ..
        } => {
            expression_uses_intrinsic(condition, predicate)
                || block_uses_intrinsic(then_block, predicate)
                || block_uses_intrinsic(else_block, predicate)
        }
        Expression::Match { value, arms, .. } => {
            expression_uses_intrinsic(value, predicate)
                || arms
                    .iter()
                    .any(|arm| block_uses_intrinsic(&arm.body, predicate))
        }
        Expression::Block(block) => block_uses_intrinsic(block, predicate),
    }
}
