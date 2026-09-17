//! Each negative changes exactly one floating-negation mapping boundary.
#[cfg(floating_c)]
use super::CFloatingNegation as Floating;
#[cfg(floating_java)]
use super::JavaFloatingNegation as Floating;
use super::*;
#[cfg(floating_c)]
use crate::c_lower::Reader;
#[cfg(floating_java)]
use crate::java_lower::Reader;
#[cfg(floating_c)]
type Output = portable_backend_c::ast::CValue;
#[cfg(floating_java)]
type Output = crate::java_lower::Value;

#[cfg(all(floating_missing, floating_c))]
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
        .build();
}

#[cfg(all(floating_missing, floating_java))]
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
        .build();
}

#[cfg(floating_duplicate)]
fn duplicate() {
    Builder::new()
        .floating_negation(Floating)
        .floating_negation(Floating);
}
#[cfg(any(
    floating_wrong_capability,
    floating_wrong_context,
    floating_wrong_output
))]
#[derive(Clone, Copy)]
struct Wrong;
#[cfg(any(
    floating_wrong_capability,
    floating_wrong_context,
    floating_wrong_output
))]
impl Mapping for Wrong {
    #[cfg(floating_wrong_capability)]
    type Capability = LiteralValues;
    #[cfg(not(floating_wrong_capability))]
    type Capability = FloatingNegation;
    #[cfg(floating_wrong_context)]
    type Context<'tcx> = ();
    #[cfg(not(floating_wrong_context))]
    type Context<'tcx> = Reader<'tcx>;
    #[cfg(floating_wrong_output)]
    type Output = ();
    #[cfg(not(floating_wrong_output))]
    type Output = Output;
    fn lower<'tcx>(
        &self,
        _: &mut Self::Context<'tcx>,
        _: <Self::Capability as Capability>::Input<'tcx>,
    ) -> Result<Self::Output, String> {
        Err("deliberately wrong floating-negation mapping".into())
    }
}
#[cfg(any(
    floating_wrong_capability,
    floating_wrong_context,
    floating_wrong_output
))]
fn wrong() {
    Builder::new().floating_negation(Wrong);
}
#[cfg(floating_wrong_input)]
fn wrong_input<'tcx>(reader: &mut Reader<'tcx>, expression: &'tcx rustc_hir::Expr<'tcx>) {
    let input = LiteralInput::read(reader.tcx, reader.checked, expression).unwrap();
    let _ = Floating.lower(reader, input);
}
#[cfg(floating_private_input)]
fn private_input<'tcx>(expression: &'tcx rustc_hir::Expr<'tcx>) {
    let _ = FloatingInput {
        source: expression,
        operand: expression,
    };
}
