//! Coarse definition ownership, not a proof of Core provenance or dependencies.

use super::super::{
    CAggregateRef, CDeclarationKey, CDeclarationKind, CDefinitionKind, CFileItem, CFileRef,
    CFileRole as F, CFunctionRef, CGeneratedOrigin as O, CRegistry, CSourceFile,
    CSynthesisReason as S, registry::CRegistered as R,
};
use super::CContextError as E;
use portable_core_ir::CoreDeclaration;
use std::collections::BTreeMap;

fn permitted(origin: &O, role: F) -> bool {
    match origin {
        O::CoreDeclaration(CoreDeclaration::Test(_)) | O::Synthesized(S::TestHarness) => {
            role == F::TestSource
        }
        O::CoreDeclaration(_) => matches!(
            role,
            F::GeneratedPublicHeader | F::GeneratedSource | F::PrivateHeader
        ),
        O::CoreExpression(_) | O::Synthesized(S::EvaluationTemporary) => {
            matches!(role, F::GeneratedSource | F::PrivateHeader | F::TestSource)
        }
        O::Synthesized(S::Runtime) => matches!(
            role,
            F::RuntimePublicHeader | F::RuntimeSource | F::PrivateHeader
        ),
        O::Synthesized(S::OwnershipAdapter | S::InterfaceAdapter) => matches!(
            role,
            F::GeneratedPublicHeader | F::GeneratedSource | F::PrivateHeader | F::TestSource
        ),
        O::Synthesized(S::PlatformAssertion) => true,
    }
}

fn require(key: &CDeclarationKey, file: &CFileRef) -> Result<(), E> {
    if permitted(&key.origin, file.key().role) {
        Ok(())
    } else {
        Err(E::OriginRoleMismatch)
    }
}

fn aggregate_file(owner: &CAggregateRef) -> &CFileRef {
    match owner {
        CAggregateRef::Struct(v) => v.file(),
        CAggregateRef::Union(v) => v.file(),
    }
}

pub(super) fn check(registry: &CRegistry, files: &[CSourceFile]) -> Result<(), E> {
    let mut bodies = BTreeMap::new();
    let mut aggregates = BTreeMap::new();
    for file in files {
        for item in file.items() {
            match item {
                CFileItem::Definition(value) => match value.kind() {
                    CDefinitionKind::Function { function, .. } => {
                        require(function.key(), file.identity())?;
                        bodies.insert(function, file.identity());
                    }
                    CDefinitionKind::Object { object, .. } => {
                        require(object.key(), file.identity())?
                    }
                },
                CFileItem::Declaration(value) => match value.kind() {
                    CDeclarationKind::Aggregate { owner, .. } => {
                        require(owner.key(), file.identity())?;
                        aggregates.insert(owner, file.identity());
                    }
                    CDeclarationKind::Enum { owner, .. } => require(owner.key(), file.identity())?,
                    CDeclarationKind::Typedef(value) => require(value.key(), file.identity())?,
                    // A forward/prototype/extern occurrence references an owner;
                    // it does not make this file the symbol's defining origin.
                    CDeclarationKind::ForwardTag(_)
                    | CDeclarationKind::FunctionPrototype { .. }
                    | CDeclarationKind::ObjectDeclaration(_) => {}
                },
                CFileItem::Comment(_) | CFileItem::StaticAssert(_) => {}
            }
        }
    }
    let body_file = |function: &CFunctionRef| -> F {
        bodies
            .get(function)
            .copied()
            .unwrap_or(function.file())
            .key()
            .role
    };
    for node in registry.contextual_inventory() {
        let (key, role) = match node {
            R::Struct(v) => (v.key(), v.file().key().role),
            R::Union(v) => (v.key(), v.file().key().role),
            R::Enum(v) => (v.key(), v.file().key().role),
            R::Typedef(v) => (v.key(), v.file().key().role),
            R::Function(v) => (v.key(), v.file().key().role),
            R::Object(v) => (v.key(), v.file().key().role),
            R::Member(v) => (
                v.key(),
                aggregates
                    .get(v.owner())
                    .copied()
                    .unwrap_or(aggregate_file(v.owner()))
                    .key()
                    .role,
            ),
            R::Enumerator(v) => (v.key(), v.owner().file().key().role),
            R::MemberOwnership(_, v) => (
                v.member().key(),
                aggregates
                    .get(v.member().owner())
                    .copied()
                    .unwrap_or(aggregate_file(v.member().owner()))
                    .key()
                    .role,
            ),
            R::Parameter(v) => (v.key(), body_file(v.function())),
            R::Scope(v) => (v.key(), body_file(v.function())),
            R::Local(v) => (v.key(), body_file(v.scope().function())),
            R::OwnerSlot(v) => (v.local().key(), body_file(v.local().scope().function())),
            R::Loop(v) => (v.key(), body_file(v.scope().function())),
            R::Switch(v) => (v.key(), body_file(v.scope().function())),
            R::CleanupExit(v) => (v.key(), body_file(v.scope().function())),
            R::Allocation(v) => (v.key(), body_file(v.scope().function())),
            R::Witness(v) => (v.key(), v.record().file().key().role),
            R::Table(v) => (v.key(), v.object().file().key().role),
            R::Adapter(v) => (v.key(), v.table().object().file().key().role),
        };
        if !permitted(&key.origin, role) {
            return Err(E::OriginRoleMismatch);
        }
    }
    Ok(())
}
