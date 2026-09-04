//! Java mapping for `BytesValues`.

use portable_build::BytesValues;

use super::support::java_ast_mapping;
use crate::ast::JavaExpr;

java_ast_mapping!(JavaBytesValues, BytesValues, JavaExpr);
