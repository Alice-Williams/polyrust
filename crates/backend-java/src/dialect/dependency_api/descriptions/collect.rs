use super::{JavaSourceDescription, JavaSourceDescriptionKind, JavaSourceTarget};
use crate::{
    ast::{
        JavaDeclaredPath, JavaFileItem, JavaMember, JavaMethodDeclaration,
        JavaRecordComponentOrigin, JavaResolvedName, ResolvedJavaFileItem,
    },
    dialect::JavaDependencyApi,
};
use portable_codegen::{GeneratedOrigin, GeneratedSymbolId, RustSourceOrigin, TargetSymbolRef};
use std::collections::BTreeMap;

pub(crate) fn collect<'a>(
    api: &'a JavaDependencyApi,
) -> Result<Vec<JavaSourceDescription<'a>>, String> {
    let [file] = api.package().ast().files() else {
        return Err("source descriptions require one owner file".into());
    };
    let [item] = file.items() else {
        return Err("source descriptions require one owner facade".into());
    };
    let JavaFileItem::Type { declaration, .. } = &item.item else {
        return Err("source descriptions require a facade".into());
    };
    let mut descriptions = BTreeMap::new();
    let mut insert = |description: JavaSourceDescription<'a>| {
        if descriptions
            .insert(description.source.declaration, description)
            .is_some()
        {
            Err("duplicate Java source description identity".to_owned())
        } else {
            Ok(())
        }
    };
    for member in &declaration.members {
        match member {
            JavaMember::Constructor(_) => {} // Synthetic private facade constructor.
            JavaMember::Method(method) => {
                let JavaMethodDeclaration::Callable(id) = method.declared else {
                    return Err("source method description has no callable identity".into());
                };
                let symbol = GeneratedSymbolId::Callable(id);
                let source = origin(item, symbol)?;
                insert(JavaSourceDescription {
                    source,
                    target: JavaSourceTarget::Declaration(path(item, symbol)?),
                    kind: JavaSourceDescriptionKind::Function {
                        parameters: &method.parameters,
                        result: &method.return_type,
                    },
                })?;
            }
            JavaMember::Field(field) => {
                let symbol = GeneratedSymbolId::Value(
                    field.declared.ok_or("source constant identity missing")?,
                );
                let source = origin(item, symbol)?;
                let constant = api
                    .constant(source.declaration)
                    .ok_or("source constant inventory missing")?;
                insert(JavaSourceDescription {
                    source,
                    target: JavaSourceTarget::Declaration(path(item, symbol)?),
                    kind: JavaSourceDescriptionKind::Constant {
                        ty: &field.ty,
                        value: constant.value(),
                    },
                })?;
            }
            JavaMember::NestedType(record) => {
                if record
                    .declared
                    .is_some_and(|id| api.package_identity().is_result_type(id))
                {
                    continue; // Synthesized transport declarations are not Rust source identities.
                }
                let symbol = GeneratedSymbolId::Type(
                    record.declared.ok_or("source record identity missing")?,
                );
                let source = origin(item, symbol)?;
                let target = path(item, symbol)?;
                insert(JavaSourceDescription {
                    source,
                    target: JavaSourceTarget::Declaration(target),
                    kind: JavaSourceDescriptionKind::Record,
                })?;
                for component in &record.record_components {
                    let JavaRecordComponentOrigin::RustSource(field) = &component.origin else {
                        return Err("source field description has no origin".into());
                    };
                    insert(JavaSourceDescription {
                        source: &field.origin,
                        target: JavaSourceTarget::Field {
                            owner: target,
                            member: &component.name,
                        },
                        kind: JavaSourceDescriptionKind::Field {
                            owner: source.declaration,
                            ty: &component.ty,
                        },
                    })?;
                }
            }
            _ => return Err("unsupported Java source description member".into()),
        }
    }
    Ok(descriptions.into_values().collect())
}

fn origin(
    item: &ResolvedJavaFileItem,
    symbol: GeneratedSymbolId,
) -> Result<&RustSourceOrigin, String> {
    let value = item
        .source_inventory
        .get(symbol)
        .ok_or("source description registration missing")?;
    match value.origin() {
        GeneratedOrigin::RustSource(origin) => Ok(origin),
        _ => Err("source description has a non-source origin".into()),
    }
}
fn path(
    item: &ResolvedJavaFileItem,
    symbol: GeneratedSymbolId,
) -> Result<&JavaDeclaredPath, String> {
    match item.names.get(&TargetSymbolRef::Generated(symbol)) {
        Some(JavaResolvedName::DeclaredPath(path)) => Ok(path),
        _ => Err("source description has no resolved declaration path".into()),
    }
}
