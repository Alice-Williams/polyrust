//! Register the target-only declaration requested by the Interfaces mapping.

use super::{Lowering, diagnostic};
use crate::capabilities::{JavaInterfacesInput, JavaInterfacesNode, JavaUninhabitedInterfaceInput};
use portable_build::{CapabilityMapping, Interfaces};
use portable_codegen::GeneratedSymbolId;
use portable_core_ir::CoreDeclaration;
use portable_diagnostics::Diagnostic;

impl Lowering<'_> {
    pub(super) fn register_interface_sealing(&mut self) -> Result<(), Vec<Diagnostic>> {
        for declaration in &self.core.module().declarations {
            let CoreDeclaration::Interface(id) = *declaration else {
                continue;
            };
            if self
                .core
                .implementations()
                .iter()
                .any(|value| value.interface == id)
            {
                continue;
            }
            let interface = self.core.interface(id).expect("verified interface");
            let request = self.features.mapping_for::<Interfaces>().lower(
                &mut (),
                JavaInterfacesInput::UninhabitedType {
                    interface: self.interfaces[&id],
                    name: self.names.uninhabited(id).as_str().to_owned(),
                    source: interface.header.source.clone(),
                },
            )?;
            let JavaInterfacesNode::UninhabitedType(request) = request else {
                return Err(vec![diagnostic(
                    "Java Interfaces mapping did not return its uninhabited declaration request",
                )]);
            };
            let name = request.name.clone();
            let declared = self.builder.generated_type(*request);
            self.declared.push(GeneratedSymbolId::Type(declared));
            self.uninhabited
                .insert(id, JavaUninhabitedInterfaceInput { declared, name });
        }
        Ok(())
    }
}
