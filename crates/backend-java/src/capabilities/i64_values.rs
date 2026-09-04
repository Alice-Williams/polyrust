//! Java mapping for `I64Values`.

use portable_build::I64Values;

use super::support::java_ast_mapping;
use crate::ast::JavaExpr;

java_ast_mapping!(JavaI64Values, I64Values, JavaExpr);
