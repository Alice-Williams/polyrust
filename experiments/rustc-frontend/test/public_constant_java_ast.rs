//! Check registered fields and generated value IDs before source rendering.
use super::{ConstantDeclarationInput, Mapping, PublicConstantReadInput};
use crate::java_lower::{Reader, Value, package::State};
use portable_backend_java::ast::*;
use portable_codegen::{GeneratedSymbolId, GeneratedValueId};
pub(super) fn declaration(
    state: &State,
    input: ConstantDeclarationInput<'_>,
    id: GeneratedValueId,
) {
    let constant = &state.constants[&input.definition()];
    assert_eq!(constant.id, id);
    assert_eq!(constant.field.declared, Some(id));
    assert_eq!(constant.value, input.value());
    assert_eq!(
        constant.field.modifiers,
        vec![
            JavaModifier::Public,
            JavaModifier::Static,
            JavaModifier::Final
        ]
    );
    assert!(matches!(
        constant.field.initializer.as_ref().unwrap().kind,
        JavaExprKind::Literal(_)
    ));
    println!("PUBLIC_CONSTANT_DECL\tjava\t{:?}", input.value());
}
pub(super) fn read<'tcx>(
    reader: &mut Reader<'tcx>,
    input: PublicConstantReadInput<'tcx>,
    value: &Value,
) {
    let constant = &reader.constants[&input.definition()];
    let expression = value.clone().into_expression();
    assert_eq!(expression.ty, constant.field.ty);
    assert_eq!(
        expression.kind,
        JavaExprKind::Value(JavaValueRef::Generated(GeneratedSymbolId::Value(
            constant.id
        )))
    );

    let saved = reader.constants.remove(&input.definition()).unwrap();
    assert!(super::JavaPublicConstantReads.lower(reader, input).is_err());
    reader.constants.insert(input.definition(), saved.clone());
    reader.constants.get_mut(&input.definition()).unwrap().value = match input.value() {
        super::ScalarConstantValue::Bool(v) => super::ScalarConstantValue::Bool(!v),
        super::ScalarConstantValue::I32(v) => super::ScalarConstantValue::I32(v.wrapping_add(1)),
        super::ScalarConstantValue::I64(v) => super::ScalarConstantValue::I64(v.wrapping_add(1)),
    };
    assert!(super::JavaPublicConstantReads.lower(reader, input).is_err());
    reader.constants.insert(input.definition(), saved.clone());
    if let Some(other) = reader
        .constants
        .iter()
        .find(|(id, c)| **id != input.definition() && c.value == input.value())
        .map(|(_, c)| c.clone())
    {
        reader.constants.insert(input.definition(), other);
        assert!(super::JavaPublicConstantReads.lower(reader, input).is_err());
        reader.constants.insert(input.definition(), saved);
    }
    println!("PUBLIC_CONSTANT_READ\tjava\t{:?}", input.value());
}
