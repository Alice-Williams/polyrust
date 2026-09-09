//! Storage-specific proof consumers; intermediate checks cannot render output.
mod actions;
mod allocation_calls;
mod allocation_edges;
mod allocations;
mod expressions;
mod flow;
mod heap;
mod index_extents;
mod numeric;
mod places;
mod state;
mod values;

use super::{
    CSafetyError,
    context_facts::ContextFacts,
    numeric_flow::{NumericFacts, State as NumericState},
};
use crate::ast::{CRegistry, CSourceFile};

#[derive(Clone)]
struct Engine<'facts, 'ast> {
    context: &'facts ContextFacts<'ast>,
    site: Option<(
        &'facts crate::ast::contextual::flow_graph::Graph<'ast>,
        crate::ast::contextual::flow_graph::Point,
    )>,
    numeric: Option<NumericState<'ast>>,
}
impl Engine<'_, '_> {
    fn registry(&self) -> &CRegistry {
        self.context.registry()
    }
}

impl CRegistry {
    /// Checks derived storage, initialization, bounds and automatic lifetimes.
    /// Default allocation/null/release and fixed typed restoration are checked;
    /// dynamic extents and other call effects remain unresolved. Success is not a full ownership or
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
    ///
    /// ```compile_fail
    /// use portable_backend_c::ownership::storage::heap::Binding;
    /// fn forge() -> Binding { Binding::Unbound }
    /// ```
    ///
    /// ```compile_fail
    /// use portable_backend_c::ownership::numeric_flow::Places;
    /// struct FakeResolver;
    /// impl<'ast> Places<'ast> for FakeResolver {}
    /// ```
    pub fn check_storage_paths(&self, files: &[CSourceFile]) -> Result<(), CSafetyError> {
        let context = ContextFacts::check(self, files)?;
        flow::check(&context)
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
