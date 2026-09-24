//! Convert original exported identities to this consuming scope, atomically.
use super::JavaDependencyScope;
use crate::{
    ast::{JavaMethodSignature, JavaType},
    dialect::{JavaDependencySignature, JavaDependencyType},
};
use portable_diagnostics::Diagnostic;

impl JavaDependencyScope {
    pub(super) fn bind_signature(
        mut self,
        signature: &JavaDependencySignature,
    ) -> Result<(Self, JavaMethodSignature), Vec<Diagnostic>> {
        let mut parameters = Vec::with_capacity(signature.parameters().len());
        for ty in signature.parameters() {
            let (next, bound) = self.bind_type(ty)?;
            self = next;
            parameters.push(bound);
        }
        let (scope, result) = self.bind_type(signature.result())?;
        Ok((
            scope,
            JavaMethodSignature {
                receiver: None,
                parameters,
                result,
                checked_exceptions: vec![],
                nullable_result: false,
                pure: true,
            },
        ))
    }
    fn bind_type(self, ty: &JavaDependencyType) -> Result<(Self, JavaType), Vec<Diagnostic>> {
        match ty {
            JavaDependencyType::Primitive(primitive) => Ok((self, JavaType::primitive(*primitive))),
            JavaDependencyType::Result(original) => {
                let (scope, imported) = self.import_result_type(original.clone())?;
                Ok((scope, imported.ty()))
            }
        }
    }
}
