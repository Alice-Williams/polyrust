//! Java AST: interface conformance.

use super::declaration_model::{
    JavaDeclarationKind, JavaHeritage, JavaMember, JavaMethod, JavaMethodDeclaration, JavaModifier,
    JavaTypeDeclaration,
};
use super::file_model::JavaFileItem;
use super::identifiers::JavaIdentifier;
use super::method_contracts::{
    interface_implementation_matches, registered_interface_implementation_matches,
    runtime_semantic_implementation_matches,
};
use super::runtime_members::JavaRuntimeMember;
use super::types::{JavaKnownType, JavaType, JavaTypeName};
use crate::dialect::{JavaDialect, JavaInvocationKind};
use portable_codegen::{
    AstViolation, GeneratedInterfaceMethodId, GeneratedTypeId, TargetAstContext,
};
use portable_diagnostics::DiagnosticCode;

pub(super) fn verify_interface_conformance(
    declaration: &JavaTypeDeclaration,
    context: &TargetAstContext<'_, JavaDialect>,
) -> Vec<AstViolation> {
    let JavaHeritage::Interfaces(interfaces) = &declaration.heritage else {
        return vec![];
    };
    let mut violations = Vec::new();
    for interface in interfaces {
        match interface {
            JavaType::Reference(JavaTypeName::Generated(interface_id)) => {
                let Some(required) = declared_interface_methods(*interface_id, context) else {
                    violations.push(AstViolation::new(
                        DiagnosticCode::InterfaceNonconformance,
                        "implemented generated Java interface has no matching AST declaration",
                    ));
                    continue;
                };
                for required_method in required {
                    let implementations = declaration
                        .members
                        .iter()
                        .filter(|member| match member {
                            JavaMember::Method(method)
                                if matches!(
                                    method.declared,
                                    JavaMethodDeclaration::Implementation { interface, .. }
                                        if interface == required_method
                                ) =>
                            {
                                registered_interface_implementation_matches(
                                    declaration,
                                    method,
                                    context,
                                )
                            }
                            _ => false,
                        })
                        .count();
                    if implementations != 1 {
                        violations.push(AstViolation::new(
                            DiagnosticCode::InterfaceNonconformance,
                            format!(
                                "Java conformance must implement interface method {required_method:?} exactly once"
                            ),
                        ));
                    }
                }
            }
            JavaType::Reference(JavaTypeName::Known(JavaKnownType::RuntimeSemanticValue)) => {
                let semantic_method = declaration.members.iter().any(|member| {
                    matches!(member, JavaMember::Method(method)
                    if runtime_semantic_implementation_matches(
                        declaration,
                        method,
                        JavaRuntimeMember::SemanticEquals,
                    ))
                });
                if !semantic_method {
                    violations.push(AstViolation::new(
                        DiagnosticCode::InterfaceNonconformance,
                        "Java runtime semantic value must implement semanticEquals(Object)",
                    ));
                }
                let deep_method = declaration.members.iter().any(|member| {
                    matches!(member, JavaMember::Method(method)
                    if runtime_semantic_implementation_matches(
                        declaration,
                        method,
                        JavaRuntimeMember::DeepEquals,
                    ))
                });
                if !deep_method {
                    violations.push(AstViolation::new(
                        DiagnosticCode::InterfaceNonconformance,
                        "Java runtime semantic value must implement deepEquals(Object)",
                    ));
                }
            }
            _ => {}
        }
    }
    violations
}

fn declared_interface_methods(
    interface: GeneratedTypeId,
    context: &TargetAstContext<'_, JavaDialect>,
) -> Option<Vec<GeneratedInterfaceMethodId>> {
    context.files().find_map(|file| {
        file.items().iter().find_map(|item| {
            let JavaFileItem::Type { declaration, .. } = item else {
                return None;
            };
            find_declared_interface_methods(declaration, interface)
        })
    })
}

fn find_declared_interface_methods(
    declaration: &JavaTypeDeclaration,
    interface: GeneratedTypeId,
) -> Option<Vec<GeneratedInterfaceMethodId>> {
    if declaration.declared == Some(interface)
        && matches!(
            declaration.kind,
            JavaDeclarationKind::Interface | JavaDeclarationKind::SealedInterface
        )
    {
        let mut methods = Vec::new();
        for member in &declaration.members {
            match member {
                JavaMember::Method(JavaMethod {
                    declared: JavaMethodDeclaration::Interface(method),
                    ..
                }) => methods.push(*method),
                JavaMember::Method(_) => return None,
                _ => {}
            }
        }
        return Some(methods);
    }
    declaration.members.iter().find_map(|member| match member {
        JavaMember::NestedType(nested) => find_declared_interface_methods(nested, interface),
        _ => None,
    })
}

pub(super) fn verify_method_registration(
    declaration: &JavaTypeDeclaration,
    method: &JavaMethod,
    context: &TargetAstContext<'_, JavaDialect>,
) -> Vec<AstViolation> {
    let (registered, expected_name, receiver_owner) = match method.declared {
        JavaMethodDeclaration::Structural => return vec![],
        JavaMethodDeclaration::Callable(id) => (
            context.callable(id).map(|value| &value.signature),
            context.callable(id).map(|value| value.name.as_str()),
            None,
        ),
        JavaMethodDeclaration::Interface(id)
        | JavaMethodDeclaration::Implementation { interface: id, .. } => (
            context.interface_method(id).map(|value| &value.signature),
            context
                .interface_method(id)
                .map(|value| value.name.as_str()),
            context.interface_method(id).map(|value| value.owner),
        ),
    };
    let Some(registered) = registered else {
        return vec![AstViolation::new(
            DiagnosticCode::UnresolvedReference,
            "Java method declaration references an unregistered callable",
        )];
    };
    let actual_parameters = method
        .parameters
        .iter()
        .map(|parameter| JavaDialect.registered_type(&parameter.ty))
        .collect::<Vec<_>>();
    let actual_return = JavaDialect.registered_type(&method.return_type);
    let name_matches =
        expected_name.map(JavaIdentifier::from_portable).as_ref() == Some(&method.name);
    let static_method = method.modifiers.contains(&JavaModifier::Static);
    let signature_matches =
        registered.parameters == actual_parameters && registered.return_type == actual_return;
    let declaration_matches = match method.declared {
        JavaMethodDeclaration::Structural => true,
        JavaMethodDeclaration::Callable(_) => {
            registered.invocation == JavaInvocationKind::Static
                && registered.receiver.is_none()
                && static_method
        }
        JavaMethodDeclaration::Interface(_) => {
            registered.invocation == JavaInvocationKind::Instance
                && !static_method
                && receiver_owner == declaration.declared
        }
        JavaMethodDeclaration::Implementation { .. } => receiver_owner.is_some_and(|owner| {
            interface_implementation_matches(
                declaration,
                method,
                registered,
                expected_name.expect("registered interface method has a name"),
                owner,
            )
        }),
    };
    if name_matches && signature_matches && declaration_matches {
        vec![]
    } else {
        vec![AstViolation::new(
            DiagnosticCode::InvalidInvocation,
            "Java method declaration does not match its authoritative registered callable",
        )]
    }
}
