//! Foreign readonly storage keeps producer evidence outside owned inventories.
use super::{CObjectRef, CRegistry, CRegistryError, identity::Identity};
use crate::dialect::CDependencyConstant;

impl CRegistry {
    /// Only an actual certified constant can be imported.
    ///
    /// ```compile_fail
    /// use portable_backend_c::ast::{CRegistry, CObjectRef};
    /// fn unchecked(registry: &mut CRegistry, object: CObjectRef) {
    ///     registry.import_constant(object).unwrap();
    /// }
    /// ```
    pub fn import_constant(
        &mut self,
        dependency: CDependencyConstant,
    ) -> Result<CObjectRef, CRegistryError> {
        let original = dependency.object();
        self.check_dependency_registration(
            original.key(),
            dependency.declaration(),
            dependency.symbol(),
            &dependency.package_identity(),
        )?;
        self.check_type(original.ty())?;
        let object = CObjectRef {
            identity: Identity::new(&self.scope, original.key().clone()),
            file: original.file().clone(),
            ty: original.ty().clone(),
        };
        self.constant_imports.insert(object.clone(), dependency);
        Ok(object)
    }

    pub fn imported_constants(&self) -> impl Iterator<Item = (&CObjectRef, &CDependencyConstant)> {
        self.constant_imports.iter()
    }

    pub fn imported_constant(
        &self,
        object: &CObjectRef,
    ) -> Result<&CDependencyConstant, CRegistryError> {
        self.check_scope(&object.identity.scope)?;
        let dependency = self
            .constant_imports
            .get(object)
            .ok_or(CRegistryError::UnregisteredReference)?;
        let original = dependency.object();
        if object.key() != original.key()
            || object.file() != original.file()
            || object.ty() != original.ty()
        {
            return Err(CRegistryError::UnregisteredReference);
        }
        Ok(dependency)
    }

    pub(crate) fn check_owned_object(&self, object: &CObjectRef) -> Result<(), CRegistryError> {
        self.check_scope(&object.identity.scope)?;
        if self.objects.contains(object) {
            Ok(())
        } else {
            Err(CRegistryError::UnregisteredReference)
        }
    }
}
