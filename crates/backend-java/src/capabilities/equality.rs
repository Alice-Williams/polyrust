//! Java mapping for `Equality`.

mod mapping_plan;

use portable_build::Equality;

use super::support::java_operation_mapping;
use crate::{
    ast::{JavaExpr, JavaType, JavaUnaryOperator},
    dialect::JavaRuntimeCallable,
    lower::{JavaIntrinsicExpr, runtime_call, unary},
};

#[doc(hidden)]
#[derive(Clone)]
pub enum JavaEqualityInput {
    Equal {
        left: JavaExpr,
        right: JavaExpr,
        result: JavaType,
    },
    NotEqual {
        left: JavaExpr,
        right: JavaExpr,
        result: JavaType,
    },
}

fn lower_equality(
    input: JavaEqualityInput,
) -> Result<JavaIntrinsicExpr, Vec<portable_diagnostics::Diagnostic>> {
    let (left, right, result, negate) = match input {
        JavaEqualityInput::Equal {
            left,
            right,
            result,
        } => (left, right, result, false),
        JavaEqualityInput::NotEqual {
            left,
            right,
            result,
        } => (left, right, result, true),
    };
    let equal = runtime_call(
        JavaRuntimeCallable::SemanticEqual,
        vec![left, right],
        result.clone(),
    );
    Ok(JavaIntrinsicExpr::Infallible(if negate {
        unary(JavaUnaryOperator::Not, equal, result)
    } else {
        equal
    }))
}

java_operation_mapping!(
    JavaEquality,
    Equality,
    JavaEqualityInput,
    JavaIntrinsicExpr,
    lower_equality
);
