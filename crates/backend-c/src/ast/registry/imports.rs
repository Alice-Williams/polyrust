//! Consumer-branded calls retain the exact immutable dependency certificate.
use super::{
    CCallableContractOrigin, CCallableContractRef, CFunctionRef, CRegistry, CRegistryError,
    identity::Identity,
};
use crate::dialect::CDependencyFunction;
use std::sync::Arc;

#[cfg(test)]
#[path = "../../tests/import_registration.rs"]
mod tests;

impl CRegistry {
    /// Import an actual public certificate witness, never a name/signature pair.
    /// The original header remains foreign: it is not a consumer output file.
    /// Registration alone does not discharge linking or composed safety checks.
    ///
    /// ```compile_fail
    /// use portable_backend_c::ast::{CRegistry, CFunctionRef};
    /// fn unchecked(registry: &mut CRegistry, function: CFunctionRef) {
    ///     registry.import_function(function).unwrap();
    /// }
    /// ```
    pub fn import_function(
        &mut self,
        dependency: CDependencyFunction,
    ) -> Result<CFunctionRef, CRegistryError> {
        let original = dependency.function();
        self.check_dependency_registration(
            original.key(),
            dependency.declaration(),
            dependency.symbol(),
            &dependency.package_identity(),
        )?;
        self.check_signature(dependency.signature())?;
        let value = CFunctionRef {
            identity: Identity::new(&self.scope, original.key().clone()),
            file: original.file().clone(),
            signature: Arc::new(dependency.signature().clone()),
            contract: CCallableContractRef {
                identity: Identity::new(&self.scope, original.key().clone()),
                file: original.file().clone(),
                origin: CCallableContractOrigin::CertifiedDependency,
                dependency: Some(dependency.authority()),
            },
        };
        self.imports.insert(value.clone(), dependency);
        Ok(value)
    }

    /// Separate from owned declaration/file inventories: no fake definitions.
    pub fn imported_functions(
        &self,
    ) -> impl Iterator<Item = (&CFunctionRef, &CDependencyFunction)> {
        self.imports.iter()
    }

    pub fn imported_function(
        &self,
        function: &CFunctionRef,
    ) -> Result<&CDependencyFunction, CRegistryError> {
        self.check_scope(&function.identity.scope)?;
        let proof = self
            .imports
            .get(function)
            .ok_or(CRegistryError::UnregisteredReference)?;
        let original = proof.function();
        if function.key() != original.key()
            || function.file() != original.file()
            || function.signature() != proof.signature()
            || function.contract().key() != original.key()
            || function.contract().file() != original.file()
            || function.contract().origin() != CCallableContractOrigin::CertifiedDependency
            || function.contract.dependency.as_ref() != Some(&proof.authority())
        {
            return Err(CRegistryError::CallableContractMismatch);
        }
        self.check_scope(&function.contract.identity.scope)?;
        Ok(proof)
    }

    pub(crate) fn check_owned_function(&self, value: &CFunctionRef) -> Result<(), CRegistryError> {
        self.check_scope(&value.identity.scope)?;
        if self.functions.contains(value) {
            Ok(())
        } else {
            Err(CRegistryError::UnregisteredReference)
        }
    }
}
