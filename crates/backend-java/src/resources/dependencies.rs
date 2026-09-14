//! Bounded frozen registrations and dynamic qualified dependency spellings.
use crate::{
    ast::{JavaFileItem, JavaResolvedName},
    dialect::{JavaDialect, JavaQualifiedName},
};
use portable_codegen::LinkedTargetPackage;
use portable_diagnostics::Diagnostic;
use std::collections::BTreeSet;

#[cfg(test)]
#[path = "../tests/dependency_resources.rs"]
mod tests;

struct Limits {
    functions: usize,
    owners: usize,
    names: usize,
    name: usize,
}
const LIMITS: Limits = Limits {
    functions: 100_000,
    owners: 1024,
    names: 64 * 1024 * 1024,
    name: super::types::MAX_UTF8,
};

pub(super) fn verify(package: &LinkedTargetPackage<JavaDialect>) -> Vec<Diagnostic> {
    check(package, &LIMITS)
}

fn check(package: &LinkedTargetPackage<JavaDialect>, limits: &Limits) -> Vec<Diagnostic> {
    let mut errors = Vec::new();
    let mut functions = BTreeSet::new();
    let mut owners = BTreeSet::new();
    let mut names = 0usize;
    for file in package.files() {
        for item in file.items() {
            if let JavaFileItem::Type { dependencies, .. } = &item.item {
                for callable in dependencies.functions() {
                    if functions.insert(callable) {
                        let path = callable.function().path();
                        let length = path.encoded_len();
                        names = names.saturating_add(length);
                        owners.insert(callable.function().package_identity());
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
        "registered dependency functions",
        functions.len(),
        limits.functions,
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
