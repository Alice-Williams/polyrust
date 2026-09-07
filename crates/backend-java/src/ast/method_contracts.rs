//! Java AST: method contracts.

use super::declaration_model::{
    JavaAnnotation, JavaHeritage, JavaMethod, JavaMethodDeclaration, JavaModifier, JavaParameter,
    JavaTypeDeclaration,
};
use super::identifiers::JavaIdentifier;
use super::runtime_members::JavaRuntimeMember;
use super::types::{JavaKnownType, JavaPrimitive, JavaType, JavaTypeName};
use crate::dialect::{JavaDialect, JavaInvocationKind};
use portable_codegen::{
    AstViolation, GeneratedTypeId, TargetAstContext, TargetCallableSignature, TargetTypeRef,
};
use portable_diagnostics::DiagnosticCode;
use std::collections::BTreeSet;

pub(super) fn verify_method_annotations(
    method: &JavaMethod,
    declaration: Option<&JavaTypeDeclaration>,
    context: &TargetAstContext<'_, JavaDialect>,
) -> Vec<AstViolation> {
    let mut violations = Vec::new();
    let distinct = method.annotations.iter().copied().collect::<BTreeSet<_>>();
    if distinct.len() != method.annotations.len() {
        violations.push(AstViolation::new(
            DiagnosticCode::DuplicateDeclaration,
            "Java method annotation is declared more than once",
        ));
    }
    for annotation in distinct {
        match annotation {
            JavaAnnotation::Override
                if !method_has_override_target(method, declaration, context) =>
            {
                violations.push(AstViolation::new(
                    DiagnosticCode::InvalidStructure,
                    "Java @Override method has no verified instance override target",
                ));
            }
            JavaAnnotation::SafeVarargs => violations.push(AstViolation::new(
                DiagnosticCode::InvalidStructure,
                "Java @SafeVarargs requires a varargs parameter form which the typed AST does not admit",
            )),
            JavaAnnotation::Override => {}
        }
    }
    violations
}

fn method_has_override_target(
    method: &JavaMethod,
    declaration: Option<&JavaTypeDeclaration>,
    context: &TargetAstContext<'_, JavaDialect>,
) -> bool {
    if matches!(
        method.declared,
        JavaMethodDeclaration::UninhabitedImplementation(_)
    ) {
        return declaration.is_some_and(|declaration| {
            super::uninhabited::method_matches(declaration, method, context)
        });
    }
    if matches!(
        method.declared,
        JavaMethodDeclaration::Implementation { .. }
    ) {
        return declaration.is_some_and(|declaration| {
            registered_interface_implementation_matches(declaration, method, context)
        });
    }
    let Some(declaration) = declaration else {
        return false;
    };
    runtime_semantic_implementation_matches(declaration, method, JavaRuntimeMember::SemanticEquals)
        || runtime_semantic_implementation_matches(
            declaration,
            method,
            JavaRuntimeMember::DeepEquals,
        )
}

fn public_concrete_instance_method(method: &JavaMethod) -> bool {
    method.modifiers.contains(&JavaModifier::Public)
        && !method.modifiers.contains(&JavaModifier::Private)
        && !method.modifiers.contains(&JavaModifier::Static)
        && !method.modifiers.contains(&JavaModifier::Abstract)
        && method.body.is_some()
}

pub(super) fn runtime_semantic_implementation_matches(
    declaration: &JavaTypeDeclaration,
    method: &JavaMethod,
    member: JavaRuntimeMember,
) -> bool {
    method.declared == JavaMethodDeclaration::Structural
        && public_concrete_instance_method(method)
        && matches!(
            &declaration.heritage,
            JavaHeritage::Interfaces(values)
                if values.contains(&JavaType::known(JavaKnownType::RuntimeSemanticValue))
        )
        && method.name.as_str() == member.name()
        && method.return_type == JavaType::primitive(JavaPrimitive::Boolean)
        && matches!(
            method.parameters.as_slice(),
            [JavaParameter { ty, .. }] if *ty == JavaType::known(JavaKnownType::Object)
        )
}

pub(super) fn registered_interface_implementation_matches(
    declaration: &JavaTypeDeclaration,
    method: &JavaMethod,
    context: &TargetAstContext<'_, JavaDialect>,
) -> bool {
    let JavaMethodDeclaration::Implementation {
        interface, witness, ..
    } = method.declared
    else {
        return false;
    };
    if !witness.matches(declaration, method.declared, context) {
        return false;
    }
    let Some(registered) = context.interface_method(interface) else {
        return false;
    };
    interface_implementation_matches(
        declaration,
        method,
        &registered.signature,
        registered.name.as_str(),
        registered.owner,
    )
}

pub(super) fn interface_implementation_matches(
    declaration: &JavaTypeDeclaration,
    method: &JavaMethod,
    registered: &TargetCallableSignature<JavaDialect>,
    expected_name: &str,
    owner: GeneratedTypeId,
) -> bool {
    let actual_parameters = method
        .parameters
        .iter()
        .map(|parameter| JavaDialect.registered_type(&parameter.ty))
        .collect::<Vec<_>>();
    public_concrete_instance_method(method)
        && method.type_parameters.is_empty()
        && method.name == JavaIdentifier::from_portable(expected_name)
        && registered.invocation == JavaInvocationKind::Instance
        && registered.receiver == Some(TargetTypeRef::Generated(owner))
        && registered.parameters == actual_parameters
        && registered.return_type == JavaDialect.registered_type(&method.return_type)
        && matches!(&declaration.heritage, JavaHeritage::Interfaces(values)
            if values.contains(&JavaType::Reference(JavaTypeName::Generated(owner))))
}
