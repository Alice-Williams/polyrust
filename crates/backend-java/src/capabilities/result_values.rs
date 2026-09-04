//! Java mapping for `ResultValues`.

use portable_build::ResultValues;

use super::support::java_ast_mapping;
use crate::ast::JavaExpr;

java_ast_mapping!(JavaResultValues, ResultValues, JavaExpr);
