//! Java AST: expression arena.

use super::expression_nodes::JavaExpr;
use super::types::type_error;
use crate::dialect::JavaDialect;
use portable_codegen::{AstViolation, TargetAstContext, TargetExpressionNode, TargetTypeRef};

impl TargetExpressionNode<JavaDialect> for JavaExpr {
    fn child_expressions(&self) -> Vec<portable_codegen::TargetExprId> {
        vec![]
    }

    fn verify(
        &self,
        stored_type: &TargetTypeRef<JavaDialect>,
        context: &TargetAstContext<'_, JavaDialect>,
    ) -> Vec<AstViolation> {
        let mut violations = self.verify(context);
        if &context.dialect().registered_type(&self.ty) != stored_type {
            violations.push(type_error(
                "Java type disagrees with shared target-AST type",
            ));
        }
        violations
    }
}
