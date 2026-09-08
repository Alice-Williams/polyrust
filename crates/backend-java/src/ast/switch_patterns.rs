//! Java AST: switch patterns.

use super::declaration_model::{JavaDeclarationKind, JavaMember};
use super::expression_model::JavaLiteral;
use super::expression_nodes::{JavaExpr, JavaExprKind};
use super::field_metadata::find_type_declaration;
use super::generated_members::generated_enum_variant_matches;
use super::invocations::{
    generated_and_known_interface_related, generated_type_implements, java_instanceof_is_legal,
    java_type_is_reference, java_type_is_reifiable,
};
use super::statement_model::{JavaPattern, JavaSwitchArm};
use super::type_context::erased_java_type;
use super::types::{JavaKnownType, JavaPrimitive, JavaType, JavaTypeName, type_error};
use crate::dialect::JavaDialect;
use portable_codegen::{AstViolation, TargetAstContext};
use portable_diagnostics::DiagnosticCode;
use std::collections::BTreeSet;

pub(super) fn is_java_statement_expression(value: &JavaExpr) -> bool {
    matches!(
        value.kind,
        JavaExprKind::Call { .. } | JavaExprKind::New { .. }
    )
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum JavaSwitchConstant {
    Integral(i64),
    String(String),
}

pub(super) fn verify_switch_patterns(
    selector: &JavaExpr,
    arms: &[JavaSwitchArm],
    context: &TargetAstContext<'_, JavaDialect>,
) -> Vec<AstViolation> {
    let mut violations = Vec::new();
    if !java_switch_selector_is_legal(&selector.ty) {
        violations.push(type_error(
            "Java switch selector must be an int-compatible primitive or reference type",
        ));
    }
    let default_count = arms
        .iter()
        .filter(|arm| matches!(arm.pattern, JavaPattern::Default))
        .count();
    let enum_patterns = arms
        .iter()
        .filter_map(|arm| match arm.pattern {
            JavaPattern::EnumVariant {
                enumeration,
                variant,
            } => Some((enumeration, variant)),
            _ => None,
        })
        .collect::<Vec<_>>();
    if !enum_patterns.is_empty() {
        let enumeration = enum_patterns[0].0;
        let expected_type = JavaType::Reference(JavaTypeName::Generated(enumeration));
        if selector.ty != expected_type {
            violations.push(type_error(
                "Java enum switch selector does not have the declared enum type",
            ));
        }
        if default_count != 1 {
            violations.push(AstViolation::new(
                DiagnosticCode::NonExhaustiveMatch,
                "portable Java enum switch must have one unreachable default arm",
            ));
        }
        if arms.iter().any(|arm| {
            !matches!(
                arm.pattern,
                JavaPattern::Default | JavaPattern::EnumVariant { .. }
            )
        }) {
            violations.push(AstViolation::new(
                DiagnosticCode::InvalidStructure,
                "portable Java enum switch cannot mix enum and non-enum labels",
            ));
        }
        let expected = find_type_declaration(&expected_type, context)
            .filter(|declaration| declaration.kind == JavaDeclarationKind::Enum)
            .map(|declaration| {
                declaration
                    .members
                    .into_iter()
                    .filter_map(|member| match member {
                        JavaMember::EnumConstant(value) => Some(value.declared),
                        _ => None,
                    })
                    .collect::<BTreeSet<_>>()
            });
        let mut actual = BTreeSet::new();
        for (owner, variant) in enum_patterns {
            if owner != enumeration
                || !generated_enum_variant_matches(owner, variant, &selector.ty, context)
                    .unwrap_or(false)
                || !actual.insert(variant)
            {
                violations.push(AstViolation::new(
                    DiagnosticCode::DuplicateDeclaration,
                    "Java enum switch contains a repeated or foreign enum variant",
                ));
            }
        }
        if expected.as_ref() != Some(&actual) {
            violations.push(AstViolation::new(
                DiagnosticCode::NonExhaustiveMatch,
                "portable Java enum switch must cover every declared variant exactly once",
            ));
        }
        return violations;
    }
    if default_count != 1 {
        violations.push(AstViolation::new(
            DiagnosticCode::NonExhaustiveMatch,
            "statement switch must have exactly one typed default arm",
        ));
    }

    let mut constants = Vec::new();
    let mut prior_types = Vec::<JavaType>::new();
    for arm in arms {
        match &arm.pattern {
            JavaPattern::Default => {}
            JavaPattern::EnumVariant { .. } => unreachable!("enum switch returned above"),
            JavaPattern::Literal(literal) => {
                violations.extend(super::literal_limits::verify_payload(literal));
                if !java_switch_literal_is_compatible(literal, &selector.ty) {
                    violations.push(type_error(
                        "Java switch literal is not compatible with its selector type",
                    ));
                    continue;
                }
                if prior_types
                    .iter()
                    .any(|prior| java_type_pattern_dominates(prior, &selector.ty, context))
                {
                    violations.push(AstViolation::new(
                        DiagnosticCode::InvalidControlFlow,
                        "Java switch literal is dominated by an earlier type pattern",
                    ));
                }
                if let Some(constant) = java_switch_constant(literal)
                    && !constants.contains(&constant)
                {
                    constants.push(constant);
                } else {
                    violations.push(AstViolation::new(
                        DiagnosticCode::DuplicateDeclaration,
                        "Java switch constant label is repeated",
                    ));
                }
            }
            JavaPattern::Type { ty, .. } => {
                if !java_type_is_reifiable(ty)
                    || !java_instanceof_is_legal(&selector.ty, ty, context)
                {
                    violations.push(type_error(
                        "Java switch type pattern is not reifiable or selector-compatible",
                    ));
                }
                if prior_types
                    .iter()
                    .any(|prior| java_type_pattern_dominates(prior, ty, context))
                {
                    violations.push(AstViolation::new(
                        DiagnosticCode::InvalidControlFlow,
                        "Java switch type pattern is dominated by an earlier pattern",
                    ));
                }
                if java_type_pattern_dominates(ty, &selector.ty, context) {
                    violations.push(AstViolation::new(
                        DiagnosticCode::InvalidStructure,
                        "Java switch cannot combine an unconditional type pattern with its required default arm",
                    ));
                }
                prior_types.push(ty.clone());
            }
        }
    }
    violations
}

fn java_switch_selector_is_legal(ty: &JavaType) -> bool {
    match ty {
        JavaType::Primitive(value) => matches!(
            value,
            JavaPrimitive::Byte | JavaPrimitive::Char | JavaPrimitive::Int
        ),
        _ => java_type_is_reference(ty),
    }
}

fn java_switch_literal_is_compatible(literal: &JavaLiteral, selector: &JavaType) -> bool {
    match literal {
        JavaLiteral::I32(value) => match selector {
            JavaType::Primitive(JavaPrimitive::Byte) | JavaType::Boxed(JavaPrimitive::Byte) => {
                i8::try_from(*value).is_ok()
            }
            JavaType::Primitive(JavaPrimitive::Char) | JavaType::Boxed(JavaPrimitive::Char) => {
                u16::try_from(*value).is_ok()
            }
            JavaType::Primitive(JavaPrimitive::Int) | JavaType::Boxed(JavaPrimitive::Int) => true,
            _ => false,
        },
        JavaLiteral::CharScalar(value) => match selector {
            JavaType::Primitive(JavaPrimitive::Byte) | JavaType::Boxed(JavaPrimitive::Byte) => {
                *value <= i8::MAX as u32
            }
            JavaType::Primitive(JavaPrimitive::Char) | JavaType::Boxed(JavaPrimitive::Char) => {
                u16::try_from(*value).is_ok()
            }
            JavaType::Primitive(JavaPrimitive::Int) | JavaType::Boxed(JavaPrimitive::Int) => true,
            _ => false,
        },
        JavaLiteral::String(_) => *selector == JavaType::known(JavaKnownType::String),
        JavaLiteral::Boolean(_)
        | JavaLiteral::I64(_)
        | JavaLiteral::Utf16Units(_)
        | JavaLiteral::InternalNull(_) => false,
    }
}

fn java_switch_constant(literal: &JavaLiteral) -> Option<JavaSwitchConstant> {
    match literal {
        JavaLiteral::I32(value) => Some(JavaSwitchConstant::Integral(i64::from(*value))),
        JavaLiteral::CharScalar(value) => Some(JavaSwitchConstant::Integral(i64::from(*value))),
        JavaLiteral::String(value) => Some(JavaSwitchConstant::String(value.clone())),
        JavaLiteral::Boolean(_)
        | JavaLiteral::I64(_)
        | JavaLiteral::Utf16Units(_)
        | JavaLiteral::InternalNull(_) => None,
    }
}

fn java_type_pattern_dominates(
    earlier: &JavaType,
    later: &JavaType,
    context: &TargetAstContext<'_, JavaDialect>,
) -> bool {
    if erased_java_type(earlier) == erased_java_type(later) {
        return true;
    }
    if *earlier == JavaType::known(JavaKnownType::Object) && java_type_is_reference(later) {
        return true;
    }
    match (earlier, later) {
        (
            JavaType::Reference(JavaTypeName::Generated(expected)),
            JavaType::Reference(JavaTypeName::Generated(actual)),
        ) => generated_type_implements(*actual, *expected, context),
        (
            JavaType::Array {
                component: expected,
                ..
            },
            JavaType::Array {
                component: actual, ..
            },
        ) => java_type_pattern_dominates(expected, actual, context),
        _ => generated_and_known_interface_related(earlier, later, context),
    }
}

pub(super) fn iterable_element_type(ty: &JavaType) -> Option<&JavaType> {
    match ty {
        JavaType::Array { component, .. } => Some(component),
        JavaType::Generic {
            raw: JavaTypeName::Known(JavaKnownType::List),
            arguments,
        } if arguments.len() == 1 => arguments.first(),
        _ => None,
    }
}
