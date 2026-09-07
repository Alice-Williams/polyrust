//! Java lowering: mapping nodes.

use super::{Lowering, diagnostic};
use crate::ast::{JavaExpr, JavaStmt};
use crate::capabilities::{
    JavaFunctionsInput, JavaFunctionsNode, JavaInterfacesInput, JavaInterfacesNode,
    JavaRecordsInput, JavaRecordsNode,
};
use portable_build::CapabilityMapping;
use portable_diagnostics::Diagnostic;

impl Lowering<'_> {
    pub(super) fn lower_function_expr(
        &self,
        input: JavaFunctionsInput,
    ) -> Result<JavaExpr, Vec<Diagnostic>> {
        match self
            .features
            .mapping_for::<portable_build::Functions>()
            .lower(&mut (), input)?
        {
            JavaFunctionsNode::Expression(value) => Ok(value),
            JavaFunctionsNode::Declaration(_) | JavaFunctionsNode::Statement(_) => {
                Err(vec![diagnostic(
                    "Java Functions mapping returned a declaration for an expression",
                )])
            }
        }
    }

    pub(super) fn lower_function_return(
        &self,
        value: JavaExpr,
    ) -> Result<JavaStmt, Vec<Diagnostic>> {
        match self
            .features
            .mapping_for::<portable_build::Functions>()
            .lower(&mut (), JavaFunctionsInput::Return { value })?
        {
            JavaFunctionsNode::Statement(statement) => Ok(*statement),
            JavaFunctionsNode::Declaration(_) | JavaFunctionsNode::Expression(_) => {
                Err(vec![diagnostic(
                    "Java Functions mapping returned a non-statement for a return",
                )])
            }
        }
    }

    pub(super) fn lower_record_expr(
        &self,
        input: JavaRecordsInput,
    ) -> Result<JavaExpr, Vec<Diagnostic>> {
        match self
            .features
            .mapping_for::<portable_build::Records>()
            .lower(&mut (), input)?
        {
            JavaRecordsNode::Expression(value) => Ok(value),
            JavaRecordsNode::Type(_) | JavaRecordsNode::Declaration(_) => Err(vec![diagnostic(
                "Java Records mapping returned a declaration for an expression",
            )]),
        }
    }

    pub(super) fn lower_interface_expr(
        &self,
        input: JavaInterfacesInput,
    ) -> Result<JavaExpr, Vec<Diagnostic>> {
        match self
            .features
            .mapping_for::<portable_build::Interfaces>()
            .lower(&mut (), input)?
        {
            JavaInterfacesNode::Expression(value) => Ok(*value),
            JavaInterfacesNode::Type(_)
            | JavaInterfacesNode::Declaration(_)
            | JavaInterfacesNode::Conformance(_) => Err(vec![diagnostic(
                "Java Interfaces mapping returned a declaration for an expression",
            )]),
        }
    }
}
