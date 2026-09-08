//! Target capacity is a checked boundary, not a portable typing failure.

mod declarations;
mod executables;
mod types;

#[cfg(test)]
#[path = "tests/resources.rs"]
mod tests;

#[cfg(test)]
#[path = "tests/resource_declarations.rs"]
mod declaration_tests;

use crate::ast::{JavaFileItem, JavaMember, JavaPackage, JavaResolvedName, JavaTypeDeclaration};
use crate::dialect::JavaDialect;
use portable_codegen::{
    GeneratedSymbolId, LinkedTargetPackage, TargetSymbolRef, TypedGenerationError,
    TypedPipelineStage,
};
use portable_diagnostics::{Diagnostic, DiagnosticCode, SourceRef};
use std::collections::BTreeMap;

/// Java representation capacity failures, never syntax/capability diagnostics.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct JavaResourceError {
    diagnostics: Vec<Diagnostic>,
}

impl JavaResourceError {
    pub fn diagnostics(&self) -> &[Diagnostic] {
        &self.diagnostics
    }

    pub(crate) fn from_typed_failure(error: TypedGenerationError) -> Self {
        if let TypedGenerationError::Phase {
            stage: TypedPipelineStage::TargetResourceValidation | TypedPipelineStage::Rendering,
            diagnostics,
        } = &error
            && !diagnostics.is_empty()
            && diagnostics
                .iter()
                .all(|value| value.code == DiagnosticCode::TargetResourceLimit)
        {
            return Self {
                diagnostics: diagnostics.clone(),
            };
        }
        panic!("TypedProgram Java invariant failure: {error:#?}")
    }
}

impl std::fmt::Display for JavaResourceError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            formatter,
            "Java target capacity exceeded ({} diagnostics)",
            self.diagnostics.len()
        )
    }
}

impl std::error::Error for JavaResourceError {}

pub(crate) fn resolved(package: &LinkedTargetPackage<JavaDialect>) -> Result<(), Vec<Diagnostic>> {
    let mut names = Names::new();
    let mut local_names = Names::new();
    let mut errors = Vec::new();
    for file in package.files() {
        for item in file.items() {
            for (symbol, spelling) in &item.names {
                let (member, binary_length) = match spelling {
                    JavaResolvedName::Local(name) => (Some(name), None),
                    JavaResolvedName::DeclaredPath(path) => (
                        Some(&path.member),
                        Some(
                            path.owners
                                .iter()
                                .fold(path.package.name().len(), |size, owner| {
                                    size.saturating_add(1).saturating_add(owner.as_str().len())
                                })
                                .saturating_add(1)
                                .saturating_add(path.member.as_str().len()),
                        ),
                    ),
                    JavaResolvedName::GeneratedMember { owner, member } => (
                        Some(member),
                        Some(
                            owner
                                .text()
                                .len()
                                .saturating_add(1)
                                .saturating_add(member.as_str().len()),
                        ),
                    ),
                    JavaResolvedName::Qualified(_) | JavaResolvedName::Member { .. } => {
                        (None, None)
                    }
                };
                if let Some(member) = member {
                    limit(
                        &mut errors,
                        file.path().as_str(),
                        "resolved identifier bytes",
                        member.as_str().len(),
                        types::MAX_UTF8,
                    );
                }
                if let TargetSymbolRef::Generated(GeneratedSymbolId::Type(id)) = symbol {
                    if let Some(length) = binary_length {
                        names.insert(*id, length);
                    } else if let Some(member) = member {
                        local_names.insert(*id, member.as_str().len());
                    }
                }
            }
        }
    }
    check_with_names(
        package
            .files()
            .iter()
            .map(|file| {
                (
                    file.path().as_str(),
                    file.items().iter().map(|item| &item.item).collect(),
                )
            })
            .collect(),
        names,
        local_names,
        errors,
    )
}

#[cfg(test)]
fn check(files: Vec<(&str, Vec<&JavaFileItem>)>) -> Result<(), Vec<Diagnostic>> {
    check_with_names(files, Names::new(), Names::new(), Vec::new())
}

fn check_with_names(
    files: Vec<(&str, Vec<&JavaFileItem>)>,
    mut names: Names,
    local_names: Names,
    mut errors: Vec<Diagnostic>,
) -> Result<(), Vec<Diagnostic>> {
    let prefix = JavaPackage::Generated.name().len() + 1;
    for (_, items) in &files {
        for item in items {
            match item {
                JavaFileItem::Type { declaration, .. } => {
                    collect_names(declaration, prefix, &local_names, &mut names)
                }
                JavaFileItem::RuntimeMembers { members, .. } => {
                    for member in members {
                        if let JavaMember::NestedType(child) = member {
                            collect_names(
                                child,
                                prefix + "Runtime$".len(),
                                &local_names,
                                &mut names,
                            );
                        }
                    }
                }
            }
        }
    }
    for (path, items) in files {
        for item in &items {
            if let JavaFileItem::Type { declaration, .. } = item {
                // Helper members belong to the same Runtime class file, not
                // separate fragment budgets. Identity is checked by dialect.
                let mut members = declaration.members.iter().collect::<Vec<_>>();
                if declaration.name.as_str() == "Runtime" {
                    for item in &items {
                        if let JavaFileItem::RuntimeMembers { members: extra, .. } = item {
                            members.extend(extra);
                        }
                    }
                }
                let mut checker = declarations::Checker::new(&names, path, &mut errors);
                checker.declaration(declaration, &members, prefix, None);
            }
        }
    }
    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}

type Names = BTreeMap<portable_codegen::GeneratedTypeId, usize>;

fn collect_names(
    value: &JavaTypeDeclaration,
    prefix: usize,
    local_names: &Names,
    names: &mut Names,
) {
    let member_length = value
        .declared
        .and_then(|id| local_names.get(&id).copied())
        .unwrap_or_else(|| value.name.as_str().len());
    let length = value
        .declared
        .and_then(|id| names.get(&id).copied())
        .unwrap_or_else(|| prefix.saturating_add(member_length));
    if let Some(id) = value.declared {
        names.entry(id).or_insert(length);
    }
    for member in &value.members {
        if let JavaMember::NestedType(child) = member {
            collect_names(child, length.saturating_add(1), local_names, names);
        }
    }
}

fn limit(errors: &mut Vec<Diagnostic>, path: &str, kind: &str, actual: usize, maximum: usize) {
    if actual > maximum {
        let mut diagnostic = Diagnostic::error(
            DiagnosticCode::TargetResourceLimit,
            format!("Java {kind} requires {actual}; limit is {maximum}"),
            SourceRef::logical(["java-resource", path, kind]),
        );
        diagnostic.target = Some("org.polyrust.java".to_owned());
        errors.push(diagnostic);
    }
}
