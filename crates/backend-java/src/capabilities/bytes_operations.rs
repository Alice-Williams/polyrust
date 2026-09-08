//! Java mapping for `BytesOperations`.

mod mapping_plan;

use portable_build::BytesOperations;

use super::support::java_operation_mapping;
use crate::{
    ast::{JavaExpr, JavaType},
    dialect::JavaRuntimeCallable,
    lower::{JavaIntrinsicExpr, runtime_call},
};

#[doc(hidden)]
#[derive(Clone)]
pub enum JavaBytesOperationsInput {
    Length {
        bytes: JavaExpr,
        result: JavaType,
    },
    IsEmpty {
        bytes: JavaExpr,
        result: JavaType,
    },
    Concat {
        left: JavaExpr,
        right: JavaExpr,
        result: JavaType,
    },
    ReplaceAll {
        source: JavaExpr,
        needle: JavaExpr,
        replacement: Box<JavaExpr>,
        result: JavaType,
    },
}

fn lower_bytes_operations(
    input: JavaBytesOperationsInput,
) -> Result<JavaIntrinsicExpr, Vec<portable_diagnostics::Diagnostic>> {
    let (callable, arguments, result) = match input {
        JavaBytesOperationsInput::Length { bytes, result } => {
            (JavaRuntimeCallable::BytesLength, vec![bytes], result)
        }
        JavaBytesOperationsInput::IsEmpty { bytes, result } => {
            (JavaRuntimeCallable::BytesIsEmpty, vec![bytes], result)
        }
        JavaBytesOperationsInput::Concat {
            left,
            right,
            result,
        } => (JavaRuntimeCallable::BytesConcat, vec![left, right], result),
        JavaBytesOperationsInput::ReplaceAll {
            source,
            needle,
            replacement,
            result,
        } => (
            JavaRuntimeCallable::BytesReplaceAll,
            vec![source, needle, *replacement],
            result,
        ),
    };
    Ok(JavaIntrinsicExpr::Infallible(runtime_call(
        callable, arguments, result,
    )))
}

java_operation_mapping!(
    JavaBytesOperations,
    BytesOperations,
    JavaBytesOperationsInput,
    JavaIntrinsicExpr,
    lower_bytes_operations
);
