//! Java AST: statement model.

use super::blocks::{JavaBlock, JavaLocalFinality};
use super::expression_model::JavaLiteral;
use super::expression_nodes::JavaExpr;
use super::identifiers::JavaIdentifier;
use super::types::JavaType;
use portable_codegen::{GeneratedTypeId, GeneratedValueId};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum JavaPattern {
    Default,
    Literal(JavaLiteral),
    EnumVariant {
        enumeration: GeneratedTypeId,
        variant: GeneratedValueId,
    },
    Type {
        ty: JavaType,
        binding: JavaIdentifier,
    },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct JavaSwitchArm {
    pub pattern: JavaPattern,
    pub body: JavaBlock,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct JavaCatch {
    pub exception_type: JavaType,
    pub binding: JavaIdentifier,
    pub body: JavaBlock,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum JavaStmt {
    Local {
        finality: JavaLocalFinality,
        ty: JavaType,
        name: JavaIdentifier,
        value: Option<JavaExpr>,
    },
    Assign {
        target: JavaExpr,
        value: JavaExpr,
    },
    Expression(JavaExpr),
    Return(Option<JavaExpr>),
    If {
        condition: JavaExpr,
        then_block: JavaBlock,
        else_block: Option<JavaBlock>,
    },
    ForEach {
        binding_type: JavaType,
        binding: JavaIdentifier,
        iterable: JavaExpr,
        body: JavaBlock,
    },
    While {
        condition: JavaExpr,
        body: JavaBlock,
    },
    Switch {
        value: JavaExpr,
        arms: Vec<JavaSwitchArm>,
    },
    TryCatch {
        try_block: JavaBlock,
        catches: Vec<JavaCatch>,
    },
    Throw(JavaExpr),
    ThrowAssertion(JavaExpr),
    Break,
    Continue,
}
