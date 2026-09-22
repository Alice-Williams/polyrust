//! Public primitive constant witnesses retain the exact producer certificate.
use super::{JavaDependencyPackage, inventory::Constant};
use crate::ast::{JavaDeclaredPath, JavaScalarConstantValue, JavaType};
use portable_codegen::{GeneratedValueId, RustDeclarationId, RustSourceOrigin};
use std::sync::Arc;

/// A read-only constant exported by an exact certified Java owner.
///
/// ```compile_fail
/// use portable_backend_java::dialect::JavaDependencyConstant;
/// let forged = JavaDependencyConstant {};
/// ```
///
/// ```compile_fail
/// use portable_backend_java::{ast::ResolvedJavaFileItem, dialect::JavaDependencyConstant};
/// fn promote(item: ResolvedJavaFileItem) -> JavaDependencyConstant { item.into() }
/// ```
#[derive(Clone, Debug)]
pub struct JavaDependencyConstant {
    owner: JavaDependencyPackage,
    generated: GeneratedValueId,
    source: Arc<RustSourceOrigin>,
    path: JavaDeclaredPath,
    ty: JavaType,
    value: JavaScalarConstantValue,
}
impl JavaDependencyConstant {
    pub(super) fn new(owner: JavaDependencyPackage, value: Constant) -> Self {
        Self {
            owner,
            generated: value.generated,
            source: value.source,
            path: value.path,
            ty: value.ty,
            value: value.value,
        }
    }
    pub fn package_identity(&self) -> &JavaDependencyPackage {
        &self.owner
    }
    pub fn generated(&self) -> GeneratedValueId {
        self.generated
    }
    pub fn declaration(&self) -> RustDeclarationId {
        self.source.declaration
    }
    pub fn source(&self) -> &RustSourceOrigin {
        &self.source
    }
    pub fn path(&self) -> &JavaDeclaredPath {
        &self.path
    }
    pub fn ty(&self) -> &JavaType {
        &self.ty
    }
    pub fn value(&self) -> &JavaScalarConstantValue {
        &self.value
    }
}
impl PartialEq for JavaDependencyConstant {
    fn eq(&self, other: &Self) -> bool {
        self.owner == other.owner && self.declaration() == other.declaration()
    }
}
impl Eq for JavaDependencyConstant {}
impl PartialOrd for JavaDependencyConstant {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}
impl Ord for JavaDependencyConstant {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        (self.declaration(), &self.owner).cmp(&(other.declaration(), &other.owner))
    }
}
