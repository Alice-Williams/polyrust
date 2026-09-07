//! Java dialect: arena nodes.

use super::JavaDialect;
use super::known_callables::JavaKnownCallable;
use super::known_constructors::JavaKnownConstructor;
use super::known_methods::JavaKnownMethod;
use portable_codegen::{
    AstViolation, TargetAstContext, TargetExprId, TargetExpressionNode, TargetStatementNode,
    TargetTypeRef,
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum JavaArenaExpression {
    KnownCall {
        callable: JavaKnownCallable,
        arguments: Vec<TargetExprId>,
    },
    KnownConstructor {
        constructor: JavaKnownConstructor,
        arguments: Vec<TargetExprId>,
    },
    KnownMethod {
        method: JavaKnownMethod,
        receiver: TargetExprId,
        arguments: Vec<TargetExprId>,
    },
}

impl TargetExpressionNode<JavaDialect> for JavaArenaExpression {
    fn child_expressions(&self) -> Vec<TargetExprId> {
        match self {
            Self::KnownCall { arguments, .. } | Self::KnownConstructor { arguments, .. } => {
                arguments.clone()
            }
            Self::KnownMethod {
                receiver,
                arguments,
                ..
            } => std::iter::once(*receiver)
                .chain(arguments.iter().copied())
                .collect(),
        }
    }

    fn verify(
        &self,
        _stored_type: &TargetTypeRef<JavaDialect>,
        _context: &TargetAstContext<'_, JavaDialect>,
    ) -> Vec<AstViolation> {
        vec![]
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct JavaArenaStatement(pub Vec<TargetExprId>);

impl TargetStatementNode<JavaDialect> for JavaArenaStatement {
    fn child_expressions(&self) -> Vec<TargetExprId> {
        self.0.clone()
    }
    fn verify(&self, _context: &TargetAstContext<'_, JavaDialect>) -> Vec<AstViolation> {
        vec![]
    }
}
