//! Requested allocation shapes retain typed origins, never trusted extent facts.
#[cfg(test)]
#[path = "../../tests/buffer_shape_reconstruction.rs"]
mod tests;
use super::super::CObjectType;
use super::CBufferCountRef;
use super::{
    CDeclarationKey, CLocalRef, CParameterRef, CRegistry, CRegistryError, CScopeRef,
    identity::Identity,
};

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum CAllocatorSource {
    Default,
    Parameter(CParameterRef),
    Local(CLocalRef),
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum CAllocationShape {
    Object,
    Elements(CBufferCountRef),
}

/// Requested shape fields are authenticated by the registry, not mutable claims.
///
/// ```compile_fail
/// use portable_backend_c::ast::{CAllocationRef, CAllocationShape};
/// fn forge(mut allocation: CAllocationRef, shape: CAllocationShape) { allocation.shape = shape; }
/// ```
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct CAllocationRef {
    identity: Identity,
    scope: CScopeRef,
    object_type: CObjectType,
    allocator: CAllocatorSource,
    shape: CAllocationShape,
}

impl CAllocationRef {
    pub fn shape(&self) -> &CAllocationShape {
        &self.shape
    }
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
        self.register_allocation_shape(scope, key, object_type, allocator, CAllocationShape::Object)
    }

    /// The typed count is a request, not evidence of actual allocated capacity.
    ///
    /// ```compile_fail
    /// use portable_backend_c::ast::*;
    /// fn arbitrary(r: &mut CRegistry, scope: &CScopeRef, key: CDeclarationKey, count: CLocalRef) {
    ///     r.register_buffer_allocation(scope, key, CObjectType::scalar(CScalarType::Int), count, CAllocatorSource::Default);
    /// }
    /// ```
    pub fn register_buffer_allocation(
        &mut self,
        scope: &CScopeRef,
        key: CDeclarationKey,
        element_type: CObjectType,
        count: CBufferCountRef,
        allocator: CAllocatorSource,
    ) -> Result<CAllocationRef, CRegistryError> {
        self.check_local(scope.function(), count.local())?;
        self.buffer_count_reference(count.local())?;
        self.register_allocation_shape(
            scope,
            key,
            element_type,
            allocator,
            CAllocationShape::Elements(count),
        )
    }

    fn register_allocation_shape(
        &mut self,
        scope: &CScopeRef,
        key: CDeclarationKey,
        object_type: CObjectType,
        allocator: CAllocatorSource,
        shape: CAllocationShape,
    ) -> Result<CAllocationRef, CRegistryError> {
        self.check_lexical_scope(scope)?;
        self.check_type(&object_type)?;
        object_type
            .require_storable()
            .map_err(CRegistryError::InvalidObjectType)?;
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
            shape,
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
