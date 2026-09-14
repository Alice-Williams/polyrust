//! Reference-derived file directives, independent of symbol import bindings.

use super::{LinkedFile, LinkedTargetPackage, LinkerDialect};
use crate::{TargetAstPackage, TargetFile, TargetFileId};
use portable_diagnostics::{Diagnostic, DiagnosticCode};

/// A directive resolved from a checked generated-file dependency.
///
/// Only the shared linker constructs these witnesses. A file directive does
/// not create a second name binding for any of the destination's symbols.
///
/// ```compile_fail
/// use portable_codegen::{LinkerDialect, ResolvedFileImport, TargetFileId};
/// fn forge<D: LinkerDialect>(destination: TargetFileId, kind: D::ImportKind)
///     -> ResolvedFileImport<D>
/// {
///     ResolvedFileImport { destination, kind }
/// }
/// ```
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ResolvedFileImport<D: LinkerDialect> {
    pub(super) destination: TargetFileId,
    pub(super) kind: D::ImportKind,
}

impl<D: LinkerDialect> ResolvedFileImport<D> {
    pub const fn destination(&self) -> TargetFileId {
        self.destination
    }

    pub fn kind(&self) -> &D::ImportKind {
        &self.kind
    }
}

pub(super) fn derive<D: LinkerDialect>(
    dialect: &D,
    package: &TargetAstPackage<D>,
    source: &TargetFile<D>,
    destinations: impl IntoIterator<Item = TargetFileId>,
    diagnostics: &mut Vec<Diagnostic>,
) -> Vec<ResolvedFileImport<D>> {
    let mut imports = Vec::new();
    // Exact destination identity, not individual reference or symbol identity,
    // controls grouping and ordering. Never trust caller-authored directives.
    let destinations = destinations
        .into_iter()
        .collect::<std::collections::BTreeSet<_>>();
    for destination in destinations {
        let Some(target) = package.file(destination) else {
            diagnostics.push(Diagnostic::error(
                DiagnosticCode::UnresolvedReference,
                "file import destination has no structural source-file identity",
                source.source().clone(),
            ));
            continue;
        };
        match dialect.resolve_file_import(source, target) {
            Ok(Some(kind)) => imports.push(ResolvedFileImport { destination, kind }),
            Ok(None) => {}
            Err(violation) => diagnostics.push(Diagnostic::error(
                violation.code,
                violation.message,
                source.source().clone(),
            )),
        }
    }
    imports
}

pub(super) fn verify<D: LinkerDialect>(
    package: &LinkedTargetPackage<D>,
    file: &LinkedFile<D>,
    source: &TargetFile<D>,
    destinations: impl IntoIterator<Item = TargetFileId>,
    diagnostics: &mut Vec<Diagnostic>,
) {
    let expected = derive(
        &package.dialect,
        &package.unresolved,
        source,
        destinations,
        diagnostics,
    );
    if file.file_imports != expected {
        diagnostics.push(Diagnostic::error(
            DiagnosticCode::InterfaceNonconformance,
            "resolved file imports are not exactly reference-derived",
            source.source().clone(),
        ));
    }
}
