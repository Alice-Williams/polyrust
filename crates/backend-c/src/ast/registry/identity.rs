//! Registry authentication is separate from deterministic source identity.

use std::{cmp::Ordering, fmt, sync::Arc};

use portable_core_ir::{CoreDeclaration, CoreExprId};

use super::super::CIdentifier;

/// Provenance is structured; a generated spelling is never a declaration ID.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum CGeneratedOrigin {
    CoreDeclaration(CoreDeclaration),
    CoreExpression(CoreExprId),
    Synthesized(CSynthesisReason),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum CSynthesisReason {
    Runtime,
    OwnershipAdapter,
    InterfaceAdapter,
    EvaluationTemporary,
    TestHarness,
    PlatformAssertion,
}

/// Ephemeral authentication only. Never serialize or use this order to name
/// symbols. Within one registry all brands compare equal; canonical inventories
/// explicitly project only stable declaration keys.
#[derive(Clone)]
pub(in crate::ast) struct RegistryScope(Arc<()>);

impl RegistryScope {
    pub(super) fn new() -> Self {
        Self(Arc::new(()))
    }
}

impl fmt::Debug for RegistryScope {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("RegistryScope")
    }
}

impl PartialEq for RegistryScope {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.0, &other.0)
    }
}

impl Eq for RegistryScope {}

impl PartialOrd for RegistryScope {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for RegistryScope {
    fn cmp(&self, other: &Self) -> Ordering {
        Arc::as_ptr(&self.0).cmp(&Arc::as_ptr(&other.0))
    }
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct CDeclarationKey {
    pub origin: CGeneratedOrigin,
    pub name: CIdentifier,
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(super) struct Identity {
    pub(super) key: CDeclarationKey,
    pub(super) scope: RegistryScope,
}

impl Identity {
    pub(super) fn new(scope: &RegistryScope, key: CDeclarationKey) -> Self {
        Self {
            key,
            scope: scope.clone(),
        }
    }
}
