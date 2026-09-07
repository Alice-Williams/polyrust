//! Java lowering: records.

use super::declaration_builders::identifier;
use super::{Lowering, diagnostic};
use crate::ast::{
    JavaHeritage, JavaKnownType, JavaMember, JavaRecordComponent, JavaRecordComponentOrigin,
    JavaRuntimeMember, JavaType, JavaTypeDeclaration,
};
use crate::capabilities::{
    JavaInterfaceConformanceInput, JavaInterfaceConformancePlan, JavaInterfacesInput,
    JavaInterfacesNode, JavaRecordDeclarationInput, JavaRecordsInput, JavaRecordsNode,
};
use crate::dialect::JavaRuntimeCallable;
use portable_build::CapabilityMapping;
use portable_core_ir::{CoreDeclaration, CoreRecordId};
use portable_diagnostics::Diagnostic;

impl Lowering<'_> {
    pub(super) fn record_declaration(
        &self,
        id: CoreRecordId,
    ) -> Result<JavaTypeDeclaration, Vec<Diagnostic>> {
        let record = self.core.record(id).expect("verified record");
        let mut members = vec![
            JavaMember::Constructor(self.generated_record_constructor(
                self.records[&id],
                &record.header.name,
                record.header.visibility,
                &record.fields,
            )?),
            JavaMember::Method(self.value_equality_method(
                self.records[&id],
                &record.fields,
                JavaRuntimeCallable::SemanticEqual,
                JavaRuntimeMember::SemanticEquals,
            )?),
            JavaMember::Method(self.value_equality_method(
                self.records[&id],
                &record.fields,
                JavaRuntimeCallable::DeepEqual,
                JavaRuntimeMember::DeepEquals,
            )?),
        ];
        let mut heritage_interfaces = vec![JavaType::known(JavaKnownType::RuntimeSemanticValue)];
        let mut conformance_interfaces = Vec::new();
        let mut conformance_methods = Vec::new();
        for declaration in &self.core.module().declarations {
            if let CoreDeclaration::Implementation(implementation_id) = *declaration {
                let implementation = self
                    .core
                    .implementation(implementation_id)
                    .expect("verified implementation");
                if implementation.record == id {
                    conformance_interfaces.push(self.interfaces[&implementation.interface]);
                    for method in &implementation.methods {
                        conformance_methods.push(self.implementation_method_input(*method)?);
                    }
                }
            }
        }
        if !conformance_interfaces.is_empty() {
            let conformance = self
                .features
                .mapping_for::<portable_build::Interfaces>()
                .lower(
                    &mut (),
                    JavaInterfacesInput::Conformance(Box::new(JavaInterfaceConformanceInput {
                        interfaces: conformance_interfaces,
                        methods: conformance_methods,
                    })),
                )?;
            let JavaInterfacesNode::Conformance(JavaInterfaceConformancePlan {
                heritage,
                members: mut implementation_members,
            }) = conformance
            else {
                return Err(vec![diagnostic(
                    "Java Interfaces mapping returned the wrong conformance node",
                )]);
            };
            let JavaHeritage::Interfaces(mut generated_interfaces) = heritage else {
                return Err(vec![diagnostic(
                    "Java Interfaces mapping returned non-interface conformance heritage",
                )]);
            };
            heritage_interfaces.append(&mut generated_interfaces);
            members.append(&mut implementation_members);
        }
        match self
            .features
            .mapping_for::<portable_build::Records>()
            .lower(
                &mut (),
                JavaRecordsInput::Declaration(Box::new(JavaRecordDeclarationInput {
                    declared: self.records[&id],
                    visibility: record.header.visibility,
                    name: record.header.name.clone(),
                    components: record
                        .fields
                        .iter()
                        .map(|field| {
                            let value = self.core.field(*field).expect("verified field");
                            Ok(JavaRecordComponent {
                                origin: JavaRecordComponentOrigin::Core(*field),
                                ty: self.ty(value.ty)?,
                                name: identifier(&value.header.name),
                            })
                        })
                        .collect::<Result<Vec<_>, Vec<Diagnostic>>>()?,
                    heritage: if heritage_interfaces.is_empty() {
                        JavaHeritage::None
                    } else {
                        JavaHeritage::Interfaces(heritage_interfaces)
                    },
                    members,
                })),
            )? {
            JavaRecordsNode::Declaration(declaration) => Ok(declaration),
            JavaRecordsNode::Type(_) | JavaRecordsNode::Expression(_) => Err(vec![diagnostic(
                "Java Records mapping returned an expression for a declaration",
            )]),
        }
    }
}
