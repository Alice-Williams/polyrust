//! Java AST: statement context.

use super::blocks::JavaBlock;
use super::declaration_model::{JavaMember, JavaModifier};
use super::identifiers::JavaIdentifier;
use super::statement_model::{JavaPattern, JavaStmt};
use super::type_context::{verify_contextual_type, verify_expression_type_context};
use crate::dialect::JavaDialect;
use portable_codegen::{AstViolation, TargetAstContext};
use std::collections::BTreeSet;

pub(super) fn verify_block_type_context(
    block: &JavaBlock,
    variables: &BTreeSet<JavaIdentifier>,
    context: &TargetAstContext<'_, JavaDialect>,
) -> Vec<AstViolation> {
    let mut violations = Vec::new();
    for statement in &block.statements {
        match statement {
            JavaStmt::Local { ty, value, .. } => {
                violations.extend(verify_contextual_type(ty, variables, context));
                if let Some(value) = value {
                    violations.extend(verify_expression_type_context(value, variables, context));
                }
            }
            JavaStmt::Assign { target, value } => {
                violations.extend(verify_expression_type_context(target, variables, context));
                violations.extend(verify_expression_type_context(value, variables, context));
            }
            JavaStmt::Expression(value)
            | JavaStmt::Throw(value)
            | JavaStmt::ThrowAssertion(value) => {
                violations.extend(verify_expression_type_context(value, variables, context));
            }
            JavaStmt::Return(value) => {
                if let Some(value) = value {
                    violations.extend(verify_expression_type_context(value, variables, context));
                }
            }
            JavaStmt::If {
                condition,
                then_block,
                else_block,
            } => {
                violations.extend(verify_expression_type_context(
                    condition, variables, context,
                ));
                violations.extend(verify_block_type_context(then_block, variables, context));
                if let Some(else_block) = else_block {
                    violations.extend(verify_block_type_context(else_block, variables, context));
                }
            }
            JavaStmt::ForEach {
                binding_type,
                iterable,
                body,
                ..
            } => {
                violations.extend(verify_contextual_type(binding_type, variables, context));
                violations.extend(verify_expression_type_context(iterable, variables, context));
                violations.extend(verify_block_type_context(body, variables, context));
            }
            JavaStmt::While { condition, body } => {
                violations.extend(verify_expression_type_context(
                    condition, variables, context,
                ));
                violations.extend(verify_block_type_context(body, variables, context));
            }
            JavaStmt::Switch { value, arms } => {
                violations.extend(verify_expression_type_context(value, variables, context));
                for arm in arms {
                    match &arm.pattern {
                        JavaPattern::Type { ty, .. } => {
                            violations.extend(verify_contextual_type(ty, variables, context));
                        }
                        JavaPattern::Default
                        | JavaPattern::Literal(_)
                        | JavaPattern::EnumVariant { .. } => {}
                    }
                    violations.extend(verify_block_type_context(&arm.body, variables, context));
                }
            }
            JavaStmt::TryCatch { try_block, catches } => {
                violations.extend(verify_block_type_context(try_block, variables, context));
                for catch in catches {
                    violations.extend(verify_contextual_type(
                        &catch.exception_type,
                        variables,
                        context,
                    ));
                    violations.extend(verify_block_type_context(&catch.body, variables, context));
                }
            }
            JavaStmt::Break | JavaStmt::Continue => {}
        }
    }
    violations
}

pub(super) fn verify_member_type_context(
    member: &JavaMember,
    declaration_variables: &BTreeSet<JavaIdentifier>,
    context: &TargetAstContext<'_, JavaDialect>,
) -> Vec<AstViolation> {
    let mut violations = Vec::new();
    match member {
        JavaMember::Field(field) => {
            let variables = if field.modifiers.contains(&JavaModifier::Static) {
                BTreeSet::new()
            } else {
                declaration_variables.clone()
            };
            violations.extend(verify_contextual_type(&field.ty, &variables, context));
            if let Some(initializer) = &field.initializer {
                violations.extend(verify_expression_type_context(
                    initializer,
                    &variables,
                    context,
                ));
            }
        }
        JavaMember::CompileFailField(field) => {
            let variables = if field.modifiers.contains(&JavaModifier::Static) {
                BTreeSet::new()
            } else {
                declaration_variables.clone()
            };
            violations.extend(verify_contextual_type(
                &field.expected_type,
                &variables,
                context,
            ));
            violations.extend(verify_expression_type_context(
                &field.initializer,
                &variables,
                context,
            ));
        }
        JavaMember::EnumConstant(_) => {}
        JavaMember::Method(method) => {
            let mut variables = method
                .type_parameters
                .iter()
                .cloned()
                .collect::<BTreeSet<_>>();
            if !method.modifiers.contains(&JavaModifier::Static) {
                variables.extend(declaration_variables.iter().cloned());
            }
            violations.extend(verify_contextual_type(
                &method.return_type,
                &variables,
                context,
            ));
            for parameter in &method.parameters {
                violations.extend(verify_contextual_type(&parameter.ty, &variables, context));
            }
            if let Some(body) = &method.body {
                violations.extend(verify_block_type_context(body, &variables, context));
            }
        }
        JavaMember::Constructor(constructor) => {
            for parameter in &constructor.parameters {
                violations.extend(verify_contextual_type(
                    &parameter.ty,
                    declaration_variables,
                    context,
                ));
            }
            violations.extend(verify_block_type_context(
                &constructor.body,
                declaration_variables,
                context,
            ));
        }
        JavaMember::NestedType(_) => {}
    }
    violations
}
