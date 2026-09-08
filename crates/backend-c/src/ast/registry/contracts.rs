//! A generated callable's effect contract has its own sealed identity.
//!
//! Registration says which body must be proved, not that its summary is true.
//! Known-library contracts are supplied by the later closed catalogue, never
//! by presenting an arbitrary generated function as a known callable.

use super::{CDeclarationKey, CFileRef, identity::Identity};

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum CCallableContractOrigin {
    GeneratedBody,
}

/// ```compile_fail
/// use portable_backend_c::ast::{CCallableContractRef, CFileRef};
/// fn retarget(mut contract: CCallableContractRef, file: CFileRef) {
///     contract.file = file;
/// }
/// ```
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct CCallableContractRef {
    pub(super) identity: Identity,
    pub(super) file: CFileRef,
    pub(super) origin: CCallableContractOrigin,
}

impl CCallableContractRef {
    pub fn key(&self) -> &CDeclarationKey {
        &self.identity.key
    }
    pub fn file(&self) -> &CFileRef {
        &self.file
    }
    pub const fn origin(&self) -> CCallableContractOrigin {
        self.origin
    }
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum CMemberBinding {
    Object,
    Callable(CCallableContractRef),
}
