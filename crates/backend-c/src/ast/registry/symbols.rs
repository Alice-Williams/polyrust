//! Function, object and lexical binding references keep exact owners and types.

use std::sync::Arc;

use super::super::{CFunctionType, CObjectType};
use super::{CCallableContractRef, CDeclarationKey, CFileRef, identity::Identity};

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct CFunctionRef {
    pub(super) identity: Identity,
    pub(super) file: CFileRef,
    pub(super) signature: Arc<CFunctionType>,
    pub(super) contract: CCallableContractRef,
}

impl CFunctionRef {
    pub fn contract(&self) -> &CCallableContractRef {
        &self.contract
    }
    pub fn key(&self) -> &CDeclarationKey {
        &self.identity.key
    }
    pub fn file(&self) -> &CFileRef {
        &self.file
    }
    pub fn signature(&self) -> &CFunctionType {
        &self.signature
    }
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct CObjectRef {
    pub(super) identity: Identity,
    pub(super) file: CFileRef,
    pub(super) ty: CObjectType,
}

impl CObjectRef {
    pub fn key(&self) -> &CDeclarationKey {
        &self.identity.key
    }
    pub fn file(&self) -> &CFileRef {
        &self.file
    }
    pub fn ty(&self) -> &CObjectType {
        &self.ty
    }
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct CParameterRef {
    pub(super) identity: Identity,
    pub(super) function: CFunctionRef,
    pub(super) index: usize,
    pub(super) ty: CObjectType,
}

impl CParameterRef {
    pub fn key(&self) -> &CDeclarationKey {
        &self.identity.key
    }
    pub fn function(&self) -> &CFunctionRef {
        &self.function
    }
    pub const fn index(&self) -> usize {
        self.index
    }
    pub fn ty(&self) -> &CObjectType {
        &self.ty
    }
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct CScopeRef {
    pub(super) identity: Identity,
    pub(super) function: CFunctionRef,
    pub(super) parent: Option<Arc<CScopeRef>>,
}

impl CScopeRef {
    pub fn key(&self) -> &CDeclarationKey {
        &self.identity.key
    }
    pub fn function(&self) -> &CFunctionRef {
        &self.function
    }
    pub fn parent(&self) -> Option<&CScopeRef> {
        self.parent.as_deref()
    }
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct CLocalRef {
    pub(super) identity: Identity,
    pub(super) scope: CScopeRef,
    pub(super) ty: CObjectType,
}

impl CLocalRef {
    pub fn key(&self) -> &CDeclarationKey {
        &self.identity.key
    }
    pub fn scope(&self) -> &CScopeRef {
        &self.scope
    }
    pub fn ty(&self) -> &CObjectType {
        &self.ty
    }
}
