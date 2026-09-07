//! Typed statement construction utilities.

use super::call_builders::new_known;
use super::expression_builders::{string_literal, structural_field, this_value};
use crate::ast::{JavaExpr, JavaKnownType, JavaStmt, JavaType};
use crate::dialect::JavaKnownConstructor;

pub(super) fn assign_component(
    owner: JavaType,
    name: &str,
    ty: JavaType,
    value: JavaExpr,
) -> JavaStmt {
    JavaStmt::Assign {
        target: structural_field(this_value(owner), name, ty),
        value,
    }
}
pub(super) fn illegal_argument(message: &str) -> JavaStmt {
    JavaStmt::Throw(new_known(
        JavaKnownConstructor::IllegalArgumentExceptionString,
        JavaType::known(JavaKnownType::IllegalArgumentException),
        vec![string_literal(message)],
    ))
}
pub(super) fn illegal_state(message: &str) -> JavaStmt {
    JavaStmt::Throw(new_known(
        JavaKnownConstructor::IllegalStateExceptionString,
        JavaType::known(JavaKnownType::IllegalStateException),
        vec![string_literal(message)],
    ))
}
