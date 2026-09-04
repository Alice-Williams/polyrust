//! Java mapping for the complete `Conditionals` capability.

use portable_build::{CapabilityMapping, Conditionals};
use portable_diagnostics::Diagnostic;

use super::support::{JavaCapabilityMapping, sealed};
use crate::{
    ast::{JavaBlock, JavaExpr, JavaIdentifier, JavaLocalFinality, JavaStmt, JavaType},
    dialect::JavaDialect,
};

#[doc(hidden)]
pub struct JavaConditionalValueInput {
    pub(crate) prefix: Vec<JavaStmt>,
    pub(crate) condition: JavaExpr,
    pub(crate) result_name: JavaIdentifier,
    pub(crate) result_type: JavaType,
    pub(crate) then_block: JavaBlock,
    pub(crate) else_block: JavaBlock,
}

#[doc(hidden)]
pub struct JavaConditionalStatementInput {
    pub condition: JavaExpr,
    pub then_block: JavaBlock,
    pub else_block: Option<JavaBlock>,
}

#[doc(hidden)]
pub enum JavaConditionalsInput {
    Statement(Box<JavaConditionalStatementInput>),
    Value(Box<JavaConditionalValueInput>),
}

#[doc(hidden)]
pub enum JavaConditionalsNode {
    Statement(Box<JavaStmt>),
    Value {
        statements: Vec<JavaStmt>,
        value: Box<JavaExpr>,
    },
}

#[doc(hidden)]
#[derive(Clone, Copy, Debug, Default)]
pub struct JavaConditionals;

impl sealed::JavaCapabilityMapping for JavaConditionals {}
impl JavaCapabilityMapping for JavaConditionals {}

impl CapabilityMapping<JavaDialect> for JavaConditionals {
    type Capability = Conditionals;
    type Context = ();
    type Input = JavaConditionalsInput;
    type Output = JavaConditionalsNode;
    type Error = Vec<Diagnostic>;

    fn lower(
        &self,
        _context: &mut Self::Context,
        input: Self::Input,
    ) -> Result<Self::Output, Self::Error> {
        Ok(match input {
            JavaConditionalsInput::Statement(input) => {
                JavaConditionalsNode::Statement(Box::new(JavaStmt::If {
                    condition: input.condition,
                    then_block: input.then_block,
                    else_block: input.else_block,
                }))
            }
            JavaConditionalsInput::Value(input) => {
                let name = input.result_name;
                let value = JavaExpr::local(input.result_type.clone(), name.clone());
                let mut statements = input.prefix;
                statements.push(JavaStmt::Local {
                    finality: JavaLocalFinality::Mutable,
                    ty: input.result_type,
                    name,
                    value: None,
                });
                statements.push(JavaStmt::If {
                    condition: input.condition,
                    then_block: input.then_block,
                    else_block: Some(input.else_block),
                });
                JavaConditionalsNode::Value {
                    statements,
                    value: Box::new(value),
                }
            }
        })
    }
}
