//! Java mapping for `StringConcatenation`.

mod mapping_plan;

use portable_build::StringConcatenation;

use super::support::java_operation_mapping;
use crate::{
    ast::{JavaExpr, JavaMemberOrigin, JavaType},
    dialect::JavaKnownMethod,
    lower::{JavaIntrinsicExpr, member_call},
};

#[doc(hidden)]
#[derive(Clone)]
pub struct JavaStringConcatenationInput {
    pub(crate) left: JavaExpr,
    pub(crate) right: JavaExpr,
    pub(crate) result: JavaType,
}

fn lower_string_concatenation(
    input: JavaStringConcatenationInput,
) -> Result<JavaIntrinsicExpr, Vec<portable_diagnostics::Diagnostic>> {
    Ok(JavaIntrinsicExpr::Infallible(member_call(
        input.left,
        JavaKnownMethod::StringConcat.name().text(),
        vec![input.right],
        input.result,
        JavaMemberOrigin::Known(JavaKnownMethod::StringConcat),
    )))
}

java_operation_mapping!(
    JavaStringConcatenation,
    StringConcatenation,
    JavaStringConcatenationInput,
    JavaIntrinsicExpr,
    lower_string_concatenation
);
