//! Java AST: resolved files.

use super::declaration_model::JavaTypeDeclaration;
use super::file_model::JavaFileItem;
use super::identifiers::JavaIdentifier;
use crate::dialect::JavaDialect;
use portable_codegen::TargetSymbolRef;
use std::collections::BTreeMap;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ResolvedJavaFileItem {
    pub item: JavaFileItem,
    pub names: BTreeMap<TargetSymbolRef<JavaDialect>, JavaResolvedName>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum JavaResolvedName {
    Local(JavaIdentifier),
    Qualified(crate::dialect::JavaQualifiedName),
    GeneratedMember {
        owner: crate::dialect::JavaGeneratedContainer,
        member: JavaIdentifier,
    },
    Member {
        owner: crate::dialect::JavaQualifiedName,
        member: crate::dialect::JavaMemberName,
    },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum JavaPackage {
    Generated,
}

impl JavaPackage {
    pub const fn name(self) -> &'static str {
        match self {
            Self::Generated => "org.polyrust.generated",
        }
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
