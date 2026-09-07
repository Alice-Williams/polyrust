//! Java mapping for the complete `Loops` capability.

use portable_build::{CapabilityMapping, Loops};
use portable_diagnostics::Diagnostic;

use super::support::{JavaCapabilityMapping, sealed};
use crate::{
    ast::{JavaBlock, JavaExpr, JavaStmt, JavaType},
    dialect::JavaDialect,
    lower::identifier,
};

#[doc(hidden)]
pub enum JavaLoopsInput {
    ForEach {
        binding_type: JavaType,
        binding: String,
        iterable: Box<JavaExpr>,
        body: JavaBlock,
    },
    BindingRead {
        binding_type: JavaType,
        binding: String,
    },
}

#[doc(hidden)]
pub enum JavaLoopsNode {
    Statement(Box<JavaStmt>),
    Expression(Box<JavaExpr>),
}

impl sealed::JavaMappingOutput for JavaLoopsNode {}
impl super::support::JavaMappingOutput for JavaLoopsNode {}

#[doc(hidden)]
#[derive(Clone, Copy, Debug, Default)]
pub struct JavaLoops;

impl sealed::JavaCapabilityMapping for JavaLoops {}
impl JavaCapabilityMapping for JavaLoops {}

impl CapabilityMapping<JavaDialect> for JavaLoops {
    type Capability = Loops;
    type Context = ();
    type Input = JavaLoopsInput;
    type Output = JavaLoopsNode;
    type Error = Vec<Diagnostic>;

    fn lower(
        &self,
        _context: &mut Self::Context,
        input: Self::Input,
    ) -> Result<Self::Output, Self::Error> {
        Ok(match input {
            JavaLoopsInput::ForEach {
                binding_type,
                binding,
                iterable,
                body,
            } => JavaLoopsNode::Statement(Box::new(JavaStmt::ForEach {
                binding_type,
                binding: identifier(&binding),
                iterable: *iterable,
                body,
            })),
            JavaLoopsInput::BindingRead {
                binding_type,
                binding,
            } => JavaLoopsNode::Expression(Box::new(JavaExpr::local(
                binding_type,
                identifier(&binding),
            ))),
        })
    }
}
