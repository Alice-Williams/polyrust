//! A typed compilation-unit payload, never source text or a second C AST.

use super::{
    CDialect, CFileGrammar, CInvocation, CProjectedUnit, CSharedTypeKind, CStdType, CUnavailable,
    CVisibility, projection, violation,
};
use crate::ast::{CFileRef, CFileRole, CObjectType, CScalarType, CSynthesisReason};
use portable_codegen::{
    AstViolation, ResolvedReference, TargetAstContext, TargetAstPackage, TargetCallableSignature,
    TargetExprId, TargetExpressionNode, TargetFile, TargetFileItemNode, TargetStatementNode,
    TargetSymbolRef, TargetTypeRef, TypedAstDialect,
};
use std::collections::BTreeMap;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CResolvedUnit {
    pub(super) unit: CProjectedUnit,
    pub(super) names: BTreeMap<TargetSymbolRef<CDialect>, ResolvedReference<CDialect>>,
    pub(super) spelling: super::resolved_names::CResolvedNames,
}

impl TargetExpressionNode<CDialect> for CUnavailable {
    fn child_expressions(&self) -> Vec<TargetExprId> {
        match *self {}
    }
    fn verify(
        &self,
        _: &TargetTypeRef<CDialect>,
        _: &TargetAstContext<'_, CDialect>,
    ) -> Vec<AstViolation> {
        match *self {}
    }
}
impl TargetStatementNode<CDialect> for CUnavailable {
    fn child_expressions(&self) -> Vec<TargetExprId> {
        match *self {}
    }
    fn verify(&self, _: &TargetAstContext<'_, CDialect>) -> Vec<AstViolation> {
        match *self {}
    }
}
impl TargetFileItemNode<CDialect> for CProjectedUnit {
    fn verify(&self, _: &TargetAstContext<'_, CDialect>) -> Vec<AstViolation> {
        // The package hook authenticates the exact entire projection, including
        // extra/missing registrations; individual nodes cannot weaken that check.
        vec![]
    }
}

impl TypedAstDialect for CDialect {
    type PrimitiveType = CScalarType;
    type KnownType = CStdType;
    type RuntimeType = CUnavailable;
    type ConstructedType = CObjectType;
    type KnownCallable = CUnavailable;
    type RuntimeCallable = CUnavailable;
    type InvocationKind = CInvocation;
    type Visibility = CVisibility;
    type DeclarationKind = CSharedTypeKind;
    type SymbolOrigin = CSynthesisReason;
    type SourceFileKind = CFileGrammar;
    type ModuleDeclaration = CFileRef;
    type FilePlacement = CFileRole;
    type Expression = CUnavailable;
    type Statement = CUnavailable;
    type FileItem = CProjectedUnit;

    fn known_callable_signature(&self, value: &CUnavailable) -> TargetCallableSignature<Self> {
        match *value {}
    }
    fn runtime_callable_signature(&self, value: &CUnavailable) -> TargetCallableSignature<Self> {
        match *value {}
    }
    fn verify_signature(&self, signature: &TargetCallableSignature<Self>) -> Vec<AstViolation> {
        if signature.receiver.is_some() {
            vec![violation("C functions cannot have a receiver")]
        } else {
            vec![]
        }
    }
    fn verify_source_file(
        &self,
        file: &TargetFile<Self>,
        _: &TargetAstContext<'_, Self>,
    ) -> Vec<AstViolation> {
        let [unit] = file.items() else {
            return vec![violation(
                "C shared file requires one typed compilation-unit payload",
            )];
        };
        let identity = unit.data.source.identity();
        if file.module() != identity
            || file.path() != &identity.key().path
            || file.placement() != &identity.key().role
        {
            vec![violation(
                "C shared file identity differs from its registered compilation unit",
            )]
        } else {
            vec![]
        }
    }
    fn verify_package(&self, package: &TargetAstPackage<Self>) -> Vec<AstViolation> {
        projection::verify(package)
    }
}
