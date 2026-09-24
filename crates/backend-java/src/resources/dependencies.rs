//! Bounded frozen registrations and dynamic qualified dependency spellings.
use crate::{
    ast::{JavaFileItem, JavaResolvedName},
    dialect::{JavaDialect, JavaQualifiedName},
};
use portable_codegen::{LinkedTargetPackage, TargetSymbolRef};
use portable_diagnostics::Diagnostic;
use std::collections::BTreeSet;

#[cfg(test)]
#[path = "../tests/result_import_resources.rs"]
mod result_tests;
#[cfg(test)]
#[path = "../tests/dependency_resources.rs"]
mod tests;
#[cfg(test)]
#[path = "../tests/dependency_value_resources.rs"]
mod value_tests;

struct Limits {
    bindings: usize,
    owners: usize,
    names: usize,
    name: usize,
}
const LIMITS: Limits = Limits {
    bindings: 100_000,
    owners: 1024,
    names: 64 * 1024 * 1024,
    name: super::types::MAX_UTF8,
};

pub(super) fn verify(package: &LinkedTargetPackage<JavaDialect>) -> Vec<Diagnostic> {
    check(package, &LIMITS)
}

fn check(package: &LinkedTargetPackage<JavaDialect>, limits: &Limits) -> Vec<Diagnostic> {
    let mut errors = Vec::new();
    let mut bindings = BTreeSet::new();
    let mut owners = BTreeSet::new();
    let mut names = 0usize;
    for file in package.files() {
        for item in file.items() {
            if let JavaFileItem::Type { dependencies, .. } = &item.item {
                let references = dependencies
                    .functions()
                    .map(|callable| {
                        (
                            TargetSymbolRef::<JavaDialect>::DependencyCallable(callable.clone()),
                            callable.function().package_identity(),
                            callable.function().path(),
                        )
                    })
                    .chain(dependencies.values().map(|value| {
                        (
                            TargetSymbolRef::DependencyValue(value.clone()),
                            value.constant().package_identity(),
                            value.constant().path(),
                        )
                    }))
                    .chain(dependencies.result_constructors().map(|value| {
                        (
                            TargetSymbolRef::KnownConstructor(value.clone().into()),
                            value.owner().original().family().package_identity(),
                            value.owner().path(),
                        )
                    }))
                    .chain(dependencies.result_accessors().map(|value| {
                        (
                            TargetSymbolRef::KnownMethod(value.clone().into()),
                            value.owner().original().family().package_identity(),
                            value.path(),
                        )
                    }))
                    .chain(dependencies.result_types().map(|ty| {
                        (
                            TargetSymbolRef::KnownType(ty.clone().into()),
                            ty.original().family().package_identity(),
                            ty.path(),
                        )
                    }));
                for (symbol, owner, path) in references {
                    if bindings.insert(symbol) {
                        let length = path.encoded_len();
                        names = names.saturating_add(length);
                        owners.insert(owner);
                        super::limit(
                            &mut errors,
                            file.path().as_str(),
                            "dependency qualified-name bytes",
                            length,
                            limits.name,
                        );
                    }
                }
            }
            for spelling in item.names.values() {
                if let JavaResolvedName::Qualified(JavaQualifiedName::Dependency(path)) = spelling {
                    super::limit(
                        &mut errors,
                        file.path().as_str(),
                        "resolved dependency qualified-name bytes",
                        path.encoded_len(),
                        limits.name,
                    );
                }
            }
        }
    }
    super::limit(
        &mut errors,
        "java-dependencies",
        "registered dependency bindings",
        bindings.len(),
        limits.bindings,
    );
    super::limit(
        &mut errors,
        "java-dependencies",
        "registered dependency owners",
        owners.len(),
        limits.owners,
    );
    super::limit(
        &mut errors,
        "java-dependencies",
        "dependency name bytes",
        names,
        limits.names,
    );
    errors
}
