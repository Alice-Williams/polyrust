//! Java mapping for `Utf8Conversions`.

use portable_build::Utf8Conversions;

use super::support::java_operation_mapping;
use crate::{
    ast::{JavaExpr, JavaType},
    dialect::JavaRuntimeCallable,
    lower::{JavaIntrinsicExpr, runtime_call, runtime_fallible},
};

#[doc(hidden)]
pub enum JavaUtf8ConversionsInput {
    Encode { operand: JavaExpr, result: JavaType },
    DecodeChecked { operand: JavaExpr, result: JavaType },
}

fn lower_utf8_conversions(
    input: JavaUtf8ConversionsInput,
) -> Result<JavaIntrinsicExpr, Vec<portable_diagnostics::Diagnostic>> {
    Ok(match input {
        JavaUtf8ConversionsInput::Encode { operand, result } => JavaIntrinsicExpr::Direct(
            runtime_call(JavaRuntimeCallable::StringToUtf8, vec![operand], result),
        ),
        JavaUtf8ConversionsInput::DecodeChecked { operand, result } => {
            runtime_fallible(JavaRuntimeCallable::StringFromUtf8, vec![operand], result)
        }
    })
}

java_operation_mapping!(
    JavaUtf8Conversions,
    Utf8Conversions,
    JavaUtf8ConversionsInput,
    JavaIntrinsicExpr,
    lower_utf8_conversions
);
