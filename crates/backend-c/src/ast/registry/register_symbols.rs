//! Registration authenticates source references, not execution/flow facts.

use std::sync::Arc;

use super::super::{CConstness, CFunctionType, CObjectType};
use super::{
    CCallableContractOrigin, CCallableContractRef, CDeclarationKey, CFileRef, CFunctionRef,
    CLocalRef, CObjectRef, CParameterRef, CRegistry, CRegistryError, CScopeRef, identity::Identity,
};

impl CRegistry {
    pub fn register_function(
        &mut self,
        file: &CFileRef,
        key: CDeclarationKey,
        signature: CFunctionType,
    ) -> Result<CFunctionRef, CRegistryError> {
        self.check_file(file)?;
        self.check_signature(&signature)?;
        if self.functions.iter().any(|old| old.key() == &key) {
            return Err(CRegistryError::DuplicateRegistration);
        }
        let value = CFunctionRef {
            contract: CCallableContractRef {
                identity: Identity::new(&self.scope, key.clone()),
                file: file.clone(),
                origin: CCallableContractOrigin::GeneratedBody,
            },
            identity: Identity::new(&self.scope, key),
            file: file.clone(),
            signature: Arc::new(signature),
        };
        self.functions.insert(value.clone());
        Ok(value)
    }

    pub fn check_function(&self, value: &CFunctionRef) -> Result<(), CRegistryError> {
        self.check_scope(&value.identity.scope)?;
        if self.functions.contains(value) {
            Ok(())
        } else {
            Err(CRegistryError::UnregisteredReference)
        }
    }

    pub fn check_callable_contract(
        &self,
        value: &CCallableContractRef,
    ) -> Result<(), CRegistryError> {
        self.check_scope(&value.identity.scope)?;
        if self
            .functions
            .iter()
            .any(|function| function.contract() == value)
        {
            Ok(())
        } else {
            Err(CRegistryError::UnregisteredReference)
        }
    }

    pub fn register_object(
        &mut self,
        file: &CFileRef,
        key: CDeclarationKey,
        ty: CObjectType,
    ) -> Result<CObjectRef, CRegistryError> {
        self.check_file(file)?;
        self.check_type(&ty)?;
        if self.objects.iter().any(|old| old.key() == &key) {
            return Err(CRegistryError::DuplicateRegistration);
        }
        let value = CObjectRef {
            identity: Identity::new(&self.scope, key),
            file: file.clone(),
            ty,
        };
        self.objects.insert(value.clone());
        Ok(value)
    }

    pub fn check_object(&self, value: &CObjectRef) -> Result<(), CRegistryError> {
        self.check_scope(&value.identity.scope)?;
        if self.objects.contains(value) {
            Ok(())
        } else {
            Err(CRegistryError::UnregisteredReference)
        }
    }

    pub fn register_parameter(
        &mut self,
        function: &CFunctionRef,
        index: usize,
        key: CDeclarationKey,
        constness: CConstness,
    ) -> Result<CParameterRef, CRegistryError> {
        self.check_function(function)?;
        let parameter = function
            .signature()
            .parameters()
            .get(index)
            .ok_or(CRegistryError::ParameterIndex)?;
        if self
            .parameters
            .iter()
            .any(|old| old.function() == function && (old.index() == index || old.key() == &key))
        {
            return Err(CRegistryError::DuplicateRegistration);
        }
        let ty = parameter
            .ty()
            .clone()
            .with_constness(constness)
            .expect("prototype parameters are not arrays");
        let value = CParameterRef {
            identity: Identity::new(&self.scope, key),
            function: function.clone(),
            index,
            ty,
        };
        self.parameters.insert(value.clone());
        Ok(value)
    }

    pub fn check_parameter(
        &self,
        function: &CFunctionRef,
        value: &CParameterRef,
    ) -> Result<(), CRegistryError> {
        self.check_function(function)?;
        self.check_scope(&value.identity.scope)?;
        if value.function() != function {
            return Err(CRegistryError::WrongOwner);
        }
        if self.parameters.contains(value) {
            Ok(())
        } else {
            Err(CRegistryError::UnregisteredReference)
        }
    }

    pub fn register_scope(
        &mut self,
        function: &CFunctionRef,
        parent: Option<&CScopeRef>,
        key: CDeclarationKey,
    ) -> Result<CScopeRef, CRegistryError> {
        self.check_function(function)?;
        if let Some(parent) = parent {
            self.check_lexical_scope(parent)?;
            if parent.function() != function {
                return Err(CRegistryError::WrongOwner);
            }
        }
        if self
            .scopes
            .iter()
            .any(|old| old.function() == function && old.key() == &key)
        {
            return Err(CRegistryError::DuplicateRegistration);
        }
        let value = CScopeRef {
            identity: Identity::new(&self.scope, key),
            function: function.clone(),
            parent: parent.cloned().map(Arc::new),
        };
        self.scopes.insert(value.clone());
        Ok(value)
    }

    pub fn check_lexical_scope(&self, value: &CScopeRef) -> Result<(), CRegistryError> {
        self.check_scope(&value.identity.scope)?;
        if self.scopes.contains(value) {
            Ok(())
        } else {
            Err(CRegistryError::UnregisteredReference)
        }
    }

    pub fn register_local(
        &mut self,
        scope: &CScopeRef,
        key: CDeclarationKey,
        ty: CObjectType,
    ) -> Result<CLocalRef, CRegistryError> {
        self.check_lexical_scope(scope)?;
        self.check_type(&ty)?;
        if self
            .locals
            .iter()
            .any(|old| old.scope() == scope && old.key() == &key)
        {
            return Err(CRegistryError::DuplicateRegistration);
        }
        let value = CLocalRef {
            identity: Identity::new(&self.scope, key),
            scope: scope.clone(),
            ty,
        };
        self.locals.insert(value.clone());
        Ok(value)
    }

    pub fn check_local(
        &self,
        function: &CFunctionRef,
        value: &CLocalRef,
    ) -> Result<(), CRegistryError> {
        self.check_function(function)?;
        self.check_scope(&value.identity.scope)?;
        if value.scope().function() != function {
            return Err(CRegistryError::WrongOwner);
        }
        if self.locals.contains(value) {
            Ok(())
        } else {
            Err(CRegistryError::UnregisteredReference)
        }
    }
}
