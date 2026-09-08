//! Nominal declarations are allocated once and defined at most once.

use std::{collections::BTreeSet, sync::Arc};

use super::super::CObjectType;
use super::{
    CAggregateRef, CDeclarationKey, CEnumRef, CEnumeratorRef, CFileRef, CFunctionRef,
    CMemberBinding, CMemberRef, CRegistry, CRegistryError, CStructRef, CTypedefRef, CUnionRef,
    identity::Identity,
};

macro_rules! register_tag {
    ($method:ident, $reference:ident, $field:ident) => {
        pub fn $method(
            &mut self,
            file: &CFileRef,
            key: CDeclarationKey,
        ) -> Result<$reference, CRegistryError> {
            self.check_file(file)?;
            if self.$field.keys().any(|old| old.key() == &key) {
                return Err(CRegistryError::DuplicateRegistration);
            }
            let value = $reference {
                identity: Identity::new(&self.scope, key),
                file: file.clone(),
            };
            self.$field.insert(value.clone(), None);
            Ok(value)
        }
    };
}

impl CRegistry {
    register_tag!(declare_struct, CStructRef, structs);
    register_tag!(declare_union, CUnionRef, unions);
    register_tag!(declare_enum, CEnumRef, enums);

    pub fn register_typedef(
        &mut self,
        file: &CFileRef,
        key: CDeclarationKey,
        target: CObjectType,
    ) -> Result<CTypedefRef, CRegistryError> {
        self.check_file(file)?;
        self.check_type(&target)?;
        if self.typedefs.iter().any(|old| old.key() == &key) {
            return Err(CRegistryError::DuplicateRegistration);
        }
        let value = CTypedefRef {
            identity: Identity::new(&self.scope, key),
            file: file.clone(),
            target: Arc::new(target),
        };
        self.typedefs.insert(value.clone());
        Ok(value)
    }

    pub fn register_member(
        &mut self,
        owner: &CAggregateRef,
        key: CDeclarationKey,
        ty: CObjectType,
    ) -> Result<CMemberRef, CRegistryError> {
        self.register_member_binding(owner, key, ty, CMemberBinding::Object)
    }

    pub fn register_callable_member(
        &mut self,
        owner: &CAggregateRef,
        key: CDeclarationKey,
        function: &CFunctionRef,
    ) -> Result<CMemberRef, CRegistryError> {
        self.check_function(function)?;
        let ty = CObjectType::pointer(super::super::CPointerTarget::Function(Box::new(
            function.signature().clone(),
        )));
        self.register_member_binding(
            owner,
            key,
            ty,
            CMemberBinding::Callable(function.contract().clone()),
        )
    }

    fn register_member_binding(
        &mut self,
        owner: &CAggregateRef,
        key: CDeclarationKey,
        ty: CObjectType,
        binding: CMemberBinding,
    ) -> Result<CMemberRef, CRegistryError> {
        self.check_aggregate(owner)?;
        self.check_type(&ty)?;
        ty.require_storable()
            .map_err(CRegistryError::InvalidObjectType)?;
        if self.members(owner)?.is_some() {
            return Err(CRegistryError::AlreadyDefined);
        }
        if self
            .members
            .iter()
            .any(|old| old.owner() == owner && old.key() == &key)
        {
            return Err(CRegistryError::DuplicateRegistration);
        }
        let value = CMemberRef {
            binding,
            identity: Identity::new(&self.scope, key),
            owner: owner.clone(),
            ty,
        };
        self.members.insert(value.clone());
        Ok(value)
    }

