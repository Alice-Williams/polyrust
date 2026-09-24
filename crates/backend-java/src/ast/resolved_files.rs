//! Java AST: resolved files.

use super::JavaPackage;
use super::declaration_model::JavaTypeDeclaration;
use super::file_model::JavaFileItem;
use super::identifiers::JavaIdentifier;
use crate::dialect::JavaDialect;
use portable_codegen::TargetSymbolRef;
use std::collections::BTreeMap;

/// Resolved rendering data does not expose the original registration table.
///
/// ```compile_fail,E0616
/// use portable_backend_java::dialect::JavaDialect;
/// use portable_codegen::RenderReadyPackage;
/// fn registrations(package: &RenderReadyPackage<JavaDialect>) {
///     let _ = &package.ast().files()[0].items()[0].source_inventory;
/// }
/// ```
///
/// ```compile_fail,E0603
/// use portable_backend_java::ast::JavaSourceInventory;
/// ```
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ResolvedJavaFileItem {
    pub item: JavaFileItem,
    pub names: BTreeMap<TargetSymbolRef<JavaDialect>, JavaResolvedName>,
    pub documentation: super::JavaDocumentation,
    pub(crate) source_inventory: super::JavaSourceInventory,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum JavaResolvedName {
    Local(JavaIdentifier),
    DeclaredPath(JavaDeclaredPath),
    Qualified(crate::dialect::JavaQualifiedName),
    GeneratedMember {
        owner: crate::dialect::JavaGeneratedContainer,
        member: JavaIdentifier,
    },
    Member {
        owner: crate::dialect::JavaQualifiedName,
        member: crate::dialect::JavaReferencedMemberName,
    },
}

/// A generated declaration's verified lexical path, never a guessed container.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct JavaDeclaredPath {
    pub(crate) package: JavaPackage,
    pub(crate) owners: Vec<JavaIdentifier>,
    pub(crate) member: JavaIdentifier,
}

impl JavaDeclaredPath {
    pub(crate) fn encoded_len(&self) -> usize {
        self.owners
            .iter()
            .fold(self.package.name().len(), |size, name| {
                size.saturating_add(1).saturating_add(name.as_str().len())
            })
            .saturating_add(1)
            .saturating_add(self.member.as_str().len())
    }
    pub(crate) fn text(&self) -> String {
        let package = self.package.name();
        std::iter::once(package.as_ref())
            .chain(self.owners.iter().map(|name| name.as_str()))
            .chain(std::iter::once(self.member.as_str()))
            .collect::<Vec<_>>()
            .join(".")
    }
    pub fn package(&self) -> JavaPackage {
        self.package
    }
    pub fn owners(&self) -> &[JavaIdentifier] {
        &self.owners
    }
    pub fn member(&self) -> &JavaIdentifier {
        &self.member
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum JavaFilePlacement {
    Main,
    Runtime,
    NativeTest,
    Conformance,
    NegativeTest,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum JavaSourceFileKind {
    CompilationUnit,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct JavaCompilationUnit {
    pub package: JavaPackage,
    pub imports: Vec<crate::dialect::JavaImportKind>,
    pub declarations: Vec<JavaTypeDeclaration>,
}
