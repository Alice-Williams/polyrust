//! Authenticated file identities and checked include spelling, before rendering.
use super::violation;
use crate::ast::{CFileRef, CFileRole, CIdentifier, CRegistry};
use crate::dialect::CHeader;
use portable_codegen::AstViolation;
use std::fmt::Write;

#[cfg(test)]
#[path = "../../tests/shared_file_imports.rs"]
mod tests;

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum CImportKind {
    Standard(CHeader),
    Generated(CGeneratedHeader),
    Dependency(super::CDependencyPackage),
}

/// No public constructor: a spelling cannot manufacture a registered header.
///
/// ```compile_fail
/// use portable_backend_c::{ast::CFileRef, dialect::{CGeneratedHeader, CHeaderGuard}};
/// fn forge(file: CFileRef, guard: CHeaderGuard) -> CGeneratedHeader {
///     CGeneratedHeader { file, path: "injected.h".into(), guard }
/// }
/// ```
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct CGeneratedHeader {
    file: CFileRef,
    path: String,
    guard: CHeaderGuard,
}

/// Byte-encoded canonical output identity, disjoint from `poly_` symbol names.
///
/// ```compile_fail
/// use portable_backend_c::{ast::{CFileRef, CIdentifier}, dialect::CHeaderGuard};
/// fn forge(file: CFileRef, identifier: CIdentifier) -> CHeaderGuard {
///     CHeaderGuard { file, identifier }
/// }
/// ```
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct CHeaderGuard {
    file: CFileRef,
    identifier: CIdentifier,
}

impl CHeaderGuard {
    pub fn file(&self) -> &CFileRef {
        &self.file
    }

    pub fn identifier(&self) -> &CIdentifier {
        &self.identifier
    }
}

impl CGeneratedHeader {
    pub(super) fn resolve(
        registry: &CRegistry,
        source: &CFileRef,
        destination: &CFileRef,
    ) -> Result<Self, AstViolation> {
        for file in [source, destination] {
            registry
                .check_file(file)
                .map_err(|error| violation(error.to_string()))?;
        }
        if !matches!(
            destination.key().role,
            CFileRole::GeneratedPublicHeader
                | CFileRole::RuntimePublicHeader
                | CFileRole::PrivateHeader
        ) {
            return Err(violation("C file dependency must name a registered header"));
        }
        if source == destination {
            return Err(violation("C header cannot include itself"));
        }
        let canonical = destination.key().path.as_str();
        let (directory, basename) = split(canonical);
        if split(source.key().path.as_str()).0 != directory {
            return Err(violation(
                "C generated include currently requires sibling output files",
            ));
        }
        // Deliberately closed, portable path grammar; reject rather than escape
        // directive syntax or platform-dependent header-name interpretation.
        if !canonical
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || b"_./-".contains(&byte))
            || !basename.ends_with(".h")
            || !basename.starts_with("polyrust_")
            || basename.len() <= "polyrust_.h".len()
        {
            return Err(violation(
                "C generated header requires a portable polyrust_<nonempty>.h basename",
            ));
        }
        let mut guard = String::from("POLYRUST_HEADER_");
        for byte in canonical.bytes() {
            write!(guard, "{byte:02X}").expect("String formatting");
        }
        guard.push_str("_INCLUDED");
        if guard.len() > 256 {
            return Err(violation(
                "C generated-header guard exceeds identifier budget",
            ));
        }
        let identifier = CIdentifier::new(&guard).map_err(|error| violation(error.to_string()))?;
        Ok(Self {
            file: destination.clone(),
            path: basename.to_owned(),
            guard: CHeaderGuard {
                file: destination.clone(),
                identifier,
            },
        })
    }

    pub fn file(&self) -> &CFileRef {
        &self.file
    }

    pub fn include_path(&self) -> &str {
        &self.path
    }

    /// C resolves the emitted header name, not the owning artifact's directory.
    /// Reject an output that would shadow this exact dependency include.
    pub(crate) fn conflicts_with_output_path(
        &self,
        path: &portable_codegen::RelativeOutputPath,
    ) -> bool {
        split(path.as_str()).1 == self.include_path()
    }

    pub fn guard(&self) -> &CHeaderGuard {
        &self.guard
    }
}

fn split(path: &str) -> (&str, &str) {
    path.rsplit_once('/').unwrap_or(("", path))
}
