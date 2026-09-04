//! Java mapping for `F64Values`.

use portable_build::F64Values;

use super::support::java_ast_mapping;
use crate::ast::JavaExpr;

java_ast_mapping!(JavaF64Values, F64Values, JavaExpr);
