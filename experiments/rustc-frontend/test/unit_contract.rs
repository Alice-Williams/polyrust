//! Each negative changes exactly one unit mapping boundary.
#[cfg(unit_c)]
use super::CUnitEffects as Unit;
#[cfg(unit_java)]
use super::JavaUnitEffects as Unit;
use super::*;
#[cfg(unit_c)]
use crate::c_lower::Reader;
#[cfg(unit_java)]
use crate::java_lower::Reader;
#[cfg(unit_c)]
type Output = Vec<portable_backend_c::ast::CStatement>;
#[cfg(unit_java)]
type Output = Vec<portable_backend_java::ast::JavaStmt>;

#[cfg(all(unit_missing, unit_c))]
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
        .wrapping_negation(CWrappingNegation)
        .floating_negation(CFloatingNegation)
        .floating_nan(CFloatingNaN)
        .build();
}

#[cfg(all(unit_missing, unit_java))]
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
        .wrapping_negation(JavaWrappingNegation)
        .floating_negation(JavaFloatingNegation)
        .floating_nan(JavaFloatingNaN)
        .build();
}

#[cfg(unit_duplicate)]
fn duplicate() {
    Builder::new().unit_effects(Unit).unit_effects(Unit);
}
#[cfg(any(unit_wrong_capability, unit_wrong_context, unit_wrong_output))]
#[derive(Clone, Copy)]
struct Wrong;
#[cfg(any(unit_wrong_capability, unit_wrong_context, unit_wrong_output))]
impl Mapping for Wrong {
    #[cfg(unit_wrong_capability)]
    type Capability = LiteralValues;
    #[cfg(not(unit_wrong_capability))]
    type Capability = UnitEffects;
    #[cfg(unit_wrong_context)]
    type Context<'tcx> = ();
    #[cfg(not(unit_wrong_context))]
    type Context<'tcx> = Reader<'tcx>;
    #[cfg(unit_wrong_output)]
    type Output = ();
    #[cfg(not(unit_wrong_output))]
    type Output = Output;
    fn lower<'tcx>(
        &self,
        _: &mut Self::Context<'tcx>,
        _: <Self::Capability as Capability>::Input<'tcx>,
    ) -> Result<Self::Output, String> {
        Err("deliberately wrong unit mapping".into())
    }
}
#[cfg(any(unit_wrong_capability, unit_wrong_context, unit_wrong_output))]
fn wrong() {
    Builder::new().unit_effects(Wrong);
}
#[cfg(unit_wrong_input)]
fn wrong_input<'tcx>(reader: &mut Reader<'tcx>, expression: &'tcx rustc_hir::Expr<'tcx>) {
    let input = LiteralInput::read(reader.tcx, reader.checked, expression).unwrap();
    let _ = Unit.lower(reader, input);
}
#[cfg(unit_private_input)]
fn private_input<'tcx>(expression: &'tcx rustc_hir::Expr<'tcx>) {
    let _ = UnitInput {
        operation: UnitOperation::Empty,
        scope: expression.hir_id,
    };
}
