//! Executable mappings for the closed Rust-source subset, not support flags.
#[cfg(binary64_ast_probe)]
#[path = "../../../test/binary64_java_ast.rs"]
mod binary64_ast;
mod boolean_negation;
#[cfg(constant_import_probe)]
#[path = "../../../test/constant_import_java_ast.rs"]
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
#[path = "../../../test/public_constant_java_ast.rs"]
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
mod slots;
#[cfg(wrapping_contract)]
#[allow(dead_code, unused_imports)]
#[path = "../../../test/wrapping_contract.rs"]
mod wrapping_contract;

#[cfg(unit_contract)]
#[allow(dead_code, unused_imports)]
#[path = "../../../test/unit_contract.rs"]
mod unit_contract;
mod unit_effects;
#[cfg(wrapping_ast_probe)]
#[path = "../../../test/wrapping_java_ast.rs"]
mod wrapping_ast;
mod wrapping_negation;

#[cfg(local_constant_contract)]
#[allow(dead_code, unused_imports)]
#[path = "../../../test/local_constant_contract.rs"]
mod local_constant_contract;

#[cfg(local_constant_ast_probe)]
#[path = "../../../test/local_constant_java_ast.rs"]
mod local_constant_ast;

#[cfg(constant_ast_probe)]
#[path = "../../../test/constant_java_ast.rs"]
mod constant_ast;

#[cfg(constant_contract)]
#[allow(dead_code, unused_imports)]
#[path = "../../../test/constant_contract.rs"]
mod constant_contract;

#[cfg(eager_ast_probe)]
#[path = "../../../test/eager_java_ast.rs"]
mod eager_ast;

#[cfg(eager_contract)]
#[allow(dead_code, unused_imports)]
#[path = "../../../test/eager_contract.rs"]
mod eager_contract;

#[cfg(bitwise_ast_probe)]
#[path = "../../../test/bitwise_java_ast.rs"]
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
#[path = "../../../test/short_circuit_java_ast.rs"]
mod lazy_ast;

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
pub(crate) use eager_booleans::JavaEagerBooleans;
pub(crate) use entry_signatures::JavaEntrySignatures;
pub(crate) use function_signatures::JavaFunctionSignatures;
pub(crate) use integer_bitwise::JavaIntegerBitwise;
pub(crate) use lexical_control::JavaLexicalControl;
pub(crate) use literal_values::JavaLiteralValues;
pub(crate) use local_constants::JavaLocalConstants;
pub(crate) use object_types::JavaObjectTypes;
pub(crate) use public_constant_imports::{ImportState, JavaPublicConstantImports};
pub(crate) use public_constant_reads::JavaPublicConstantReads;
pub(crate) use public_constants::JavaPublicConstants;
pub(crate) use record_initializers::JavaRecordInitializers;
pub(crate) use resolved_places::JavaResolvedPlaces;
pub(crate) use scalar_comparisons::JavaScalarComparisons;
pub(crate) use scalar_constants::JavaScalarConstants;
pub(crate) use shared_borrows::JavaSharedBorrows;
pub(crate) use short_circuit_booleans::JavaShortCircuitBooleans;
pub(crate) use slots::{Bindings, Builder};
pub(crate) use unit_effects::JavaUnitEffects;
pub(crate) use wrapping_negation::JavaWrappingNegation;

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
    JavaShortCircuitBooleans,
    JavaIntegerBitwise,
    JavaEagerBooleans,
    JavaScalarConstants,
    JavaLocalConstants,
    JavaPublicConstants,
    JavaPublicConstantReads,
    JavaPublicConstantImports,
    JavaUnitEffects,
    JavaWrappingNegation,
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
        .short_circuit_booleans(JavaShortCircuitBooleans)
        .integer_bitwise(JavaIntegerBitwise)
        .eager_booleans(JavaEagerBooleans)
        .scalar_constants(JavaScalarConstants)
        .local_constants(JavaLocalConstants)
        .public_constants(JavaPublicConstants)
        .public_constant_reads(JavaPublicConstantReads)
        .public_constant_imports(JavaPublicConstantImports)
        .unit_effects(JavaUnitEffects)
        .wrapping_negation(JavaWrappingNegation)
        .build()
}
