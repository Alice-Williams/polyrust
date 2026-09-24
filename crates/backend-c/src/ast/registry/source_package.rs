//! Selected source crate metadata, independent of owned declarations.
use super::{CFileRef, CFileRole, CPackageRegistration, CRegistry, CRegistryError};
use portable_codegen::RustCrateExports;
use std::sync::Arc;

/// Descriptive registration, NOT compiler evidence or a render certificate.
/// It deliberately does not manufacture a declaration or symbol.
///
/// ```compile_fail
/// use portable_backend_c::ast::{CSourcePackage, CFileRef};
/// fn retarget(package: &mut CSourcePackage, header: CFileRef) {
///     package.header = header;
/// }
/// ```
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CSourcePackage {
    header: CFileRef,
    exports: Arc<RustCrateExports>,
}

impl CSourcePackage {
    pub fn header(&self) -> &CFileRef {
        &self.header
    }

    pub fn exports(&self) -> &Arc<RustCrateExports> {
        &self.exports
    }
}

impl CRegistry {
    pub fn register_source_package(
        &mut self,
        header: &CFileRef,
        exports: Arc<RustCrateExports>,
    ) -> Result<(), CRegistryError> {
        self.check_file(header)?;
        if header.key().role != CFileRole::GeneratedPublicHeader {
            return Err(CRegistryError::WrongOwner);
        }
        if self.package_registration.is_some() {
            return Err(CRegistryError::DuplicateRegistration);
        }
        self.package_registration = Some(CPackageRegistration::Source(CSourcePackage {
            header: header.clone(),
            exports,
        }));
        Ok(())
    }

    pub fn source_package(&self) -> Option<&CSourcePackage> {
        match &self.package_registration {
            Some(CPackageRegistration::Source(package)) => Some(package),
            _ => None,
        }
    }
}
