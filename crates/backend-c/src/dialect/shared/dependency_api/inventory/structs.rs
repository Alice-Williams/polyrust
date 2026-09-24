//! Collect only actual owned public declarations and their certified spellings.
use super::super::structs::StructExport;
use crate::dialect::shared::bindings::CValueBinding;
use crate::{ast::*, dialect::CDialect};
use portable_codegen::RenderReadyPackage;
use std::collections::{BTreeMap, BTreeSet};

pub(in super::super) fn collect(
    package: &RenderReadyPackage<CDialect>,
) -> Result<BTreeMap<CStructRef, StructExport>, String> {
    let mut exports = BTreeMap::new();
    let mut tags = BTreeSet::new();
    for file in package.ast().files() {
        if file.module().key().role != CFileRole::GeneratedPublicHeader {
            continue;
        }
        for unit in file.items() {
            let registry = unit.unit.projection.registry.registrations();
            for item in unit.unit.data.source.items() {
                let CFileItem::Declaration(declaration) = item else {
                    continue;
                };
                let CDeclarationKind::Aggregate {
                    owner: CAggregateRef::Struct(record),
                    members,
                } = declaration.kind()
                else {
                    continue;
                };
                if record.file() != file.module()
                    || !crate::ownership::value_transport::scalar_result(
                        Some(registry),
                        &CObjectType::structure(record.clone()),
                    )
                {
                    return Err(
                        "C dependency struct lacks its exact owned scalar-result layout".into(),
                    );
                }
                let symbol = unit
                    .spelling
                    .types
                    .get(record)
                    .ok_or("C dependency struct lacks a certified type spelling")?
                    .clone();
                if !tags.insert(symbol.clone()) {
                    return Err("C dependency public tag names collide".into());
                }
                let names = members
                    .iter()
                    .map(|member| {
                        unit.spelling
                            .values
                            .get(&CValueBinding::Member(member.clone()))
                            .cloned()
                            .map(|name| (member.clone(), name))
                            .ok_or_else(|| {
                                "C dependency struct lacks a certified member spelling".to_owned()
                            })
                    })
                    .collect::<Result<_, String>>()?;
                let export = StructExport {
                    record: record.clone(),
                    symbol,
                    members: members.clone(),
                    names,
                };
                if exports.insert(record.clone(), export).is_some() {
                    return Err("C dependency struct is defined more than once".into());
                }
            }
        }
    }
    Ok(exports)
}

/// A relay republishes only original foreign types required by its public ABI.
/// Its complete registry still retains private dependencies for safety checks.
pub(super) fn imported_for_signatures<'a>(
    registry: &CRegistry,
    functions: impl Iterator<Item = &'a CFunctionRef>,
    owned: &BTreeMap<CStructRef, StructExport>,
) -> Result<Vec<super::super::CDependencyStruct>, String> {
    let mut required = BTreeSet::new();
    for function in functions {
        let result = match function.signature().return_type() {
            CReturnType::Void => None,
            CReturnType::Value(value) => Some(value.declared_type()),
        };
        for ty in result.into_iter().chain(
            function
                .signature()
                .parameters()
                .iter()
                .map(|p| p.declared_type()),
        ) {
            if let CObjectTypeKind::Struct(record) = ty.kind()
                && !owned.contains_key(record)
            {
                required.insert(record.clone());
            }
        }
    }
    required
        .into_iter()
        .map(|record| {
            registry
                .imported_struct(&record)
                .cloned()
                .map_err(|error| error.to_string())
        })
        .collect()
}
