//! Each negative changes exactly one truncation mapping boundary.
#[cfg(truncation_c)]
use super::CFloatingTruncation as Floating;
#[cfg(truncation_java)]
use super::JavaFloatingTruncation as Floating;
use super::*;
#[cfg(truncation_c)]
use crate::c_lower::Reader;
#[cfg(truncation_java)]
use crate::java_lower::Reader;
#[cfg(truncation_c)]
type Output = portable_backend_c::ast::CValue;
#[cfg(truncation_java)]
type Output = crate::java_lower::Value;

#[cfg(all(truncation_missing, truncation_c))]
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
        .floating_absolute(CFloatingAbsolute)
        .floating_arithmetic(CFloatingArithmetic)
        .floating_remainder(CFloatingRemainder)
        .wrapping_addition(CWrappingAddition)
        .wrapping_subtraction(CWrappingSubtraction)
        .build();
}

#[cfg(all(truncation_missing, truncation_java))]
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
        .floating_absolute(JavaFloatingAbsolute)
        .floating_arithmetic(JavaFloatingArithmetic)
        .floating_remainder(JavaFloatingRemainder)
        .wrapping_addition(JavaWrappingAddition)
        .wrapping_subtraction(JavaWrappingSubtraction)
        .build();
}

#[cfg(truncation_duplicate)]
fn duplicate() {
    Builder::new()
        .floating_truncation(Floating)
        .floating_truncation(Floating);
}
#[cfg(any(
    truncation_wrong_capability,
    truncation_wrong_context,
    truncation_wrong_output
))]
#[derive(Clone, Copy)]
struct Wrong;
#[cfg(any(
    truncation_wrong_capability,
    truncation_wrong_context,
    truncation_wrong_output
))]
impl Mapping for Wrong {
    #[cfg(truncation_wrong_capability)]
    type Capability = LiteralValues;
    #[cfg(not(truncation_wrong_capability))]
    type Capability = FloatingTruncation;
    #[cfg(truncation_wrong_context)]
    type Context<'tcx> = ();
    #[cfg(not(truncation_wrong_context))]
    type Context<'tcx> = Reader<'tcx>;
    #[cfg(truncation_wrong_output)]
    type Output = ();
    #[cfg(not(truncation_wrong_output))]
    type Output = Output;
    fn lower<'tcx>(
        &self,
        _: &mut Self::Context<'tcx>,
        _: <Self::Capability as Capability>::Input<'tcx>,
    ) -> Result<Self::Output, String> {
        Err("deliberately wrong truncation-value mapping".into())
    }
}
#[cfg(any(
    truncation_wrong_capability,
    truncation_wrong_context,
    truncation_wrong_output
))]
fn wrong() {
    Builder::new().floating_truncation(Wrong);
}
#[cfg(truncation_wrong_input)]
fn wrong_input<'tcx>(reader: &mut Reader<'tcx>, expression: &'tcx rustc_hir::Expr<'tcx>) {
    let input = LiteralInput::read(reader.tcx, reader.checked, expression).unwrap();
    let _ = Floating.lower(reader, input);
}
#[cfg(truncation_private_input)]
fn private_input<'tcx>(expression: &'tcx rustc_hir::Expr<'tcx>) {
    let _ = TruncationInput {
        source: expression,
        receiver: expression,
        definition: expression.hir_id.owner.def_id.to_def_id(),
    };
}
