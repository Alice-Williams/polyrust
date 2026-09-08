use super::*;
use crate::{
    ast::*,
    capabilities as c,
    lower::{JavaIntrinsicExpr, i32_literal},
};
use c::support::plans::JavaRepresentation as R;
mod containers;
mod numeric;
mod strings;

fn verify<M>(mapping: M, input: M::Input, expected: R)
where
    M: JavaCapabilityMapping<Context = (), Output = JavaIntrinsicExpr> + 'static,
    c::JavaCapabilitySet:
        Supports<M::Capability, Dialect = JavaDialect, Mapping = c::support::CheckedJavaMapping<M>>,
{
    let (plan, mut output) = checked(mapping, input);
    assert_eq!(plan.representation(), expected);
    let (expression, value_type) = match &mut output {
        JavaIntrinsicExpr::Infallible(value) => (value, None),
        JavaIntrinsicExpr::Fallible { call, value_type } => (call, Some(value_type.clone())),
    };
    let original = expression.clone();
    *expression = i32_literal(123);
    assert!(!plan.verify_output(&output), "wrong root must be rejected");
    let reversed = match value_type {
        None => JavaIntrinsicExpr::Fallible {
            call: original.clone(),
            value_type: original.ty.clone(),
        },
        Some(_) => JavaIntrinsicExpr::Infallible(original),
    };
    assert!(
        !plan.verify_output(&reversed),
        "fallibility is part of the certificate"
    );
}
fn value(ty: JavaType) -> JavaExpr {
    JavaExpr::local(ty, JavaIdentifier::from_portable("operand"))
}
fn integer() -> JavaExpr {
    value(JavaType::primitive(JavaPrimitive::Int))
}
fn wide() -> JavaExpr {
    value(JavaType::primitive(JavaPrimitive::Long))
}
fn float() -> JavaExpr {
    value(JavaType::primitive(JavaPrimitive::Double))
}
fn text() -> JavaExpr {
    value(JavaType::known(JavaKnownType::String))
}
fn boolean() -> JavaType {
    JavaType::primitive(JavaPrimitive::Boolean)
}
fn long() -> JavaType {
    JavaType::primitive(JavaPrimitive::Long)
}
fn string() -> JavaType {
    JavaType::known(JavaKnownType::String)
}
fn bytes() -> JavaType {
    JavaType::known(JavaKnownType::RuntimeBytes)
}
fn list() -> JavaType {
    JavaType::generic(
        JavaKnownType::List,
        vec![JavaType::Boxed(JavaPrimitive::Int)],
    )
}
fn option() -> JavaType {
    JavaType::generic(
        JavaKnownType::RuntimeOption,
        vec![JavaType::Boxed(JavaPrimitive::Int)],
    )
}
fn result() -> JavaType {
    JavaType::generic(
        JavaKnownType::RuntimeValueResult,
        vec![JavaType::Boxed(JavaPrimitive::Int), string()],
    )
}
