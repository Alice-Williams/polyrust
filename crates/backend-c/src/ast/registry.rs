//! Authoritative C declaration registrations, not a validity certificate.
//!
//! The registry is moved into the C dialect's shared unresolved package at
//! integration. No alternate renderer or public proof constructor exists here.
//!
//! ```compile_fail
//! use portable_backend_c::ast::{CFileRef, CStructRef};
//! fn retarget(mut value: CStructRef, file: CFileRef) { value.file = file; }
//! ```
//!
//! ```compile_fail
//! use portable_backend_c::ast::{CObjectType, CUnionRef};
//! fn wrong_kind(value: CUnionRef) { let _ = CObjectType::structure(value); }
//! ```

use std::collections::{BTreeMap, BTreeSet};

mod allocations;
mod buffer_counts;
mod contextual_inventory;
mod contracts;
mod control;
mod files;
mod frozen;
mod identity;
mod interfaces;
mod inventory;
mod nominals;
mod register_interfaces;
mod register_nominals;
mod register_symbols;
mod symbols;
mod type_membership;

pub use allocations::{CAllocationRef, CAllocationShape, CAllocatorSource};
pub use buffer_counts::CBufferCountRef;
pub use contracts::{CCallableContractOrigin, CCallableContractRef, CMemberBinding};
pub use control::{CCleanupExitRef, CLoopRef, CSwitchRef};
pub use files::{CFileKey, CFileRef, CFileRole};
pub use frozen::CFrozenRegistry;
pub use identity::{CDeclarationKey, CGeneratedOrigin, CSynthesisReason};
pub use interfaces::{
    CInterfaceAdapterRef, CInterfaceTableRef, CInterfaceWitnessRef, CWitnessMethod,
};
pub use inventory::{CRegistrationKind, CRegistrationOwner, CRegistrationSummary};
pub use nominals::{
    CAggregateRef, CEnumRef, CEnumeratorRef, CMemberRef, CStructRef, CTypedefRef, CUnionRef,
};
pub use symbols::{CFunctionRef, CLocalRef, CObjectRef, CParameterRef, CScopeRef};

pub(crate) use contextual_inventory::CRegistered;
pub(super) use identity::RegistryScope;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CRegistryError {
    InvalidObjectType(super::CTypeError),
    CrossRegistry,
    UnregisteredReference,
    DuplicateRegistration,
    AlreadyDefined,
    WrongOwner,
    EmptyDefinition,
    DefinitionInventoryMismatch,
    ParameterIndex,
    InterfaceTableType,
    CallableContractMismatch,
    InvalidBufferCount,
}

impl std::fmt::Display for CRegistryError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Self::InvalidObjectType(error) = self {
            return error.fmt(f);
        }
        f.write_str(match self {
            Self::InvalidObjectType(_) => unreachable!("handled above"),
            Self::CrossRegistry => "C reference belongs to a different registry",
            Self::UnregisteredReference => "C reference has no matching registration",
            Self::DuplicateRegistration => "C declaration is already registered",
            Self::AlreadyDefined => "C declaration already has a definition",
            Self::WrongOwner => "C reference belongs to a different declaration owner",
            Self::ParameterIndex => "C parameter index is outside the registered exact signature",
            Self::InvalidBufferCount => "C buffer count requires an immutable size_t local",
            Self::CallableContractMismatch => {
                "C callable member does not bind this function contract"
            }
            Self::InterfaceTableType => {
                "C interface tables require a const registered struct object"
            }
            Self::EmptyDefinition => "C17 aggregate/enum definitions must be nonempty",
            Self::DefinitionInventoryMismatch => {
                "C definition does not match its complete registered member inventory"
            }
        })
    }
}
impl std::error::Error for CRegistryError {}

