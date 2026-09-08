//! Java mapping for `ListOperations`.

mod mapping_plan;

use portable_build::ListOperations;

use super::support::java_operation_mapping;
use crate::{
    ast::{JavaExpr, JavaType},
    dialect::JavaRuntimeCallable,
    lower::{JavaIntrinsicExpr, runtime_call, runtime_fallible},
};

#[doc(hidden)]
#[derive(Clone)]
pub enum JavaListOperationsInput {
    Length {
        list: JavaExpr,
        result: JavaType,
    },
    IsEmpty {
        list: JavaExpr,
        result: JavaType,
    },
    GetChecked {
        list: JavaExpr,
        index: JavaExpr,
        result: JavaType,
    },
    Append {
        list: JavaExpr,
        value: JavaExpr,
        result: JavaType,
    },
    Concat {
        left: JavaExpr,
        right: JavaExpr,
        result: JavaType,
    },
    Contains {
        list: JavaExpr,
        value: JavaExpr,
        result: JavaType,
    },
    IndexOf {
        list: JavaExpr,
        value: JavaExpr,
        result: JavaType,
    },
}

fn lower_list_operations(
    input: JavaListOperationsInput,
) -> Result<JavaIntrinsicExpr, Vec<portable_diagnostics::Diagnostic>> {
    Ok(match input {
        JavaListOperationsInput::Length { list, result } => JavaIntrinsicExpr::Infallible(
            runtime_call(JavaRuntimeCallable::ListLength, vec![list], result),
        ),
        JavaListOperationsInput::IsEmpty { list, result } => JavaIntrinsicExpr::Infallible(
            runtime_call(JavaRuntimeCallable::ListIsEmpty, vec![list], result),
        ),
        JavaListOperationsInput::GetChecked {
            list,
            index,
            result,
        } => runtime_fallible(JavaRuntimeCallable::ListGet, vec![list, index], result),
        JavaListOperationsInput::Append {
            list,
            value,
            result,
        } => JavaIntrinsicExpr::Infallible(runtime_call(
            JavaRuntimeCallable::ListAppend,
            vec![list, value],
            result,
        )),
        JavaListOperationsInput::Concat {
            left,
            right,
            result,
        } => JavaIntrinsicExpr::Infallible(runtime_call(
            JavaRuntimeCallable::ListConcat,
            vec![left, right],
            result,
        )),
        JavaListOperationsInput::Contains {
            list,
            value,
            result,
        } => JavaIntrinsicExpr::Infallible(runtime_call(
            JavaRuntimeCallable::ListContains,
            vec![list, value],
            result,
        )),
        JavaListOperationsInput::IndexOf {
            list,
            value,
            result,
        } => JavaIntrinsicExpr::Infallible(runtime_call(
            JavaRuntimeCallable::ListIndexOf,
            vec![list, value],
            result,
        )),
    })
}

java_operation_mapping!(
    JavaListOperations,
    ListOperations,
    JavaListOperationsInput,
    JavaIntrinsicExpr,
    lower_list_operations
);
