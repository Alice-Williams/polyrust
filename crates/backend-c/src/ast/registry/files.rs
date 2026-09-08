//! Private file identities use the shared normalized relative-path type.

use portable_codegen::RelativeOutputPath;

use super::identity::RegistryScope;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum CFileRole {
    GeneratedPublicHeader,
    GeneratedSource,
    RuntimePublicHeader,
    RuntimeSource,
    PrivateHeader,
    TestSource,
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct CFileKey {
    pub path: RelativeOutputPath,
    pub role: CFileRole,
}

/// A spelling alone cannot manufacture a registered file.
///
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct CFileRef {
    pub(super) key: CFileKey,
    pub(super) scope: RegistryScope,
}

impl CFileRef {
    pub fn key(&self) -> &CFileKey {
        &self.key
    }
}
