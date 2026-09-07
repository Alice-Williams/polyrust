//! Java lowering: callable declarations.

use super::{BlockMode, Lowering, diagnostic};
use crate::ast::{
    JavaBlock, JavaExpr, JavaField, JavaIdentifier, JavaLocalFinality, JavaMethod,
    JavaMethodDeclaration, JavaParameter, JavaStmt, JavaType,
};
use crate::capabilities::{
    JavaConstantsInput, JavaConstantsNode, JavaFunctionDeclarationInput, JavaFunctionsInput,
    JavaFunctionsNode, JavaInterfaceImplementationInput,
};
use portable_build::CapabilityMapping;
use portable_core_ir::{CoreConstantId, CoreFunctionId, CoreImplementationMethodId};
use portable_diagnostics::Diagnostic;

impl Lowering<'_> {
    pub(super) fn parameters(
        &self,
        values: &[portable_core_ir::CoreParameter],
    ) -> Result<Vec<JavaParameter>, Vec<Diagnostic>> {
        values
            .iter()
            .zip(self.names.parameters(values))
            .map(|(parameter, name)| {
                Ok(JavaParameter {
                    ty: self.ty(parameter.ty)?,
                    name,
                    final_parameter: true,
                })
            })
            .collect()
    }

    pub(super) fn constant_field(&self, id: CoreConstantId) -> Result<JavaField, Vec<Diagnostic>> {
        let value = self.core.constant(id).expect("verified constant");
        match self
            .features
            .mapping_for::<portable_build::Constants>()
            .lower(
                &mut (),
                JavaConstantsInput::Declaration {
                    declared: self.constants[&id],
                    visibility: value.header.visibility,
                    name: self.names.constant(id).as_str().to_owned(),
                    ty: self.ty(value.ty)?,
                    initializer: Box::new(self.constant_expr(&value.value, value.ty)?),
                },
            )? {
            JavaConstantsNode::Declaration(field) => Ok(field),
            JavaConstantsNode::Expression(_) => Err(vec![diagnostic(
                "Java Constants mapping returned an expression for a declaration",
            )]),
        }
    }

    pub(super) fn function_method(
        &self,
        id: CoreFunctionId,
    ) -> Result<JavaMethod, Vec<Diagnostic>> {
        let value = self.core.function(id).expect("verified function");
        let (parameters, mut boundary) = self.callable_parameters(&value.parameters)?;
        let mut body = self.block(value.body, BlockMode::ReturnResult, value.return_type)?;
        boundary.append(&mut body.statements);
        match self
            .features
            .mapping_for::<portable_build::Functions>()
            .lower(
                &mut (),
                JavaFunctionsInput::Declaration(Box::new(JavaFunctionDeclarationInput {
                    declared: JavaMethodDeclaration::Callable(self.functions[&id]),
                    visibility: value.header.visibility,
                    name: self.names.function(id).as_str().to_owned(),
                    parameters,
                    return_type: self.poly_result_type(value.return_type)?,
                    body: JavaBlock::new(boundary),
                })),
            )? {
            JavaFunctionsNode::Declaration(method) => Ok(method),
            JavaFunctionsNode::Expression(_) | JavaFunctionsNode::Statement(_) => {
                Err(vec![diagnostic(
                    "Java Functions mapping returned an expression for a declaration",
                )])
            }
        }
    }

    pub(super) fn implementation_method_input(
        &self,
        id: CoreImplementationMethodId,
    ) -> Result<JavaInterfaceImplementationInput, Vec<Diagnostic>> {
        let value = self
            .core
            .implementation_method(id)
            .expect("verified implementation method");
        let (parameters, mut boundary) = self.callable_parameters(&value.parameters)?;
        let mut body = self.block(value.body, BlockMode::ReturnResult, value.return_type)?;
        boundary.append(&mut body.statements);
        Ok(JavaInterfaceImplementationInput {
            method: id,
            witness: crate::ast::JavaImplementationWitness::from_checked(
                self.core,
                id,
                self.records[&self
                    .core
                    .implementation(value.implementation)
                    .expect("checked implementation")
                    .record],
                self.interface_methods[&value.interface_method],
            ),
            interface_method: self.interface_methods[&value.interface_method],
            interface_method_name: self
                .names
                .method(value.interface_method)
                .as_str()
                .to_owned(),
            parameters,
            return_type: self.poly_result_type(value.return_type)?,
            body: JavaBlock::new(boundary),
        })
    }

    fn callable_parameters(
        &self,
        values: &[portable_core_ir::CoreParameter],
    ) -> Result<(Vec<JavaParameter>, Vec<JavaStmt>), Vec<Diagnostic>> {
        let mut parameters = Vec::with_capacity(values.len());
        let mut boundary = Vec::new();
        for (index, parameter) in values.iter().enumerate() {
            let name = self
                .names
                .local(parameter.local.expect("checked callable parameter binding"))
                .clone();
            let ty = self.ty(parameter.ty)?;
            if matches!(ty, JavaType::Primitive(_)) {
                parameters.push(JavaParameter {
                    ty,
                    name,
                    final_parameter: true,
                });
                continue;
            }
            let input_name = JavaIdentifier::new(format!("__polyrust_input_{index}"))
                .expect("internal Java parameter identifier is valid");
            let input = JavaExpr::local(ty.clone(), input_name.clone());
            let normalized = self.normalize_boundary_value(parameter.ty, input)?;
            parameters.push(JavaParameter {
                ty: ty.clone(),
                name: input_name,
                final_parameter: true,
            });
            boundary.extend(normalized.statements);
            boundary.push(JavaStmt::Local {
                finality: JavaLocalFinality::Final,
                ty,
                name,
                value: Some(normalized.value),
            });
        }
        Ok((parameters, boundary))
    }
}
