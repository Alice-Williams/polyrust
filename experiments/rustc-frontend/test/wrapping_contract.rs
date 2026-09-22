//! Each negative changes exactly one wrapping-negation mapping boundary.
#[cfg(wrapping_c)]
use super::CWrappingNegation as Wrapping;
#[cfg(wrapping_java)]
use super::JavaWrappingNegation as Wrapping;
use super::*;
#[cfg(wrapping_c)]
use crate::c_lower::Reader;
#[cfg(wrapping_java)]
use crate::java_lower::Reader;
#[cfg(wrapping_c)]
type Output = portable_backend_c::ast::CValue;
#[cfg(wrapping_java)]
type Output = crate::java_lower::Value;

#[cfg(all(wrapping_missing, wrapping_c))]
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
        .floating_negation(CFloatingNegation)
        .floating_nan(CFloatingNaN)
        .floating_truncation(CFloatingTruncation)
        .floating_absolute(CFloatingAbsolute)
        .floating_arithmetic(CFloatingArithmetic)
        .floating_remainder(CFloatingRemainder)
        .wrapping_addition(CWrappingAddition)
        .wrapping_subtraction(CWrappingSubtraction)
        .wrapping_multiplication(CWrappingMultiplication)
        .signed_widening(CSignedWidening)
        .build();
}

#[cfg(all(wrapping_missing, wrapping_java))]
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
        .floating_negation(JavaFloatingNegation)
        .floating_nan(JavaFloatingNaN)
        .floating_truncation(JavaFloatingTruncation)
        .floating_absolute(JavaFloatingAbsolute)
        .floating_arithmetic(JavaFloatingArithmetic)
        .floating_remainder(JavaFloatingRemainder)
        .wrapping_addition(JavaWrappingAddition)
        .wrapping_subtraction(JavaWrappingSubtraction)
        .wrapping_multiplication(JavaWrappingMultiplication)
        .signed_widening(JavaSignedWidening)
        .build();
}

#[cfg(wrapping_duplicate)]
fn duplicate() {
    Builder::new()
        .wrapping_negation(Wrapping)
        .wrapping_negation(Wrapping);
}
#[cfg(any(
    wrapping_wrong_capability,
    wrapping_wrong_context,
    wrapping_wrong_output
))]
#[derive(Clone, Copy)]
struct Wrong;
#[cfg(any(
    wrapping_wrong_capability,
    wrapping_wrong_context,
    wrapping_wrong_output
))]
impl Mapping for Wrong {
    #[cfg(wrapping_wrong_capability)]
    type Capability = LiteralValues;
    #[cfg(not(wrapping_wrong_capability))]
    type Capability = WrappingNegation;
    #[cfg(wrapping_wrong_context)]
    type Context<'tcx> = ();
    #[cfg(not(wrapping_wrong_context))]
    type Context<'tcx> = Reader<'tcx>;
    #[cfg(wrapping_wrong_output)]
    type Output = ();
    #[cfg(not(wrapping_wrong_output))]
    type Output = Output;
    fn lower<'tcx>(
        &self,
        _: &mut Self::Context<'tcx>,
        _: <Self::Capability as Capability>::Input<'tcx>,
    ) -> Result<Self::Output, String> {
        Err("deliberately wrong wrapping-negation mapping".into())
    }
}
#[cfg(any(
    wrapping_wrong_capability,
    wrapping_wrong_context,
    wrapping_wrong_output
))]
fn wrong() {
    Builder::new().wrapping_negation(Wrong);
}
#[cfg(wrapping_wrong_input)]
fn wrong_input<'tcx>(reader: &mut Reader<'tcx>, expression: &'tcx rustc_hir::Expr<'tcx>) {
    let input = LiteralInput::read(reader.tcx, reader.checked, expression).unwrap();
    let _ = Wrapping.lower(reader, input);
}
#[cfg(wrapping_private_input)]
fn private_input<'tcx>(expression: &'tcx rustc_hir::Expr<'tcx>) {
    let _ = WrappingInput {
        source: expression,
        receiver: expression,
        definition: expression.hir_id.owner.def_id.to_def_id(),
        width: WrappingWidth::I32,
    };
}
