//! Compare compiler-emitted class metrics with reservations for the same AST.

use crate::ast::{JavaFileItem, JavaMember};
use crate::dialect::JavaDialect;
use crate::resources::budget::{self, ClassBudget};
use portable_codegen::LinkedTargetPackage;
use std::path::Path;

pub(crate) fn collect(package: &LinkedTargetPackage<JavaDialect>) -> Vec<ClassBudget> {
    crate::resources::resolved(package).expect("oracle package must fit the admission policy");
    let mut result = Vec::new();
    let count = package
        .files()
        .iter()
        .flat_map(|file| file.items())
        .fold(0usize, |sum, item| {
            sum.saturating_add(match &item.item {
                JavaFileItem::Type { declaration, .. } => budget::declaration_count(declaration),
                JavaFileItem::RuntimeMembers { members, .. } => members
                    .iter()
                    .filter_map(|member| {
                        if let JavaMember::NestedType(child) = member {
                            Some(budget::declaration_count(child))
                        } else {
                            None
                        }
                    })
                    .sum(),
            })
        });
    for file in package.files() {
        for item in file.items() {
            if let JavaFileItem::Type { declaration, .. } = &item.item {
                let mut members = declaration.members.iter().collect::<Vec<_>>();
                for extra in file.items() {
                    if let JavaFileItem::RuntimeMembers { members: extra, .. } = &extra.item {
                        members.extend(extra);
                    }
                }
                result.extend(budget::report(declaration, &members, count));
            }
        }
    }
    result
}

pub(crate) fn verify(classes: &Path, budgets: &[ClassBudget]) {
    let directory = classes.join("org/polyrust/generated");
    let mut checked = 0usize;
    for entry in std::fs::read_dir(directory).expect("read compiler output") {
        let path = entry.unwrap().path();
        if path
            .extension()
            .is_none_or(|extension| extension != "class")
        {
            continue;
        }
        let name = path.file_stem().unwrap().to_str().unwrap();
        let budget = budgets
            .iter()
            .find(|budget| budget.name == name)
            .or_else(|| {
                let (owner, suffix) = name.rsplit_once('$')?;
                if suffix.parse::<usize>().is_err() {
                    return None;
                }
                budgets
                    .iter()
                    .find(|budget| budget.name == format!("{owner}$enum-switch-helper"))
            })
            .unwrap_or_else(|| panic!("unaccounted compiler-generated class {name}"));
        let actual = super::classfile_metrics::read(&path);
        assert!(
            actual.pool <= budget.pool,
            "{name}: pool {actual:?} > {budget:?}"
        );
        assert!(
            actual.fields <= budget.fields,
            "{name}: fields {actual:?} > {budget:?}"
        );
        assert!(
            actual.methods.len() <= budget.methods.len(),
            "{name}: method count"
        );
        assert!(
            actual.metadata <= budget.metadata,
            "{name}: inner/nest metadata"
        );
        assert!(
            actual.bootstraps <= budget.methods.iter().map(|method| method.bootstraps).sum(),
            "{name}: bootstraps"
        );
        assert!(
            actual.bootstrap_arguments
                <= budget
                    .methods
                    .iter()
                    .map(|method| method.bootstrap_arguments)
                    .max()
                    .unwrap_or(0),
            "{name}: bootstrap arguments"
        );
        // Overloads share a named-method envelope; unrelated large methods
        // cannot mask an underestimated constructor, accessor or helper.
        for method in &actual.methods {
            let bounds = budget
                .methods
                .iter()
                .filter(|bound| bound.name == method.name)
                .collect::<Vec<_>>();
            assert!(
                !bounds.is_empty(),
                "{name}: unaccounted method {}",
                method.name
            );
            assert!(
                method.code <= bounds.iter().map(|m| m.bytes).max().unwrap_or(0),
                "{name}: code {method:?}"
            );
            assert!(
                method.locals <= bounds.iter().map(|m| m.locals).max().unwrap_or(0),
                "{name}: locals {method:?}"
            );
            assert!(
                method.stack <= bounds.iter().map(|m| m.stack).max().unwrap_or(0),
                "{name}: stack {method:?}"
            );
            assert!(
                method.exceptions <= bounds.iter().map(|m| m.exceptions).max().unwrap_or(0),
                "{name}: exceptions {method:?}"
            );
            assert!(
                method.frames <= bounds.iter().map(|m| m.bytes).max().unwrap_or(0),
                "{name}: frames {method:?}"
            );
        }
        checked += 1;
    }
    assert!(checked > 0, "native budget corpus unexpectedly empty");
}

pub(crate) fn verify_manifest(
    manifest: &portable_codegen::OutputManifest,
    budgets: &[ClassBudget],
) {
    let package = super::totality_oracle::CompiledPackage::new(manifest, "runtime-budget");
    package.compile_harnesses(manifest);
    package.verify_budgets(budgets);
}
