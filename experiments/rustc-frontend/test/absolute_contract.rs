//! Each negative changes exactly one absolute-value mapping boundary.
#[cfg(absolute_c)]
use super::CFloatingAbsolute as Floating;
#[cfg(absolute_java)]
use super::JavaFloatingAbsolute as Floating;
use super::*;
#[cfg(absolute_c)]
use crate::c_lower::Reader;
#[cfg(absolute_java)]
use crate::java_lower::Reader;
#[cfg(absolute_c)]
type Output = portable_backend_c::ast::CValue;
#[cfg(absolute_java)]
type Output = crate::java_lower::Value;

#[cfg(all(absolute_missing, absolute_c))]
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
        .floating_arithmetic(CFloatingArithmetic)
        .floating_remainder(CFloatingRemainder)
        .wrapping_addition(CWrappingAddition)
        .wrapping_subtraction(CWrappingSubtraction)
        .wrapping_multiplication(CWrappingMultiplication)
        .build();
}

#[cfg(all(absolute_missing, absolute_java))]
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
        .floating_arithmetic(JavaFloatingArithmetic)
        .floating_remainder(JavaFloatingRemainder)
        .wrapping_addition(JavaWrappingAddition)
        .wrapping_subtraction(JavaWrappingSubtraction)
        .wrapping_multiplication(JavaWrappingMultiplication)
        .build();
}

#[cfg(absolute_duplicate)]
fn duplicate() {
    Builder::new()
        .floating_absolute(Floating)
        .floating_absolute(Floating);
}
#[cfg(any(
    absolute_wrong_capability,
    absolute_wrong_context,
    absolute_wrong_output
))]
#[derive(Clone, Copy)]
struct Wrong;
#[cfg(any(
    absolute_wrong_capability,
    absolute_wrong_context,
    absolute_wrong_output
))]
impl Mapping for Wrong {
    #[cfg(absolute_wrong_capability)]
    type Capability = LiteralValues;
    #[cfg(not(absolute_wrong_capability))]
    type Capability = FloatingAbsolute;
    #[cfg(absolute_wrong_context)]
    type Context<'tcx> = ();
    #[cfg(not(absolute_wrong_context))]
    type Context<'tcx> = Reader<'tcx>;
    #[cfg(absolute_wrong_output)]
    type Output = ();
    #[cfg(not(absolute_wrong_output))]
    type Output = Output;
    fn lower<'tcx>(
        &self,
        _: &mut Self::Context<'tcx>,
        _: <Self::Capability as Capability>::Input<'tcx>,
    ) -> Result<Self::Output, String> {
        Err("deliberately wrong absolute-value mapping".into())
    }
}
#[cfg(any(
    absolute_wrong_capability,
    absolute_wrong_context,
    absolute_wrong_output
))]
fn wrong() {
    Builder::new().floating_absolute(Wrong);
}
#[cfg(absolute_wrong_input)]
fn wrong_input<'tcx>(reader: &mut Reader<'tcx>, expression: &'tcx rustc_hir::Expr<'tcx>) {
    let input = LiteralInput::read(reader.tcx, reader.checked, expression).unwrap();
    let _ = Floating.lower(reader, input);
}
#[cfg(absolute_private_input)]
fn private_input<'tcx>(expression: &'tcx rustc_hir::Expr<'tcx>) {
    let _ = AbsoluteInput {
        source: expression,
        receiver: expression,
        definition: expression.hir_id.owner.def_id.to_def_id(),
    };
}