    pub fn define_aggregate(
        &mut self,
        owner: &CAggregateRef,
        members: Vec<CMemberRef>,
    ) -> Result<(), CRegistryError> {
        self.check_aggregate(owner)?;
        if self.members(owner)?.is_some() {
            return Err(CRegistryError::AlreadyDefined);
        }
        if members.is_empty() {
            return Err(CRegistryError::EmptyDefinition);
        }
        for member in &members {
            self.check_member(owner, member)?;
        }
        let actual: BTreeSet<_> = members.iter().collect();
        let registered: BTreeSet<_> = self
            .members
            .iter()
            .filter(|member| member.owner() == owner)
            .collect();
        if actual.len() != members.len() || actual != registered {
            return Err(CRegistryError::DefinitionInventoryMismatch);
        }
        let slot = match owner {
            CAggregateRef::Struct(value) => self.structs.get_mut(value),
            CAggregateRef::Union(value) => self.unions.get_mut(value),
        }
        .ok_or(CRegistryError::UnregisteredReference)?;
        *slot = Some(members);
        Ok(())
    }

    pub fn check_enum(&self, value: &CEnumRef) -> Result<(), CRegistryError> {
        self.check_scope(&value.identity.scope)?;
        if self.enums.contains_key(value) {
            Ok(())
        } else {
            Err(CRegistryError::UnregisteredReference)
        }
    }

    pub fn check_callable_member(
        &self,
        owner: &CAggregateRef,
        member: &CMemberRef,
        function: &CFunctionRef,
    ) -> Result<(), CRegistryError> {
        self.check_member(owner, member)?;
        self.check_function(function)?;
        match member.binding() {
            CMemberBinding::Callable(contract) if contract == function.contract() => Ok(()),
            CMemberBinding::Callable(_) | CMemberBinding::Object => {
                Err(CRegistryError::CallableContractMismatch)
            }
        }
    }

    pub fn register_enumerator(
        &mut self,
        owner: &CEnumRef,
        key: CDeclarationKey,
        value: i32,
    ) -> Result<CEnumeratorRef, CRegistryError> {
        self.check_enum(owner)?;
        if self.enums[owner].is_some() {
            return Err(CRegistryError::AlreadyDefined);
        }
        if self
            .enumerators
            .iter()
            .any(|old| old.owner() == owner && old.key() == &key)
        {
            return Err(CRegistryError::DuplicateRegistration);
        }
        let value = CEnumeratorRef {
            identity: Identity::new(&self.scope, key),
            owner: owner.clone(),
            value,
        };
        self.enumerators.insert(value.clone());
        Ok(value)
    }

    pub fn check_enumerator(&self, value: &CEnumeratorRef) -> Result<(), CRegistryError> {
        self.check_enum(value.owner())?;
        self.check_scope(&value.identity.scope)?;
        if self.enumerators.contains(value) {
            Ok(())
        } else {
            Err(CRegistryError::UnregisteredReference)
        }
    }

    pub fn define_enum(
        &mut self,
        owner: &CEnumRef,
        values: Vec<CEnumeratorRef>,
    ) -> Result<(), CRegistryError> {
        self.check_enum(owner)?;
        if self.enums[owner].is_some() {
            return Err(CRegistryError::AlreadyDefined);
        }
        if values.is_empty() {
            return Err(CRegistryError::EmptyDefinition);
        }
        for value in &values {
            self.check_scope(&value.identity.scope)?;
            if value.owner() != owner {
                return Err(CRegistryError::WrongOwner);
            }
        }
        let actual: BTreeSet<_> = values.iter().collect();
        let registered: BTreeSet<_> = self
            .enumerators
            .iter()
            .filter(|value| value.owner() == owner)
            .collect();
        if actual.len() != values.len() || actual != registered {
            return Err(CRegistryError::DefinitionInventoryMismatch);
        }
        *self
            .enums
            .get_mut(owner)
            .ok_or(CRegistryError::UnregisteredReference)? = Some(values);
        Ok(())
    }

    pub fn enumerators(
        &self,
        owner: &CEnumRef,
    ) -> Result<Option<&[CEnumeratorRef]>, CRegistryError> {
        self.check_enum(owner)?;
        Ok(self.enums[owner].as_deref())
    }
}
