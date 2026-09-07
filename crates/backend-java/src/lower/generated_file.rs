//! Java lowering: generated file.

use super::Lowering;
use crate::ast::JavaMember;
use crate::capabilities::{JavaModuleInput, JavaTypeAliasInput};
use portable_build::CapabilityMapping;
use portable_core_ir::CoreDeclaration;
use portable_diagnostics::Diagnostic;

impl Lowering<'_> {
    pub(super) fn generated_file(
        &mut self,
    ) -> Result<portable_codegen::TargetFileId, Vec<Diagnostic>> {
        let mut members = Vec::new();
        for id in self.ordered_constant_ids()? {
            members.push(JavaMember::Field(self.constant_field(id)?));
        }
        for declaration in &self.core.module().declarations {
            match *declaration {
                CoreDeclaration::Record(id) => {
                    members.push(JavaMember::NestedType(self.record_declaration(id)?));
                }
                CoreDeclaration::Enum(id) => {
                    members.extend(
                        self.enum_declarations(id)?
                            .into_iter()
                            .map(JavaMember::NestedType),
                    );
                }
                CoreDeclaration::Interface(id) => {
                    members.push(JavaMember::NestedType(self.interface_declaration(id)?));
                }
                CoreDeclaration::Function(id) => {
                    members.push(JavaMember::Method(self.function_method(id)?))
                }
                CoreDeclaration::Alias(id) => {
                    let alias = self.core.alias(id).expect("verified alias");
                    let _erased = self
                        .features
                        .mapping_for::<portable_build::TypeAliases>()
                        .lower(
                            &mut (),
                            JavaTypeAliasInput {
                                name: alias.header.name.clone(),
                                target: self.ty(alias.target)?,
                            },
                        )?;
                }
                CoreDeclaration::Constant(_)
                | CoreDeclaration::Implementation(_)
                | CoreDeclaration::Test(_) => {}
            }
        }
        members.extend(self.public_tagged_value_factories()?);
        let file = self
            .features
            .mapping_for::<portable_build::Modules>()
            .lower(
                &mut (),
                JavaModuleInput {
                    entry: self.entry.expect("Java module entry registered"),
                    declared: self.declared.clone(),
                    members,
                },
            )?;
        Ok(self.builder.file(file))
    }
}
