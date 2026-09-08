//! Distinct nominal categories; references retain source and owning file.

use std::sync::Arc;

use super::super::CObjectType;
use super::{CDeclarationKey, CFileRef, CMemberBinding, identity::Identity};

macro_rules! nominal_reference {
    ($name:ident) => {
        #[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
        pub struct $name {
            pub(super) identity: Identity,
            pub(super) file: CFileRef,
        }
        impl $name {
            pub fn key(&self) -> &CDeclarationKey {
                &self.identity.key
            }
            pub fn file(&self) -> &CFileRef {
                &self.file
            }
        }
    };
}

nominal_reference!(CStructRef);
nominal_reference!(CUnionRef);
nominal_reference!(CEnumRef);

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum CAggregateRef {
    Struct(CStructRef),
    Union(CUnionRef),
}

impl CAggregateRef {
    pub fn key(&self) -> &CDeclarationKey {
        match self {
            Self::Struct(value) => value.key(),
            Self::Union(value) => value.key(),
        }
    }
}

/// An alias is immutable and can only target already registered aliases.
/// Recursive pointers go through forward nominal tags, never alias cycles.
///
/// ```compile_fail
/// use portable_backend_c::ast::{CTypedefRef, CObjectType};
/// fn rewrite(mut alias: CTypedefRef, ty: CObjectType) {
///     alias.target = std::sync::Arc::new(ty);
/// }
/// ```
///
/// Bare function aliases cannot enter the object-type registry:
///
/// ```compile_fail
/// use portable_backend_c::ast::{CRegistry, CFileRef, CDeclarationKey, CFunctionType};
/// fn function_alias(registry: &mut CRegistry, file: &CFileRef,
///                   key: CDeclarationKey, function: CFunctionType) {
///     registry.register_typedef(file, key, function).unwrap();
/// }
/// ```
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct CTypedefRef {
    pub(super) identity: Identity,
    pub(super) file: CFileRef,
    pub(super) target: Arc<CObjectType>,
}

impl CTypedefRef {
    pub fn key(&self) -> &CDeclarationKey {
        &self.identity.key
    }
    pub fn file(&self) -> &CFileRef {
        &self.file
    }
    pub fn target(&self) -> &CObjectType {
        &self.target
    }
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct CMemberRef {
    pub(super) identity: Identity,
    pub(super) owner: CAggregateRef,
    pub(super) ty: CObjectType,
    pub(super) binding: CMemberBinding,
}

impl CMemberRef {
    pub fn binding(&self) -> &CMemberBinding {
        &self.binding
    }
    pub fn key(&self) -> &CDeclarationKey {
        &self.identity.key
    }
    pub fn owner(&self) -> &CAggregateRef {
        &self.owner
    }
    pub fn ty(&self) -> &CObjectType {
        &self.ty
    }
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct CEnumeratorRef {
    pub(super) identity: Identity,
    pub(super) owner: CEnumRef,
    pub(super) value: i32,
}

impl CEnumeratorRef {
    pub fn key(&self) -> &CDeclarationKey {
        &self.identity.key
    }
    pub fn owner(&self) -> &CEnumRef {
        &self.owner
    }
    pub const fn value(&self) -> i32 {
        self.value
    }
}
