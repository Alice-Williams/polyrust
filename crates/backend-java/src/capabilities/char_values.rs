//! Java mapping for `CharValues`.

use portable_build::CharValues;

use super::support::java_ast_mapping;
use crate::ast::JavaExpr;

java_ast_mapping!(JavaCharValues, CharValues, JavaExpr);
