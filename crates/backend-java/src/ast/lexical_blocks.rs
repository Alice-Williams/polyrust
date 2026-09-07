//! Java AST: lexical blocks.

use super::blocks::{JavaBlock, JavaLocalFinality};
use super::completion::{
    JavaLoopCondition, block_guarantees_exit, java_loop_condition, statement_can_complete_normally,
};
use super::expression_model::{JavaBinaryOperator, JavaUnaryOperator};
use super::expression_nodes::{JavaExpr, JavaExprKind};
use super::lexical_expressions::{
    verify_assignment_target, verify_assignment_target_reads, verify_expr_scope,
};
use super::lexical_scope::{JavaLexicalBinding, JavaLexicalScope};
use super::operator_signatures::invocation_types_match;
use super::statement_model::{JavaPattern, JavaStmt};
use super::types::{JavaPrimitive, JavaType, type_error};
use crate::dialect::JavaDialect;
use portable_codegen::{AstViolation, TargetAstContext};
use portable_diagnostics::DiagnosticCode;

#[cfg(test)]
pub(super) fn verify_block_scope(
    block: &JavaBlock,
    scope: &mut JavaLexicalScope,
    expected_return: &JavaType,
    in_loop: bool,
) -> Vec<AstViolation> {
    verify_block_scope_in_context(block, scope, expected_return, in_loop, None)
}

