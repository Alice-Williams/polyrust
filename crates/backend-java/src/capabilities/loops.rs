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
        iterable: JavaExpr,
        body: JavaBlock,
    },
    While {
        condition: JavaExpr,
        body: JavaBlock,
    },
    Break,
    Continue,
}

#[doc(hidden)]
#[derive(Clone, Copy, Debug, Default)]
pub struct JavaLoops;

impl sealed::JavaCapabilityMapping for JavaLoops {}
impl JavaCapabilityMapping for JavaLoops {}

impl CapabilityMapping<JavaDialect> for JavaLoops {
    type Capability = Loops;
    type Context = ();
    type Input = JavaLoopsInput;
    type Output = JavaStmt;
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
            } => JavaStmt::ForEach {
                binding_type,
                binding: identifier(&binding),
                iterable,
                body,
            },
            JavaLoopsInput::While { condition, body } => JavaStmt::While { condition, body },
            JavaLoopsInput::Break => JavaStmt::Break,
            JavaLoopsInput::Continue => JavaStmt::Continue,
        })
    }
}
