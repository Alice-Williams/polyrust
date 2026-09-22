//! Source descriptions are reconciled with exact certified target declarations.
use super::{JavaDependencyApi, JavaDialect, JavaSourceDescriptionKind};
use crate::ast::{JavaFileItem, JavaPrimitive, JavaType};
use portable_codegen::{RenderReadyPackage, RustResultKind, RustScalarKind, RustSourceTypes};
use std::sync::Arc;

pub(super) fn read(package: &RenderReadyPackage<JavaDialect>) -> Option<Arc<RustSourceTypes>> {
    package
        .ast()
        .files()
        .iter()
        .flat_map(|file| file.items())
        .find_map(|item| match &item.item {
            JavaFileItem::Type {
                source_package: Some(source),
                ..
            } => source.source_types().cloned(),
            _ => None,
        })
}

fn scalar(kind: RustScalarKind) -> JavaType {
    JavaType::primitive(match kind {
        RustScalarKind::I32 | RustScalarKind::Char => JavaPrimitive::Int,
        RustScalarKind::I64 => JavaPrimitive::Long,
        RustScalarKind::Bool => JavaPrimitive::Boolean,
        RustScalarKind::F64 => JavaPrimitive::Double,
    })
}

pub(super) fn verify(api: &JavaDependencyApi) -> Result<(), String> {
    let Some(types) = api.source_types() else {
        return Ok(());
    };
    verify_inventory(api.root(), &api.source_descriptions()?, types)
}

fn verify_inventory(
    root: portable_codegen::RustDeclarationId,
    descriptions: &[super::JavaSourceDescription<'_>],
    types: &RustSourceTypes,
) -> Result<(), String> {
    if types.root() != root {
        return Err("Java source type facts belong to another owner".into());
    }
    let (mut functions, mut fields) = (0, 0);
    for description in descriptions {
        let id = description.source().declaration;
        match description.kind() {
            JavaSourceDescriptionKind::Function { parameters, result } => {
                let original = types
                    .functions()
                    .get(&id)
                    .ok_or("Java source function type facts are missing")?;
                let expected = match original.result {
                    RustResultKind::Unit => JavaType::primitive(JavaPrimitive::Void),
                    RustResultKind::Scalar(kind) => scalar(kind),
                };
                if &expected != result
                    || original.parameters.len() != parameters.len()
                    || !original
                        .parameters
                        .iter()
                        .zip(parameters)
                        .all(|(kind, parameter)| scalar(*kind) == parameter.ty)
                {
                    return Err(
                        "Java source function type facts disagree with target signature".into(),
                    );
                }
                functions += 1;
            }
            JavaSourceDescriptionKind::Field { ty, owner } => {
                let original = types
                    .fields()
                    .get(&id)
                    .ok_or("Java source field type facts are missing")?;
                if original.owner != owner || scalar(original.kind) != *ty {
                    return Err(
                        "Java source field type facts disagree with target representation".into(),
                    );
                }
                fields += 1;
            }
            JavaSourceDescriptionKind::Record | JavaSourceDescriptionKind::Constant { .. } => {}
        }
    }
    if functions != types.functions().len() || fields != types.fields().len() {
        return Err("Java source type facts include unknown declarations".into());
    }
    Ok(())
}

#[cfg(test)]
#[path = "../../tests/source_type_facts.rs"]
mod tests;
