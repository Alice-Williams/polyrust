//! Read-only registry payload for the later shared unresolved C package.

use std::sync::Arc;

use super::CRegistry;

/// This is a frozen declaration inventory, NOT a verified AST certificate.
/// It cannot render and makes no capability, flow, ABI or ownership claims.
///
/// ```compile_fail
/// use portable_backend_c::ast::{CFrozenRegistry, CFileKey};
/// fn mutate(value: &CFrozenRegistry, key: CFileKey) {
///     value.registrations().register_file(key).unwrap();
/// }
/// ```
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CFrozenRegistry(Arc<CRegistry>);

impl CFrozenRegistry {
    pub fn registrations(&self) -> &CRegistry {
        &self.0
    }
}

impl CRegistry {
    pub fn freeze(self) -> CFrozenRegistry {
        CFrozenRegistry(Arc::new(self))
    }
}
