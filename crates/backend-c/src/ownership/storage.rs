//! Storage-specific proof consumers; intermediate checks cannot render output.
mod actions;
mod allocation_calls;
mod allocation_edges;
mod allocations;
mod expressions;
mod flow;
mod index_extents;
mod places;
mod state;
mod values;

use super::{
    CSafetyError,
    numeric_flow::{Cursor, NumericFacts},
};
use crate::ast::{CRegistry, CSourceFile};

#[derive(Clone)]
struct Engine<'facts, 'ast> {
    facts: &'facts NumericFacts<'ast>,
    cursor: Option<Cursor<'ast>>,
}
impl Engine<'_, '_> {
    fn registry(&self) -> &CRegistry {
        self.facts.context().registry()
    }
}

impl CRegistry {
    /// Checks derived storage, initialization, bounds and automatic lifetimes.
    /// Default allocation/null/release flow is checked; typed restoration and
    /// other call effects remain unresolved. Success is not a full ownership or
    /// rendering certificate.
    ///
    /// ```compile_fail
    /// use portable_backend_c::ownership::storage::state::State;
    /// fn forge() -> State { State::default() }
    /// ```
    ///
    /// ```compile_fail
    /// use portable_backend_c::ownership::numeric_flow::AllocationOrigin;
    /// fn forge() -> AllocationOrigin { todo!() }
    /// ```
    pub fn check_storage_paths(&self, files: &[CSourceFile]) -> Result<(), CSafetyError> {
        let numeric = NumericFacts::check(self, files)?;
        flow::check(&numeric)
    }

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
