//! Primary declaration placement differs from a file's referenced bindings.
use super::bindings::{CBindings, CValueBinding};
use crate::ast::{CAggregateRef, CDefinitionKind, CFileItem, CSourceFile};
use crate::dialect::{CFileDependencies, CTagDependency};
use portable_codegen::GeneratedSymbolId as Symbol;
use std::collections::{BTreeMap, BTreeSet};

#[cfg(test)]
#[path = "../../tests/shared_unit_bindings.rs"]
mod tests;

pub(super) fn project(
    bindings: &CBindings,
    sources: &[CSourceFile],
    dependencies: &CFileDependencies,
) -> Result<(CBindings, Vec<Symbol>), String> {
    let mut bodies = BTreeMap::new();
    let mut object_bodies = BTreeMap::new();
    for source in sources {
        for item in source.items() {
            if let CFileItem::Definition(definition) = item
                && let CDefinitionKind::Object { object, .. } = definition.kind()
                && object_bodies.insert(object, source.identity()).is_some()
            {
                return Err("C object has more than one defining source file".into());
            }
            if let CFileItem::Definition(definition) = item
                && let CDefinitionKind::Function { function, .. } = definition.kind()
                && bodies.insert(function, source.identity()).is_some()
            {
                return Err("C function has more than one defining source file".into());
            }
        }
    }
    let file = dependencies.file();
    let mut declarations = BTreeSet::new();
    for (record, id) in &bindings.types {
        if record.file() == file {
            declarations.insert(Symbol::Type(*id));
        }
    }
    for (function, id) in &bindings.functions {
        if function.file() == file {
            declarations.insert(Symbol::Callable(*id));
        }
    }
    for (value, id) in &bindings.values {
        let owner = match value {
            CValueBinding::Global(object) => object.file(),
            CValueBinding::Member(member) => match member.owner() {
                CAggregateRef::Struct(record) => record.file(),
                CAggregateRef::Union(record) => record.file(),
            },
            CValueBinding::Parameter(parameter) => *bodies
                .get(parameter.function())
                .ok_or("C parameter has no defining source body")?,
            CValueBinding::Local(local) => *bodies
                .get(local.scope().function())
                .ok_or("C local has no defining source body")?,
        };
        if owner == file {
            declarations.insert(Symbol::Value(*id));
        }
    }
    let mut used = declarations.clone();
    for tag in dependencies.tags().keys() {
        let CTagDependency::Struct(record) = tag else {
            return Err("C unit references an unsupported aggregate category".into());
        };
        used.insert(Symbol::Type(
            *bindings
                .types
                .get(record)
                .ok_or("C used record has no binding")?,
        ));
    }
    for function in dependencies.functions().iter().chain(
        bodies
            .iter()
            .filter(|(_, owner)| **owner == file)
            .map(|(function, _)| *function),
    ) {
        if bindings.imports.contains_key(function) {
            continue;
        }
        used.insert(Symbol::Callable(
            *bindings
                .functions
                .get(function)
                .ok_or("C used function has no binding")?,
        ));
    }
    for object in dependencies.objects().iter().chain(
        object_bodies
            .iter()
            .filter(|(_, owner)| **owner == file)
            .map(|(object, _)| *object),
    ) {
        if bindings.imported_values.contains_key(object) {
            continue;
        }
        used.insert(Symbol::Value(
            *bindings
                .values
                .get(&CValueBinding::Global(object.clone()))
                .ok_or("C used global object has no binding")?,
        ));
    }
    for member in dependencies.members() {
        used.insert(Symbol::Value(
            *bindings
                .values
                .get(&CValueBinding::Member(member.clone()))
                .ok_or("C used member has no exact registered binding")?,
        ));
    }
    let mut selected = select(bindings, &used)?;
    for function in dependencies.functions() {
        if let Some(import) = bindings.imports.get(function) {
            selected.imports.insert(function.clone(), import.clone());
        }
    }
    for object in dependencies.objects() {
        if let Some(import) = bindings.imported_values.get(object) {
            selected
                .imported_values
                .insert(object.clone(), import.clone());
        }
    }
    Ok((selected, declarations.into_iter().collect()))
}

fn select(bindings: &CBindings, used: &BTreeSet<Symbol>) -> Result<CBindings, String> {
    let mut selected = CBindings::default();
    for symbol in used {
        match symbol {
            Symbol::Type(id) => {
                let record = bindings
                    .reverse_types
                    .get(id)
                    .ok_or("C type binding has no registered owner")?;
                selected.types.insert(record.clone(), *id);
                selected.reverse_types.insert(*id, record.clone());
            }
            Symbol::Callable(id) => {
                let function = bindings
                    .reverse_functions
                    .get(id)
                    .ok_or("C callable binding has no registered owner")?;
                selected.functions.insert(function.clone(), *id);
                selected.reverse_functions.insert(*id, function.clone());
            }
            Symbol::Value(id) => {
                let value = bindings
                    .reverse_values
                    .get(id)
                    .ok_or("C value binding has no registered owner")?;
                selected.values.insert(value.clone(), *id);
                selected.reverse_values.insert(*id, value.clone());
            }
            Symbol::InterfaceMethod(_) => {
                return Err("C profile has no interface-method binding".into());
            }
        }
    }
    Ok(selected)
}
