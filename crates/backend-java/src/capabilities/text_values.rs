//! Java mapping for `TextValues`.

use portable_build::TextValues;

use super::support::java_ast_mapping;
use crate::ast::JavaExpr;

java_ast_mapping!(JavaTextValues, TextValues, JavaExpr);
