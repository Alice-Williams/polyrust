//! Exact checked conformance evidence retained by Java interface coercions.

use super::declaration_model::{
    JavaDeclarationKind, JavaMember, JavaMethod, JavaMethodDeclaration,
};
use super::field_metadata::find_type_declaration;
use super::invocations::generated_type_implements;
use super::types::{JavaType, JavaTypeName};
use crate::dialect::JavaDialect;
use portable_codegen::{
    AstViolation, GeneratedOrigin, GeneratedTypeId, SynthesisReason, TargetAstContext,
};
use portable_core_ir::{CoreImplementationId, CoreImplementationMethodId, CoreProgram};
use portable_diagnostics::DiagnosticCode;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct JavaInterfaceWitness {
    origin: Origin,
    record: GeneratedTypeId,
    interface: GeneratedTypeId,
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum Origin {
    Core {
        implementation: CoreImplementationId,
        methods: Vec<CoreImplementationMethodId>,
    },
    GeneratedAdapter,
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
            origin: Origin::Core {
                implementation,
                methods: conformance.methods.clone(),
            },
            record,
            interface,
        }
    }

    /// Descriptive local adapter edge, not checked Core conformance authority.
    /// Verification requires the exact synthesized record or constant-only enum
    /// and empty sealed interface
    /// declarations and their actual implements edge in this package.
    pub fn for_generated_adapter(record: GeneratedTypeId, interface: GeneratedTypeId) -> Self {
        Self {
            origin: Origin::GeneratedAdapter,
            record,
            interface,
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
            && interface.as_ref().is_some_and(|declaration| {
                matches!(
                    declaration.kind,
                    JavaDeclarationKind::Interface | JavaDeclarationKind::SealedInterface
                )
            })
            && record
                .as_ref()
                .is_some_and(|declaration| match &self.origin {
                    Origin::Core { methods, .. } => methods.iter().all(|expected| {
                        declaration.members.iter().any(|member| {
                            matches!(member, JavaMember::Method(JavaMethod {
                            declared: JavaMethodDeclaration::Implementation { method, .. },
                            ..
                        }) if method == expected)
                        })
                    }),
                    Origin::GeneratedAdapter => {
                        matches!(
                            declaration.kind,
                            JavaDeclarationKind::Record | JavaDeclarationKind::Enum
                        ) && declaration.type_parameters.is_empty()
                            && interface.as_ref().is_some_and(|interface| {
                                interface.kind == JavaDeclarationKind::SealedInterface
                                    && interface.type_parameters.is_empty()
                                    && interface.members.is_empty()
                            })
                            && [self.record, self.interface].into_iter().all(|id| {
                                context.generated_type(id).is_some_and(|registered| {
                                    registered.origin
                                        == GeneratedOrigin::Synthesized(
                                            SynthesisReason::InterfaceAdapter,
                                        )
                                })
                            })
                    }
                });
        if valid {
            vec![]
        } else {
            let conformance = match &self.origin {
                Origin::Core { implementation, .. } => format!("{implementation:?}"),
                Origin::GeneratedAdapter => format!(
                    "generated adapter {:?} -> {:?}",
                    self.record, self.interface
                ),
            };
            vec![AstViolation::new(
                DiagnosticCode::UnresolvedReference,
                format!("Java interface coercion disagrees with conformance {conformance}"),
            )]
        }
    }
}
