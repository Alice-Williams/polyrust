//! Independent negative controls for the new executable slot and checked input.
#[cfg(constant_c)]
use super::CFloatingNaN as NaN;
#[cfg(constant_c)]
use super::CFloatingNegation as Floating;
#[cfg(constant_c)]
use super::CIntegerBitwise as IntegerBits;
#[cfg(constant_c)]
use super::CLocalConstants as Locals;
#[cfg(constant_c)]
use super::CUnitEffects as Unit;
#[cfg(constant_c)]
use super::CWrappingNegation as Wrapping;
#[cfg(constant_java)]
use super::JavaFloatingNaN as NaN;
#[cfg(constant_java)]
use super::JavaFloatingNegation as Floating;
#[cfg(constant_java)]
use super::JavaIntegerBitwise as IntegerBits;
#[cfg(constant_java)]
use super::JavaLocalConstants as Locals;
#[cfg(constant_java)]
use super::JavaUnitEffects as Unit;
#[cfg(constant_java)]
use super::JavaWrappingNegation as Wrapping;
use super::*;
#[cfg(constant_c)]
use super::{
    CBooleanNegation as Negate, CDirectCalls as Calls, CEagerBooleans as Eager,
    CEntrySignatures as Entry, CFunctionSignatures as Functions, CLexicalControl as Control,
    CLiteralValues as Literals, CObjectTypes as Objects, CRecordInitializers as Records,
    CResolvedPlaces as Places, CScalarComparisons as Comparisons, CScalarConstants as Constants,
    CSharedBorrows as Borrows, CShortCircuitBooleans as Lazy,
};
#[cfg(constant_c)]
use super::{
    CFloatingAbsolute as Absolute, CFloatingArithmetic as Arithmetic,
    CFloatingRemainder as Remainder, CFloatingTruncation as Truncation,
    CWrappingAddition as Addition, CWrappingSubtraction as Subtraction,
};
#[cfg(constant_c)]
use super::{
    CPublicConstantImports as PublicImports, CPublicConstantReads as PublicReads,
    CPublicConstants as PublicDeclarations,
};
#[cfg(constant_java)]
use super::{
    JavaBooleanNegation as Negate, JavaDirectCalls as Calls, JavaEagerBooleans as Eager,
    JavaEntrySignatures as Entry, JavaFunctionSignatures as Functions,
    JavaLexicalControl as Control, JavaLiteralValues as Literals, JavaObjectTypes as Objects,
    JavaRecordInitializers as Records, JavaResolvedPlaces as Places,
    JavaScalarComparisons as Comparisons, JavaScalarConstants as Constants,
    JavaSharedBorrows as Borrows, JavaShortCircuitBooleans as Lazy,
};
#[cfg(constant_java)]
use super::{
    JavaFloatingAbsolute as Absolute, JavaFloatingArithmetic as Arithmetic,
    JavaFloatingRemainder as Remainder, JavaFloatingTruncation as Truncation,
    JavaWrappingAddition as Addition, JavaWrappingSubtraction as Subtraction,
};
#[cfg(constant_java)]
use super::{
    JavaPublicConstantImports as PublicImports, JavaPublicConstantReads as PublicReads,
    JavaPublicConstants as PublicDeclarations,
};
#[cfg(constant_c)]
use crate::c_lower::Reader;
#[cfg(constant_java)]
use crate::java_lower::Reader;
#[cfg(constant_c)]
type Output = portable_backend_c::ast::CValue;
#[cfg(constant_java)]
type Output = crate::java_lower::Value;

#[cfg(constant_missing)]
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
        .local_constants(Locals)
        .public_constants(PublicDeclarations)
        .public_constant_imports(PublicImports)
        .public_constant_reads(PublicReads)
        .unit_effects(Unit)
        .wrapping_negation(Wrapping)
        .floating_negation(Floating)
        .floating_nan(NaN)
        .floating_truncation(Truncation)
        .floating_arithmetic(Arithmetic)
        .floating_remainder(Remainder)
        .wrapping_addition(Addition)
        .wrapping_subtraction(Subtraction)
        .floating_absolute(Absolute)
        .build();
}

#[cfg(constant_duplicate)]
fn duplicate() {
    Builder::new()
        .scalar_constants(Constants)
        .scalar_constants(Constants);
}

#[cfg(any(
    constant_wrong_capability,
    constant_wrong_context,
    constant_wrong_output
))]
#[derive(Clone, Copy)]
struct Wrong;
#[cfg(any(
    constant_wrong_capability,
    constant_wrong_context,
    constant_wrong_output
))]
impl Mapping for Wrong {
    #[cfg(constant_wrong_capability)]
    type Capability = LiteralValues;
    #[cfg(not(constant_wrong_capability))]
    type Capability = ScalarConstants;
    #[cfg(constant_wrong_context)]
    type Context<'tcx> = ();
    #[cfg(not(constant_wrong_context))]
    type Context<'tcx> = Reader<'tcx>;
    #[cfg(constant_wrong_output)]
    type Output = ();
    #[cfg(not(constant_wrong_output))]
    type Output = Output;

    fn lower<'tcx>(
        &self,
        _: &mut Self::Context<'tcx>,
        _: <Self::Capability as Capability>::Input<'tcx>,
    ) -> Result<Self::Output, String> {
        Err("deliberately wrong Boolean mapping".into())
    }
}

#[cfg(any(
    constant_wrong_capability,
    constant_wrong_context,
    constant_wrong_output
))]
fn wrong() {
    Builder::new().scalar_constants(Wrong);
}

#[cfg(constant_wrong_input)]
fn wrong_input<'tcx>(reader: &mut Reader<'tcx>, expression: &'tcx rustc_hir::Expr<'tcx>) {
    let input = LiteralInput::read(reader.tcx, reader.checked, expression).unwrap();
    let _ = Constants.lower(reader, input);
}

#[cfg(constant_private_input)]
fn private_input<'tcx>(expression: &'tcx rustc_hir::Expr<'tcx>) {
    let _ = ConstantInput {
        value: ScalarConstantValue::I32(0),
        _definition: rustc_hir::def_id::CRATE_DEF_ID.to_def_id(),
        _expression: expression,
    };
}
