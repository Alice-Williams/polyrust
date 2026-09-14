use super::{
    Builder, JavaDirectCalls, JavaEntrySignatures, JavaFunctionSignatures, JavaLexicalControl,
    JavaLiteralValues, JavaObjectTypes, JavaRecordInitializers, JavaResolvedPlaces,
    JavaScalarComparisons, JavaSharedBorrows,
};

#[allow(dead_code)]
fn must_not_compile() {
    Builder::new()
        .literal_values(JavaLiteralValues)
        .literal_values(JavaLiteralValues);
    Builder::new()
        .scalar_comparisons(JavaScalarComparisons)
        .scalar_comparisons(JavaScalarComparisons);
    Builder::new()
        .resolved_places(JavaResolvedPlaces)
        .resolved_places(JavaResolvedPlaces);
    Builder::new()
        .shared_borrows(JavaSharedBorrows)
        .shared_borrows(JavaSharedBorrows);
    Builder::new()
        .object_types(JavaObjectTypes)
        .object_types(JavaObjectTypes);
    Builder::new()
        .record_initializers(JavaRecordInitializers)
        .record_initializers(JavaRecordInitializers);
    Builder::new()
        .lexical_control(JavaLexicalControl)
        .lexical_control(JavaLexicalControl);
    Builder::new()
        .entry_signatures(JavaEntrySignatures)
        .entry_signatures(JavaEntrySignatures);
    Builder::new()
        .direct_calls(JavaDirectCalls)
        .direct_calls(JavaDirectCalls);
    Builder::new()
        .function_signatures(JavaFunctionSignatures)
        .function_signatures(JavaFunctionSignatures);
}
