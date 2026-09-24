//! Nominal layout evidence retains the exact producer certificate and members.
use super::{Authority, CDependencyAuthority, CDependencyPackage};
use crate::ast::{CIdentifier, CMemberRef, CStructRef};
use std::{collections::BTreeMap, sync::Arc};

#[cfg(test)]
#[path = "../../../tests/shared_nominal_authority.rs"]
mod tests;

/// An original public struct, never a caller-authored name or copied layout.
///
/// ```compile_fail
/// use portable_backend_c::dialect::CDependencyStruct;
/// fn forge() { let _ = CDependencyStruct {}; }
/// ```
#[derive(Clone, Debug)]
pub struct CDependencyStruct {
    authority: Arc<Authority>,
    export: StructExport,
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(super) struct StructExport {
    pub record: CStructRef,
    pub symbol: CIdentifier,
    pub members: Vec<CMemberRef>,
    pub names: BTreeMap<CMemberRef, CIdentifier>,
}

impl CDependencyStruct {
    pub(crate) fn authenticate(&self) -> Result<(), crate::ast::CRegistryError> {
        let exports = super::inventory::structs::collect(&self.authority.package)
            .map_err(|_| crate::ast::CRegistryError::WrongOwner)?;
        if exports.get(self.record()) != Some(&self.export)
            || self.record().file() != self.authority.header.file()
        {
            return Err(crate::ast::CRegistryError::WrongOwner);
        }
        Ok(())
    }

    pub(super) fn new(authority: Arc<Authority>, export: StructExport) -> Self {
        Self { authority, export }
    }

    pub fn record(&self) -> &CStructRef {
        &self.export.record
    }
    pub fn symbol(&self) -> &CIdentifier {
        &self.export.symbol
    }
    pub fn members(&self) -> &[CMemberRef] {
        &self.export.members
    }
    pub fn member_name(&self, member: &CMemberRef) -> Option<&CIdentifier> {
        self.export.names.get(member)
    }
    pub fn public_header(&self) -> &super::CGeneratedHeader {
        &self.authority.header
    }
    pub fn package_identity(&self) -> CDependencyPackage {
        CDependencyPackage {
            owner: self.authority.owner,
            header: self.authority.header.clone(),
            authority: CDependencyAuthority(self.authority.clone()),
        }
    }
}

impl PartialEq for CDependencyStruct {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.authority, &other.authority) && self.export == other.export
    }
}
impl Eq for CDependencyStruct {}
impl PartialOrd for CDependencyStruct {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}
impl Ord for CDependencyStruct {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.package_identity()
            .cmp(&other.package_identity())
            .then_with(|| self.export.cmp(&other.export))
    }
}
