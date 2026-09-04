//! Java mapping for `I32Values`.

use portable_build::I32Values;

use super::support::java_ast_mapping;
use crate::ast::JavaExpr;

java_ast_mapping!(JavaI32Values, I32Values, JavaExpr);
