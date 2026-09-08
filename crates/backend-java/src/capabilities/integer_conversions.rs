//! Java mapping for `IntegerConversions`.

mod mapping_plan;

use portable_build::IntegerConversions;

use super::support::java_operation_mapping;
use crate::{
    ast::{JavaExpr, JavaExprKind, JavaPrecedence, JavaType},
    dialect::JavaRuntimeCallable,
    lower::{JavaIntrinsicExpr, runtime_fallible},
};

#[doc(hidden)]
#[derive(Clone)]
pub enum JavaIntegerConversionsInput {
    WidenI32ToI64 { operand: JavaExpr, result: JavaType },
    NarrowI64ToI32Checked { operand: JavaExpr, result: JavaType },
}

fn lower_integer_conversions(
    input: JavaIntegerConversionsInput,
) -> Result<JavaIntrinsicExpr, Vec<portable_diagnostics::Diagnostic>> {
    Ok(match input {
        JavaIntegerConversionsInput::WidenI32ToI64 { operand, result } => {
            JavaIntrinsicExpr::Infallible(JavaExpr {
                ty: result.clone(),
                precedence: JavaPrecedence::Unary,
                kind: JavaExprKind::Cast {
                    target: result,
                    value: Box::new(operand),
                },
            })
        }
        JavaIntegerConversionsInput::NarrowI64ToI32Checked { operand, result } => {
            runtime_fallible(JavaRuntimeCallable::NarrowI64ToI32, vec![operand], result)
        }
    })
}

java_operation_mapping!(
    JavaIntegerConversions,
    IntegerConversions,
    JavaIntegerConversionsInput,
    JavaIntrinsicExpr,
    lower_integer_conversions
);
