//! Independent negative controls for the new executable slot and checked input.
#[cfg(boolean_c)]
use super::CIntegerBitwise as Bits;
#[cfg(boolean_java)]
use super::JavaIntegerBitwise as Bits;
use super::*;
#[cfg(boolean_c)]
use super::{
    CBooleanNegation as Negate, CDirectCalls as Calls, CEntrySignatures as Entry,
    CFunctionSignatures as Functions, CLexicalControl as Control, CLiteralValues as Literals,
    CObjectTypes as Objects, CRecordInitializers as Records, CResolvedPlaces as Places,
    CScalarComparisons as Comparisons, CSharedBorrows as Borrows, CShortCircuitBooleans as Lazy,
};
#[cfg(boolean_java)]
use super::{
    JavaBooleanNegation as Negate, JavaDirectCalls as Calls, JavaEntrySignatures as Entry,
    JavaFunctionSignatures as Functions, JavaLexicalControl as Control,
    JavaLiteralValues as Literals, JavaObjectTypes as Objects, JavaRecordInitializers as Records,
    JavaResolvedPlaces as Places, JavaScalarComparisons as Comparisons,
    JavaSharedBorrows as Borrows, JavaShortCircuitBooleans as Lazy,
};
#[cfg(boolean_c)]
use crate::c_lower::Reader;
#[cfg(boolean_java)]
use crate::java_lower::Reader;
#[cfg(boolean_c)]
type Output = portable_backend_c::ast::CValue;
#[cfg(boolean_java)]
type Output = crate::java_lower::Value;

#[cfg(boolean_missing)]
fn missing() {
    Builder::new()
        .literal_values(Literals)
        .scalar_comparisons(Comparisons)
        .resolved_places(Places)
        .shared_borrows(Borrows)
        .object_types(Objects)
        .record_initializers(Records)
        .lexical_control(Control)
        .entry_signatures(Entry)
        .direct_calls(Calls)
        .function_signatures(Functions)
        .short_circuit_booleans(Lazy)
        .integer_bitwise(Bits)
        .build();
}

#[cfg(boolean_duplicate)]
fn duplicate() {
    Builder::new()
        .boolean_negation(Negate)
        .boolean_negation(Negate);
}

#[cfg(any(boolean_wrong_capability, boolean_wrong_context, boolean_wrong_output))]
#[derive(Clone, Copy)]
struct Wrong;
#[cfg(any(boolean_wrong_capability, boolean_wrong_context, boolean_wrong_output))]
impl Mapping for Wrong {
    #[cfg(boolean_wrong_capability)]
    type Capability = LiteralValues;
    #[cfg(not(boolean_wrong_capability))]
    type Capability = BooleanNegation;
    #[cfg(boolean_wrong_context)]
    type Context<'tcx> = ();
    #[cfg(not(boolean_wrong_context))]
    type Context<'tcx> = Reader<'tcx>;
    #[cfg(boolean_wrong_output)]
    type Output = ();
    #[cfg(not(boolean_wrong_output))]
    type Output = Output;

    fn lower<'tcx>(
        &self,
        _: &mut Self::Context<'tcx>,
        _: <Self::Capability as Capability>::Input<'tcx>,
    ) -> Result<Self::Output, String> {
        Err("deliberately wrong Boolean mapping".into())
    }
}

#[cfg(any(boolean_wrong_capability, boolean_wrong_context, boolean_wrong_output))]
fn wrong() {
    Builder::new().boolean_negation(Wrong);
}

#[cfg(boolean_wrong_input)]
fn wrong_input<'tcx>(reader: &mut Reader<'tcx>, expression: &'tcx rustc_hir::Expr<'tcx>) {
    let input = LiteralInput::read(reader.checked, expression).unwrap();
    let _ = Negate.lower(reader, input);
}

#[cfg(boolean_private_input)]
fn private_input<'tcx>(expression: &'tcx rustc_hir::Expr<'tcx>) {
    let _ = NegationInput {
        operand: expression,
    };
}
