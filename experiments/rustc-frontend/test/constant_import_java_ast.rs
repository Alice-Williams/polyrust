//! Imported Java value nodes retain exact producer identities and primitive types.
use super::{Mapping, PublicConstantReadInput};
use crate::java_lower::{Reader, Value};
use portable_backend_java::ast::*;

pub(super) fn read<'tcx>(
    reader: &mut Reader<'tcx>,
    input: PublicConstantReadInput<'tcx>,
    value: &Value,
) {
    let expression = value.clone().into_expression();
    let JavaExprKind::Value(JavaValueRef::Dependency(imported)) = &expression.kind else {
        panic!("foreign constant must be a typed dependency value")
    };
    assert_eq!(imported, &reader.foreign_constants[&input.definition()]);
    let proof = imported.constant();
    let (_, expected) = crate::java_lower::constants::value(input.value());
    assert_eq!(
        proof.declaration(),
        crate::source_origin::identity(reader.tcx, input.definition())
    );
    assert_eq!(proof.value(), &expected);
    assert_eq!(proof.ty(), &expression.ty);
    assert!(!reader.constants.contains_key(&input.definition()));
    let saved = reader
        .foreign_constants
        .remove(&input.definition())
        .unwrap();
    assert!(super::JavaPublicConstantReads.lower(reader, input).is_err());
    for other in reader
        .foreign_constants
        .values()
        .cloned()
        .collect::<Vec<_>>()
    {
        reader.foreign_constants.insert(input.definition(), other);
        assert!(super::JavaPublicConstantReads.lower(reader, input).is_err());
    }
    reader.foreign_constants.insert(input.definition(), saved);
    println!("CONSTANT_IMPORT_READ\tjava\t{:?}", input.value());
}
