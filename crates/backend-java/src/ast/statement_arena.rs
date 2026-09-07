//! Java AST: statement arena.

use super::statement_model::JavaStmt;
use crate::dialect::JavaDialect;
use portable_codegen::{AstViolation, TargetAstContext, TargetStatementNode};

impl TargetStatementNode<JavaDialect> for JavaStmt {
    fn child_expressions(&self) -> Vec<portable_codegen::TargetExprId> {
        vec![]
    }
    fn verify(&self, context: &TargetAstContext<'_, JavaDialect>) -> Vec<AstViolation> {
        self.verify(context)
    }
}
