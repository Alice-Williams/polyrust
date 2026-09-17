//! Every new executable slot has isolated compile-negative boundary controls.
#[cfg(public_constant_c)]
use super::CUnitEffects as Unit;
#[cfg(public_constant_c)]
use super::CWrappingNegation as Wrapping;
#[cfg(public_constant_java)]
use super::JavaUnitEffects as Unit;
#[cfg(public_constant_java)]
use super::JavaWrappingNegation as Wrapping;
use super::*;
#[cfg(public_constant_c)]
use super::{
    CPublicConstantImports as Imports, CPublicConstantReads as Reads,
    CPublicConstants as Declarations,
};
#[cfg(public_constant_java)]
use super::{
    JavaPublicConstantImports as Imports, JavaPublicConstantReads as Reads,
    JavaPublicConstants as Declarations,
};
#[cfg(public_constant_c)]
use crate::c_lower::{Reader, package::State};
#[cfg(public_constant_java)]
use crate::java_lower::{Reader, package::State};
#[cfg(public_constant_import)]
type Selected = PublicConstantImports;
#[cfg(public_constant_import)]
type Context<'tcx> = ImportState;
#[cfg(all(public_constant_import, public_constant_c))]
type Output = portable_backend_c::ast::CObjectRef;
#[cfg(all(public_constant_import, public_constant_java))]
type Output = portable_backend_java::dialect::JavaImportedValue;
#[cfg(public_constant_declaration)]
type Selected = PublicConstants;
#[cfg(public_constant_read)]
type Selected = PublicConstantReads;
#[cfg(public_constant_declaration)]
type Context<'tcx> = State;
#[cfg(public_constant_read)]
type Context<'tcx> = Reader<'tcx>;
#[cfg(all(public_constant_declaration, public_constant_c))]
type Output = portable_backend_c::ast::CObjectRef;
#[cfg(all(public_constant_declaration, public_constant_java))]
type Output = portable_codegen::GeneratedValueId;
#[cfg(all(public_constant_read, public_constant_c))]
type Output = portable_backend_c::ast::CValue;
#[cfg(all(public_constant_read, public_constant_java))]
type Output = crate::java_lower::Value;

#[cfg(public_constant_missing)]
fn missing() {
    let builder = Builder::new();
    #[cfg(public_constant_c)]
    let builder = builder
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
        .local_constants(CLocalConstants);
    #[cfg(public_constant_java)]
    let builder = builder
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
        .local_constants(JavaLocalConstants);
    #[cfg(public_constant_declaration)]
    let builder = builder.public_constant_reads(Reads);
    #[cfg(public_constant_read)]
    let builder = builder.public_constants(Declarations);
    #[cfg(not(public_constant_import))]
    let builder = builder.public_constant_imports(Imports);
    #[cfg(public_constant_import)]
    let builder = builder
        .public_constants(Declarations)
        .public_constant_reads(Reads);
    let builder = builder.unit_effects(Unit).wrapping_negation(Wrapping);
    #[cfg(all(public_constant_complete_control, public_constant_declaration))]
    let builder = builder.public_constants(Declarations);
    #[cfg(all(public_constant_complete_control, public_constant_read))]
    let builder = builder.public_constant_reads(Reads);
    #[cfg(all(public_constant_complete_control, public_constant_import))]
    let builder = builder.public_constant_imports(Imports);
    builder.build();
}
#[cfg(public_constant_duplicate)]
fn duplicate() {
    #[cfg(public_constant_import)]
    let _ = Builder::new()
        .public_constant_imports(Imports)
        .public_constant_imports(Imports);
    #[cfg(public_constant_declaration)]
    let _ = Builder::new()
        .public_constants(Declarations)
        .public_constants(Declarations);
    #[cfg(public_constant_read)]
    let _ = Builder::new()
        .public_constant_reads(Reads)
        .public_constant_reads(Reads);
}
#[cfg(any(
    public_constant_wrong_capability,
    public_constant_wrong_context,
    public_constant_wrong_output
))]
#[derive(Clone, Copy)]
struct Wrong;
#[cfg(any(
    public_constant_wrong_capability,
    public_constant_wrong_context,
    public_constant_wrong_output
))]
impl Mapping for Wrong {
    #[cfg(public_constant_wrong_capability)]
    type Capability = LiteralValues;
    #[cfg(not(public_constant_wrong_capability))]
    type Capability = Selected;
    #[cfg(public_constant_wrong_context)]
    type Context<'tcx> = ();
    #[cfg(not(public_constant_wrong_context))]
    type Context<'tcx> = Context<'tcx>;
    #[cfg(public_constant_wrong_output)]
    type Output = bool;
    #[cfg(not(public_constant_wrong_output))]
    type Output = Output;
    fn lower<'tcx>(
        &self,
        _: &mut Self::Context<'tcx>,
        _: <Self::Capability as Capability>::Input<'tcx>,
    ) -> Result<Self::Output, String> {
        Err("deliberately incorrect mapping".into())
    }
}
#[cfg(any(
    public_constant_wrong_capability,
    public_constant_wrong_context,
    public_constant_wrong_output
))]
fn wrong() {
    #[cfg(public_constant_import)]
    let _ = Builder::new().public_constant_imports(Wrong);
    #[cfg(public_constant_declaration)]
    let _ = Builder::new().public_constants(Wrong);
    #[cfg(public_constant_read)]
    let _ = Builder::new().public_constant_reads(Wrong);
}
#[cfg(public_constant_wrong_input)]
fn wrong_input<'tcx>(context: &mut Context<'tcx>, input: LiteralInput<'tcx>) {
    #[cfg(public_constant_declaration)]
    let _ = Declarations.lower(context, input);
    #[cfg(public_constant_read)]
    let _ = Reads.lower(context, input);
    #[cfg(public_constant_import)]
    let _ = Imports.lower(context, input);
}
#[cfg(all(public_constant_private_input, public_constant_declaration))]
fn private_declaration(tcx: rustc_middle::ty::TyCtxt<'_>) {
    let _ = ConstantDeclarationInput {
        tcx,
        definition: rustc_hir::def_id::CRATE_DEF_ID.to_def_id(),
        value: LiteralValue::I32(0),
    };
}
#[cfg(all(public_constant_private_input, public_constant_read))]
fn private_read(checked: ConstantInput<'_>) {
    let _ = PublicConstantReadInput { checked };
}

#[cfg(all(public_constant_private_input, public_constant_import))]
fn private_import(tcx: rustc_middle::ty::TyCtxt<'_>) {
    let _ = ConstantImportInput {
        tcx,
        definition: rustc_hir::def_id::CRATE_DEF_ID.to_def_id(),
        value: LiteralValue::I32(0),
    };
}
