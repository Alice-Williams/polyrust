//! Java AST: generated members.

use super::declaration_model::{
    JavaDeclarationKind, JavaMember, JavaMethodDeclaration, JavaModifier,
    JavaRecordComponentOrigin, JavaTypeDeclaration,
};
use super::expression_model::JavaMethodSignature;
use super::file_model::JavaFileItem;
use super::identifiers::JavaIdentifier;
use super::types::{JavaType, JavaTypeName};
use crate::dialect::JavaDialect;
use portable_codegen::{GeneratedTypeId, GeneratedValueId, TargetAstContext};
use portable_core_ir::CoreFieldId;

pub(super) fn generated_constructor_matches(
    owner: GeneratedTypeId,
    parameters: &[JavaType],
    context: &TargetAstContext<'_, JavaDialect>,
) -> bool {
    context.files().any(|file| {
        file.items().iter().any(|item| {
            matches!(item, JavaFileItem::Type { declaration, .. }
                if declaration_has_constructor(declaration, owner, parameters))
        })
    })
}

pub(super) fn generated_value_matches(
    value: GeneratedValueId,
    ty: &JavaType,
    context: &TargetAstContext<'_, JavaDialect>,
) -> Option<bool> {
    let registered = context.value(value)?;
    let declared_type = context.files().find_map(|file| {
        file.items().iter().find_map(|item| {
            let JavaFileItem::Type { declaration, .. } = item else {
                return None;
            };
            find_generated_value_type(declaration, value)
        })
    });
    Some(
        registered.ty == JavaDialect.registered_type(ty)
            && declared_type.is_some_and(|declared| declared == *ty),
    )
}

fn find_generated_value_type(
    declaration: &JavaTypeDeclaration,
    value: GeneratedValueId,
) -> Option<JavaType> {
    declaration.members.iter().find_map(|member| match member {
        JavaMember::Field(field) if field.declared == Some(value) => Some(field.ty.clone()),
        JavaMember::EnumConstant(constant) if constant.declared == value => declaration
            .declared
            .map(|owner| JavaType::Reference(JavaTypeName::Generated(owner))),
        JavaMember::NestedType(nested) => find_generated_value_type(nested, value),
        _ => None,
    })
}

pub(super) fn generated_enum_variant_matches(
    enumeration: GeneratedTypeId,
    variant: GeneratedValueId,
    ty: &JavaType,
    context: &TargetAstContext<'_, JavaDialect>,
) -> Option<bool> {
    let expected = JavaType::Reference(JavaTypeName::Generated(enumeration));
    let registered = generated_value_matches(variant, ty, context)?;
    let declared = context.files().any(|file| {
        file.items().iter().any(|item| {
            matches!(item, JavaFileItem::Type { declaration, .. }
                if declaration_has_enum_variant(declaration, enumeration, variant))
        })
    });
    Some(registered && *ty == expected && declared)
}

fn declaration_has_enum_variant(
    declaration: &JavaTypeDeclaration,
    enumeration: GeneratedTypeId,
    variant: GeneratedValueId,
) -> bool {
    (declaration.declared == Some(enumeration)
        && declaration.kind == JavaDeclarationKind::Enum
        && declaration.members.iter().any(
            |member| matches!(member, JavaMember::EnumConstant(value) if value.declared == variant),
        ))
        || declaration.members.iter().any(|member| {
            matches!(member, JavaMember::NestedType(nested)
                if declaration_has_enum_variant(nested, enumeration, variant))
        })
}

fn declaration_has_constructor(
    declaration: &JavaTypeDeclaration,
    owner: GeneratedTypeId,
    parameters: &[JavaType],
) -> bool {
    (declaration.declared == Some(owner)
        && declaration.members.iter().any(|member| {
            matches!(member, JavaMember::Constructor(constructor)
                if constructor.name == declaration.name
                    && constructor.parameters.iter().map(|value| &value.ty).eq(parameters.iter()))
        }))
        || declaration.members.iter().any(|member| {
            matches!(member, JavaMember::NestedType(nested)
                if declaration_has_constructor(nested, owner, parameters))
        })
}

