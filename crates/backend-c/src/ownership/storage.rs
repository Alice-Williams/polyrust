//! Storage-specific proof consumers; intermediate checks cannot render output.
mod index_extents;

use super::{CSafetyError, numeric_flow::NumericFacts};
use crate::ast::{CRegistry, CSourceFile};

impl CRegistry {
    /// Checks numeric safety and actual fixed-array index extents.
    /// Pointer-based indices reject until their storage extent is established.
    /// Initialization, dereference validity, active members and ownership are
    /// separate obligations; this diagnostic success is not a safety certificate.
    ///
    /// ```compile_fail
    /// use portable_backend_c::ownership::numeric_flow::facts::indices::IndexObservation;
    /// fn forge<'a>() -> IndexObservation<'a, 'a> { todo!() }
    /// ```
    pub fn check_index_extents(&self, files: &[CSourceFile]) -> Result<(), CSafetyError> {
        let numeric = NumericFacts::check(self, files)?;
        index_extents::check(&numeric)
    }
}
