//! Narrow Rust-source capabilities, not full portable catalogue support.
#[cfg(binary64_ast_probe)]
#[path = "../../../test/binary64_c_ast.rs"]
mod binary64_ast;
mod boolean_negation;
#[cfg(constant_import_probe)]
#[path = "../../../test/constant_import_c_ast.rs"]
mod constant_import_ast;
mod direct_calls;
mod eager_booleans;
mod entry_signatures;
mod function_signatures;
mod integer_bitwise;
mod lexical_control;
mod literal_values;
mod local_constants;
mod object_types;
#[cfg(public_constant_ast_probe)]
#[path = "../../../test/public_constant_c_ast.rs"]
mod public_constant_ast;
#[cfg(public_constant_contract)]
#[allow(dead_code, unused_imports)]
#[path = "../../../test/public_constant_contract.rs"]
mod public_constant_contract;
mod public_constant_imports;
mod public_constant_reads;
mod public_constants;
mod record_initializers;
mod resolved_places;
mod scalar_comparisons;
mod scalar_constants;
mod shared_borrows;
mod short_circuit_booleans;
mod signed_widening;
mod slots;
#[cfg(widening_contract)]
#[allow(dead_code, unused_imports)]
#[path = "../../../test/widening_contract.rs"]
mod widening_contract;
#[cfg(wrapping_contract)]
#[allow(dead_code, unused_imports)]
#[path = "../../../test/wrapping_contract.rs"]
mod wrapping_contract;

#[cfg(addition_ast_probe)]
#[path = "../../../test/addition_c_ast.rs"]
mod addition_ast;
#[cfg(addition_contract)]
#[allow(dead_code, unused_imports)]
#[path = "../../../test/addition_contract.rs"]
mod addition_contract;
mod floating_absolute;
mod floating_arithmetic;
mod floating_nan;
mod floating_negation;
mod floating_remainder;
mod floating_truncation;
#[cfg(unit_contract)]
#[allow(dead_code, unused_imports)]
#[path = "../../../test/unit_contract.rs"]
mod unit_contract;
mod unit_effects;
mod wrapping_addition;
#[cfg(wrapping_ast_probe)]
#[path = "../../../test/wrapping_c_ast.rs"]
mod wrapping_ast;
mod wrapping_multiplication;
mod wrapping_negation;
mod wrapping_subtraction;

#[cfg(local_constant_contract)]
#[allow(dead_code, unused_imports)]
#[path = "../../../test/local_constant_contract.rs"]
mod local_constant_contract;

#[cfg(local_constant_ast_probe)]
#[path = "../../../test/local_constant_c_ast.rs"]
mod local_constant_ast;

#[cfg(constant_ast_probe)]
#[path = "../../../test/constant_c_ast.rs"]
mod constant_ast;

