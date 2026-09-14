use super::{
    JavaDirectCalls, JavaEntrySignatures, JavaFunctionSignatures, JavaLexicalControl,
    JavaLiteralValues, JavaObjectTypes, JavaRecordInitializers, JavaResolvedPlaces,
    JavaScalarComparisons, JavaSharedBorrows, Mapping,
};
use crate::java_lower::Reader;

#[allow(dead_code)]
fn must_not_compile(reader: &mut Reader<'_>) {
    let _ = JavaDirectCalls.lower(reader, ());
    let _ = JavaEntrySignatures.lower(&mut (), ());
    let _ = JavaFunctionSignatures.lower(&mut (), ());
    let _ = JavaLexicalControl.lower(reader, ());
    let _ = JavaLiteralValues.lower(reader, ());
    let _ = JavaObjectTypes.lower(reader, ());
    let _ = JavaRecordInitializers.lower(reader, ());
    let _ = JavaResolvedPlaces.lower(reader, ());
    let _ = JavaScalarComparisons.lower(reader, ());
    let _ = JavaSharedBorrows.lower(reader, ());
}
