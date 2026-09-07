//! Java mapping for `StringConcatenation`.

use portable_build::StringConcatenation;

use super::support::java_operation_mapping;
use crate::{
    ast::{JavaBinaryOperator, JavaExpr, JavaType},
    lower::{JavaIntrinsicExpr, binary},
};

#[doc(hidden)]
pub struct JavaStringConcatenationInput {
    pub(crate) left: JavaExpr,
    pub(crate) right: JavaExpr,
    pub(crate) result: JavaType,
}

fn lower_string_concatenation(
    input: JavaStringConcatenationInput,
) -> Result<JavaIntrinsicExpr, Vec<portable_diagnostics::Diagnostic>> {
    Ok(JavaIntrinsicExpr::Direct(binary(
        JavaBinaryOperator::Add,
        input.left,
        input.right,
        input.result,
    )))
}

java_operation_mapping!(
    JavaStringConcatenation,
    StringConcatenation,
    JavaStringConcatenationInput,
    JavaIntrinsicExpr,
    lower_string_concatenation
);
