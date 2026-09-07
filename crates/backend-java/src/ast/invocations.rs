//! Java AST: invocations.

use super::declaration_model::{JavaHeritage, JavaMember, JavaTypeDeclaration};
use super::expression_model::JavaMethodSignature;
use super::expression_nodes::JavaExpr;
use super::field_metadata::find_type_declaration;
use super::file_model::JavaFileItem;
use super::operator_signatures::invocation_types_match;
use super::type_context::erased_java_type;
use super::types::{JavaKnownType, JavaPrimitive, JavaType, JavaTypeName};
use crate::dialect::JavaDialect;
use portable_codegen::{AstViolation, GeneratedTypeId, TargetAstContext};
use portable_diagnostics::DiagnosticCode;

pub(super) fn verify_call(
    violations: &mut Vec<AstViolation>,
    signature: &JavaMethodSignature,
    receiver: Option<&JavaExpr>,
    arguments: &[JavaExpr],
    result: &JavaType,
    context: &TargetAstContext<'_, JavaDialect>,
) {
    let receiver_valid = match (&signature.receiver, receiver) {
        (None, None) => true,
        (Some(expected), Some(actual)) => {
            invocation_types_match_in_context(expected, &actual.ty, context)
        }
        _ => false,
    };
    if !receiver_valid
        || signature.parameters.len() != arguments.len()
        || signature
            .parameters
            .iter()
            .zip(arguments)
            .any(|(a, b)| !invocation_types_match_in_context(a, &b.ty, context))
        || &signature.result != result
    {
        violations.push(AstViolation::new(
            DiagnosticCode::InvalidInvocation,
            "call does not match its authoritative owner/receiver/parameter/result signature",
        ));
    }
}

fn invocation_types_match_in_context(
    expected: &JavaType,
    actual: &JavaType,
    context: &TargetAstContext<'_, JavaDialect>,
) -> bool {
    invocation_types_match(expected, actual)
        || matches!(
            (expected, actual),
            (
                JavaType::Reference(JavaTypeName::Generated(expected_interface)),
                JavaType::Reference(JavaTypeName::Generated(actual_type)),
            ) if generated_type_implements(*actual_type, *expected_interface, context)
        )
}

pub(super) fn generated_type_implements(
    actual: GeneratedTypeId,
    expected_interface: GeneratedTypeId,
    context: &TargetAstContext<'_, JavaDialect>,
) -> bool {
    context.files().any(|file| {
        file.items().iter().any(|item| {
            matches!(item, JavaFileItem::Type { declaration, .. }
            if declaration_contains_conformance(
                declaration,
                actual,
                expected_interface,
            ))
        })
    })
}

fn declaration_contains_conformance(
    declaration: &JavaTypeDeclaration,
    actual: GeneratedTypeId,
    expected_interface: GeneratedTypeId,
) -> bool {
    (declaration.declared == Some(actual)
        && matches!(&declaration.heritage, JavaHeritage::Interfaces(values)
            if values.contains(&JavaType::Reference(JavaTypeName::Generated(expected_interface)))))
        || declaration.members.iter().any(|member| {
            matches!(member, JavaMember::NestedType(nested)
                if declaration_contains_conformance(nested, actual, expected_interface))
        })
}

