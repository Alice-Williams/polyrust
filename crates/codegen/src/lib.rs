#![forbid(unsafe_code)]

//! Backend boundary and deterministic in-memory artifact manifest.

mod document;

pub use document::*;

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
