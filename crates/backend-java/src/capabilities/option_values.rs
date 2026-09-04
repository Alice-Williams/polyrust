//! Java mapping for `OptionValues`.

use portable_build::OptionValues;

use super::support::java_ast_mapping;
use crate::ast::JavaExpr;

java_ast_mapping!(JavaOptionValues, OptionValues, JavaExpr);
