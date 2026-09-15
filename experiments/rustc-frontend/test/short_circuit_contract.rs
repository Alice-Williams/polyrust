//! Independent negative controls for the new executable slot and checked input.
use super::*;
#[cfg(lazy_c)]
use super::{
    CBooleanNegation as Negate, CDirectCalls as Calls, CEntrySignatures as Entry,
    CFunctionSignatures as Functions, CLexicalControl as Control, CLiteralValues as Literals,
    CObjectTypes as Objects, CRecordInitializers as Records, CResolvedPlaces as Places,
    CScalarComparisons as Comparisons, CSharedBorrows as Borrows, CShortCircuitBooleans as Lazy,
};
#[cfg(lazy_java)]
use super::{
    JavaBooleanNegation as Negate, JavaDirectCalls as Calls, JavaEntrySignatures as Entry,
    JavaFunctionSignatures as Functions, JavaLexicalControl as Control,
    JavaLiteralValues as Literals, JavaObjectTypes as Objects, JavaRecordInitializers as Records,
    JavaResolvedPlaces as Places, JavaScalarComparisons as Comparisons,
    JavaSharedBorrows as Borrows, JavaShortCircuitBooleans as Lazy,
};
#[cfg(lazy_c)]
use crate::c_lower::Reader;
#[cfg(lazy_java)]
use crate::java_lower::Reader;
#[cfg(lazy_c)]
type Output = portable_backend_c::ast::CValue;
#[cfg(lazy_java)]
type Output = crate::java_lower::Value;

#[cfg(lazy_missing)]
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
        .boolean_negation(Negate)
        .build();
}

#[cfg(lazy_duplicate)]
fn duplicate() {
    Builder::new()
        .short_circuit_booleans(Lazy)
        .short_circuit_booleans(Lazy);
}

#[cfg(any(lazy_wrong_capability, lazy_wrong_context, lazy_wrong_output))]
#[derive(Clone, Copy)]
struct Wrong;
#[cfg(any(lazy_wrong_capability, lazy_wrong_context, lazy_wrong_output))]
impl Mapping for Wrong {
    #[cfg(lazy_wrong_capability)]
    type Capability = LiteralValues;
    #[cfg(not(lazy_wrong_capability))]
    type Capability = ShortCircuitBooleans;
    #[cfg(lazy_wrong_context)]
    type Context<'tcx> = ();
    #[cfg(not(lazy_wrong_context))]
    type Context<'tcx> = Reader<'tcx>;
    #[cfg(lazy_wrong_output)]
    type Output = ();
    #[cfg(not(lazy_wrong_output))]
    type Output = Output;

    fn lower<'tcx>(
        &self,
        _: &mut Self::Context<'tcx>,
        _: <Self::Capability as Capability>::Input<'tcx>,
    ) -> Result<Self::Output, String> {
        Err("deliberately wrong Boolean mapping".into())
    }
}

#[cfg(any(lazy_wrong_capability, lazy_wrong_context, lazy_wrong_output))]
fn wrong() {
    Builder::new().short_circuit_booleans(Wrong);
}

#[cfg(lazy_wrong_input)]
fn wrong_input<'tcx>(reader: &mut Reader<'tcx>, expression: &'tcx rustc_hir::Expr<'tcx>) {
    let _ = Lazy.lower(reader, LiteralInput(expression));
}

#[cfg(lazy_private_input)]
fn private_input<'tcx>(expression: &'tcx rustc_hir::Expr<'tcx>) {
    let _ = LazyBooleanInput {
        operator: LazyBooleanOperator::And,
        left: expression,
        right: expression,
    };
}