pub(super) fn generated_field_matches(
    owner: GeneratedTypeId,
    field: CoreFieldId,
    name: &JavaIdentifier,
    ty: &JavaType,
    context: &TargetAstContext<'_, JavaDialect>,
) -> bool {
    context.files().any(|file| {
        file.items().iter().any(|item| {
            matches!(item, JavaFileItem::Type { declaration, .. }
                if declaration_has_generated_field(declaration, owner, field, name, ty))
        })
    })
}

fn declaration_has_generated_field(
    declaration: &JavaTypeDeclaration,
    owner: GeneratedTypeId,
    field: CoreFieldId,
    name: &JavaIdentifier,
    ty: &JavaType,
) -> bool {
    (declaration.declared == Some(owner)
        && declaration.record_components.iter().any(|component| {
            component.origin == JavaRecordComponentOrigin::Core(field)
                && &component.name == name
                && &component.ty == ty
        }))
        || declaration.members.iter().any(|member| {
            matches!(member, JavaMember::NestedType(nested)
                if declaration_has_generated_field(nested, owner, field, name, ty))
        })
}

pub(super) fn generated_accessor_matches(
    owner: &JavaType,
    field: CoreFieldId,
    name: &JavaIdentifier,
    signature: &JavaMethodSignature,
    context: &TargetAstContext<'_, JavaDialect>,
) -> bool {
    let JavaType::Reference(JavaTypeName::Generated(owner_id)) = owner else {
        return false;
    };
    signature.receiver.as_ref() == Some(owner)
        && signature.parameters.is_empty()
        && generated_field_matches(*owner_id, field, name, &signature.result, context)
}

pub(super) fn generated_member_matches(
    owner: &JavaType,
    name: &JavaIdentifier,
    signature: &JavaMethodSignature,
    implementation: Option<portable_core_ir::CoreImplementationMethodId>,
    context: &TargetAstContext<'_, JavaDialect>,
) -> bool {
    context.files().any(|file| {
        file.items().iter().any(|item| {
            matches!(item, JavaFileItem::Type { declaration, .. }
            if declaration_has_member(
                declaration,
                owner,
                name,
                signature,
                implementation,
            ))
        })
    })
}

fn declaration_has_member(
    declaration: &JavaTypeDeclaration,
    owner: &JavaType,
    name: &JavaIdentifier,
    signature: &JavaMethodSignature,
    implementation: Option<portable_core_ir::CoreImplementationMethodId>,
) -> bool {
    let represents_owner = match owner {
        JavaType::Reference(JavaTypeName::Generated(id)) => declaration.declared == Some(*id),
        JavaType::Reference(JavaTypeName::Known(known))
        | JavaType::Generic {
            raw: JavaTypeName::Known(known),
            ..
        } if known.runtime_helper().is_some() => known
            .qualified_name()
            .rsplit('.')
            .next()
            .is_some_and(|simple| simple == declaration.name.as_str()),
        _ => false,
    };
    let member_matches = represents_owner
        && declaration.members.iter().any(|member| match member {
            JavaMember::Method(method) => {
                let origin_matches = match (implementation, method.declared) {
                    (Some(expected), JavaMethodDeclaration::Implementation { method, .. }) => {
                        expected == method
                    }
                    (None, JavaMethodDeclaration::Structural) => true,
                    _ => false,
                };
                origin_matches
                    && method.name == *name
                    && !method.modifiers.contains(&JavaModifier::Static)
                    && method
                        .parameters
                        .iter()
                        .map(|value| &value.ty)
                        .eq(signature.parameters.iter())
                    && method.return_type == signature.result
            }
            _ => false,
        });
    let implicit_record_accessor = represents_owner
        && implementation.is_none()
        && signature.parameters.is_empty()
        && declaration
            .record_components
            .iter()
            .any(|component| component.name == *name && component.ty == signature.result);
    member_matches
        || implicit_record_accessor
        || declaration.members.iter().any(|member| {
            matches!(member, JavaMember::NestedType(nested)
            if declaration_has_member(
                nested,
                owner,
                name,
                signature,
                implementation,
            ))
        })
}
