//! Mapping-owned target-only representation of an interface with no instances.

use super::JavaInterfaceMethodInput;
use crate::ast::{
    JavaAnnotation, JavaDeclarationKind, JavaHeritage, JavaMember, JavaMethod,
    JavaMethodDeclaration, JavaModifier, JavaType, JavaTypeDeclaration, JavaTypeName,
    JavaVisibility,
};
use crate::dialect::JavaDialect;
use crate::lower::identifier;
use portable_codegen::{GeneratedOrigin, GeneratedType, GeneratedTypeId, SynthesisReason};
use portable_diagnostics::SourceRef;

#[derive(Clone)]
pub(crate) struct JavaUninhabitedInterfaceInput {
    pub(crate) declared: GeneratedTypeId,
    pub(crate) name: String,
}

pub(super) fn uninhabited_type(
    interface: GeneratedTypeId,
    name: &str,
    source: SourceRef,
) -> GeneratedType<JavaDialect> {
    GeneratedType {
        name: name.to_owned(),
        kind: JavaDeclarationKind::UninhabitedEnum(interface),
        visibility: JavaVisibility::Private,
        origin: GeneratedOrigin::Synthesized(SynthesisReason::UninhabitedInterface),
        source,
    }
}

pub(super) fn uninhabited_declaration(
    interface: GeneratedTypeId,
    input: &JavaUninhabitedInterfaceInput,
    methods: &[JavaInterfaceMethodInput],
) -> JavaTypeDeclaration {
    JavaTypeDeclaration {
        declared: Some(input.declared),
        kind: JavaDeclarationKind::UninhabitedEnum(interface),
        visibility: JavaVisibility::Private,
        modifiers: vec![],
        name: identifier(&input.name),
        type_parameters: vec![],
        record_components: vec![],
        heritage: JavaHeritage::Interfaces(vec![JavaType::Reference(JavaTypeName::Generated(
            interface,
        ))]),
        permits: vec![],
        members: methods
            .iter()
            .map(|method| {
                JavaMember::Method(JavaMethod {
                    declared: JavaMethodDeclaration::UninhabitedImplementation(method.declared),
                    annotations: vec![JavaAnnotation::Override],
                    modifiers: vec![JavaModifier::Public],
                    type_parameters: vec![],
                    return_type: method.return_type.clone(),
                    name: identifier(&method.name),
                    parameters: method.parameters.clone(),
                    body: Some(crate::ast::uninhabited::unreachable_body()),
                })
            })
            .collect(),
    }
}
