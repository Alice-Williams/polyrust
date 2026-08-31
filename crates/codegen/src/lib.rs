#![forbid(unsafe_code)]

//! Checked-v0 backend boundary and deterministic in-memory artifact manifests.

mod backend;
mod document;
mod manifest;

pub use backend::*;
pub use document::*;
pub use manifest::*;

/// Temporary adapter for the pre-v0 prototype emitters. New backends must use
/// the checked [`Backend`] contract at this crate's root.
#[doc(hidden)]
pub mod legacy {
    use portable_check::CheckedModule;

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
