//! Independent negative controls for the new executable slot and checked input.
#[cfg(eager_c)]
use super::CFloatingNaN as NaN;
#[cfg(eager_c)]
use super::CFloatingNegation as Floating;
#[cfg(eager_c)]
use super::CIntegerBitwise as IntegerBits;
#[cfg(eager_c)]
use super::CLocalConstants as Locals;
#[cfg(eager_c)]
use super::CScalarConstants as Constants;
#[cfg(eager_c)]
use super::CUnitEffects as Unit;
#[cfg(eager_c)]
use super::CWrappingNegation as Wrapping;
#[cfg(eager_java)]
use super::JavaFloatingNaN as NaN;
#[cfg(eager_java)]
use super::JavaFloatingNegation as Floating;
#[cfg(eager_java)]
use super::JavaIntegerBitwise as IntegerBits;
#[cfg(eager_java)]
use super::JavaLocalConstants as Locals;
#[cfg(eager_java)]
use super::JavaScalarConstants as Constants;
#[cfg(eager_java)]
use super::JavaUnitEffects as Unit;
#[cfg(eager_java)]
use super::JavaWrappingNegation as Wrapping;
use super::*;
#[cfg(eager_c)]
use super::{
    CBooleanNegation as Negate, CDirectCalls as Calls, CEagerBooleans as Bits,
    CEntrySignatures as Entry, CFunctionSignatures as Functions, CLexicalControl as Control,
    CLiteralValues as Literals, CObjectTypes as Objects, CRecordInitializers as Records,
    CResolvedPlaces as Places, CScalarComparisons as Comparisons, CSharedBorrows as Borrows,
    CShortCircuitBooleans as Lazy,
};
#[cfg(eager_c)]
use super::{
    CFloatingAbsolute as Absolute, CFloatingArithmetic as Arithmetic,
    CFloatingRemainder as Remainder, CFloatingTruncation as Truncation,
    CWrappingAddition as Addition, CWrappingMultiplication as Multiplication,
    CWrappingSubtraction as Subtraction,
};
#[cfg(eager_c)]
use super::{
    CPublicConstantImports as PublicImports, CPublicConstantReads as PublicReads,
    CPublicConstants as PublicDeclarations,
};
#[cfg(eager_java)]
use super::{
    JavaBooleanNegation as Negate, JavaDirectCalls as Calls, JavaEagerBooleans as Bits,
    JavaEntrySignatures as Entry, JavaFunctionSignatures as Functions,
    JavaLexicalControl as Control, JavaLiteralValues as Literals, JavaObjectTypes as Objects,
    JavaRecordInitializers as Records, JavaResolvedPlaces as Places,
    JavaScalarComparisons as Comparisons, JavaSharedBorrows as Borrows,
    JavaShortCircuitBooleans as Lazy,
};
#[cfg(eager_java)]
use super::{
    JavaFloatingAbsolute as Absolute, JavaFloatingArithmetic as Arithmetic,
    JavaFloatingRemainder as Remainder, JavaFloatingTruncation as Truncation,
    JavaWrappingAddition as Addition, JavaWrappingMultiplication as Multiplication,
    JavaWrappingSubtraction as Subtraction,
};
#[cfg(eager_java)]
use super::{
    JavaPublicConstantImports as PublicImports, JavaPublicConstantReads as PublicReads,
    JavaPublicConstants as PublicDeclarations,
};
#[cfg(eager_c)]
use crate::c_lower::Reader;
#[cfg(eager_java)]
use crate::java_lower::Reader;
#[cfg(eager_c)]
type Output = portable_backend_c::ast::CValue;
#[cfg(eager_java)]
type Output = crate::java_lower::Value;

#[cfg(eager_missing)]
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
        .scalar_constants(Constants)
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
        .wrapping_multiplication(Multiplication)
        .floating_absolute(Absolute)
        .build();
}

#[cfg(eager_duplicate)]
fn duplicate() {
    Builder::new().eager_booleans(Bits).eager_booleans(Bits);
}

#[cfg(any(eager_wrong_capability, eager_wrong_context, eager_wrong_output))]
#[derive(Clone, Copy)]
struct Wrong;
#[cfg(any(eager_wrong_capability, eager_wrong_context, eager_wrong_output))]
impl Mapping for Wrong {
    #[cfg(eager_wrong_capability)]
    type Capability = LiteralValues;
    #[cfg(not(eager_wrong_capability))]
    type Capability = EagerBooleans;
    #[cfg(eager_wrong_context)]
    type Context<'tcx> = ();
    #[cfg(not(eager_wrong_context))]
    type Context<'tcx> = Reader<'tcx>;
    #[cfg(eager_wrong_output)]
    type Output = ();
    #[cfg(not(eager_wrong_output))]
    type Output = Output;

    fn lower<'tcx>(
        &self,
        _: &mut Self::Context<'tcx>,
        _: <Self::Capability as Capability>::Input<'tcx>,
    ) -> Result<Self::Output, String> {
        Err("deliberately wrong Boolean mapping".into())
    }
}

#[cfg(any(eager_wrong_capability, eager_wrong_context, eager_wrong_output))]
fn wrong() {
    Builder::new().eager_booleans(Wrong);
}

#[cfg(eager_wrong_input)]
fn wrong_input<'tcx>(reader: &mut Reader<'tcx>, expression: &'tcx rustc_hir::Expr<'tcx>) {
    let input = LiteralInput::read(reader.tcx, reader.checked, expression).unwrap();
    let _ = Bits.lower(reader, input);
}

#[cfg(eager_private_input)]
fn private_input<'tcx>(expression: &'tcx rustc_hir::Expr<'tcx>) {
    let _ = EagerBooleanInput {
        operator: EagerBooleanOperator::And,
        left: expression,
        right: expression,
    };
}
