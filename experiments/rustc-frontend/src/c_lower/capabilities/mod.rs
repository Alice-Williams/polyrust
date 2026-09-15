//! Narrow Rust-source capabilities, not full portable catalogue support.
mod boolean_negation;
mod direct_calls;
mod eager_booleans;
mod entry_signatures;
mod function_signatures;
mod integer_bitwise;
mod lexical_control;
mod literal_values;
mod object_types;
mod record_initializers;
mod resolved_places;
mod scalar_comparisons;
mod shared_borrows;
mod short_circuit_booleans;
mod slots;

#[cfg(eager_ast_probe)]
#[path = "../../../test/eager_c_ast.rs"]
mod eager_ast;

#[cfg(eager_contract)]
#[allow(dead_code, unused_imports)]
#[path = "../../../test/eager_contract.rs"]
mod eager_contract;

#[cfg(bitwise_ast_probe)]
#[path = "../../../test/bitwise_c_ast.rs"]
mod bitwise_ast;

#[cfg(bitwise_contract)]
#[allow(dead_code, unused_imports)]
#[path = "../../../test/bitwise_contract.rs"]
mod bitwise_contract;

#[cfg(literal_private_input)]
#[path = "../../../test/literal_private_input.rs"]
mod literal_private_input;

#[cfg(boolean_contract)]
#[allow(dead_code, unused_imports)]
#[path = "../../../test/boolean_negation_contract.rs"]
mod boolean_contract;

#[cfg(lazy_contract)]
#[allow(dead_code, unused_imports)]
#[path = "../../../test/short_circuit_contract.rs"]
mod lazy_contract;

#[cfg(lazy_ast_probe)]
#[path = "../../../test/short_circuit_c_ast.rs"]
mod lazy_ast;

#[cfg(mapping_contract_duplicate)]
#[path = "../../../test/capability_duplicate.rs"]
mod contract_duplicate;
#[cfg(mapping_contract_missing)]
#[path = "../../../test/capability_missing.rs"]
mod contract_missing;
#[cfg(mapping_contract_wrong_input)]
#[path = "../../../test/capability_wrong_input.rs"]
mod contract_wrong_input;
#[cfg(mapping_contract_wrong_mapping)]
#[path = "../../../test/capability_wrong_mapping.rs"]
mod contract_wrong_mapping;
#[cfg(mapping_contract_wrong_output)]
#[path = "../../../test/capability_wrong_output.rs"]
mod contract_wrong_output;

#[cfg(mapping_contract_wrong_context)]
#[path = "../../../test/capability_wrong_context.rs"]
mod contract_wrong_context;
#[cfg(mapping_contract_wrong_mapping_output)]
#[path = "../../../test/capability_wrong_mapping_output.rs"]
mod contract_wrong_mapping_output;

#[cfg(mapping_contract_target_scope)]
#[path = "../../../test/capability_target_scope.rs"]
mod contract_target_scope;

pub(crate) use boolean_negation::CBooleanNegation;
pub(crate) use direct_calls::CDirectCalls;
pub(crate) use eager_booleans::CEagerBooleans;
pub(crate) use entry_signatures::CEntrySignatures;
pub(crate) use function_signatures::CFunctionSignatures;
pub(crate) use integer_bitwise::CIntegerBitwise;
pub(crate) use lexical_control::CLexicalControl;
pub(crate) use literal_values::CLiteralValues;
pub(crate) use object_types::CObjectTypes;
pub(crate) use record_initializers::CRecordInitializers;
pub(crate) use resolved_places::CResolvedPlaces;
pub(crate) use scalar_comparisons::CScalarComparisons;
pub(crate) use shared_borrows::CSharedBorrows;
pub(crate) use short_circuit_booleans::CShortCircuitBooleans;
pub(crate) use slots::{Bindings, Builder};

pub(crate) use crate::source_capabilities::*;

pub(crate) type CBindings = Bindings<
    CLiteralValues,
    CScalarComparisons,
    CResolvedPlaces,
    CSharedBorrows,
    CObjectTypes,
    CRecordInitializers,
    CLexicalControl,
    CEntrySignatures,
    CDirectCalls,
    CFunctionSignatures,
    CBooleanNegation,
    CShortCircuitBooleans,
    CIntegerBitwise,
    CEagerBooleans,
>;

pub(crate) fn c_bindings() -> CBindings {
    Builder::new()
        .literal_values(CLiteralValues)
        .scalar_comparisons(CScalarComparisons)
        .resolved_places(CResolvedPlaces)
        .shared_borrows(CSharedBorrows)
        .object_types(CObjectTypes)
        .record_initializers(CRecordInitializers)
        .lexical_control(CLexicalControl)
        .entry_signatures(CEntrySignatures)
        .direct_calls(CDirectCalls)
        .function_signatures(CFunctionSignatures)
        .boolean_negation(CBooleanNegation)
        .short_circuit_booleans(CShortCircuitBooleans)
        .integer_bitwise(CIntegerBitwise)
        .eager_booleans(CEagerBooleans)
        .build()
}
