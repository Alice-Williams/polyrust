//! Each negative changes exactly one signed-widening mapping boundary.
#[cfg(widening_c)]
use super::CSignedWidening as Widening;
#[cfg(widening_java)]
use super::JavaSignedWidening as Widening;
use super::*;
#[cfg(widening_c)]
use crate::c_lower::Reader;
#[cfg(widening_java)]
use crate::java_lower::Reader;
#[cfg(widening_c)]
type Output = portable_backend_c::ast::CValue;
#[cfg(widening_java)]
type Output = crate::java_lower::Value;

#[cfg(all(widening_missing, widening_c))]
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
        .wrapping_multiplication(CWrappingMultiplication)
        .build();
}

#[cfg(all(widening_missing, widening_java))]
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
        .wrapping_multiplication(JavaWrappingMultiplication)
        .build();
}

#[cfg(widening_duplicate)]
fn duplicate() {
    Builder::new()
        .signed_widening(Widening)
        .signed_widening(Widening);
}
#[cfg(any(
    widening_wrong_capability,
    widening_wrong_context,
    widening_wrong_output
))]
#[derive(Clone, Copy)]
struct Wrong;
#[cfg(any(
    widening_wrong_capability,
    widening_wrong_context,
    widening_wrong_output
))]
impl Mapping for Wrong {
    #[cfg(widening_wrong_capability)]
    type Capability = LiteralValues;
    #[cfg(not(widening_wrong_capability))]
    type Capability = SignedWidening;
    #[cfg(widening_wrong_context)]
    type Context<'tcx> = ();
    #[cfg(not(widening_wrong_context))]
    type Context<'tcx> = Reader<'tcx>;
    #[cfg(widening_wrong_output)]
    type Output = ();
    #[cfg(not(widening_wrong_output))]
    type Output = Output;
    fn lower<'tcx>(
        &self,
        _: &mut Self::Context<'tcx>,
        _: <Self::Capability as Capability>::Input<'tcx>,
    ) -> Result<Self::Output, String> {
        Err("deliberately wrong signed-widening mapping".into())
    }
}
#[cfg(any(
    widening_wrong_capability,
    widening_wrong_context,
    widening_wrong_output
))]
fn wrong() {
    Builder::new().signed_widening(Wrong);
}
#[cfg(widening_wrong_input)]
fn wrong_input<'tcx>(reader: &mut Reader<'tcx>, expression: &'tcx rustc_hir::Expr<'tcx>) {
    let input = LiteralInput::read(reader.tcx, reader.checked, expression).unwrap();
    let _ = Widening.lower(reader, input);
}
#[cfg(widening_private_input)]
fn private_input<'tcx>(expression: &'tcx rustc_hir::Expr<'tcx>) {
    let _ = WideningInput {
        source: expression,
        operand: expression,
    };
}
