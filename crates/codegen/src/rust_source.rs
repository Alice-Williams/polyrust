//! Compiler-source metadata shared by target plugins, not a validity certificate.
//!
//! The isolated rustc adapter populates this data only after analysis succeeds.
//! Constructing metadata never authenticates a Rust program or makes target
//! syntax render-ready. Target registries still authenticate their references.

mod exports;
pub use exports::*;
mod constant_values;
mod types;
pub use constant_values::RustConstantValue;
pub use types::{
    RustFieldTypes, RustFunctionTypes, RustResultKind, RustScalarKind, RustSourceTypes,
};
mod documentation;
pub use documentation::{CheckedRustDocumentation, RustDocumentationError};

/// Stable compiler declaration identity; neither spelling nor a HIR arena index.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct RustDeclarationId {
    pub crate_id: u64,
    pub definition_path_hash: u64,
}

/// A body-local node is meaningful only together with its owning declaration.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum RustSourceNode {
    Declaration,
    Parameter(u32),
    Binding(u32),
    LexicalScope(u32),
}

/// Declared visibility and effective export status are distinct compiler facts.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum RustVisibility {
    Public,
    RestrictedTo(RustDeclarationId),
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct RustSourceLocation {
    /// Compiler-reported source path, for diagnostics only, never an output path.
    pub file: String,
    pub line: usize,
    pub column: usize,
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct RustModuleDocumentation {
    pub declaration: RustDeclarationId,
    pub parent: Option<RustDeclarationId>,
    pub location: RustSourceLocation,
    /// Compiler-resolved attributes in source order, not target syntax.
    pub documentation: Vec<String>,
}

/// Shared across declarations in one module; ancestor payloads are shared across
/// different module chains too. Cloning origins does not copy module doc text.
pub type RustModuleAncestry = std::sync::Arc<[std::sync::Arc<RustModuleDocumentation>]>;

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct RustSourceOrigin {
    pub declaration: RustDeclarationId,
    pub node: RustSourceNode,
    pub module: RustDeclarationId,
    pub location: RustSourceLocation,
    pub visibility: RustVisibility,
    pub externally_reachable: bool,
    /// Resolved documentation attribute text, not executable target text.
    pub documentation: Vec<String>,
    /// Root-to-owning-module ancestry for declarations; empty on body nodes.
    pub module_ancestors: RustModuleAncestry,
    /// One shared compiler crate inventory, independent of the owning module's
    /// private logical ancestry and of the target's later public API mapping.
    pub crate_exports: std::sync::Arc<RustCrateExports>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn declaration_and_body_identity_are_distinct_from_names() {
        let first = RustDeclarationId {
            crate_id: 1,
            definition_path_hash: 2,
        };
        let other_crate = RustDeclarationId {
            crate_id: 2,
            ..first
        };
        let other_definition = RustDeclarationId {
            definition_path_hash: 3,
            ..first
        };
        assert_ne!(first, other_crate);
        assert_ne!(first, other_definition);
        assert_ne!(RustSourceNode::Binding(7), RustSourceNode::LexicalScope(7));
        assert_ne!(RustVisibility::Public, RustVisibility::RestrictedTo(first));
    }
}