pub(super) fn verify_block_scope_in_context(
    block: &JavaBlock,
    scope: &mut JavaLexicalScope,
    expected_return: &JavaType,
    in_loop: bool,
    context: Option<&TargetAstContext<'_, JavaDialect>>,
) -> Vec<AstViolation> {
    let mut violations = Vec::new();
    if let Some(context) = context {
        violations.extend(scope.protect_qualifiers(context));
    }
    let mut can_complete_normally = true;
    for statement in &block.statements {
        if !can_complete_normally {
            violations.push(AstViolation::new(
                DiagnosticCode::InvalidControlFlow,
                "unreachable Java statement follows control flow that cannot complete normally",
            ));
        }
        match statement {
            JavaStmt::Local {
                finality,
                ty,
                name,
                value,
            } => {
                if let Some(value) = value {
                    violations.extend(verify_expr_scope(value, scope));
                }
                scope.bind(
                    name.clone(),
                    JavaLexicalBinding {
                        ty: ty.clone(),
                        mutable: *finality == JavaLocalFinality::Mutable,
                        definitely_assigned: value.is_some(),
                    },
                    "Java local shadows a name already declared in this portable scope",
                    &mut violations,
                );
            }
            JavaStmt::Assign { target, value } => {
                violations.extend(verify_assignment_target_reads(target, scope));
                violations.extend(verify_expr_scope(value, scope));
                violations.extend(verify_assignment_target(target, scope, context));
                scope.mark_local_assigned(target);
            }
            JavaStmt::Expression(value)
            | JavaStmt::Throw(value)
            | JavaStmt::ThrowAssertion(value) => {
                violations.extend(verify_expr_scope(value, scope));
            }
            JavaStmt::Return(value) => match value {
                Some(value) => {
                    violations.extend(verify_expr_scope(value, scope));
                    if !invocation_types_match(expected_return, &value.ty) {
                        violations.push(type_error(
                            "Java return value does not match the declared return type",
                        ));
                    }
                }
                None if *expected_return != JavaType::primitive(JavaPrimitive::Void) => {
                    violations.push(type_error(
                        "non-void Java method cannot return without a value",
                    ));
                }
                None => {}
            },
            JavaStmt::If {
                condition,
                then_block,
                else_block,
            } => {
                violations.extend(verify_expr_scope(condition, scope));
                let mut then_scope = scope.clone();
                collect_positive_pattern_bindings(condition, &mut then_scope, &mut violations);
                violations.extend(verify_block_scope_in_context(
                    then_block,
                    &mut then_scope,
                    expected_return,
                    in_loop,
                    context,
                ));
                if let Some(else_block) = else_block {
                    let mut else_scope = scope.clone();
                    violations.extend(verify_block_scope_in_context(
                        else_block,
                        &mut else_scope,
                        expected_return,
                        in_loop,
                        context,
                    ));
                    let mut normal_alternatives = Vec::new();
                    if !block_guarantees_exit(then_block) {
                        normal_alternatives.push(then_scope.clone());
                    }
                    if !block_guarantees_exit(else_block) {
                        normal_alternatives.push(else_scope);
                    }
                    if !normal_alternatives.is_empty() {
                        scope.merge_definite_assignments(&normal_alternatives);
                    }
                }
                if else_block.is_none()
                    && block_guarantees_exit(then_block)
                    && let JavaExprKind::Unary {
                        operator: JavaUnaryOperator::Not,
                        operand,
                    } = &condition.kind
                    && let JavaExprKind::InstanceOf {
                        target,
                        binding: Some(binding),
                        ..
                    } = &operand.kind
                    && !scope.bindings.contains_key(binding)
                {
                    scope.bind(
                        binding.clone(),
                        JavaLexicalBinding {
                            ty: target.clone(),
                            mutable: false,
                            definitely_assigned: true,
                        },
                        "Java pattern binding conflicts with an overlapping lexical binding",
                        &mut violations,
                    );
                }
            }
            JavaStmt::ForEach {
                binding_type,
                binding,
                iterable,
                body,
            } => {
                violations.extend(verify_expr_scope(iterable, scope));
                let mut body_scope = scope.clone();
                body_scope.bind(
                    binding.clone(),
                    JavaLexicalBinding {
                        ty: binding_type.clone(),
                        mutable: false,
                        definitely_assigned: true,
                    },
                    "Java foreach binding conflicts with an overlapping lexical binding",
                    &mut violations,
                );
                violations.extend(verify_block_scope_in_context(
                    body,
                    &mut body_scope,
                    expected_return,
                    true,
                    context,
                ));
            }
            JavaStmt::While { condition, body } => {
                violations.extend(verify_expr_scope(condition, scope));
                match java_loop_condition(condition) {
                    JavaLoopCondition::Never => {
                        violations.push(AstViolation::new(
                            DiagnosticCode::InvalidControlFlow,
                            "Java while(false) body is unreachable",
                        ));
                    }
                    JavaLoopCondition::UnsupportedConstantForm => {
                        violations.push(AstViolation::new(
                            DiagnosticCode::InvalidControlFlow,
                            "Java loop condition uses a compile-time form outside the admitted reachability grammar",
                        ));
                    }
                    JavaLoopCondition::Always | JavaLoopCondition::Dynamic => {}
                }
                let mut body_scope = scope.clone();
                collect_positive_pattern_bindings(condition, &mut body_scope, &mut violations);
                violations.extend(verify_block_scope_in_context(
                    body,
                    &mut body_scope,
                    expected_return,
                    true,
                    context,
                ));
            }
            JavaStmt::Switch { value, arms } => {
                violations.extend(verify_expr_scope(value, scope));
                let mut arm_scopes = Vec::new();
                for arm in arms {
                    let mut arm_scope = scope.clone();
                    if let JavaPattern::Type { ty, binding } = &arm.pattern {
                        arm_scope.bind(
                            binding.clone(),
                            JavaLexicalBinding {
                                ty: ty.clone(),
                                mutable: false,
                                definitely_assigned: true,
                            },
                            "Java switch type-pattern binding conflicts with an overlapping lexical binding",
                            &mut violations,
                        );
                    }
                    violations.extend(verify_block_scope_in_context(
                        &arm.body,
                        &mut arm_scope,
                        expected_return,
                        in_loop,
                        context,
                    ));
                    if !block_guarantees_exit(&arm.body) {
                        arm_scopes.push(arm_scope);
                    }
                }
                if arms
                    .iter()
                    .any(|arm| matches!(arm.pattern, JavaPattern::Default))
                    && !arm_scopes.is_empty()
                {
                    scope.merge_definite_assignments(&arm_scopes);
                }
            }
            JavaStmt::TryCatch { try_block, catches } => {
                let mut try_scope = scope.clone();
                violations.extend(verify_block_scope_in_context(
                    try_block,
                    &mut try_scope,
                    expected_return,
                    in_loop,
                    context,
                ));
                for catch in catches {
                    let mut catch_scope = scope.clone();
                    catch_scope.bind(
                        catch.binding.clone(),
                        JavaLexicalBinding {
                            ty: catch.exception_type.clone(),
                            mutable: false,
                            definitely_assigned: true,
                        },
                        "Java catch binding conflicts with an overlapping lexical binding",
                        &mut violations,
                    );
                    violations.extend(verify_block_scope_in_context(
                        &catch.body,
                        &mut catch_scope,
                        expected_return,
                        in_loop,
                        context,
                    ));
                }
            }
            JavaStmt::Break | JavaStmt::Continue if !in_loop => violations.push(AstViolation::new(
                DiagnosticCode::InvalidControlFlow,
                "Java break/continue is outside a loop",
            )),
            JavaStmt::Break | JavaStmt::Continue => {}
        }
        if can_complete_normally {
            can_complete_normally = statement_can_complete_normally(statement);
        }
    }
    violations
}

fn collect_positive_pattern_bindings(
    value: &JavaExpr,
    scope: &mut JavaLexicalScope,
    violations: &mut Vec<AstViolation>,
) {
    match &value.kind {
        JavaExprKind::InstanceOf {
            target,
            binding: Some(binding),
            ..
        } => {
            if !scope.bindings.contains_key(binding) {
                scope.bind(
                    binding.clone(),
                    JavaLexicalBinding {
                        ty: target.clone(),
                        mutable: false,
                        definitely_assigned: true,
                    },
                    "Java pattern binding conflicts with an overlapping lexical binding",
                    violations,
                );
            }
        }
        JavaExprKind::Binary {
            operator: JavaBinaryOperator::LogicalAnd,
            left,
            right,
        } => {
            collect_positive_pattern_bindings(left, scope, violations);
            collect_positive_pattern_bindings(right, scope, violations);
        }
        _ => {}
    }
}
