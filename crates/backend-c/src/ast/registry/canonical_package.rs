//! Closed descriptive type-owner registration; never compiler evidence.
use super::{
    CAggregateRef, CDeclarationKey, CFileRef, CFileRole, CGeneratedOrigin, CMemberRef,
    CPackageRegistration, CRegistry, CRegistryError, CStructRef, CSynthesisReason,
};
use crate::ast::{CIdentifier, CNameError};
use portable_codegen::{RustCanonicalErrorKindFacts, RustCanonicalInstanceKey, TargetPackageOwner};

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum CCanonicalTypeProfile {
    ScalarResultV2,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum CCanonicalTypeRole {
    Result,
    SuccessTag,
    Value,
}

impl CCanonicalTypeProfile {
    pub fn basename(self, key: RustCanonicalInstanceKey) -> String {
        match self {
            Self::ScalarResultV2 => format!(
                "polyrust_t2_c{:016x}_r{:016x}_i32_e{:016x}",
                key.result_definition().crate_id,
                key.result_definition().definition_path_hash,
                key.error_definition().definition_path_hash
            ),
        }
    }

    pub fn declaration_key(
        self,
        key: RustCanonicalInstanceKey,
        role: CCanonicalTypeRole,
    ) -> Result<CDeclarationKey, CNameError> {
        let suffix = match role {
            CCanonicalTypeRole::Result => "result",
            CCanonicalTypeRole::SuccessTag => "success",
            CCanonicalTypeRole::Value => "value",
        };
        Ok(CDeclarationKey {
            origin: CGeneratedOrigin::Synthesized(CSynthesisReason::OwnershipAdapter),
            name: CIdentifier::new(&format!("{}_{}", self.basename(key), suffix))?,
        })
    }
}

/// The immutable descriptor binds original facts to registered target roles.
/// It is not a compiler witness, source crate or RenderReadyPackage.
///
/// ```compile_fail
/// use portable_backend_c::ast::{CCanonicalTypePackage, CFileRef};
/// fn retarget(package: &mut CCanonicalTypePackage, file: CFileRef) {
///     package.header = file;
/// }
/// ```
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CCanonicalTypePackage {
    header: CFileRef,
    implementation: CFileRef,
    profile: CCanonicalTypeProfile,
    facts: RustCanonicalErrorKindFacts,
    record: CStructRef,
    tag: CMemberRef,
    value: CMemberRef,
}

impl CCanonicalTypePackage {
    pub fn header(&self) -> &CFileRef {
        &self.header
    }
    pub fn implementation(&self) -> &CFileRef {
        &self.implementation
    }
    pub fn profile(&self) -> CCanonicalTypeProfile {
        self.profile
    }
    pub fn facts(&self) -> RustCanonicalErrorKindFacts {
        self.facts
    }
    pub fn record(&self) -> &CStructRef {
        &self.record
    }
    pub fn tag(&self) -> &CMemberRef {
        &self.tag
    }
    pub fn value(&self) -> &CMemberRef {
        &self.value
    }
    pub fn owner(&self) -> TargetPackageOwner<CCanonicalTypeProfile> {
        TargetPackageOwner::CanonicalInstance {
            instance: self.facts.instance().key(),
            profile: self.profile,
        }
    }
}

impl CRegistry {
    pub(crate) fn package_header(&self) -> Option<&CFileRef> {
        match &self.package_registration {
            Some(CPackageRegistration::Source(package)) => Some(package.header()),
            Some(CPackageRegistration::Canonical(package)) => Some(package.header()),
            None => None,
        }
    }

    pub fn register_canonical_type_package(
        &mut self,
        header: &CFileRef,
        implementation: &CFileRef,
        profile: CCanonicalTypeProfile,
        facts: RustCanonicalErrorKindFacts,
        record: &CStructRef,
    ) -> Result<(), CRegistryError> {
        if self.package_registration.is_some() {
            return Err(CRegistryError::DuplicateRegistration);
        }
        self.check_file(header)?;
        self.check_file(implementation)?;
        let owner = CAggregateRef::Struct(record.clone());
        self.check_owned_aggregate(&owner)?;
        if header.key().role != CFileRole::GeneratedPublicHeader
            || implementation.key().role != CFileRole::GeneratedSource
            || record.file() != header
        {
            return Err(CRegistryError::WrongOwner);
        }
        let Some([tag, value]) = self.members(&owner)? else {
            return Err(CRegistryError::DefinitionInventoryMismatch);
        };
        let descriptor = CCanonicalTypePackage {
            header: header.clone(),
            implementation: implementation.clone(),
            profile,
            facts,
            record: record.clone(),
            tag: tag.clone(),
            value: value.clone(),
        };
        self.package_registration = Some(CPackageRegistration::Canonical(Box::new(descriptor)));
        Ok(())
    }

    pub fn canonical_type_package(&self) -> Option<&CCanonicalTypePackage> {
        match &self.package_registration {
            Some(CPackageRegistration::Canonical(package)) => Some(package),
            _ => None,
        }
    }
}
