//! Resource effects of one already-admitted C syntax node.
use super::{Measurements, add, profile::Node};
use crate::ast::{CDeclarationKind, CDefinitionKind, CFileItem, CReturnType, CStatementKind};
use crate::ownership::layout::Layouts;

pub(super) fn observe(
    measured: &mut Measurements,
    layouts: &mut Layouts<'_>,
    node: Node<'_>,
    depth: usize,
) -> Result<(), String> {
    add(&mut measured.nodes, 1)?;
    measured.depth = measured.depth.max(depth);
    match node {
        Node::Item(CFileItem::StaticAssert(assertion)) => {
            measured.max_diagnostic_bytes = measured
                .max_diagnostic_bytes
                .max(assertion.diagnostic().display_text().len());
            // Fixed closed platform grammar: at most eight child/type nodes.
            add(&mut measured.nodes, 8)?;
            measured.depth = measured.depth.max(4);
            add(
                &mut measured.diagnostic_bytes,
                assertion.diagnostic().bytes().len() as u64,
            )?;
        }
        Node::Item(CFileItem::Comment(comment)) => {
            add(&mut measured.comment_bytes, comment.text().len() as u64)?
        }
        Node::Item(CFileItem::Declaration(declaration)) => match declaration.kind() {
            CDeclarationKind::Aggregate { members, .. } => {
                measured.max_fields = measured.max_fields.max(members.len());
            }
            CDeclarationKind::FunctionPrototype { function, .. } => {
                measured.max_parameters = measured
                    .max_parameters
                    .max(function.signature().parameters().len());
                // Parameter declarators are not child nodes in the profile walk.
                add(
                    &mut measured.nodes,
                    function.signature().parameters().len() as u64,
                )?;
            }
            _ => {}
        },
        Node::Item(CFileItem::Definition(definition)) => {
            if let CDefinitionKind::Function {
                function,
                parameters,
                ..
            } = definition.kind()
            {
                measured.max_parameters = measured.max_parameters.max(parameters.len());
                add(&mut measured.nodes, parameters.len() as u64)?;
                for parameter in parameters {
                    add(&mut measured.automatic_objects, 1)?;
                    add(
                        &mut measured.automatic_bytes,
                        layouts
                            .object(parameter.ty())
                            .map_err(|e| e.to_string())?
                            .size(),
                    )?;
                }
                if let CReturnType::Value(result) = function.signature().return_type() {
                    add(
                        &mut measured.value_bytes,
                        layouts
                            .object(result.declared_type())
                            .map_err(|e| e.to_string())?
                            .size(),
                    )?;
                }
            }
        }
        Node::Statement(statement) => {
            if let CStatementKind::Declare(local) = statement.kind() {
                add(&mut measured.automatic_objects, 1)?;
                add(
                    &mut measured.automatic_bytes,
                    layouts
                        .object(local.local().ty())
                        .map_err(|e| e.to_string())?
                        .size(),
                )?;
            }
        }
        Node::Value(value) => add(
            &mut measured.value_bytes,
            layouts
                .object(value.ty())
                .map_err(|e| e.to_string())?
                .size(),
        )?,
        _ => {}
    }
    Ok(())
}
