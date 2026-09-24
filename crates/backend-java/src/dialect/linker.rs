//! Java dialect: linker.

use super::arena_nodes::JavaArenaExpression;
use super::catalogue::java_symbol_catalogue;
use super::file_checks::{
    declares_reserved_runtime_type, verify_composed_java_file, verify_java_file_identity,
};
use super::known_fields::JavaKnownField;
use super::runtime_helpers::{JavaHelperCapability, JavaRuntimeHelper};
use super::{
    JavaDialect, JavaExternalPackage, JavaGeneratedContainer, JavaImportKind, JavaNameKey,
    JavaNamespace, JavaPackageFeature, JavaPreludeSymbol, JavaQualifiedName, JavaStandardLibrary,
};
use crate::ast::{
    JavaIdentifier, JavaPackage, JavaResolvedName, JavaVisibility, ResolvedJavaFileItem,
};
use portable_codegen::{
    AstViolation, FileItemRoots, GeneratedOrigin, GeneratedSymbolId, LinkedFile, LinkerDialect,
    PackageEcosystem, ResolvedReference, ResolvedReferenceMap, SymbolCatalogue, SynthesisReason,
    TargetAstContext, TargetAstPackage, TargetExprId, TargetSymbolRef,
};
use portable_diagnostics::DiagnosticCode;
use std::collections::{BTreeMap, BTreeSet};

impl LinkerDialect for JavaDialect {
    type DependencyValue = super::JavaImportedValue;
    type DependencyCallable = super::JavaImportedCallable;
    type DependencyPackage = super::JavaDependencyPackage;
    type KnownField = JavaKnownField;
    type KnownConstructor = super::JavaReferencedConstructor;
    type KnownMethod = super::JavaReferencedMethod;
    type PreludeSymbol = JavaPreludeSymbol;
    type StandardLibrary = JavaStandardLibrary;
    type ExternalPackage = JavaExternalPackage;
    type PackageFeature = JavaPackageFeature;
    type HelperId = JavaRuntimeHelper;
    type HelperCapability = JavaHelperCapability;
    type Identifier = JavaIdentifier;
    type QualifiedName = JavaQualifiedName;
    type MemberName = super::JavaReferencedMemberName;
    type Namespace = JavaNamespace;
    type NameKey = JavaNameKey;
    type ImportKind = JavaImportKind;
    type ResolvedModule = JavaPackage;
    type ResolvedFileItem = ResolvedJavaFileItem;

    fn dependency_value_spec(
        &self,
        value: &Self::DependencyValue,
    ) -> portable_codegen::DependencyValueSpec<Self> {
        value.spec()
    }

    fn verify_dependency_value_type(
        &self,
        value: &Self::DependencyValue,
        ty: &portable_codegen::TargetTypeRef<Self>,
    ) -> Result<(), AstViolation> {
        value.verify_type(ty)
    }

    fn dependency_callable_spec(
        &self,
        callable: &Self::DependencyCallable,
    ) -> portable_codegen::DependencyCallableSpec<Self> {
        callable.spec()
    }

    fn package_symbol_catalogue(
        &self,
        package: &TargetAstPackage<Self>,
    ) -> Result<SymbolCatalogue<Self>, Vec<portable_diagnostics::Diagnostic>> {
        super::dependency_scope::catalogue(package)
    }

    fn package_ecosystem(&self, package: &Self::ExternalPackage) -> PackageEcosystem {
        match package {
            JavaExternalPackage::None => PackageEcosystem::Maven,
        }
    }

