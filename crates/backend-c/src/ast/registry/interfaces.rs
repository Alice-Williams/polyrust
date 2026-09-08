//! Interface bundle registrations retain exact CoreIR and target identities.
//!
//! These references are not conformance/ownership proofs. Stage 02D checks
//! callback ABI and body facts; mapping certificates authenticate CoreIR uses.

use portable_core_ir::{CoreImplementationId, CoreImplementationMethodId, CoreInterfaceMethodId};

use super::{CDeclarationKey, CFunctionRef, CObjectRef, CStructRef, identity::Identity};

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct CWitnessMethod {
    pub interface_method: CoreInterfaceMethodId,
    pub implementation_method: CoreImplementationMethodId,
    pub concrete: CFunctionRef,
    pub adapter: CFunctionRef,
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct CInterfaceWitnessRef {
    pub(super) identity: Identity,
    pub(super) implementation: CoreImplementationId,
    pub(super) interface: CStructRef,
    pub(super) record: CStructRef,
    pub(super) methods: Vec<CWitnessMethod>,
}

impl CInterfaceWitnessRef {
    pub fn key(&self) -> &CDeclarationKey {
        &self.identity.key
    }
    pub const fn implementation(&self) -> CoreImplementationId {
        self.implementation
    }
    pub fn interface(&self) -> &CStructRef {
        &self.interface
    }
    pub fn record(&self) -> &CStructRef {
        &self.record
    }
    pub fn methods(&self) -> &[CWitnessMethod] {
        &self.methods
    }
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct CInterfaceTableRef {
    pub(super) identity: Identity,
    pub(super) witness: CInterfaceWitnessRef,
    pub(super) object: CObjectRef,
    pub(super) clone_context: CFunctionRef,
    pub(super) drop_context: CFunctionRef,
}

impl CInterfaceTableRef {
    pub fn key(&self) -> &CDeclarationKey {
        &self.identity.key
    }
    pub fn witness(&self) -> &CInterfaceWitnessRef {
        &self.witness
    }
    pub fn object(&self) -> &CObjectRef {
        &self.object
    }
    pub fn clone_context(&self) -> &CFunctionRef {
        &self.clone_context
    }
    pub fn drop_context(&self) -> &CFunctionRef {
        &self.drop_context
    }
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct CInterfaceAdapterRef {
    pub(super) identity: Identity,
    pub(super) table: CInterfaceTableRef,
}

impl CInterfaceAdapterRef {
    pub fn key(&self) -> &CDeclarationKey {
        &self.identity.key
    }
    pub fn table(&self) -> &CInterfaceTableRef {
        &self.table
    }
    pub fn witness(&self) -> &CInterfaceWitnessRef {
        self.table.witness()
    }
}
