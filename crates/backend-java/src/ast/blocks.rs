//! Java AST: blocks.

use super::statement_model::JavaStmt;
use crate::dialect::JavaDialect;
use portable_codegen::{AstViolation, TargetAstContext, TargetSymbolRef};
use std::collections::BTreeSet;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum JavaLocalFinality {
    Final,
    Mutable,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct JavaBlock {
    pub statements: Vec<JavaStmt>,
}

impl JavaBlock {
    pub fn new(statements: Vec<JavaStmt>) -> Self {
        Self { statements }
    }
    pub fn symbols(&self, symbols: &mut BTreeSet<TargetSymbolRef<JavaDialect>>) {
        for statement in &self.statements {
            statement.symbols(symbols);
        }
    }
    pub fn verify(&self, context: &TargetAstContext<'_, JavaDialect>) -> Vec<AstViolation> {
        self.statements
            .iter()
            .flat_map(|value| value.verify(context))
            .collect()
    }
}
