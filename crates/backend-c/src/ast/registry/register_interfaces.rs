//! Bundle identity comes from actual registrations, not matching spellings.

use std::collections::BTreeSet;

use portable_core_ir::CoreImplementationId;

use super::super::{CConstness, CObjectTypeKind};
use super::{
    CAggregateRef, CDeclarationKey, CFunctionRef, CInterfaceAdapterRef, CInterfaceTableRef,
    CInterfaceWitnessRef, CObjectRef, CRegistry, CRegistryError, CStructRef, CWitnessMethod,
    identity::Identity,
};

impl CRegistry {
    pub fn register_interface_witness(
        &mut self,
        key: CDeclarationKey,
        implementation: CoreImplementationId,
        interface: &CStructRef,
        record: &CStructRef,
        methods: Vec<CWitnessMethod>,
    ) -> Result<CInterfaceWitnessRef, CRegistryError> {
        self.check_aggregate(&CAggregateRef::Struct(interface.clone()))?;
        self.check_aggregate(&CAggregateRef::Struct(record.clone()))?;
        let mut interface_methods = BTreeSet::new();
        let mut implementation_methods = BTreeSet::new();
        for method in &methods {
            self.check_function(&method.concrete)?;
            self.check_function(&method.adapter)?;
            if !interface_methods.insert(method.interface_method)
                || !implementation_methods.insert(method.implementation_method)
            {
                return Err(CRegistryError::DuplicateRegistration);
            }
        }
        if self
            .witnesses
            .iter()
            .any(|old| old.key() == &key || old.implementation() == implementation)
        {
            return Err(CRegistryError::DuplicateRegistration);
        }
        let value = CInterfaceWitnessRef {
            identity: Identity::new(&self.scope, key),
            implementation,
            interface: interface.clone(),
            record: record.clone(),
            methods,
        };
        self.witnesses.insert(value.clone());
        Ok(value)
    }

    pub fn check_interface_witness(
        &self,
        value: &CInterfaceWitnessRef,
    ) -> Result<(), CRegistryError> {
        self.check_scope(&value.identity.scope)?;
        if self.witnesses.contains(value) {
            Ok(())
        } else {
            Err(CRegistryError::UnregisteredReference)
        }
    }

    pub fn register_interface_table(
        &mut self,
        key: CDeclarationKey,
        witness: &CInterfaceWitnessRef,
        object: &CObjectRef,
        clone_context: &CFunctionRef,
        drop_context: &CFunctionRef,
    ) -> Result<CInterfaceTableRef, CRegistryError> {
        self.check_interface_witness(witness)?;
        self.check_object(object)?;
        self.check_function(clone_context)?;
        self.check_function(drop_context)?;
        let ty = object.ty().canonical();
        if ty.constness() != CConstness::Const || !matches!(ty.kind(), CObjectTypeKind::Struct(_)) {
            return Err(CRegistryError::InterfaceTableType);
        }
        if self
            .tables
            .iter()
            .any(|old| old.key() == &key || old.witness() == witness || old.object() == object)
        {
            return Err(CRegistryError::DuplicateRegistration);
        }
        let value = CInterfaceTableRef {
            identity: Identity::new(&self.scope, key),
            witness: witness.clone(),
            object: object.clone(),
            clone_context: clone_context.clone(),
            drop_context: drop_context.clone(),
        };
        self.tables.insert(value.clone());
        Ok(value)
    }

    pub fn check_interface_table(&self, value: &CInterfaceTableRef) -> Result<(), CRegistryError> {
        self.check_scope(&value.identity.scope)?;
        if self.tables.contains(value) {
            Ok(())
        } else {
            Err(CRegistryError::UnregisteredReference)
        }
    }

    pub fn register_interface_adapter(
        &mut self,
        key: CDeclarationKey,
        table: &CInterfaceTableRef,
    ) -> Result<CInterfaceAdapterRef, CRegistryError> {
        self.check_interface_table(table)?;
        if self
            .adapters
            .iter()
            .any(|old| old.key() == &key || old.table() == table)
        {
            return Err(CRegistryError::DuplicateRegistration);
        }
        let value = CInterfaceAdapterRef {
            identity: Identity::new(&self.scope, key),
            table: table.clone(),
        };
        self.adapters.insert(value.clone());
        Ok(value)
    }

    pub fn check_interface_adapter(
        &self,
        value: &CInterfaceAdapterRef,
    ) -> Result<(), CRegistryError> {
        self.check_scope(&value.identity.scope)?;
        if self.adapters.contains(value) {
            Ok(())
        } else {
            Err(CRegistryError::UnregisteredReference)
        }
    }
}
