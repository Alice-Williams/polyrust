//! Exact checked conformance evidence retained by Java interface coercions.

use super::declaration_model::{
    JavaDeclarationKind, JavaMember, JavaMethod, JavaMethodDeclaration,
};
use super::field_metadata::find_type_declaration;
use super::invocations::generated_type_implements;
use super::types::{JavaType, JavaTypeName};
use crate::dialect::JavaDialect;
use portable_codegen::{AstViolation, GeneratedTypeId, TargetAstContext};
use portable_core_ir::{CoreImplementationId, CoreImplementationMethodId, CoreProgram};
use portable_diagnostics::DiagnosticCode;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct JavaInterfaceWitness {
    implementation: CoreImplementationId,
    record: GeneratedTypeId,
    interface: GeneratedTypeId,
    methods: Vec<CoreImplementationMethodId>,
}

impl JavaInterfaceWitness {
    pub(crate) fn from_checked(
        core: &CoreProgram,
        implementation: CoreImplementationId,
        record: GeneratedTypeId,
        interface: GeneratedTypeId,
    ) -> Self {
        let conformance = core
            .implementation(implementation)
            .expect("verified conformance");
        Self {
            implementation,
            record,
            interface,
            methods: conformance.methods.clone(),
        }
    }

    pub(crate) fn verify(
        &self,
        source: &JavaType,
        target: &JavaType,
        context: &TargetAstContext<'_, JavaDialect>,
    ) -> Vec<AstViolation> {
        let record_type = JavaType::Reference(JavaTypeName::Generated(self.record));
        let interface_type = JavaType::Reference(JavaTypeName::Generated(self.interface));
        let record = find_type_declaration(&record_type, context);
        let interface = find_type_declaration(&interface_type, context);
        let valid = source == &record_type
            && target == &interface_type
            && generated_type_implements(self.record, self.interface, context)
            && interface.is_some_and(|declaration| {
                matches!(
                    declaration.kind,
                    JavaDeclarationKind::Interface | JavaDeclarationKind::SealedInterface
                )
            })
            && record.is_some_and(|declaration| {
                self.methods.iter().all(|expected| {
                    declaration.members.iter().any(|member| {
                        matches!(member, JavaMember::Method(JavaMethod {
                            declared: JavaMethodDeclaration::Implementation { method, .. },
                            ..
                        }) if method == expected)
                    })
                })
            });
        if valid {
            vec![]
        } else {
            vec![AstViolation::new(
                DiagnosticCode::UnresolvedReference,
                format!(
                    "Java interface coercion disagrees with conformance {:?}",
                    self.implementation
                ),
            )]
        }
    }
}
