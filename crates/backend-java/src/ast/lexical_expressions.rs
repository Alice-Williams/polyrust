//! Java AST: lexical expressions.

use super::expression_model::JavaValueRef;
use super::expression_nodes::{JavaExpr, JavaExprKind, JavaFieldRef};
use super::field_metadata::{JavaFieldMetadata, structural_field_metadata};
use super::generated_members::generated_field_matches;
use super::lexical_scope::{JavaLexicalScope, verify_pattern_binding_uniqueness};
use super::types::{JavaArrayOwnership, JavaPrimitive, JavaType, JavaTypeName, type_error};
use crate::dialect::JavaDialect;
use portable_codegen::{AstViolation, TargetAstContext};
use portable_diagnostics::DiagnosticCode;

pub(super) fn verify_expr_scope(value: &JavaExpr, scope: &JavaLexicalScope) -> Vec<AstViolation> {
    let mut violations = verify_pattern_binding_uniqueness(value, scope);
    violations.extend(verify_expr_scope_structure(value, scope));
    violations
}

fn verify_expr_scope_structure(value: &JavaExpr, scope: &JavaLexicalScope) -> Vec<AstViolation> {
    let mut violations = Vec::new();
    match &value.kind {
        JavaExprKind::Value(JavaValueRef::Local(name)) => match scope.bindings.get(name) {
            Some(binding) if binding.ty == value.ty && binding.definitely_assigned => {}
            Some(binding) if binding.ty == value.ty => violations.push(AstViolation::new(
                DiagnosticCode::InvalidControlFlow,
                format!(
                    "Java local {:?} is read before it is definitely assigned",
                    name.as_str()
                ),
            )),
            Some(_) => violations.push(type_error(
                "local reference type disagrees with its lexical declaration",
            )),
            None => violations.push(AstViolation::new(
                DiagnosticCode::UnresolvedReference,
                format!(
                    "local reference {:?} is outside its lexical scope",
                    name.as_str()
                ),
            )),
        },
        JavaExprKind::Value(JavaValueRef::This) => {
            if !scope.allows_this {
                violations.push(AstViolation::new(
                    DiagnosticCode::UnresolvedReference,
                    "this is unavailable in a static Java scope",
                ));
            } else if scope.owner.as_ref() != Some(&value.ty) {
                violations.push(AstViolation::new(
                    DiagnosticCode::UnresolvedReference,
                    "this type does not match its declaring Java owner",
                ));
            }
        }
        JavaExprKind::Value(_) | JavaExprKind::Literal(_) => {}
        JavaExprKind::Unary { operand, .. } => {
            violations.extend(verify_expr_scope_structure(operand, scope));
        }
        JavaExprKind::Binary { left, right, .. } => {
            violations.extend(verify_expr_scope_structure(left, scope));
            violations.extend(verify_expr_scope_structure(right, scope));
        }
        JavaExprKind::Conditional {
            condition,
            when_true,
            when_false,
        } => {
            violations.extend(verify_expr_scope_structure(condition, scope));
            violations.extend(verify_expr_scope_structure(when_true, scope));
            violations.extend(verify_expr_scope_structure(when_false, scope));
        }
        JavaExprKind::Call {
            receiver,
            arguments,
            ..
        } => {
            if let Some(receiver) = receiver {
                violations.extend(verify_expr_scope_structure(receiver, scope));
            }
            for argument in arguments {
                violations.extend(verify_expr_scope_structure(argument, scope));
            }
        }
        JavaExprKind::New { arguments, .. } => {
            for argument in arguments {
                violations.extend(verify_expr_scope_structure(argument, scope));
            }
        }
        JavaExprKind::NewArray { length, .. } => {
            violations.extend(verify_expr_scope_structure(length, scope));
        }
        JavaExprKind::ArrayIndex { array, index } => {
            violations.extend(verify_expr_scope_structure(array, scope));
            violations.extend(verify_expr_scope_structure(index, scope));
        }
        JavaExprKind::Field { receiver, field } => {
            violations.extend(verify_expr_scope_structure(receiver, scope));
            if let JavaFieldRef::Structural { name, ty } = field {
                let valid =
                    if matches!(receiver.ty, JavaType::Array { .. }) && name.as_str() == "length" {
                        *ty == JavaType::primitive(JavaPrimitive::Int)
                    } else {
                        scope.owner.as_ref() == Some(&receiver.ty)
                            && scope
                                .owner_fields
                                .get(name)
                                .is_some_and(|metadata| metadata.ty == *ty)
                    };
                if !valid {
                    violations.push(AstViolation::new(
                        DiagnosticCode::UnresolvedReference,
                        "structural Java field is absent from its lexical owner declaration",
                    ));
                }
            }
        }
        JavaExprKind::Cast { value, .. }
        | JavaExprKind::InterfaceCoercion { value, .. }
        | JavaExprKind::ArrayOwnershipTransition { value, .. }
        | JavaExprKind::InstanceOf { value, .. } => {
            violations.extend(verify_expr_scope_structure(value, scope));
        }
        JavaExprKind::Lambda { .. } => {
            // The ordinary AST verifier rejects lambdas before rendering.
        }
    }
    violations
}

