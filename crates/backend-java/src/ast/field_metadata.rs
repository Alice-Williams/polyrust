//! Java AST: field metadata.

use super::declaration_model::{JavaMember, JavaModifier, JavaTypeDeclaration};
use super::file_model::JavaFileItem;
use super::identifiers::JavaIdentifier;
use super::types::{JavaPrimitive, JavaType, JavaTypeName};
use crate::dialect::JavaDialect;
use portable_codegen::TargetAstContext;
use std::collections::BTreeMap;

#[derive(Clone, Debug)]
pub(super) struct JavaFieldMetadata {
    pub(super) ty: JavaType,
    pub(super) final_field: bool,
    pub(super) blank_final: bool,
}

pub(super) fn structural_field_metadata(
    owner: &JavaType,
    name: &JavaIdentifier,
    context: &TargetAstContext<'_, JavaDialect>,
) -> Option<JavaFieldMetadata> {
    if matches!(owner, JavaType::Array { .. }) && name.as_str() == "length" {
        return Some(JavaFieldMetadata {
            ty: JavaType::primitive(JavaPrimitive::Int),
            final_field: true,
            blank_final: false,
        });
    }
    let declaration = find_type_declaration(owner, context)?;
    let substitutions = declaration_type_substitutions(&declaration, owner)?;
    if let Some(component) = declaration
        .record_components
        .iter()
        .find(|component| component.name == *name)
    {
        return Some(JavaFieldMetadata {
            ty: substitute_type_variables(&component.ty, &substitutions),
            final_field: true,
            blank_final: true,
        });
    }
    declaration.members.iter().find_map(|member| match member {
        JavaMember::Field(field)
            if field.name == *name && !field.modifiers.contains(&JavaModifier::Static) =>
        {
            Some(JavaFieldMetadata {
                ty: substitute_type_variables(&field.ty, &substitutions),
                final_field: field.modifiers.contains(&JavaModifier::Final),
                blank_final: field.modifiers.contains(&JavaModifier::Final)
                    && field.initializer.is_none(),
            })
        }
        _ => None,
    })
}

fn declaration_type_substitutions(
    declaration: &JavaTypeDeclaration,
    owner: &JavaType,
) -> Option<BTreeMap<JavaIdentifier, JavaType>> {
    match owner {
        JavaType::Generic { arguments, .. }
            if arguments.len() == declaration.type_parameters.len() =>
        {
            Some(
                declaration
                    .type_parameters
                    .iter()
                    .cloned()
                    .zip(arguments.iter().cloned())
                    .collect(),
            )
        }
        JavaType::Reference(_) if declaration.type_parameters.is_empty() => Some(BTreeMap::new()),
        _ => None,
    }
}

fn substitute_type_variables(
    ty: &JavaType,
    substitutions: &BTreeMap<JavaIdentifier, JavaType>,
) -> JavaType {
    match ty {
        JavaType::TypeVariable(name) => substitutions
            .get(name)
            .cloned()
            .unwrap_or_else(|| ty.clone()),
        JavaType::Array {
            component,
            ownership,
        } => JavaType::Array {
            component: Box::new(substitute_type_variables(component, substitutions)),
            ownership: *ownership,
        },
        JavaType::Generic { raw, arguments } => JavaType::Generic {
            raw: raw.clone(),
            arguments: arguments
                .iter()
                .map(|argument| substitute_type_variables(argument, substitutions))
                .collect(),
        },
        JavaType::Wildcard {
            bound: Some((kind, bound)),
        } => JavaType::Wildcard {
            bound: Some((
                *kind,
                Box::new(substitute_type_variables(bound, substitutions)),
            )),
        },
        JavaType::Primitive(_)
        | JavaType::Boxed(_)
        | JavaType::Reference(_)
        | JavaType::Wildcard { bound: None } => ty.clone(),
    }
}

pub(super) fn find_type_declaration(
    owner: &JavaType,
    context: &TargetAstContext<'_, JavaDialect>,
) -> Option<JavaTypeDeclaration> {
    context.files().find_map(|file| {
        file.items().iter().find_map(|item| match item {
            JavaFileItem::Type { declaration, .. } => find_type_declaration_in(declaration, owner),
            JavaFileItem::RuntimeMembers { members, .. } => {
                members.iter().find_map(|member| match member {
                    JavaMember::NestedType(nested) => find_type_declaration_in(nested, owner),
                    _ => None,
                })
            }
        })
    })
}

fn find_type_declaration_in(
    declaration: &JavaTypeDeclaration,
    owner: &JavaType,
) -> Option<JavaTypeDeclaration> {
    if declaration_represents_type(declaration, owner) {
        return Some(declaration.clone());
    }
    declaration.members.iter().find_map(|member| match member {
        JavaMember::NestedType(nested) => find_type_declaration_in(nested, owner),
        _ => None,
    })
}

fn declaration_represents_type(declaration: &JavaTypeDeclaration, owner: &JavaType) -> bool {
    match owner {
        JavaType::Reference(JavaTypeName::Generated(id))
        | JavaType::Generic {
            raw: JavaTypeName::Generated(id),
            ..
        } => declaration.declared == Some(*id),
        JavaType::Reference(JavaTypeName::Known(known))
        | JavaType::Generic {
            raw: JavaTypeName::Known(known),
            ..
        } if known.runtime_helper().is_some() => known.simple_name() == declaration.name.as_str(),
        _ => false,
    }
}