#[cfg(constant_contract)]
#[allow(dead_code, unused_imports)]
#[path = "../../../test/constant_contract.rs"]
mod constant_contract;

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
pub(crate) use floating_absolute::CFloatingAbsolute;
pub(crate) use floating_arithmetic::CFloatingArithmetic;
pub(crate) use floating_nan::CFloatingNaN;
pub(crate) use floating_negation::CFloatingNegation;
pub(crate) use floating_remainder::CFloatingRemainder;
pub(crate) use floating_truncation::CFloatingTruncation;
pub(crate) use function_signatures::CFunctionSignatures;
pub(crate) use integer_bitwise::CIntegerBitwise;
pub(crate) use lexical_control::CLexicalControl;
pub(crate) use literal_values::CLiteralValues;
pub(crate) use local_constants::CLocalConstants;
pub(crate) use object_types::CObjectTypes;
pub(crate) use public_constant_imports::{CPublicConstantImports, ImportState};
pub(crate) use public_constant_reads::CPublicConstantReads;
pub(crate) use public_constants::CPublicConstants;
pub(crate) use record_initializers::CRecordInitializers;
pub(crate) use resolved_places::CResolvedPlaces;
pub(crate) use scalar_comparisons::CScalarComparisons;
pub(crate) use scalar_constants::CScalarConstants;
pub(crate) use shared_borrows::CSharedBorrows;
pub(crate) use short_circuit_booleans::CShortCircuitBooleans;
pub(crate) use signed_widening::CSignedWidening;
pub(crate) use slots::{Bindings, Builder};
pub(crate) use unit_effects::CUnitEffects;
pub(crate) use wrapping_addition::CWrappingAddition;
pub(crate) use wrapping_multiplication::CWrappingMultiplication;
pub(crate) use wrapping_negation::CWrappingNegation;
pub(crate) use wrapping_subtraction::CWrappingSubtraction;

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
    CScalarConstants,
    CLocalConstants,
    CPublicConstants,
    CPublicConstantReads,
    CPublicConstantImports,
    CUnitEffects,
    CWrappingNegation,
    CFloatingNegation,
    CFloatingNaN,
    CFloatingAbsolute,
    CFloatingTruncation,
    CFloatingArithmetic,
    CFloatingRemainder,
    CWrappingAddition,
    CWrappingSubtraction,
    CWrappingMultiplication,
    CSignedWidening,
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
        .scalar_constants(CScalarConstants)
        .local_constants(CLocalConstants)
        .public_constants(CPublicConstants)
        .public_constant_reads(CPublicConstantReads)
        .public_constant_imports(CPublicConstantImports)
        .unit_effects(CUnitEffects)
        .wrapping_negation(CWrappingNegation)
        .floating_negation(CFloatingNegation)
        .floating_nan(CFloatingNaN)
        .floating_absolute(CFloatingAbsolute)
        .floating_truncation(CFloatingTruncation)
        .floating_arithmetic(CFloatingArithmetic)
        .floating_remainder(CFloatingRemainder)
        .wrapping_addition(CWrappingAddition)
        .wrapping_subtraction(CWrappingSubtraction)
        .wrapping_multiplication(CWrappingMultiplication)
        .signed_widening(CSignedWidening)
        .build()
}

#[cfg(floating_ast_probe)]
#[path = "../../../test/floating_c_ast.rs"]
mod floating_ast;
#[cfg(floating_contract)]
#[allow(dead_code, unused_imports)]
#[path = "../../../test/floating_contract.rs"]
mod floating_contract;
#[cfg(widening_ast_probe)]
#[path = "../../../test/widening_c_ast.rs"]
mod widening_ast;

#[cfg(nan_contract)]
#[allow(dead_code, unused_imports)]
#[path = "../../../test/nan_contract.rs"]
mod nan_contract;

#[cfg(nan_ast_probe)]
#[path = "../../../test/nan_c_ast.rs"]
mod nan_ast;

#[cfg(absolute_contract)]
#[allow(dead_code, unused_imports)]
#[path = "../../../test/absolute_contract.rs"]
mod absolute_contract;

#[cfg(absolute_ast_probe)]
#[path = "../../../test/absolute_c_ast.rs"]
mod absolute_ast;

#[cfg(truncation_contract)]
#[allow(dead_code, unused_imports)]
#[path = "../../../test/truncation_contract.rs"]
mod truncation_contract;

#[cfg(truncation_ast_probe)]
#[path = "../../../test/truncation_c_ast.rs"]
mod truncation_ast;

#[cfg(arithmetic_contract)]
#[allow(dead_code, unused_imports)]
#[path = "../../../test/arithmetic_contract.rs"]
mod arithmetic_contract;

#[cfg(arithmetic_ast_probe)]
#[path = "../../../test/arithmetic_c_ast.rs"]
mod arithmetic_ast;

#[cfg(remainder_contract)]
#[allow(dead_code, unused_imports)]
#[path = "../../../test/remainder_contract.rs"]
mod remainder_contract;

#[cfg(remainder_ast_probe)]
#[path = "../../../test/remainder_c_ast.rs"]
mod remainder_ast;

#[cfg(subtraction_contract)]
#[allow(dead_code, unused_imports)]
#[path = "../../../test/subtraction_contract.rs"]
mod subtraction_contract;

#[cfg(subtraction_ast_probe)]
#[path = "../../../test/subtraction_c_ast.rs"]
mod subtraction_ast;

#[cfg(multiplication_contract)]
#[allow(dead_code, unused_imports)]
#[path = "../../../test/multiplication_contract.rs"]
mod multiplication_contract;

#[cfg(multiplication_ast_probe)]
#[path = "../../../test/multiplication_c_ast.rs"]
mod multiplication_ast;