pub(super) fn verify_assignment_target_reads(
    target: &JavaExpr,
    scope: &JavaLexicalScope,
) -> Vec<AstViolation> {
    match &target.kind {
        JavaExprKind::Value(JavaValueRef::Local(_)) => vec![],
        JavaExprKind::Field { receiver, .. } => verify_expr_scope(receiver, scope),
        JavaExprKind::ArrayIndex { array, index } => {
            let mut violations = verify_expr_scope(array, scope);
            violations.extend(verify_expr_scope(index, scope));
            violations
        }
        _ => verify_expr_scope(target, scope),
    }
}

pub(super) fn verify_assignment_target(
    target: &JavaExpr,
    scope: &JavaLexicalScope,
    context: Option<&TargetAstContext<'_, JavaDialect>>,
) -> Vec<AstViolation> {
    match &target.kind {
        JavaExprKind::Value(JavaValueRef::Local(name)) => match scope.bindings.get(name) {
            Some(binding) if binding.mutable && binding.ty == target.ty => vec![],
            Some(binding) if !binding.mutable => vec![AstViolation::new(
                DiagnosticCode::InvalidControlFlow,
                "cannot assign to a final Java local or parameter",
            )],
            Some(_) => vec![type_error(
                "assignment target type disagrees with its lexical declaration",
            )],
            None => vec![AstViolation::new(
                DiagnosticCode::UnresolvedReference,
                "assignment target is outside its lexical scope",
            )],
        },
        JavaExprKind::Field { receiver, field } => {
            let metadata = match (field, context) {
                (JavaFieldRef::Known(_), _) => Some(JavaFieldMetadata {
                    ty: field.ty(),
                    final_field: true,
                    blank_final: false,
                }),
                (JavaFieldRef::Structural { name, .. }, Some(context)) => {
                    structural_field_metadata(&receiver.ty, name, context).or_else(|| {
                        (scope.owner.as_ref() == Some(&receiver.ty))
                            .then(|| scope.owner_fields.get(name).cloned())
                            .flatten()
                    })
                }
                (
                    JavaFieldRef::Generated {
                        owner,
                        field,
                        name,
                        ty,
                    },
                    Some(context),
                ) if receiver.ty == JavaType::Reference(JavaTypeName::Generated(*owner))
                    && generated_field_matches(*owner, *field, name, ty, context) =>
                {
                    Some(JavaFieldMetadata {
                        ty: ty.clone(),
                        final_field: true,
                        blank_final: true,
                    })
                }
                _ => None,
            };
            match metadata {
                Some(metadata) if metadata.ty != target.ty => vec![type_error(
                    "assignment field type disagrees with its declaration",
                )],
                Some(metadata) if !metadata.final_field => vec![],
                Some(metadata) if metadata.final_field && !metadata.blank_final => {
                    vec![AstViolation::new(
                        DiagnosticCode::InvalidControlFlow,
                        "initialized final Java fields cannot be assigned",
                    )]
                }
                Some(_)
                    if scope.constructor
                        && matches!(receiver.kind, JavaExprKind::Value(JavaValueRef::This))
                        && scope.owner.as_ref() == Some(&receiver.ty) =>
                {
                    vec![]
                }
                Some(_) => vec![AstViolation::new(
                    DiagnosticCode::InvalidControlFlow,
                    "final Java fields may be assigned only through this in their declaring constructor",
                )],
                None => vec![AstViolation::new(
                    DiagnosticCode::UnresolvedReference,
                    format!(
                        "assignment target does not resolve to a declared Java field on {:?}",
                        receiver.ty
                    ),
                )],
            }
        }
        JavaExprKind::ArrayIndex { array, .. }
            if matches!(
                array.ty,
                JavaType::Array {
                    ownership: JavaArrayOwnership::InternalMutable,
                    ..
                }
            ) =>
        {
            vec![]
        }
        JavaExprKind::ArrayIndex { .. } => vec![AstViolation::new(
            DiagnosticCode::InvalidControlFlow,
            "cannot assign through a defensive-copy Java array boundary",
        )],
        _ => vec![AstViolation::new(
            DiagnosticCode::InvalidControlFlow,
            "Java assignment target is not an lvalue",
        )],
    }
}
