//! Control and allocation identities do not themselves prove flow or ranges.

use super::super::CObjectType;
use super::{
    CDeclarationKey, CLocalRef, CParameterRef, CRegistry, CRegistryError, CScopeRef,
    identity::Identity,
};

macro_rules! control_reference {
    ($name:ident, $method:ident, $check:ident, $field:ident) => {
        #[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
        pub struct $name {
            identity: Identity,
            scope: CScopeRef,
        }
        impl $name {
            pub fn key(&self) -> &CDeclarationKey {
                &self.identity.key
            }
            pub fn scope(&self) -> &CScopeRef {
                &self.scope
            }
        }
        impl CRegistry {
            pub fn $method(
                &mut self,
                scope: &CScopeRef,
                key: CDeclarationKey,
            ) -> Result<$name, CRegistryError> {
                self.check_lexical_scope(scope)?;
                if self
                    .$field
                    .iter()
                    .any(|old| old.scope.function() == scope.function() && old.key() == &key)
                {
                    return Err(CRegistryError::DuplicateRegistration);
                }
                let value = $name {
                    identity: Identity::new(&self.scope, key),
                    scope: scope.clone(),
                };
                self.$field.insert(value.clone());
                Ok(value)
            }
            pub fn $check(&self, scope: &CScopeRef, value: &$name) -> Result<(), CRegistryError> {
                self.check_lexical_scope(scope)?;
                self.check_scope(&value.identity.scope)?;
                if scope.function() != value.scope.function() {
                    return Err(CRegistryError::WrongOwner);
                }
                if self.$field.contains(value) {
                    Ok(())
                } else {
                    Err(CRegistryError::UnregisteredReference)
                }
            }
        }
    };
}

control_reference!(CLoopRef, register_loop, check_loop, loops);
control_reference!(CSwitchRef, register_switch, check_switch, switches);
control_reference!(
    CCleanupExitRef,
    register_cleanup_exit,
    check_cleanup_exit,
    cleanup_exits
);

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum CAllocatorSource {
    Default,
    Parameter(CParameterRef),
    Local(CLocalRef),
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct CAllocationRef {
    identity: Identity,
    scope: CScopeRef,
    object_type: CObjectType,
    allocator: CAllocatorSource,
}

impl CAllocationRef {
    pub fn key(&self) -> &CDeclarationKey {
        &self.identity.key
    }
    pub fn scope(&self) -> &CScopeRef {
        &self.scope
    }
    pub fn object_type(&self) -> &CObjectType {
        &self.object_type
    }
    pub fn allocator(&self) -> &CAllocatorSource {
        &self.allocator
    }
}

impl CRegistry {
    pub fn register_allocation(
        &mut self,
        scope: &CScopeRef,
        key: CDeclarationKey,
        object_type: CObjectType,
        allocator: CAllocatorSource,
    ) -> Result<CAllocationRef, CRegistryError> {
        self.check_lexical_scope(scope)?;
        self.check_type(&object_type)?;
        match &allocator {
            CAllocatorSource::Default => {}
            CAllocatorSource::Parameter(value) => self.check_parameter(scope.function(), value)?,
            CAllocatorSource::Local(value) => self.check_local(scope.function(), value)?,
        }
        if self
            .allocations
            .iter()
            .any(|old| old.scope.function() == scope.function() && old.key() == &key)
        {
            return Err(CRegistryError::DuplicateRegistration);
        }
        let value = CAllocationRef {
            identity: Identity::new(&self.scope, key),
            scope: scope.clone(),
            object_type,
            allocator,
        };
        self.allocations.insert(value.clone());
        Ok(value)
    }

    pub fn check_allocation(
        &self,
        scope: &CScopeRef,
        value: &CAllocationRef,
    ) -> Result<(), CRegistryError> {
        self.check_lexical_scope(scope)?;
        self.check_scope(&value.identity.scope)?;
        if scope.function() != value.scope.function() {
            return Err(CRegistryError::WrongOwner);
        }
        if self.allocations.contains(value) {
            Ok(())
        } else {
            Err(CRegistryError::UnregisteredReference)
        }
    }
}
