//! Java AST: privileged literals.

use super::blocks::JavaBlock;
use super::declaration_model::{JavaDeclarationKind, JavaMember, JavaTypeDeclaration};
use super::expression_model::{JavaBinaryOperator, JavaLiteral, JavaNullPurpose, JavaValueRef};
use super::expression_nodes::{JavaConstructorRef, JavaExpr, JavaExprKind};
use super::statement_model::{JavaPattern, JavaStmt};
use super::types::JavaKnownType;
use crate::dialect::{JavaKnownConstructor, JavaRuntimeHelper};
use portable_codegen::AstViolation;
use portable_diagnostics::DiagnosticCode;

#[derive(Clone, Copy)]
struct JavaPrivilegedLiteralScope {
    tagged_storage_helper: Option<JavaRuntimeHelper>,
    tagged_constructor: Option<JavaKnownType>,
}

impl JavaPrivilegedLiteralScope {
    const FORBIDDEN: Self = Self {
        tagged_storage_helper: None,
        tagged_constructor: None,
    };
}

pub(super) fn verify_privileged_literals_in_declaration(
    declaration: &JavaTypeDeclaration,
    helper: Option<JavaRuntimeHelper>,
) -> Vec<AstViolation> {
    let tagged_owner =
        helper.and_then(|helper| registered_tagged_runtime_type(helper, declaration));
    declaration
        .members
        .iter()
        .flat_map(|member| verify_privileged_literals_in_member(member, helper, tagged_owner))
        .collect()
}

fn registered_tagged_runtime_type(
    helper: JavaRuntimeHelper,
    declaration: &JavaTypeDeclaration,
) -> Option<JavaKnownType> {
    let candidate = match helper {
        JavaRuntimeHelper::Core => JavaKnownType::RuntimeResult,
        JavaRuntimeHelper::TaggedValues
            if declaration.name.as_str() == JavaKnownType::RuntimeOption.simple_name() =>
        {
            JavaKnownType::RuntimeOption
        }
        JavaRuntimeHelper::TaggedValues => JavaKnownType::RuntimeValueResult,
        _ => return None,
    };
    (declaration.name.as_str() == candidate.simple_name()
        && declaration.kind == JavaDeclarationKind::FinalClass)
        .then_some(candidate)
}

pub(super) fn verify_privileged_literals_in_member(
    member: &JavaMember,
    helper: Option<JavaRuntimeHelper>,
    tagged_owner: Option<JavaKnownType>,
) -> Vec<AstViolation> {
    let tagged_storage_helper = helper.filter(|helper| {
        matches!(
            helper,
            JavaRuntimeHelper::Core | JavaRuntimeHelper::TaggedValues
        )
    });
    match member {
        JavaMember::Field(field) => field.initializer.as_ref().map_or_else(Vec::new, |value| {
            verify_privileged_literals_in_expression(value, JavaPrivilegedLiteralScope::FORBIDDEN)
        }),
        JavaMember::CompileFailField(field) => verify_privileged_literals_in_expression(
            &field.initializer,
            JavaPrivilegedLiteralScope::FORBIDDEN,
        ),
        JavaMember::EnumConstant(_) => vec![],
        JavaMember::Method(method) => method.body.as_ref().map_or_else(Vec::new, |body| {
            verify_privileged_literals_in_block(
                body,
                JavaPrivilegedLiteralScope {
                    tagged_storage_helper,
                    tagged_constructor: None,
                },
            )
        }),
        JavaMember::Constructor(constructor) => verify_privileged_literals_in_block(
            &constructor.body,
            JavaPrivilegedLiteralScope {
                tagged_storage_helper,
                tagged_constructor: tagged_owner,
            },
        ),
        JavaMember::NestedType(declaration) => {
            verify_privileged_literals_in_declaration(declaration, helper)
        }
    }
}

