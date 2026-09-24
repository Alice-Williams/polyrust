//! Foreign nominal references are readable only through immutable certificates.
use super::{CAggregateRef, CRegistry, CRegistryError, CStructRef};
use crate::dialect::CDependencyStruct;

impl CRegistry {
    /// Import an exact producer type, retaining its original nominal identity.
    ///
    /// ```compile_fail
    /// use portable_backend_c::ast::{CRegistry, CStructRef};
    /// fn unchecked(registry: &mut CRegistry, record: CStructRef) {
    ///     registry.import_struct(record).unwrap();
    /// }
    /// ```
    pub fn import_struct(
        &mut self,
        proof: CDependencyStruct,
    ) -> Result<CStructRef, CRegistryError> {
        proof.authenticate()?;
        let record = proof.record();
        self.check_dependency_owner(&proof.package_identity())?;
        if self.structs.contains_key(record)
            || self.struct_imports.contains_key(record)
            || self
                .struct_imports
                .values()
                .any(|old| old.symbol() == proof.symbol())
            || record.file() != proof.public_header().file()
        {
            return Err(CRegistryError::DuplicateRegistration);
        }
        let record = record.clone();
        self.struct_imports.insert(record.clone(), proof);
        Ok(record)
    }

    pub fn imported_structs(&self) -> impl Iterator<Item = (&CStructRef, &CDependencyStruct)> {
        self.struct_imports.iter()
    }

    pub fn imported_struct(
        &self,
        record: &CStructRef,
    ) -> Result<&CDependencyStruct, CRegistryError> {
        let proof = self
            .struct_imports
            .get(record)
            .ok_or(CRegistryError::UnregisteredReference)?;
        proof.authenticate()?;
        if proof.record() != record
            || record.file() != proof.public_header().file()
            || proof.members().iter().any(|member| {
                member.owner() != &CAggregateRef::Struct(record.clone())
                    || proof.member_name(member).is_none()
            })
        {
            return Err(CRegistryError::WrongOwner);
        }
        Ok(proof)
    }
}
