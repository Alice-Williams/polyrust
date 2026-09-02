#![forbid(unsafe_code)]

//! Checked-v0 backend boundary and deterministic in-memory artifact manifests.

mod backend;
mod capability;
mod compliance;
mod document;
mod heritage;
mod language;
mod linking;
mod manifest;
mod rendering;
mod target_ast;
mod typed_pipeline;

pub use backend::*;
pub use capability::*;
pub use compliance::*;
pub use document::*;
pub use heritage::*;
pub use language::*;
pub use linking::*;
pub use manifest::*;
pub use rendering::*;
pub use target_ast::*;
pub use typed_pipeline::*;

/// Temporary adapter for the pre-v0 prototype emitters. New backends must use
/// the checked [`Backend`] contract at this crate's root.
#[doc(hidden)]
pub mod legacy {
    use portable_check::CheckedModule;

    use crate::{DeclaredDependency, InjectedHelper, OutputFile, OutputManifest};

    /// Transitional manifest assembler for pre-M34A backends and CLI tests.
    ///
    /// This is deliberately isolated under `legacy`; typed plugins cannot
    /// provide a manifest to the sealed compiler adapter. M34A-18 removes this
    /// escape after every built-in and external example uses the typed path.
    pub fn assemble_output_manifest(
        files: Vec<OutputFile>,
        dependencies: Vec<DeclaredDependency>,
        helpers: Vec<InjectedHelper>,
    ) -> Result<OutputManifest, Vec<portable_diagnostics::Diagnostic>> {
        OutputManifest::new(files, dependencies, helpers)
    }

    #[derive(Clone, Debug, PartialEq, Eq)]
    pub struct GeneratedFile {
        pub path: String,
        pub contents: String,
    }

    #[derive(Clone, Debug, PartialEq, Eq)]
    pub struct GeneratedPackage {
        pub files: Vec<GeneratedFile>,
    }

    impl GeneratedPackage {
        pub fn file(&self, path: &str) -> Option<&str> {
            self.files
                .iter()
                .find(|file| file.path == path)
                .map(|file| file.contents.as_str())
        }
    }

    pub trait Backend {
        fn target_name(&self) -> &'static str;
        fn emit(&self, module: &CheckedModule) -> GeneratedPackage;
    }
}