    fn package_name(&self, package: &Self::ExternalPackage) -> &'static str {
        match package {
            JavaExternalPackage::None => "none",
        }
    }

    fn package_feature_name(&self, feature: &Self::PackageFeature) -> &'static str {
        match feature {
            JavaPackageFeature::None => "none",
        }
    }

    fn helper_name(&self, helper: &Self::HelperId) -> &'static str {
        helper.name()
    }

    fn helper_capability_name(&self, capability: &Self::HelperCapability) -> &'static str {
        capability.name()
    }

    fn symbol_catalogue(&self) -> SymbolCatalogue<Self> {
        java_symbol_catalogue()
    }

    fn identifier_from_candidate(
        &self,
        candidate: &str,
        _namespace: &Self::Namespace,
    ) -> Result<Self::Identifier, AstViolation> {
        Ok(JavaIdentifier::from_portable(candidate))
    }

    fn identifier_key(&self, identifier: &Self::Identifier) -> Self::NameKey {
        JavaNameKey(identifier.as_str().to_owned())
    }

    fn is_public(&self, visibility: &Self::Visibility) -> bool {
        *visibility == JavaVisibility::Public
    }

    fn type_namespace(&self, _kind: &Self::DeclarationKind) -> Self::Namespace {
        JavaNamespace::Type
    }
    fn type_namespace_from_known(&self, _known: &Self::KnownType) -> Self::Namespace {
        JavaNamespace::Type
    }
    fn callable_namespace(&self) -> Self::Namespace {
        JavaNamespace::Value
    }
    fn member_namespace(&self) -> Self::Namespace {
        JavaNamespace::Value
    }
    fn value_namespace(&self) -> Self::Namespace {
        JavaNamespace::Value
    }

    fn known_call_expression(
        &self,
        callable: Self::KnownCallable,
        _invocation: Self::InvocationKind,
        arguments: Vec<TargetExprId>,
    ) -> Self::Expression {
        JavaArenaExpression::KnownCall {
            callable,
            arguments,
        }
    }

    fn known_constructor_expression(
        &self,
        constructor: Self::KnownConstructor,
        arguments: Vec<TargetExprId>,
    ) -> Self::Expression {
        JavaArenaExpression::KnownConstructor {
            constructor,
            arguments,
        }
    }

    fn known_method_expression(
        &self,
        method: Self::KnownMethod,
        receiver: TargetExprId,
        arguments: Vec<TargetExprId>,
    ) -> Self::Expression {
        JavaArenaExpression::KnownMethod {
            method,
            receiver,
            arguments,
        }
    }

    fn expression_references(&self, value: &Self::Expression) -> Vec<TargetSymbolRef<Self>> {
        match value {
            JavaArenaExpression::KnownCall { callable, .. } => {
                vec![TargetSymbolRef::KnownCallable(*callable)]
            }
            JavaArenaExpression::KnownConstructor { constructor, .. } => {
                vec![TargetSymbolRef::KnownConstructor(constructor.clone())]
            }
            JavaArenaExpression::KnownMethod { method, .. } => {
                vec![TargetSymbolRef::KnownMethod(method.clone())]
            }
        }
    }

    fn statement_references(&self, _statement: &Self::Statement) -> Vec<TargetSymbolRef<Self>> {
        vec![]
    }

    fn file_item_roots(&self, item: &Self::FileItem) -> FileItemRoots<Self> {
        FileItemRoots {
            declarations: item.declared_symbols(),
            expressions: vec![],
            statements: vec![],
            symbols: item.symbols(),
        }
    }

    fn resolve_module(
        &self,
        module: &Self::ModuleDeclaration,
    ) -> Result<Self::ResolvedModule, AstViolation> {
        Ok(*module)
    }

    fn resolve_file_item(
        &self,
        package: &TargetAstPackage<Self>,
        item: &Self::FileItem,
        references: &ResolvedReferenceMap<Self>,
    ) -> Result<Self::ResolvedFileItem, AstViolation> {
        let locally_declared = item.declared_symbols().into_iter().collect::<BTreeSet<_>>();
        let mut names = BTreeMap::new();
        for symbol in item.symbols() {
            if let TargetSymbolRef::Generated(id) = symbol
                && let Some(path) = super::declaration_paths::noncanonical_path(package, id)?
            {
                names.insert(symbol, JavaResolvedName::DeclaredPath(path));
                continue;
            }
            let Some(resolved) = references.get(&symbol) else {
                return Err(AstViolation::new(
                    DiagnosticCode::UnresolvedReference,
                    "Java item symbol has no resolver-owned spelling",
                ));
            };
            let is_public_container_member = match symbol {
                TargetSymbolRef::Generated(GeneratedSymbolId::Type(id)) => {
                    !matches!(
                        package.generated_type(id).map(|value| &value.origin),
                        Some(GeneratedOrigin::Synthesized(
                            SynthesisReason::PackageEntryPoint
                        ))
                    ) && !locally_declared.contains(&GeneratedSymbolId::Type(id))
                }
                TargetSymbolRef::Generated(GeneratedSymbolId::Callable(id)) => {
                    package.callable(id).is_some_and(|value| {
                        matches!(
                            value.origin,
                            GeneratedOrigin::CoreDeclaration(
                                portable_core_ir::CoreDeclaration::Function(_)
                            )
                        )
                    }) || !locally_declared.contains(&GeneratedSymbolId::Callable(id))
                }
                TargetSymbolRef::Generated(GeneratedSymbolId::Value(id)) => {
                    package.value(id).is_some_and(|value| {
                        matches!(
                            value.origin,
                            GeneratedOrigin::CoreDeclaration(
                                portable_core_ir::CoreDeclaration::Constant(_)
                            )
                        )
                    }) || !locally_declared.contains(&GeneratedSymbolId::Value(id))
                }
                _ => false,
            };
            let name = match resolved {
                ResolvedReference::Local(value) if is_public_container_member => {
                    JavaResolvedName::GeneratedMember {
                        owner: JavaGeneratedContainer::PublicApi,
                        member: value.clone(),
                    }
                }
                ResolvedReference::Local(value)
                | ResolvedReference::Imported { binding: value, .. } => {
                    JavaResolvedName::Local(value.clone())
                }
                ResolvedReference::Qualified(value) => JavaResolvedName::Qualified(value.clone()),
                ResolvedReference::Member { owner, member } => JavaResolvedName::Member {
                    owner: owner.clone(),
                    member: member.clone(),
                },
            };
            names.insert(symbol, name);
        }
        Ok(ResolvedJavaFileItem {
            item: item.clone(),
            names,
            documentation: super::documentation::lower(package, item)?,
            source_inventory: crate::ast::JavaSourceInventory::derive(package, item)?,
        })
    }

    fn verify_resolved_file_item(&self, item: &Self::ResolvedFileItem) -> Vec<AstViolation> {
        let expected = item.item.symbols().into_iter().collect::<BTreeSet<_>>();
        let actual = item.names.keys().cloned().collect::<BTreeSet<_>>();
        let dependencies_match = item.names.iter().all(|(symbol, name)| match symbol {
            TargetSymbolRef::KnownConstructor(super::JavaReferencedConstructor::Dependency(
                value,
            )) => {
                matches!(name, JavaResolvedName::Qualified(JavaQualifiedName::Dependency(path))
                    if path == value.owner().path())
            }
            TargetSymbolRef::KnownMethod(super::JavaReferencedMethod::Dependency(value)) => {
                matches!(name, JavaResolvedName::Member {
                    owner: JavaQualifiedName::Dependency(path),
                    member: super::JavaReferencedMemberName::Dependency(member)
                } if path == value.owner().path() && member == value)
            }
            TargetSymbolRef::KnownType(super::JavaReferencedType::Dependency(ty)) => {
                matches!(name, JavaResolvedName::Qualified(JavaQualifiedName::Dependency(path))
                    if path == ty.path())
            }
            TargetSymbolRef::DependencyValue(value) => {
                matches!(name, JavaResolvedName::Qualified(JavaQualifiedName::Dependency(path))
                    if path == value.constant().path())
            }
            TargetSymbolRef::DependencyCallable(callable) => {
                matches!(name, JavaResolvedName::Qualified(JavaQualifiedName::Dependency(path))
                    if path == callable.function().path())
            }
            _ => true,
        });
        if expected == actual && dependencies_match {
            vec![]
        } else {
            vec![AstViolation::new(
                DiagnosticCode::UnresolvedReference,
                "resolved Java item does not contain the exact linker-derived spelling map",
            )]
        }
    }

    fn verify_resolved_file(
        &self,
        file: &LinkedFile<Self>,
        context: &TargetAstContext<'_, Self>,
    ) -> Vec<AstViolation> {
        let mut violations = verify_java_file_identity(
            file.role(),
            file.path().as_str(),
            file.module(),
            file.placement(),
            file.items()
                .iter()
                .any(|item| declares_reserved_runtime_type(&item.item)),
        );
        violations.extend(verify_composed_java_file(
            file.module(),
            file.placement(),
            file.items().iter().map(|item| &item.item).collect(),
            context,
        ));
        violations
    }
}
