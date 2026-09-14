//! Shared compiler input contracts; target mappings own context and output.
mod contracts;
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

pub(crate) use contracts::{Capability, Mapping, Supports};
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
