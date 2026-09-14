//! Shared compiler input contracts; target mappings own context and output.
mod direct_calls;
mod entry_signatures;
mod function_signatures;
mod lexical_control;
mod literal_values;
mod object_types;
mod record_initializers;
mod resolved_places;
mod scalar_comparisons;
mod shared_borrows;

pub(crate) use direct_calls::{CallInput, DirectCalls};
pub(crate) use entry_signatures::{EntryInput, EntrySignatures};
pub(crate) use function_signatures::{FunctionInput, FunctionSignatures};
pub(crate) use lexical_control::{ControlInput, LexicalControl};
pub(crate) use literal_values::{LiteralInput, LiteralValues};
pub(crate) use object_types::{ObjectTypes, TypeInput};
pub(crate) use record_initializers::{RecordInitializers, RecordInput};
pub(crate) use resolved_places::{PlaceInput, ResolvedPlaces};
pub(crate) use scalar_comparisons::{ComparisonInput, ScalarComparisons};
pub(crate) use shared_borrows::{BorrowInput, SharedBorrows};

type Result<T> = std::result::Result<T, String>;

/// Session-bound compiler input, independent of a target AST or reader.
pub(crate) trait Capability {
    type Input<'tcx>;
}

/// A stored executable implementation, never an independent support flag.
pub(crate) trait Mapping: Copy {
    type Capability: Capability;
    type Context<'tcx>;
    type Output;

    fn lower<'tcx>(
        &self,
        context: &mut Self::Context<'tcx>,
        input: <Self::Capability as Capability>::Input<'tcx>,
    ) -> Result<Self::Output>;
}

pub(crate) trait Supports<C: Capability> {
    type Mapping: Mapping<Capability = C>;
    fn mapping(&self) -> Self::Mapping;
}
