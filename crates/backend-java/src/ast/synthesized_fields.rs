//! Target-created components retain an exact owner without inventing source IDs.
use super::{
    JavaDeclarationKind, JavaIdentifier, JavaPrimitive, JavaRecordComponent,
    JavaRecordComponentOrigin, JavaType, JavaTypeDeclaration, JavaTypeName,
};
use crate::dialect::JavaDialect;
use portable_codegen::{GeneratedOrigin, GeneratedTypeId, SynthesisReason, TargetAstContext};

/// A role describes storage, not proof of a source Result or a complete family.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum JavaSynthesizedFieldRole {
    ScalarResultPayload,
}

impl JavaSynthesizedFieldRole {
    fn ty(self) -> JavaType {
        match self {
            Self::ScalarResultPayload => JavaType::primitive(JavaPrimitive::Int),
        }
    }
}

/// Descriptive identity. Certification checks the actual declaration and owner.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct JavaSynthesizedField {
    pub owner: GeneratedTypeId,
    pub role: JavaSynthesizedFieldRole,
}

pub(super) fn valid_component(
    field: JavaSynthesizedField,
    component: &JavaRecordComponent,
    declaration: &JavaTypeDeclaration,
    context: &TargetAstContext<'_, JavaDialect>,
) -> bool {
    declaration.declared == Some(field.owner)
        && declaration.kind == JavaDeclarationKind::Record
        && declaration.type_parameters.is_empty()
        && component.ty == field.role.ty()
        && context
            .generated_type(field.owner)
            .is_some_and(|registered| {
                registered.origin == GeneratedOrigin::Synthesized(SynthesisReason::InterfaceAdapter)
            })
}

pub(super) fn matches(
    field: JavaSynthesizedField,
    name: &JavaIdentifier,
    ty: &JavaType,
    context: &TargetAstContext<'_, JavaDialect>,
) -> bool {
    let owner = JavaType::Reference(JavaTypeName::Generated(field.owner));
    super::field_metadata::find_type_declaration(&owner, context).is_some_and(|declaration| {
        declaration.record_components.iter().any(|component| {
            component.origin == JavaRecordComponentOrigin::Synthesized(field)
                && &component.name == name
                && &component.ty == ty
                && valid_component(field, component, &declaration, context)
        })
    })
}
