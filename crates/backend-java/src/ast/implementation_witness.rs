//! Unforgeable checked identity for each concrete interface implementation method.

use super::{JavaMethodDeclaration, JavaTypeDeclaration};
use crate::dialect::JavaDialect;
use portable_codegen::{
    GeneratedInterfaceMethodId, GeneratedOrigin, GeneratedTypeId, TargetAstContext,
};
use portable_core_ir::{
    CoreDeclaration, CoreImplementationMethodId, CoreInterfaceId, CoreProgram, CoreRecordId,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct JavaImplementationWitness {
    method: CoreImplementationMethodId,
    record: CoreRecordId,
    interface: CoreInterfaceId,
    owner: GeneratedTypeId,
    target_method: GeneratedInterfaceMethodId,
}

impl JavaImplementationWitness {
    pub(crate) fn from_checked(
        core: &CoreProgram,
        method: CoreImplementationMethodId,
        owner: GeneratedTypeId,
        target_method: GeneratedInterfaceMethodId,
    ) -> Self {
        let value = core.implementation_method(method).expect("checked method");
        let implementation = core
            .implementation(value.implementation)
            .expect("checked implementation");
        assert!(implementation.methods.contains(&method));
        Self {
            method,
            record: implementation.record,
            interface: implementation.interface,
            owner,
            target_method,
        }
    }

    pub(super) fn matches(
        self,
        declaration: &JavaTypeDeclaration,
        method: JavaMethodDeclaration,
        context: &TargetAstContext<'_, JavaDialect>,
    ) -> bool {
        matches!(method, JavaMethodDeclaration::Implementation { method, interface, .. }
            if method == self.method && interface == self.target_method)
            && declaration.declared == Some(self.owner)
            && context.generated_type(self.owner).is_some_and(|owner| {
                owner.origin
                    == GeneratedOrigin::CoreDeclaration(CoreDeclaration::Record(self.record))
            })
            && context
                .interface_method(self.target_method)
                .is_some_and(|method| {
                    method.origin
                        == GeneratedOrigin::CoreDeclaration(CoreDeclaration::Interface(
                            self.interface,
                        ))
                        && context.generated_type(method.owner).is_some_and(|owner| {
                            owner.origin
                                == GeneratedOrigin::CoreDeclaration(CoreDeclaration::Interface(
                                    self.interface,
                                ))
                        })
                })
    }
}
