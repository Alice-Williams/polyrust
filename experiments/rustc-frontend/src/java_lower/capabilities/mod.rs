//! Executable mappings for the closed Rust-source subset, not support flags.
mod boolean_negation;
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
mod slots;

#[cfg(boolean_contract)]
#[allow(dead_code, unused_imports)]
#[path = "../../../test/boolean_negation_contract.rs"]
mod boolean_contract;

#[cfg(java_contract_duplicate)]
#[path = "../../../test/java_capability_duplicate.rs"]
mod contract_duplicate;
#[cfg(java_contract_missing)]
#[path = "../../../test/java_capability_missing.rs"]
mod contract_missing;
#[cfg(any(
    java_contract_wrong_capability,
    java_contract_wrong_context,
    java_contract_wrong_output
))]
#[path = "../../../test/java_capability_signatures.rs"]
mod contract_signatures;
#[cfg(java_contract_wrong_input)]
#[path = "../../../test/java_capability_wrong_input.rs"]
mod contract_wrong_input;

pub(crate) use crate::source_capabilities::*;
pub(crate) use boolean_negation::JavaBooleanNegation;
pub(crate) use direct_calls::JavaDirectCalls;
pub(crate) use entry_signatures::JavaEntrySignatures;
pub(crate) use function_signatures::JavaFunctionSignatures;
pub(crate) use lexical_control::JavaLexicalControl;
pub(crate) use literal_values::JavaLiteralValues;
pub(crate) use object_types::JavaObjectTypes;
pub(crate) use record_initializers::JavaRecordInitializers;
pub(crate) use resolved_places::JavaResolvedPlaces;
pub(crate) use scalar_comparisons::JavaScalarComparisons;
pub(crate) use shared_borrows::JavaSharedBorrows;
pub(crate) use slots::{Bindings, Builder};

pub(crate) type JavaBindings = Bindings<
    JavaLiteralValues,
    JavaScalarComparisons,
    JavaResolvedPlaces,
    JavaSharedBorrows,
    JavaObjectTypes,
    JavaRecordInitializers,
    JavaLexicalControl,
    JavaEntrySignatures,
    JavaDirectCalls,
    JavaFunctionSignatures,
    JavaBooleanNegation,
>;

pub(crate) fn java_bindings() -> JavaBindings {
    Builder::new()
        .literal_values(JavaLiteralValues)
        .scalar_comparisons(JavaScalarComparisons)
        .resolved_places(JavaResolvedPlaces)
        .shared_borrows(JavaSharedBorrows)
        .object_types(JavaObjectTypes)
        .record_initializers(JavaRecordInitializers)
        .lexical_control(JavaLexicalControl)
        .entry_signatures(JavaEntrySignatures)
        .direct_calls(JavaDirectCalls)
        .function_signatures(JavaFunctionSignatures)
        .boolean_negation(JavaBooleanNegation)
        .build()
}
