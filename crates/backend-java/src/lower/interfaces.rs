//! Java lowering: interfaces.

use super::{Lowering, diagnostic};
use crate::ast::{JavaType, JavaTypeDeclaration, JavaTypeName};
use crate::capabilities::{
    JavaInterfaceDeclarationInput, JavaInterfaceMethodInput, JavaInterfacesInput,
    JavaInterfacesNode,
};
use portable_build::CapabilityMapping;
use portable_core_ir::CoreInterfaceId;
use portable_diagnostics::Diagnostic;

impl Lowering<'_> {
    pub(super) fn interface_declaration(
        &self,
        id: CoreInterfaceId,
    ) -> Result<JavaTypeDeclaration, Vec<Diagnostic>> {
        let interface = self.core.interface(id).expect("verified interface");
        let permits = self
            .core
            .implementations()
            .iter()
            .filter(|implementation| implementation.interface == id)
            .map(|implementation| {
                JavaType::Reference(JavaTypeName::Generated(
                    self.records[&implementation.record],
                ))
            })
            .collect();
        let methods = interface
            .methods
            .iter()
            .map(|method_id| {
                let method = self
                    .core
                    .interface_method(*method_id)
                    .expect("verified method");
                Ok(JavaInterfaceMethodInput {
                    declared: self.interface_methods[method_id],
                    name: method.header.name.clone(),
                    parameters: self.parameters(&method.parameters)?,
                    return_type: self.poly_result_type(method.return_type)?,
                })
            })
            .collect::<Result<Vec<_>, Vec<Diagnostic>>>()?;
        match self
            .features
            .mapping_for::<portable_build::Interfaces>()
            .lower(
                &mut (),
                JavaInterfacesInput::Declaration(Box::new(JavaInterfaceDeclarationInput {
                    declared: self.interfaces[&id],
                    visibility: interface.header.visibility,
                    name: interface.header.name.clone(),
                    permits,
                    methods,
                })),
            )? {
            JavaInterfacesNode::Declaration(declaration) => Ok(*declaration),
            JavaInterfacesNode::Type(_)
            | JavaInterfacesNode::Conformance(_)
            | JavaInterfacesNode::Expression(_) => Err(vec![diagnostic(
                "Java Interfaces mapping returned the wrong declaration node",
            )]),
        }
    }
}
