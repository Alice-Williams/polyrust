//! Catalogue entries are derived from scope-bound original member authority.
use super::{JavaImportedResultAccessor, JavaImportedResultConstructor};
use crate::{
    ast::JavaVisibility,
    dialect::{JavaDialect, JavaInvocationKind, JavaQualifiedName, JavaReferencedMemberName},
};
use portable_codegen::{
    DependencyPolicy, KnownConstructorSpec, KnownMethodSpec, SymbolOrigin, TargetCallableSignature,
};
use portable_diagnostics::SourceRef;

impl JavaImportedResultConstructor {
    pub(in crate::dialect) fn spec(&self) -> KnownConstructorSpec<JavaDialect> {
        let signature = TargetCallableSignature {
            invocation: JavaInvocationKind::Constructor,
            receiver: None,
            parameters: self
                .parameters()
                .iter()
                .map(|ty| JavaDialect.registered_type(ty))
                .collect(),
            return_type: JavaDialect.registered_type(&self.owner().ty()),
        };
        KnownConstructorSpec {
            symbol: self.clone().into(),
            owner: self.owner().clone().into(),
            name: self.owner().path().member().clone(),
            alias_stem: self.owner().path().member().as_str().to_owned(),
            qualified_name: Some(JavaQualifiedName::Dependency(self.owner().path().clone())),
            origin: SymbolOrigin::CertifiedDependency(
                self.owner().original().family().package_identity().clone(),
            ),
            signature: super::super::catalogue::callable_pattern(&signature),
            visibility: JavaVisibility::Public,
            policy: DependencyPolicy::Qualified,
            dependency: None,
            source: SourceRef::logical([
                "java",
                "certified-result-constructor",
                self.owner().path().member().as_str(),
            ]),
        }
    }
}

impl JavaImportedResultAccessor {
    pub(in crate::dialect) fn spec(&self) -> KnownMethodSpec<JavaDialect> {
        KnownMethodSpec {
            symbol: self.clone().into(),
            owner: self.owner().clone().into(),
            name: self.name().clone(),
            origin: SymbolOrigin::CertifiedDependency(
                self.owner().original().family().package_identity().clone(),
            ),
            signature: super::super::catalogue::callable_pattern(
                &JavaDialect.coarse_signature(&self.signature()),
            ),
            visibility: JavaVisibility::Public,
            policy: DependencyPolicy::Member {
                owner: JavaQualifiedName::Dependency(self.owner().path().clone()),
                member: JavaReferencedMemberName::Dependency(self.clone()),
            },
            dependency: None,
            source: SourceRef::logical(["java", "certified-result-accessor", self.name().as_str()]),
        }
    }
}
