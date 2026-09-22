//! Each negative changes exactly one wrapping-multiplication mapping boundary.
#[cfg(multiplication_c)]
use super::CWrappingMultiplication as Wrapping;
#[cfg(multiplication_java)]
use super::JavaWrappingMultiplication as Wrapping;
use super::*;
#[cfg(multiplication_c)]
use crate::c_lower::Reader;
#[cfg(multiplication_java)]
use crate::java_lower::Reader;
#[cfg(multiplication_c)]
type Output = portable_backend_c::ast::CValue;
#[cfg(multiplication_java)]
type Output = crate::java_lower::Value;

#[cfg(all(multiplication_missing, multiplication_c))]
fn missing() {
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
        .floating_truncation(CFloatingTruncation)
        .floating_absolute(CFloatingAbsolute)
        .floating_arithmetic(CFloatingArithmetic)
        .floating_remainder(CFloatingRemainder)
        .wrapping_addition(CWrappingAddition)
        .wrapping_subtraction(CWrappingSubtraction)
        .signed_widening(CSignedWidening)
        .build();
}

#[cfg(all(multiplication_missing, multiplication_java))]
fn missing() {
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
        .floating_negation(JavaFloatingNegation)
        .floating_nan(JavaFloatingNaN)
        .floating_truncation(JavaFloatingTruncation)
        .floating_absolute(JavaFloatingAbsolute)
        .floating_arithmetic(JavaFloatingArithmetic)
        .floating_remainder(JavaFloatingRemainder)
        .wrapping_addition(JavaWrappingAddition)
        .wrapping_subtraction(JavaWrappingSubtraction)
        .signed_widening(JavaSignedWidening)
        .build();
}

#[cfg(multiplication_duplicate)]
fn duplicate() {
    Builder::new()
        .wrapping_multiplication(Wrapping)
        .wrapping_multiplication(Wrapping);
}
#[cfg(any(
    multiplication_wrong_capability,
    multiplication_wrong_context,
    multiplication_wrong_output
))]
#[derive(Clone, Copy)]
struct Wrong;
#[cfg(any(
    multiplication_wrong_capability,
    multiplication_wrong_context,
    multiplication_wrong_output
))]
impl Mapping for Wrong {
    #[cfg(multiplication_wrong_capability)]
    type Capability = LiteralValues;
    #[cfg(not(multiplication_wrong_capability))]
    type Capability = WrappingMultiplication;
    #[cfg(multiplication_wrong_context)]
    type Context<'tcx> = ();
    #[cfg(not(multiplication_wrong_context))]
    type Context<'tcx> = Reader<'tcx>;
    #[cfg(multiplication_wrong_output)]
    type Output = ();
    #[cfg(not(multiplication_wrong_output))]
    type Output = Output;
    fn lower<'tcx>(
        &self,
        _: &mut Self::Context<'tcx>,
        _: <Self::Capability as Capability>::Input<'tcx>,
    ) -> Result<Self::Output, String> {
        Err("deliberately wrong wrapping-multiplication mapping".into())
    }
}
#[cfg(any(
    multiplication_wrong_capability,
    multiplication_wrong_context,
    multiplication_wrong_output
))]
fn wrong() {
    Builder::new().wrapping_multiplication(Wrong);
}
#[cfg(multiplication_wrong_input)]
fn wrong_input<'tcx>(reader: &mut Reader<'tcx>, expression: &'tcx rustc_hir::Expr<'tcx>) {
    let input = LiteralInput::read(reader.tcx, reader.checked, expression).unwrap();
    let _ = Wrapping.lower(reader, input);
}
#[cfg(multiplication_private_input)]
fn private_input<'tcx>(expression: &'tcx rustc_hir::Expr<'tcx>) {
    let _ = MultiplicationInput {
        source: expression,
        left: expression,
        right: expression,
        definition: expression.hir_id.owner.def_id.to_def_id(),
        width: MultiplicationWidth::I32,
    };
}
