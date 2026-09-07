//! Java AST: file symbols.

use super::file_model::JavaFileItem;
use crate::dialect::JavaDialect;
use portable_codegen::{GeneratedSymbolId, TargetSymbolRef};
use std::collections::BTreeSet;

impl JavaFileItem {
    pub fn declared_symbols(&self) -> Vec<GeneratedSymbolId> {
        match self {
            Self::Type { declared, .. } => declared.clone(),
            Self::RuntimeMembers { .. } => vec![],
        }
    }

    pub fn symbols(&self) -> Vec<TargetSymbolRef<JavaDialect>> {
        let mut symbols = BTreeSet::new();
        match self {
            Self::Type { declaration, .. } => declaration.symbols(&mut symbols),
            Self::RuntimeMembers { helper, members } => {
                for member in members {
                    member.symbols(&mut symbols);
                }
                symbols.remove(&TargetSymbolRef::RuntimeHelper(*helper));
            }
        }
        symbols.into_iter().collect()
    }
}
