//! Independent negative controls for the new executable slot and checked input.
#[cfg(local_constant_c)]
use super::CIntegerBitwise as IntegerBits;
#[cfg(local_constant_c)]
use super::CScalarConstants as ScalarReads;
#[cfg(local_constant_c)]
use super::CUnitEffects as Unit;
#[cfg(local_constant_c)]
use super::CWrappingNegation as Wrapping;
#[cfg(local_constant_java)]
use super::JavaIntegerBitwise as IntegerBits;
#[cfg(local_constant_java)]
use super::JavaScalarConstants as ScalarReads;
#[cfg(local_constant_java)]
use super::JavaUnitEffects as Unit;
#[cfg(local_constant_java)]
use super::JavaWrappingNegation as Wrapping;
use super::*;
#[cfg(local_constant_c)]
use super::{
    CBooleanNegation as Negate, CDirectCalls as Calls, CEagerBooleans as Eager,
    CEntrySignatures as Entry, CFunctionSignatures as Functions, CLexicalControl as Control,
    CLiteralValues as Literals, CLocalConstants as Constants, CObjectTypes as Objects,
    CRecordInitializers as Records, CResolvedPlaces as Places, CScalarComparisons as Comparisons,
    CSharedBorrows as Borrows, CShortCircuitBooleans as Lazy,
};
#[cfg(local_constant_c)]
use super::{
    CPublicConstantImports as PublicImports, CPublicConstantReads as PublicReads,
    CPublicConstants as PublicDeclarations,
};
#[cfg(local_constant_java)]
use super::{
    JavaBooleanNegation as Negate, JavaDirectCalls as Calls, JavaEagerBooleans as Eager,
    JavaEntrySignatures as Entry, JavaFunctionSignatures as Functions,
    JavaLexicalControl as Control, JavaLiteralValues as Literals, JavaLocalConstants as Constants,
    JavaObjectTypes as Objects, JavaRecordInitializers as Records, JavaResolvedPlaces as Places,
    JavaScalarComparisons as Comparisons, JavaSharedBorrows as Borrows,
    JavaShortCircuitBooleans as Lazy,
};
#[cfg(local_constant_java)]
use super::{
    JavaPublicConstantImports as PublicImports, JavaPublicConstantReads as PublicReads,
    JavaPublicConstants as PublicDeclarations,
};
#[cfg(local_constant_c)]
use crate::c_lower::Reader;
#[cfg(local_constant_java)]
use crate::java_lower::Reader;
type Output = ();

#[cfg(local_constant_missing)]
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
        .short_circuit_booleans(Lazy)
        .integer_bitwise(IntegerBits)
        .eager_booleans(Eager)
        .scalar_constants(ScalarReads)
        .public_constants(PublicDeclarations)
        .public_constant_imports(PublicImports)
        .public_constant_reads(PublicReads)
        .unit_effects(Unit)
        .wrapping_negation(Wrapping)
        .build();
}

#[cfg(local_constant_duplicate)]
fn duplicate() {
    Builder::new()
        .local_constants(Constants)
        .local_constants(Constants);
}

#[cfg(any(
    local_constant_wrong_capability,
    local_constant_wrong_context,
    local_constant_wrong_output
))]
#[derive(Clone, Copy)]
struct Wrong;
#[cfg(any(
    local_constant_wrong_capability,
    local_constant_wrong_context,
    local_constant_wrong_output
))]
impl Mapping for Wrong {
    #[cfg(local_constant_wrong_capability)]
    type Capability = LiteralValues;
    #[cfg(not(local_constant_wrong_capability))]
    type Capability = LocalConstants;
    #[cfg(local_constant_wrong_context)]
    type Context<'tcx> = ();
    #[cfg(not(local_constant_wrong_context))]
    type Context<'tcx> = Reader<'tcx>;
    #[cfg(local_constant_wrong_output)]
    type Output = bool;
    #[cfg(not(local_constant_wrong_output))]
    type Output = Output;

    fn lower<'tcx>(
        &self,
        _: &mut Self::Context<'tcx>,
        _: <Self::Capability as Capability>::Input<'tcx>,
    ) -> Result<Self::Output, String> {
        Err("deliberately wrong local constant mapping".into())
    }
}

#[cfg(any(
    local_constant_wrong_capability,
    local_constant_wrong_context,
    local_constant_wrong_output
))]
fn wrong() {
    Builder::new().local_constants(Wrong);
}

#[cfg(local_constant_wrong_input)]
fn wrong_input<'tcx>(reader: &mut Reader<'tcx>, expression: &'tcx rustc_hir::Expr<'tcx>) {
    let input = LiteralInput::read(reader.checked, expression).unwrap();
    let _ = Constants.lower(reader, input);
}

#[cfg(local_constant_private_input)]
fn private_input<'tcx>(statement: &'tcx rustc_hir::Stmt<'tcx>) {
    let _ = LocalConstantInput {
        value: LiteralValue::I32(0),
        _definition: rustc_hir::def_id::CRATE_DEF_ID.to_def_id(),
        _statement: statement,
    };
}
