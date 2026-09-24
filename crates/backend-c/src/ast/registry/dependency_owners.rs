//! Values and calls share one authenticated producer namespace.
use super::{CDeclarationKey, CRegistry, CRegistryError};
use crate::{ast::CIdentifier, dialect::CDependencyPackage};
use portable_codegen::RustDeclarationId;

impl CRegistry {
    pub(crate) fn dependency_packages(
        &self,
    ) -> impl Iterator<Item = Result<CDependencyPackage, CRegistryError>> {
        self.imports
            .keys()
            .map(|function| {
                self.imported_function(function)
                    .map(|proof| proof.package_identity())
            })
            .chain(self.constant_imports.keys().map(|object| {
                self.imported_constant(object)
                    .map(|proof| proof.package_identity())
            }))
            .chain(self.struct_imports.keys().map(|record| {
                self.imported_struct(record)
                    .map(|proof| proof.package_identity())
            }))
    }

    pub(super) fn check_dependency_owner(
        &self,
        owner: &CDependencyPackage,
    ) -> Result<(), CRegistryError> {
        for old in self.dependency_packages() {
            let old = old?;
            if old != *owner
                && (old.conflicts_with(owner)
                    || old.public_header().include_path() == owner.public_header().include_path())
            {
                return Err(CRegistryError::DuplicateRegistration);
            }
        }
        if self.files.iter().any(|file| {
            owner
                .public_header()
                .conflicts_with_output_path(&file.key().path)
        }) {
            return Err(CRegistryError::DuplicateRegistration);
        }
        Ok(())
    }

    pub(super) fn check_dependency_registration(
        &self,
        key: &CDeclarationKey,
        declaration: RustDeclarationId,
        symbol: &CIdentifier,
        owner: &CDependencyPackage,
    ) -> Result<(), CRegistryError> {
        self.check_dependency_owner(owner)?;
        let old = self
            .imports
            .iter()
            .map(|(reference, proof)| {
                (
                    reference.key(),
                    proof.declaration(),
                    proof.symbol(),
                    proof.package_identity(),
                )
            })
            .chain(self.constant_imports.iter().map(|(reference, proof)| {
                (
                    reference.key(),
                    proof.declaration(),
                    proof.symbol(),
                    proof.package_identity(),
                )
            }));
        if self
            .functions
            .iter()
            .any(|reference| reference.key() == key)
            || self.objects.iter().any(|reference| reference.key() == key)
            || old
                .into_iter()
                .any(|(old_key, old_declaration, old_symbol, old_owner)| {
                    old_key == key
                        || old_declaration == declaration
                        || old_symbol == symbol
                        || old_owner.conflicts_with(owner)
                        || (old_owner.public_header().include_path()
                            == owner.public_header().include_path()
                            && old_owner != *owner)
                })
            || self.files.iter().any(|file| {
                owner
                    .public_header()
                    .conflicts_with_output_path(&file.key().path)
            })
        {
            return Err(CRegistryError::DuplicateRegistration);
        }
        Ok(())
    }
}
