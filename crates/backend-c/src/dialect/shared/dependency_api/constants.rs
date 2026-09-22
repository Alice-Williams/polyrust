//! Opaque constant evidence retains its independent producer certificate.
use super::{Authority, CDependencyAuthority, CDependencyPackage};
use crate::ast::{CFileRef, CIdentifier, CObjectRef, CObjectType, CScalarConstantValue};
use crate::dialect::CGeneratedHeader;
use portable_codegen::RustDeclarationId;
use std::{cmp::Ordering, sync::Arc};

/// An actual public constant, never a caller-supplied name/type/value triple.
///
/// ```compile_fail
/// use portable_backend_c::dialect::CDependencyConstant;
/// fn forge() -> CDependencyConstant {
///     CDependencyConstant {
///         authority: todo!(), declaration: todo!(), object: todo!(),
///         symbol: todo!(), value: todo!(), read_type: todo!(),
///     }
/// }
/// ```
#[derive(Clone, Debug)]
pub struct CDependencyConstant {
    authority: Arc<Authority>,
    declaration: RustDeclarationId,
    object: CObjectRef,
    symbol: CIdentifier,
    value: CScalarConstantValue,
    read_type: CObjectType,
}

impl CDependencyConstant {
    pub(super) fn new(
        authority: Arc<Authority>,
        declaration: RustDeclarationId,
        constant: super::inventory::Constant,
    ) -> Self {
        Self {
            authority,
            declaration,
            object: constant.object,
            symbol: constant.symbol,
            value: constant.value,
            read_type: constant.read_type,
        }
    }

    pub fn declaration(&self) -> RustDeclarationId {
        self.declaration
    }
    pub fn object(&self) -> &CObjectRef {
        &self.object
    }
    pub fn symbol(&self) -> &CIdentifier {
        &self.symbol
    }
    pub fn value(&self) -> &CScalarConstantValue {
        &self.value
    }
    pub fn read_type(&self) -> &CObjectType {
        &self.read_type
    }
    pub fn public_header(&self) -> &CGeneratedHeader {
        &self.authority.header
    }
    pub fn implementation(&self) -> &CFileRef {
        &self.authority.implementation
    }

    pub fn package_identity(&self) -> CDependencyPackage {
        CDependencyPackage {
            root: self.authority.root,
            header: self.authority.header.clone(),
            authority: self.authority(),
        }
    }

    fn authority(&self) -> CDependencyAuthority {
        CDependencyAuthority(self.authority.clone())
    }
}

impl PartialEq for CDependencyConstant {
    fn eq(&self, other: &Self) -> bool {
        self.cmp(other).is_eq()
    }
}
impl Eq for CDependencyConstant {}
impl PartialOrd for CDependencyConstant {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}
impl Ord for CDependencyConstant {
    fn cmp(&self, other: &Self) -> Ordering {
        self.declaration
            .cmp(&other.declaration)
            .then_with(|| self.object.cmp(&other.object))
            .then_with(|| self.symbol.cmp(&other.symbol))
            .then_with(|| self.value.cmp(&other.value))
            .then_with(|| self.read_type.cmp(&other.read_type))
            .then_with(|| self.authority().cmp(&other.authority()))
    }
}
