//! Closed declaration/definition categories; linkage is not a source string.

use super::{
    CAggregateRef, CBlock, CEnumRef, CEnumeratorRef, CFileRef, CFunctionRef, CInitializer,
    CMemberRef, CObjectRef, CParameterRef, CTypedefRef,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CLinkage {
    External,
    Internal,
    None,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CStorage {
    Automatic,
    Static,
    Extern,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CDeclaration {
    pub(super) file: CFileRef,
    pub(super) kind: CDeclarationKind,
}
impl CDeclaration {
    pub const fn file(&self) -> &CFileRef {
        &self.file
    }
    pub const fn kind(&self) -> &CDeclarationKind {
        &self.kind
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CDeclarationKind {
    ForwardTag(CAggregateRef),
    Typedef(CTypedefRef),
    Aggregate {
        owner: CAggregateRef,
        members: Vec<CMemberRef>,
    },
    Enum {
        owner: CEnumRef,
        values: Vec<CEnumeratorRef>,
    },
    FunctionPrototype {
        function: CFunctionRef,
        linkage: CLinkage,
    },
    /// Always extern; a public declaration is never a tentative definition.
    ObjectDeclaration(CObjectRef),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CDefinition {
    pub(super) file: CFileRef,
    pub(super) kind: CDefinitionKind,
}
impl CDefinition {
    pub const fn file(&self) -> &CFileRef {
        &self.file
    }
    pub const fn kind(&self) -> &CDefinitionKind {
        &self.kind
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CDefinitionKind {
    Function {
        function: CFunctionRef,
        linkage: CLinkage,
        parameters: Vec<CParameterRef>,
        body: Box<CBlock>,
    },
    Object {
        object: CObjectRef,
        linkage: CLinkage,
        initializer: Box<CInitializer>,
    },
}

impl CDeclaration {
    pub fn linkage(&self) -> CLinkage {
        match self.kind() {
            CDeclarationKind::FunctionPrototype { linkage, .. } => *linkage,
            CDeclarationKind::ObjectDeclaration(_) => CLinkage::External,
            CDeclarationKind::ForwardTag(_)
            | CDeclarationKind::Typedef(_)
            | CDeclarationKind::Aggregate { .. }
            | CDeclarationKind::Enum { .. } => CLinkage::None,
        }
    }
    pub fn storage(&self) -> Option<CStorage> {
        match self.linkage() {
            CLinkage::External => Some(CStorage::Extern),
            CLinkage::Internal => Some(CStorage::Static),
            CLinkage::None => None,
        }
    }
}

impl CDefinition {
    pub fn linkage(&self) -> CLinkage {
        match self.kind() {
            CDefinitionKind::Function { linkage, .. } | CDefinitionKind::Object { linkage, .. } => {
                *linkage
            }
        }
    }
    pub fn storage(&self) -> Option<CStorage> {
        match self.linkage() {
            CLinkage::Internal => Some(CStorage::Static),
            CLinkage::External | CLinkage::None => None,
        }
    }
}

impl super::CLocalDeclaration {
    pub const fn storage(&self) -> CStorage {
        CStorage::Automatic
    }
}
