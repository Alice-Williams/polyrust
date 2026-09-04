//! Java mapping for `BoolValues`.

use portable_build::BoolValues;

use super::support::java_ast_mapping;
use crate::ast::JavaExpr;

java_ast_mapping!(JavaBoolValues, BoolValues, JavaExpr);
