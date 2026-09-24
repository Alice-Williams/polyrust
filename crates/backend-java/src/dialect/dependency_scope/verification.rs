//! Original-package scope reconciliation, not a lookup by target spelling.
use super::{JavaDependencyBindings, JavaImportedCallable};
use crate::{
    ast::JavaFileItem,
    dialect::{JavaDialect, JavaQualifiedName},
};
use portable_codegen::{
    DependencyCallableSpec, DependencySpelling, GeneratedOrigin, GeneratedSymbolId, LinkerDialect,
    SymbolCatalogue, TargetAstPackage, TargetSymbolRef,
};
use portable_diagnostics::{Diagnostic, DiagnosticCode, SourceRef};
use std::collections::{BTreeMap, BTreeSet};

pub(super) const MAX_OWNERS: usize = 1024;

impl JavaImportedCallable {
    pub(in crate::dialect) fn spec(&self) -> DependencyCallableSpec<JavaDialect> {
        DependencyCallableSpec {
            symbol: self.clone(),
            owner: self.function.package_identity().clone(),
            name: self.function.path().member().clone(),
            signature: JavaDialect.coarse_signature(self.signature()),
            spelling: DependencySpelling::Qualified(JavaQualifiedName::Dependency(
                self.function.path().clone(),
            )),
            source: SourceRef::logical([
                "java",
                "certified-dependency",
                self.function.path().member().as_str(),
            ]),
        }
    }
}

fn error(message: &str) -> Vec<Diagnostic> {
    vec![Diagnostic::error(
        DiagnosticCode::InterfaceNonconformance,
        message,
        SourceRef::logical(["java", "dependency-scope"]),
    )]
}

pub(in crate::dialect) fn catalogue(
    package: &TargetAstPackage<JavaDialect>,
) -> Result<SymbolCatalogue<JavaDialect>, Vec<Diagnostic>> {
    let mut scope: Option<&JavaDependencyBindings> = None;
    let mut owned = BTreeSet::new();
    for file in package.files() {
        for item in file.items() {
            let JavaFileItem::Type {
                declared,
                dependencies,
                ..
            } = item
            else {
                continue;
            };
            if dependencies.0.is_some() {
                if scope.is_some_and(|previous| previous != dependencies) {
                    return Err(error(
                        "Java package mixes independently frozen dependency scopes",
                    ));
                }
                scope = Some(dependencies);
            }
            for symbol in item.symbols() {
                if let TargetSymbolRef::DependencyCallable(callable) = symbol
                    && !dependencies.contains(&callable)
                {
                    return Err(error(
                        "Java dependency call is absent from its original consumer scope",
                    ));
                }
            }
            for value in item.symbols() {
                match &value {
                    TargetSymbolRef::KnownConstructor(
                        crate::dialect::JavaReferencedConstructor::Dependency(value),
                    ) if !dependencies.contains_result_constructor(value) => {
                        return Err(error(
                            "Java dependency constructor is absent from its original consumer scope",
                        ));
                    }
                    TargetSymbolRef::KnownMethod(
                        crate::dialect::JavaReferencedMethod::Dependency(value),
                    ) if !dependencies.contains_result_accessor(value) => {
                        return Err(error(
                            "Java dependency accessor is absent from its original consumer scope",
                        ));
                    }
                    _ => {}
                }
                if let TargetSymbolRef::KnownType(crate::dialect::JavaReferencedType::Dependency(
                    ref ty,
                )) = value
                    && !dependencies.contains_result_type(ty)
                {
                    return Err(error(
                        "Java dependency type is absent from its original consumer scope",
                    ));
                }
                if let TargetSymbolRef::DependencyValue(value) = value
                    && !dependencies.contains_value(&value)
                {
                    return Err(error(
                        "Java dependency value is absent from its original consumer scope",
                    ));
                }
            }
            if let crate::ast::JavaPackage::RustCrate(id) = file.module() {
                owned.insert(*id);
            }
            for symbol in declared {
                let origin = match symbol {
                    GeneratedSymbolId::Type(id) => {
                        package.generated_type(*id).map(|value| &value.origin)
                    }
                    GeneratedSymbolId::Callable(id) => {
                        package.callable(*id).map(|value| &value.origin)
                    }
                    GeneratedSymbolId::InterfaceMethod(id) => {
                        package.interface_method(*id).map(|value| &value.origin)
                    }
                    GeneratedSymbolId::Value(id) => package.value(*id).map(|value| &value.origin),
                };
                if let Some(GeneratedOrigin::RustSource(origin)) = origin {
                    owned.insert(origin.declaration.crate_id);
                    owned.insert(origin.crate_exports.root.crate_id);
                }
            }
        }
    }
    let mut catalogue = JavaDialect.symbol_catalogue();
    let mut pending = Vec::new();
    if let Some(scope) = scope {
        catalogue
            .constructors
            .extend(scope.result_constructors().map(|value| value.spec()));
        catalogue
            .methods
            .extend(scope.result_accessors().map(|value| value.spec()));
        for ty in scope.result_types() {
            pending.push(ty.original().family().package_identity());
            catalogue.types.push(ty.spec());
        }
        for value in scope.values() {
            pending.push(value.constant().package_identity());
            catalogue.dependency_values.push(value.spec());
        }
        for callable in scope.functions() {
            let owner = callable.function.package_identity();
            pending.push(owner);
            catalogue.dependency_callables.push(callable.spec());
        }
    }
    verify_owners(pending, &owned, MAX_OWNERS)?;
    Ok(catalogue)
}

pub(super) fn verify_owners(
    mut pending: Vec<&crate::dialect::JavaDependencyPackage>,
    owned: &BTreeSet<u64>,
    max_owners: usize,
) -> Result<(), Vec<Diagnostic>> {
    let mut owners = BTreeMap::new();
    while let Some(owner) = pending.pop() {
        let crate_id = owner.root().crate_id;
        if owned.contains(&crate_id) {
            return Err(error(
                "Java consumer crate overlaps a certified dependency owner",
            ));
        }
        if let Some(previous) = owners.insert(crate_id, owner) {
            if previous != owner {
                return Err(error(
                    "Java dependency source crate has conflicting owner certificates",
                ));
            }
            continue;
        }
        if owners.len() > max_owners {
            return Err(vec![Diagnostic::error(
                DiagnosticCode::TargetResourceLimit,
                format!("Java dependency closure exceeds {max_owners} owners"),
                SourceRef::logical(["java", "dependency-scope"]),
            )]);
        }
        pending.extend(owner.dependencies());
    }
    Ok(())
}
