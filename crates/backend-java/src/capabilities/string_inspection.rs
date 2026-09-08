//! Java mapping for `StringInspection`.

mod mapping_plan;

use portable_build::StringInspection;

use super::support::java_operation_mapping;
use crate::{
    ast::{JavaExpr, JavaExprKind, JavaMemberOrigin, JavaPrecedence, JavaPrimitive, JavaType},
    dialect::{JavaKnownMethod, JavaRuntimeCallable},
    lower::{JavaIntrinsicExpr, member_call, runtime_call, runtime_fallible},
};

#[doc(hidden)]
#[derive(Clone)]
pub enum JavaStringInspectionInput {
    ScalarLength {
        source: JavaExpr,
        result: JavaType,
    },
    Utf16Length {
        source: JavaExpr,
        result: JavaType,
    },
    IsEmpty {
        source: JavaExpr,
        result: JavaType,
    },
    IndexOfLiteral {
        source: JavaExpr,
        needle: JavaExpr,
        result: JavaType,
    },
    Contains {
        source: JavaExpr,
        needle: JavaExpr,
        result: JavaType,
    },
    StartsWith {
        source: JavaExpr,
        prefix: JavaExpr,
        result: JavaType,
    },
    EndsWith {
        source: JavaExpr,
        suffix: JavaExpr,
        result: JavaType,
    },
}

fn lower_string_inspection(
    input: JavaStringInspectionInput,
) -> Result<JavaIntrinsicExpr, Vec<portable_diagnostics::Diagnostic>> {
    Ok(match input {
        JavaStringInspectionInput::ScalarLength { source, result } => {
            runtime_fallible(JavaRuntimeCallable::ScalarLength, vec![source], result)
        }
        JavaStringInspectionInput::Utf16Length { source, result } => {
            let length = member_call(
                source,
                "length",
                vec![],
                JavaType::primitive(JavaPrimitive::Int),
                JavaMemberOrigin::Known(JavaKnownMethod::StringLength),
            );
            JavaIntrinsicExpr::Infallible(JavaExpr {
                ty: result.clone(),
                precedence: JavaPrecedence::Unary,
                kind: JavaExprKind::Cast {
                    target: result,
                    value: Box::new(length),
                },
            })
        }
        JavaStringInspectionInput::IsEmpty { source, result } => {
            JavaIntrinsicExpr::Infallible(member_call(
                source,
                "isEmpty",
                vec![],
                result,
                JavaMemberOrigin::Known(JavaKnownMethod::StringIsEmpty),
            ))
        }
        JavaStringInspectionInput::IndexOfLiteral {
            source,
            needle,
            result,
        } => JavaIntrinsicExpr::Infallible(runtime_call(
            JavaRuntimeCallable::StringIndexOfLiteral,
            vec![source, needle],
            result,
        )),
        JavaStringInspectionInput::Contains {
            source,
            needle,
            result,
        } => JavaIntrinsicExpr::Infallible(member_call(
            source,
            "contains",
            vec![needle],
            result,
            JavaMemberOrigin::Known(JavaKnownMethod::StringContains),
        )),
        JavaStringInspectionInput::StartsWith {
            source,
            prefix,
            result,
        } => JavaIntrinsicExpr::Infallible(member_call(
            source,
            "startsWith",
            vec![prefix],
            result,
            JavaMemberOrigin::Known(JavaKnownMethod::StringStartsWith),
        )),
        JavaStringInspectionInput::EndsWith {
            source,
            suffix,
            result,
        } => JavaIntrinsicExpr::Infallible(member_call(
            source,
            "endsWith",
            vec![suffix],
            result,
            JavaMemberOrigin::Known(JavaKnownMethod::StringEndsWith),
        )),
    })
}

java_operation_mapping!(
    JavaStringInspection,
    StringInspection,
    JavaStringInspectionInput,
    JavaIntrinsicExpr,
    lower_string_inspection
);
