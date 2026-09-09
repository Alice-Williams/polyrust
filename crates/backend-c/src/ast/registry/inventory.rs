//! Address-free canonical declaration inventory; never a parallel symbol table.

use super::{
    CAggregateRef, CDeclarationKey, CFileKey, CFileRef, CFunctionRef, CRegistry, CScopeRef,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum CRegistrationKind {
    Struct,
    Union,
    Enum,
    Typedef,
    Member,
    Enumerator,
    Function,
    Object,
    Parameter,
    Scope,
    Local,
    OwnerSlot,
    Loop,
    Switch,
    CleanupExit,
    Allocation,
    Witness,
    Table,
    Adapter,
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum CRegistrationOwner {
    File(CFileKey),
    Declaration {
        kind: CRegistrationKind,
        file: CFileKey,
        key: CDeclarationKey,
    },
    Scope {
        file: CFileKey,
        function: CDeclarationKey,
        scope: CDeclarationKey,
    },
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct CRegistrationSummary {
    pub kind: CRegistrationKind,
    pub key: CDeclarationKey,
    pub owner: CRegistrationOwner,
}

fn declaration(
    kind: CRegistrationKind,
    file: &CFileRef,
    key: &CDeclarationKey,
) -> CRegistrationOwner {
    CRegistrationOwner::Declaration {
        kind,
        file: file.key().clone(),
        key: key.clone(),
    }
}

fn function(value: &CFunctionRef) -> CRegistrationOwner {
    declaration(CRegistrationKind::Function, value.file(), value.key())
}

fn scope(value: &CScopeRef) -> CRegistrationOwner {
    CRegistrationOwner::Scope {
        file: value.function().file().key().clone(),
        function: value.function().key().clone(),
        scope: value.key().clone(),
    }
}

impl CRegistry {
    /// Recomputed from authoritative registrations; no cached names or IDs.
    pub fn inventory(&self) -> Vec<CRegistrationSummary> {
        let mut result = Vec::new();
        let mut add = |kind, key: &CDeclarationKey, owner| {
            result.push(CRegistrationSummary {
                kind,
                key: key.clone(),
                owner,
            });
        };
        for value in self.structs.keys() {
            add(
                CRegistrationKind::Struct,
                value.key(),
                CRegistrationOwner::File(value.file().key().clone()),
            );
        }
        for value in self.unions.keys() {
            add(
                CRegistrationKind::Union,
                value.key(),
                CRegistrationOwner::File(value.file().key().clone()),
            );
        }
        for value in self.enums.keys() {
            add(
                CRegistrationKind::Enum,
                value.key(),
                CRegistrationOwner::File(value.file().key().clone()),
            );
        }
        for value in &self.typedefs {
            add(
                CRegistrationKind::Typedef,
                value.key(),
                CRegistrationOwner::File(value.file().key().clone()),
            );
        }
        for value in &self.members {
            let owner = match value.owner() {
                CAggregateRef::Struct(owner) => {
                    declaration(CRegistrationKind::Struct, owner.file(), owner.key())
                }
                CAggregateRef::Union(owner) => {
                    declaration(CRegistrationKind::Union, owner.file(), owner.key())
                }
            };
            add(CRegistrationKind::Member, value.key(), owner);
        }
        for value in &self.enumerators {
            add(
                CRegistrationKind::Enumerator,
                value.key(),
                declaration(
                    CRegistrationKind::Enum,
                    value.owner().file(),
                    value.owner().key(),
                ),
            );
        }
        for value in &self.functions {
            add(
                CRegistrationKind::Function,
                value.key(),
                CRegistrationOwner::File(value.file().key().clone()),
            );
        }
        for value in &self.objects {
            add(
                CRegistrationKind::Object,
                value.key(),
                CRegistrationOwner::File(value.file().key().clone()),
            );
        }
        for value in &self.parameters {
            add(
                CRegistrationKind::Parameter,
                value.key(),
                function(value.function()),
            );
        }
        for value in &self.scopes {
            add(
                CRegistrationKind::Scope,
                value.key(),
                value
                    .parent()
                    .map_or_else(|| function(value.function()), scope),
            );
        }
        for value in &self.locals {
            add(CRegistrationKind::Local, value.key(), scope(value.scope()));
        }
        for value in &self.owner_slots {
            let local = value.local();
            add(
                CRegistrationKind::OwnerSlot,
                local.key(),
                scope(local.scope()),
            );
        }
        for value in &self.loops {
            add(CRegistrationKind::Loop, value.key(), scope(value.scope()));
        }
        for value in &self.switches {
            add(CRegistrationKind::Switch, value.key(), scope(value.scope()));
        }
        for value in &self.cleanup_exits {
            add(
                CRegistrationKind::CleanupExit,
                value.key(),
                scope(value.scope()),
            );
        }
        for value in &self.allocations {
            add(
                CRegistrationKind::Allocation,
                value.key(),
                scope(value.scope()),
            );
        }
        for value in &self.witnesses {
            add(
                CRegistrationKind::Witness,
                value.key(),
                CRegistrationOwner::File(value.interface().file().key().clone()),
            );
        }
        for value in &self.tables {
            add(
                CRegistrationKind::Table,
                value.key(),
                declaration(
                    CRegistrationKind::Witness,
                    value.witness().interface().file(),
                    value.witness().key(),
                ),
            );
        }
        for value in &self.adapters {
            add(
                CRegistrationKind::Adapter,
                value.key(),
                declaration(
                    CRegistrationKind::Table,
                    value.table().object().file(),
                    value.table().key(),
                ),
            );
        }
        result.sort();
        result
    }
}