fn verify_privileged_literals_in_block(
    block: &JavaBlock,
    scope: JavaPrivilegedLiteralScope,
) -> Vec<AstViolation> {
    let mut violations = Vec::new();
    for statement in &block.statements {
        match statement {
            JavaStmt::Local { value, .. } | JavaStmt::Return(value) => {
                if let Some(value) = value {
                    violations.extend(verify_privileged_literals_in_expression(value, scope));
                }
            }
            JavaStmt::Assign { target, value } => {
                violations.extend(verify_privileged_literals_in_expression(target, scope));
                violations.extend(verify_privileged_literals_in_expression(value, scope));
            }
            JavaStmt::Expression(value)
            | JavaStmt::Throw(value)
            | JavaStmt::ThrowAssertion(value) => {
                violations.extend(verify_privileged_literals_in_expression(value, scope));
            }
            JavaStmt::If {
                condition,
                then_block,
                else_block,
            } => {
                violations.extend(verify_privileged_literals_in_expression(condition, scope));
                violations.extend(verify_privileged_literals_in_block(then_block, scope));
                if let Some(else_block) = else_block {
                    violations.extend(verify_privileged_literals_in_block(else_block, scope));
                }
            }
            JavaStmt::ForEach { iterable, body, .. } => {
                violations.extend(verify_privileged_literals_in_expression(iterable, scope));
                violations.extend(verify_privileged_literals_in_block(body, scope));
            }
            JavaStmt::While { condition, body } => {
                violations.extend(verify_privileged_literals_in_expression(condition, scope));
                violations.extend(verify_privileged_literals_in_block(body, scope));
            }
            JavaStmt::Switch { value, arms } => {
                violations.extend(verify_privileged_literals_in_expression(value, scope));
                for arm in arms {
                    if matches!(
                        arm.pattern,
                        JavaPattern::Literal(
                            JavaLiteral::Utf16Units(_) | JavaLiteral::InternalNull(_)
                        )
                    ) {
                        violations.push(privileged_literal_error(
                            "privileged Java literal cannot be used as a switch label",
                        ));
                    }
                    violations.extend(verify_privileged_literals_in_block(&arm.body, scope));
                }
            }
            JavaStmt::TryCatch { try_block, catches } => {
                violations.extend(verify_privileged_literals_in_block(try_block, scope));
                for catch in catches {
                    violations.extend(verify_privileged_literals_in_block(&catch.body, scope));
                }
            }
            JavaStmt::Break | JavaStmt::Continue => {}
        }
    }
    violations
}

fn verify_privileged_literals_in_expression(
    expression: &JavaExpr,
    scope: JavaPrivilegedLiteralScope,
) -> Vec<AstViolation> {
    let mut violations = Vec::new();
    match &expression.kind {
        JavaExprKind::Literal(JavaLiteral::Utf16Units(_)) => {
            violations.push(privileged_literal_error(
                "raw UTF-16-unit literal is not admitted in verified executable Java AST",
            ))
        }
        JavaExprKind::Literal(JavaLiteral::InternalNull(_)) => {
            violations.push(privileged_literal_error(
                "internal null literal is outside exact registered tagged runtime storage",
            ))
        }
        JavaExprKind::Literal(_) | JavaExprKind::Value(_) => {}
        JavaExprKind::Unary { operand, .. } => {
            violations.extend(verify_privileged_literals_in_expression(operand, scope));
        }
        JavaExprKind::Binary {
            operator,
            left,
            right,
        } => {
            let tagged_null_check = matches!(
                operator,
                JavaBinaryOperator::Equal | JavaBinaryOperator::NotEqual
            ) && scope.tagged_constructor.is_some();
            if !(tagged_null_check
                && registered_tagged_null_check(left, right, scope.tagged_constructor.unwrap()))
            {
                violations.extend(verify_privileged_literals_in_expression(left, scope));
            }
            if !(tagged_null_check
                && registered_tagged_null_check(right, left, scope.tagged_constructor.unwrap()))
            {
                violations.extend(verify_privileged_literals_in_expression(right, scope));
            }
        }
        JavaExprKind::Conditional {
            condition,
            when_true,
            when_false,
        } => {
            violations.extend(verify_privileged_literals_in_expression(condition, scope));
            violations.extend(verify_privileged_literals_in_expression(when_true, scope));
            violations.extend(verify_privileged_literals_in_expression(when_false, scope));
        }
        JavaExprKind::Call {
            receiver,
            arguments,
            ..
        } => {
            if let Some(receiver) = receiver {
                violations.extend(verify_privileged_literals_in_expression(receiver, scope));
            }
            for argument in arguments {
                violations.extend(verify_privileged_literals_in_expression(argument, scope));
            }
        }
        JavaExprKind::New {
            constructor,
            arguments,
        } => {
            for (index, argument) in arguments.iter().enumerate() {
                if !scope.tagged_storage_helper.is_some_and(|helper| {
                    registered_inactive_tagged_storage_argument(
                        helper,
                        constructor,
                        arguments,
                        index,
                        argument,
                    )
                }) {
                    violations.extend(verify_privileged_literals_in_expression(argument, scope));
                }
            }
        }
        JavaExprKind::NewArray { length, .. } => {
            violations.extend(verify_privileged_literals_in_expression(length, scope));
        }
        JavaExprKind::ArrayIndex { array, index } => {
            violations.extend(verify_privileged_literals_in_expression(array, scope));
            violations.extend(verify_privileged_literals_in_expression(index, scope));
        }
        JavaExprKind::Field { receiver, .. } => {
            violations.extend(verify_privileged_literals_in_expression(receiver, scope));
        }
        JavaExprKind::Cast { value, .. }
        | JavaExprKind::InterfaceCoercion { value, .. }
        | JavaExprKind::ArrayOwnershipTransition { value, .. }
        | JavaExprKind::InstanceOf { value, .. } => {
            violations.extend(verify_privileged_literals_in_expression(value, scope));
        }
        JavaExprKind::Lambda { body, .. } => {
            violations.extend(verify_privileged_literals_in_block(body, scope));
        }
    }
    violations
}