pub(super) fn java_cast_is_legal(
    target: &JavaType,
    source: &JavaType,
    context: &TargetAstContext<'_, JavaDialect>,
) -> bool {
    if target == source {
        return !matches!(target, JavaType::Primitive(JavaPrimitive::Void));
    }
    match (target, source) {
        (JavaType::Primitive(target), JavaType::Primitive(source)) => {
            java_numeric_primitive(*target) && java_numeric_primitive(*source)
        }
        (JavaType::Primitive(target), JavaType::Boxed(source)) => {
            (*target == *source && *target != JavaPrimitive::Void)
                || (java_numeric_primitive(*target) && java_numeric_primitive(*source))
        }
        (JavaType::Boxed(target), JavaType::Primitive(source)) => {
            target == source && *target != JavaPrimitive::Void
        }
        (JavaType::Reference(JavaTypeName::Known(JavaKnownType::Object)), source) => {
            !matches!(source, JavaType::Primitive(JavaPrimitive::Void))
        }
        (target, JavaType::Reference(JavaTypeName::Known(JavaKnownType::Object))) => match target {
            JavaType::Primitive(value) => *value != JavaPrimitive::Void,
            JavaType::Boxed(_) | JavaType::Reference(_) => true,
            JavaType::Generic { .. } => java_type_is_reifiable(target),
            JavaType::Array { .. } | JavaType::Wildcard { .. } | JavaType::TypeVariable(_) => false,
        },
        (
            JavaType::Array {
                component: target,
                ownership: target_ownership,
            },
            JavaType::Array {
                component: source,
                ownership: source_ownership,
            },
        ) => {
            target_ownership == source_ownership
                && (target == source || java_reference_cast_is_legal(target, source, context))
        }
        (target, source) => java_reference_cast_is_legal(target, source, context),
    }
}

fn java_reference_cast_is_legal(
    target: &JavaType,
    source: &JavaType,
    context: &TargetAstContext<'_, JavaDialect>,
) -> bool {
    if !java_type_is_reference(target) || !java_type_is_reference(source) {
        return false;
    }
    if target != source && !java_type_is_reifiable(target) {
        return false;
    }
    if matches!(source, JavaType::TypeVariable(_)) && !matches!(target, JavaType::TypeVariable(_)) {
        return !matches!(target, JavaType::Array { .. });
    }
    if erased_java_type(target) == erased_java_type(source) {
        return target == source;
    }
    invocation_types_match_in_context(target, source, context)
        || invocation_types_match_in_context(source, target, context)
        || generated_and_known_interface_related(target, source, context)
        || generated_and_known_interface_related(source, target, context)
}

pub(super) fn generated_and_known_interface_related(
    expected_interface: &JavaType,
    actual: &JavaType,
    context: &TargetAstContext<'_, JavaDialect>,
) -> bool {
    let JavaType::Reference(JavaTypeName::Known(JavaKnownType::RuntimeSemanticValue)) =
        expected_interface
    else {
        return false;
    };
    find_type_declaration(actual, context).is_some_and(|declaration| {
        matches!(
            declaration.heritage,
            JavaHeritage::Interfaces(ref interfaces)
                if interfaces.contains(expected_interface)
        )
    })
}

pub(super) fn java_instanceof_is_legal(
    source: &JavaType,
    target: &JavaType,
    context: &TargetAstContext<'_, JavaDialect>,
) -> bool {
    java_type_is_reference(source)
        && java_type_is_reference(target)
        && java_cast_is_legal(target, source, context)
}

pub(super) fn java_type_is_reference(ty: &JavaType) -> bool {
    matches!(
        ty,
        JavaType::Boxed(_)
            | JavaType::Reference(_)
            | JavaType::Array { .. }
            | JavaType::Generic { .. }
            | JavaType::TypeVariable(_)
    )
}

pub(super) fn java_type_is_reifiable(ty: &JavaType) -> bool {
    match ty {
        JavaType::Boxed(_) | JavaType::Reference(_) => true,
        JavaType::Array { component, .. } => java_type_is_reifiable(component),
        JavaType::Generic { arguments, .. } => arguments
            .iter()
            .all(|argument| matches!(argument, JavaType::Wildcard { bound: None })),
        JavaType::Primitive(_) | JavaType::Wildcard { .. } | JavaType::TypeVariable(_) => false,
    }
}

fn java_numeric_primitive(value: JavaPrimitive) -> bool {
    matches!(
        value,
        JavaPrimitive::Byte
            | JavaPrimitive::Char
            | JavaPrimitive::Int
            | JavaPrimitive::Long
            | JavaPrimitive::Double
    )
}
