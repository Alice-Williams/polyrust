//! Public layouts derived from a certificate, not standalone owner authority.
use super::super::{JavaDialect, JavaScalarResultFamily, JavaScalarResultTypes};
use crate::ast::{
    JavaDeclaredPath, JavaFileItem, JavaIdentifier, JavaResolvedName, JavaSourceDeclaration,
    JavaVisibility,
};
use portable_codegen::{GeneratedSymbolId, GeneratedTypeId, RenderReadyPackage, TargetSymbolRef};
use std::{collections::BTreeMap, sync::Arc};

#[cfg(test)]
#[path = "../../tests/result_export_layout.rs"]
mod tests;

#[derive(Clone, Debug)]
pub(super) struct ResultLayout {
    pub types: JavaScalarResultTypes,
    pub paths: [JavaDeclaredPath; 3],
    pub payload_name: JavaIdentifier,
}

pub(super) fn collect(
    package: &RenderReadyPackage<JavaDialect>,
    selections: &[JavaScalarResultTypes],
) -> Result<BTreeMap<GeneratedTypeId, Arc<ResultLayout>>, String> {
    collect_with_limits(package, selections, 1024, 100_000)
}

fn collect_with_limits(
    package: &RenderReadyPackage<JavaDialect>,
    selections: &[JavaScalarResultTypes],
    max_families: usize,
    max_search_steps: usize,
) -> Result<BTreeMap<GeneratedTypeId, Arc<ResultLayout>>, String> {
    // Bound work before scanning selection products.
    if selections.len() > max_families {
        return Err("Java result-family selection limit exceeded".into());
    }
    if selections.is_empty() {
        return Ok(BTreeMap::new());
    }
    // Three family lookups and three path lookups per selection. Charge all
    // direct members/items conservatively before the repeated searches.
    let direct_members = package
        .ast()
        .files()
        .iter()
        .flat_map(|file| file.items())
        .fold(0usize, |count, item| {
            count.saturating_add(match &item.item {
                JavaFileItem::Type { declaration, .. } => {
                    declaration.members.len().saturating_add(1)
                }
                _ => 1,
            })
        });
    if selections
        .len()
        .saturating_mul(6)
        .saturating_mul(direct_members)
        > max_search_steps
    {
        return Err("Java result-family declaration search limit exceeded".into());
    }
    let mut families = BTreeMap::new();
    let mut members = std::collections::BTreeSet::new();
    for types in selections {
        let payload_name = JavaScalarResultFamily::checked_payload_name(package, *types)?;
        let ids = [types.interface, types.success, types.error];
        if ids.iter().any(|id| !members.insert(*id)) {
            return Err("Java result-family selections overlap or repeat".into());
        }
        let mut paths = Vec::new();
        for id in ids {
            let symbol = GeneratedSymbolId::Type(id);
            let mut found = None;
            for item in package.ast().files().iter().flat_map(|file| file.items()) {
                let JavaFileItem::Type { declaration, .. } = &item.item else {
                    continue;
                };
                let Some(JavaSourceDeclaration::Type(registration)) =
                    item.source_inventory.get(symbol)
                else {
                    continue;
                };
                if registration.visibility != JavaVisibility::Public
                    || declaration.visibility != JavaVisibility::Public
                {
                    return Err(
                        "Java result-family export requires public enclosing types and variants"
                            .into(),
                    );
                }
                let Some(JavaResolvedName::DeclaredPath(path)) =
                    item.names.get(&TargetSymbolRef::Generated(symbol))
                else {
                    return Err(
                        "Java result-family export lacks its original resolved type path".into(),
                    );
                };
                if found.replace(path.clone()).is_some() {
                    return Err("Java result-family export repeats a declaration".into());
                }
            }
            paths.push(
                found.ok_or("Java result-family export lacks original declaration metadata")?,
            );
        }
        let paths: [JavaDeclaredPath; 3] = paths.try_into().expect("three selected identities");
        families.insert(
            types.interface,
            Arc::new(ResultLayout {
                types: *types,
                paths,
                payload_name,
            }),
        );
    }
    Ok(families)
}