fn registered_tagged_null_check(null: &JavaExpr, payload: &JavaExpr, owner: JavaKnownType) -> bool {
    matches!(
        &null.kind,
        JavaExprKind::Literal(JavaLiteral::InternalNull(
            JavaNullPurpose::AbsentTaggedPayload
        ))
    ) && null.ty == payload.ty
        && matches!(
            &payload.kind,
            JavaExprKind::Value(JavaValueRef::Local(name))
                if match owner {
                    JavaKnownType::RuntimeOption => name.as_str() == "value",
                    JavaKnownType::RuntimeResult | JavaKnownType::RuntimeValueResult => {
                        matches!(name.as_str(), "value" | "error")
                    }
                    _ => false,
                }
        )
}

fn registered_inactive_tagged_storage_argument(
    helper: JavaRuntimeHelper,
    constructor: &JavaConstructorRef,
    arguments: &[JavaExpr],
    index: usize,
    argument: &JavaExpr,
) -> bool {
    if !matches!(
        argument.kind,
        JavaExprKind::Literal(JavaLiteral::InternalNull(
            JavaNullPurpose::AbsentTaggedPayload
        ))
    ) {
        return false;
    }
    let Some(JavaExpr {
        kind: JavaExprKind::Literal(JavaLiteral::Boolean(active)),
        ..
    }) = arguments.first()
    else {
        return false;
    };
    let JavaConstructorRef::Known { constructor, .. } = constructor else {
        return false;
    };
    if !matches!(
        (helper, constructor),
        (JavaRuntimeHelper::Core, JavaKnownConstructor::RuntimeResult)
            | (
                JavaRuntimeHelper::TaggedValues,
                JavaKnownConstructor::RuntimeOption | JavaKnownConstructor::RuntimeValueResult
            )
    ) {
        return false;
    }
    match constructor {
        JavaKnownConstructor::RuntimeOption => !active && index == 1 && arguments.len() == 2,
        JavaKnownConstructor::RuntimeResult | JavaKnownConstructor::RuntimeValueResult => {
            ((*active && index == 2) || (!active && index == 1)) && arguments.len() == 3
        }
        _ => false,
    }
}

fn privileged_literal_error(message: &str) -> AstViolation {
    AstViolation::new(DiagnosticCode::InvalidStructure, message)
}
