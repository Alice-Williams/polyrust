//! A count is an authenticated immutable size local, not a caller-proved range.
#[cfg(test)]
#[path = "../../tests/buffer_count_reconstruction.rs"]
mod tests;
use super::super::{CConstness, CObjectType, CObjectTypeKind, CScalarType};
use super::{CDeclarationKey, CLocalRef, CRegistry, CRegistryError as E, CScopeRef};
use std::sync::Arc;

/// A typed immutable size_t binding; declaration and numeric proof are separate.
///
/// ```compile_fail
/// use portable_backend_c::ast::{CBufferCountRef, CLocalRef};
/// fn forge(local: CLocalRef) -> CBufferCountRef { CBufferCountRef { local: std::sync::Arc::new(local) } }
/// ```
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct CBufferCountRef {
    local: Arc<CLocalRef>,
}
impl CBufferCountRef {
    pub fn local(&self) -> &CLocalRef {
        &self.local
    }
}
impl CRegistry {
    pub fn register_buffer_count(
        &mut self,
        scope: &CScopeRef,
        key: CDeclarationKey,
    ) -> Result<CBufferCountRef, E> {
        let ty = CObjectType::scalar(CScalarType::Size)
            .with_constness(CConstness::Const)
            .expect("a scalar accepts const qualification");
        let local = self.register_local(scope, key, ty)?;
        self.buffer_count_reference(&local)
    }
    pub fn buffer_count_reference(&self, local: &CLocalRef) -> Result<CBufferCountRef, E> {
        self.check_local(local.scope().function(), local)?;
        let ty = local.ty().canonical();
        if ty.constness() != CConstness::Const
            || ty.kind() != &CObjectTypeKind::Scalar(CScalarType::Size)
        {
            return Err(E::InvalidBufferCount);
        }
        Ok(CBufferCountRef {
            local: Arc::new(local.clone()),
        })
    }
}