/// This mutable registration builder is intentionally not Clone: cloning a
/// scope and then diverging its authoritative inventory would weaken identity.
#[derive(Debug, PartialEq, Eq)]
pub struct CRegistry {
    scope: RegistryScope,
    files: BTreeSet<CFileRef>,
    structs: BTreeMap<CStructRef, Option<Vec<CMemberRef>>>,
    unions: BTreeMap<CUnionRef, Option<Vec<CMemberRef>>>,
    enums: BTreeMap<CEnumRef, Option<Vec<CEnumeratorRef>>>,
    typedefs: BTreeSet<CTypedefRef>,
    members: BTreeSet<CMemberRef>,
    enumerators: BTreeSet<CEnumeratorRef>,
    functions: BTreeSet<CFunctionRef>,
    objects: BTreeSet<CObjectRef>,
    parameters: BTreeSet<CParameterRef>,
    scopes: BTreeSet<CScopeRef>,
    locals: BTreeSet<CLocalRef>,
    loops: BTreeSet<CLoopRef>,
    switches: BTreeSet<CSwitchRef>,
    cleanup_exits: BTreeSet<CCleanupExitRef>,
    allocations: BTreeSet<CAllocationRef>,
    witnesses: BTreeSet<CInterfaceWitnessRef>,
    tables: BTreeSet<CInterfaceTableRef>,
    adapters: BTreeSet<CInterfaceAdapterRef>,
}

impl Default for CRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl CRegistry {
    pub(super) fn expression_brand(&self) -> RegistryScope {
        self.scope.clone()
    }

    pub fn new() -> Self {
        Self {
            scope: RegistryScope::new(),
            files: BTreeSet::new(),
            structs: BTreeMap::new(),
            unions: BTreeMap::new(),
            enums: BTreeMap::new(),
            typedefs: BTreeSet::new(),
            members: BTreeSet::new(),
            enumerators: BTreeSet::new(),
            functions: BTreeSet::new(),
            objects: BTreeSet::new(),
            parameters: BTreeSet::new(),
            scopes: BTreeSet::new(),
            locals: BTreeSet::new(),
            loops: BTreeSet::new(),
            switches: BTreeSet::new(),
            cleanup_exits: BTreeSet::new(),
            allocations: BTreeSet::new(),
            witnesses: BTreeSet::new(),
            tables: BTreeSet::new(),
            adapters: BTreeSet::new(),
        }
    }

    pub fn register_file(&mut self, key: CFileKey) -> Result<CFileRef, CRegistryError> {
        let value = CFileRef {
            key,
            scope: self.scope.clone(),
        };
        if self.files.iter().any(|old| old.key.path == value.key.path) {
            return Err(CRegistryError::DuplicateRegistration);
        }
        self.files.insert(value.clone());
        Ok(value)
    }

    pub fn files(&self) -> impl Iterator<Item = &CFileKey> {
        self.files.iter().map(CFileRef::key)
    }

    fn check_scope(&self, scope: &RegistryScope) -> Result<(), CRegistryError> {
        if *scope == self.scope {
            Ok(())
        } else {
            Err(CRegistryError::CrossRegistry)
        }
    }

    pub fn check_file(&self, value: &CFileRef) -> Result<(), CRegistryError> {
        self.check_scope(&value.scope)?;
        if self.files.contains(value) {
            Ok(())
        } else {
            Err(CRegistryError::UnregisteredReference)
        }
    }

    pub fn check_aggregate(&self, owner: &CAggregateRef) -> Result<(), CRegistryError> {
        match owner {
            CAggregateRef::Struct(value) => {
                self.check_scope(&value.identity.scope)?;
                if self.structs.contains_key(value) {
                    Ok(())
                } else {
                    Err(CRegistryError::UnregisteredReference)
                }
            }
            CAggregateRef::Union(value) => {
                self.check_scope(&value.identity.scope)?;
                if self.unions.contains_key(value) {
                    Ok(())
                } else {
                    Err(CRegistryError::UnregisteredReference)
                }
            }
        }
    }

    pub fn members(&self, owner: &CAggregateRef) -> Result<Option<&[CMemberRef]>, CRegistryError> {
        self.check_aggregate(owner)?;
        let members = match owner {
            CAggregateRef::Struct(value) => &self.structs[value],
            CAggregateRef::Union(value) => &self.unions[value],
        };
        Ok(members.as_deref())
    }

    pub fn check_member(
        &self,
        owner: &CAggregateRef,
        member: &CMemberRef,
    ) -> Result<(), CRegistryError> {
        self.check_aggregate(owner)?;
        self.check_scope(&member.identity.scope)?;
        if member.owner != *owner {
            return Err(CRegistryError::WrongOwner);
        }
        if self.members.contains(member) {
            Ok(())
        } else {
            Err(CRegistryError::UnregisteredReference)
        }
    }
}
