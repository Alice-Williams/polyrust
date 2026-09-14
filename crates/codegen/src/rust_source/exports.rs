//! Finite compiler export metadata; names here are not executable target syntax.
use super::RustDeclarationId;
use std::collections::BTreeMap;

/// Rust names may coincide across namespaces without naming the same object.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum RustExportNamespace {
    Type,
    Value,
    Macro,
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct RustExportName {
    pub namespace: RustExportNamespace,
    /// Compiler-resolved identifier spelling, without a raw-identifier prefix.
    pub name: String,
}

/// A module alias is an edge, including aliases back to an already seen module.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum RustExportTarget {
    Module(RustDeclarationId),
    Declaration(RustDeclarationId),
}

/// Public module bindings form a finite graph, not an expanded list of paths.
///
/// Only reachable local modules have entries. A foreign module edge retains
/// its crate identity and must be resolved against that crate's own inventory.
/// Construction is not evidence that a Rust compiler analyzed a program.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct RustCrateExports {
    pub root: RustDeclarationId,
    pub modules: BTreeMap<RustDeclarationId, BTreeMap<RustExportName, RustExportTarget>>,
    /// Compiler-owned ancestry for public modules, including alias-only modules.
    /// Metadata remains non-authenticating; the compiler bridge supplies all entries.
    pub module_ancestries: BTreeMap<RustDeclarationId, super::RustModuleAncestry>,
}
