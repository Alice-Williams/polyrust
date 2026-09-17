//! Independent negative controls for the new executable slot and checked input.
#[cfg(bitwise_c)]
use super::CLocalConstants as Locals;
#[cfg(bitwise_c)]
use super::CScalarConstants as Constants;
#[cfg(bitwise_c)]
use super::CUnitEffects as Unit;
#[cfg(bitwise_java)]
use super::JavaLocalConstants as Locals;
#[cfg(bitwise_java)]
use super::JavaScalarConstants as Constants;
#[cfg(bitwise_java)]
use super::JavaUnitEffects as Unit;
use super::*;
#[cfg(bitwise_c)]
use super::{
    CBooleanNegation as Negate, CDirectCalls as Calls, CEagerBooleans as Eager,
    CEntrySignatures as Entry, CFunctionSignatures as Functions, CIntegerBitwise as Bits,
    CLexicalControl as Control, CLiteralValues as Literals, CObjectTypes as Objects,
    CRecordInitializers as Records, CResolvedPlaces as Places, CScalarComparisons as Comparisons,
    CSharedBorrows as Borrows, CShortCircuitBooleans as Lazy,
};
#[cfg(bitwise_c)]
use super::{
    CPublicConstantImports as PublicImports, CPublicConstantReads as PublicReads,
    CPublicConstants as PublicDeclarations,
};
#[cfg(bitwise_java)]
use super::{
    JavaBooleanNegation as Negate, JavaDirectCalls as Calls, JavaEagerBooleans as Eager,
    JavaEntrySignatures as Entry, JavaFunctionSignatures as Functions, JavaIntegerBitwise as Bits,
    JavaLexicalControl as Control, JavaLiteralValues as Literals, JavaObjectTypes as Objects,
    JavaRecordInitializers as Records, JavaResolvedPlaces as Places,
    JavaScalarComparisons as Comparisons, JavaSharedBorrows as Borrows,
    JavaShortCircuitBooleans as Lazy,
};
#[cfg(bitwise_java)]
use super::{
    JavaPublicConstantImports as PublicImports, JavaPublicConstantReads as PublicReads,
    JavaPublicConstants as PublicDeclarations,
};
#[cfg(bitwise_c)]
use crate::c_lower::Reader;
#[cfg(bitwise_java)]
use crate::java_lower::Reader;
#[cfg(bitwise_c)]
type Output = portable_backend_c::ast::CValue;
#[cfg(bitwise_java)]
type Output = crate::java_lower::Value;

#[cfg(bitwise_missing)]
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
        .eager_booleans(Eager)
        .scalar_constants(Constants)
        .local_constants(Locals)
        .public_constants(PublicDeclarations)
        .public_constant_imports(PublicImports)
        .public_constant_reads(PublicReads)
        .unit_effects(Unit)
        .build();
}

#[cfg(bitwise_duplicate)]
fn duplicate() {
    Builder::new().integer_bitwise(Bits).integer_bitwise(Bits);
}

#[cfg(any(bitwise_wrong_capability, bitwise_wrong_context, bitwise_wrong_output))]
#[derive(Clone, Copy)]
struct Wrong;
#[cfg(any(bitwise_wrong_capability, bitwise_wrong_context, bitwise_wrong_output))]
impl Mapping for Wrong {
    #[cfg(bitwise_wrong_capability)]
    type Capability = LiteralValues;
    #[cfg(not(bitwise_wrong_capability))]
    type Capability = IntegerBitwise;
    #[cfg(bitwise_wrong_context)]
    type Context<'tcx> = ();
    #[cfg(not(bitwise_wrong_context))]
    type Context<'tcx> = Reader<'tcx>;
    #[cfg(bitwise_wrong_output)]
    type Output = ();
    #[cfg(not(bitwise_wrong_output))]
    type Output = Output;

    fn lower<'tcx>(
        &self,
        _: &mut Self::Context<'tcx>,
        _: <Self::Capability as Capability>::Input<'tcx>,
    ) -> Result<Self::Output, String> {
        Err("deliberately wrong Boolean mapping".into())
    }
}

#[cfg(any(bitwise_wrong_capability, bitwise_wrong_context, bitwise_wrong_output))]
fn wrong() {
    Builder::new().integer_bitwise(Wrong);
}

#[cfg(bitwise_wrong_input)]
fn wrong_input<'tcx>(reader: &mut Reader<'tcx>, expression: &'tcx rustc_hir::Expr<'tcx>) {
    let input = LiteralInput::read(reader.checked, expression).unwrap();
    let _ = Bits.lower(reader, input);
}

#[cfg(bitwise_private_input)]
fn private_input<'tcx>(expression: &'tcx rustc_hir::Expr<'tcx>) {
    let _ = BitwiseInput {
        operands: BitwiseOperands::Complement(expression),
    };
}
