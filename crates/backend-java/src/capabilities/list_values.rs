//! Java mapping for `ListValues`.

use portable_build::ListValues;

use super::support::java_ast_mapping;
use crate::ast::JavaExpr;

java_ast_mapping!(JavaListValues, ListValues, JavaExpr);
