use super::{
    Builder, CDirectCalls, CEntrySignatures, CFunctionSignatures, CLexicalControl, CLiteralValues,
    CObjectTypes, CRecordInitializers, CResolvedPlaces, CScalarComparisons, CSharedBorrows,
};

#[allow(dead_code)]
fn must_not_compile() {
    Builder::new()
        .literal_values(CLiteralValues)
        .literal_values(CLiteralValues);
    Builder::new()
        .scalar_comparisons(CScalarComparisons)
        .scalar_comparisons(CScalarComparisons);
    Builder::new()
        .resolved_places(CResolvedPlaces)
        .resolved_places(CResolvedPlaces);
    Builder::new()
        .shared_borrows(CSharedBorrows)
        .shared_borrows(CSharedBorrows);
    Builder::new()
        .object_types(CObjectTypes)
        .object_types(CObjectTypes);
    Builder::new()
        .record_initializers(CRecordInitializers)
        .record_initializers(CRecordInitializers);
    Builder::new()
        .lexical_control(CLexicalControl)
        .lexical_control(CLexicalControl);
    Builder::new()
        .entry_signatures(CEntrySignatures)
        .entry_signatures(CEntrySignatures);
    Builder::new()
        .direct_calls(CDirectCalls)
        .direct_calls(CDirectCalls);
    Builder::new()
        .function_signatures(CFunctionSignatures)
        .function_signatures(CFunctionSignatures);
}
