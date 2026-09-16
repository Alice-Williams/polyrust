//! File-only references retain target ownership without inventing a symbol.
use super::{LinkerDialect, file_graph};
use crate::{RelativeOutputPath, TargetAstPackage, TargetFileId, TypedAstDialect};
use portable_diagnostics::{Diagnostic, DiagnosticCode};
use std::collections::{BTreeMap, BTreeSet};

/// Unresolved structural data, not a checked import or file authority.
/// Only shared linking/certification may turn this into a resolved directive.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TargetFileRequirement<D: TypedAstDialect> {
    module: D::ModuleDeclaration,
    path: RelativeOutputPath,
}

impl<D: TypedAstDialect> TargetFileRequirement<D> {
    pub fn new(module: D::ModuleDeclaration, path: RelativeOutputPath) -> Self {
        Self { module, path }
    }

    pub fn module(&self) -> &D::ModuleDeclaration {
        &self.module
    }
    pub fn path(&self) -> &RelativeOutputPath {
        &self.path
    }
}

const MAX_REQUESTS: usize = 100_000;

pub(super) fn derive<D: LinkerDialect>(
    dialect: &D,
    package: &TargetAstPackage<D>,
    diagnostics: &mut Vec<Diagnostic>,
) -> BTreeMap<TargetFileId, BTreeSet<TargetFileId>> {
    let mut catalogue = BTreeMap::new();
    for (index, file) in package.files().enumerate() {
        catalogue
            .entry(file.path())
            .and_modify(|slot| *slot = None)
            .or_insert(Some((TargetFileId::from_index(index), file)));
    }
    let mut requests = 0usize;
    let mut graph = BTreeMap::new();
    for (index, file) in package.files().enumerate() {
        let source = TargetFileId::from_index(index);
        let requirements = dialect.file_requirements(file);
        let Some(total) = requests
            .checked_add(requirements.len())
            .filter(|total| *total <= MAX_REQUESTS)
        else {
            diagnostics.push(Diagnostic::error(
                DiagnosticCode::TargetResourceLimit,
                "explicit file requirement budget exceeded",
                file.source().clone(),
            ));
            return graph;
        };
        requests = total;
        let mut destinations = BTreeSet::new();
        for requirement in requirements {
            let error = match catalogue.get(requirement.path()) {
                Some(Some((destination, target))) if target.module() == requirement.module() => {
                    if *destination == source {
                        Some("explicit file requirement cannot target itself")
                    } else if file_graph::violates_role(file.role(), target.role()) {
                        Some("explicit file requirement violates source-role visibility")
                    } else {
                        destinations.insert(*destination);
                        None
                    }
                }
                Some(Some(_)) => {
                    Some("explicit file requirement has a different target module owner")
                }
                Some(None) => Some("explicit file requirement has an ambiguous target path"),
                None => Some("explicit file requirement has no structural target file"),
            };
            if let Some(error) = error {
                diagnostics.push(Diagnostic::error(
                    DiagnosticCode::InvalidStructure,
                    error,
                    file.source().clone(),
                ));
            }
        }
        graph.insert(source, destinations);
    }
    graph
}
