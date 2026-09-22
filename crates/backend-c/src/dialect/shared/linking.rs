//! Shared linker hooks; all emitted references retain resolver-owned names.

use super::{
    CDialect, CInvocation, CNamespace, CProjectedUnit, CResolvedUnit, CSharedTypeKind, CStdType,
    CUnavailable, CVisibility, violation,
};
use crate::ast::{CFileRef, CIdentifier};
use crate::dialect::CHeader;
use portable_codegen::{
    AstViolation, DependencyPolicy, FileItemRoots, GeneratedSymbolId, KnownTypeSpec, LinkedFile,
    LinkerDialect, PackageEcosystem, ResolvedReference, ResolvedReferenceMap, SymbolCatalogue,
    SymbolOrigin, TargetAstContext, TargetAstPackage, TargetExprId, TargetSymbolRef,
};
use portable_diagnostics::SourceRef;
use std::collections::{BTreeMap, BTreeSet};

impl CProjectedUnit {
    fn declarations(&self) -> Vec<GeneratedSymbolId> {
        self.data.declarations.clone()
    }
    pub(super) fn symbols(&self) -> Vec<TargetSymbolRef<CDialect>> {
        self.data
            .bindings
            .symbols()
            .into_iter()
            .map(TargetSymbolRef::Generated)
            .chain(
                self.data
                    .bindings
                    .imports
                    .values()
                    .cloned()
                    .map(TargetSymbolRef::DependencyCallable),
            )
            .chain(
                self.data
                    .bindings
                    .imported_values
                    .values()
                    .cloned()
                    .map(TargetSymbolRef::DependencyValue),
            )
            .chain(
                self.data
                    .standards
                    .iter()
                    .copied()
                    .map(TargetSymbolRef::KnownType),
            )
            .collect()
    }
}

impl LinkerDialect for CDialect {
    type DependencyValue = super::CImportedValue;
    type DependencyCallable = super::CImportedCallable;
    type DependencyPackage = super::CDependencyPackage;
    type KnownField = CUnavailable;
    type KnownConstructor = CUnavailable;
    type KnownMethod = CUnavailable;
    type PreludeSymbol = CUnavailable;
    type StandardLibrary = CHeader;
    type ExternalPackage = CUnavailable;
    type PackageFeature = CUnavailable;
    type HelperId = CUnavailable;
    type HelperCapability = CUnavailable;
    type Identifier = CIdentifier;
    type QualifiedName = CUnavailable;
    type MemberName = CUnavailable;
    type Namespace = CNamespace;
    type NameKey = String;
    type ImportKind = super::CImportKind;
    type ResolvedModule = CFileRef;
    type ResolvedFileItem = CResolvedUnit;

    fn package_ecosystem(&self, value: &CUnavailable) -> PackageEcosystem {
        match *value {}
    }
    fn package_name(&self, value: &CUnavailable) -> &'static str {
        match *value {}
    }
    fn package_feature_name(&self, value: &CUnavailable) -> &'static str {
        match *value {}
    }
    fn helper_name(&self, value: &CUnavailable) -> &'static str {
        match *value {}
    }
    fn helper_capability_name(&self, value: &CUnavailable) -> &'static str {
        match *value {}
    }

    fn symbol_catalogue(&self) -> SymbolCatalogue<Self> {
        SymbolCatalogue {
            dependency_callables: vec![],
            dependency_values: vec![],
            types: [
                CStdType::I32,
                CStdType::I64,
                CStdType::U32,
                CStdType::U64,
                CStdType::Size,
            ]
            .into_iter()
            .map(|symbol| KnownTypeSpec {
                symbol,
                name: CIdentifier::new(symbol.spelling()).expect("catalogue identifier"),
                alias_stem: symbol.spelling().into(),
                qualified_name: None,
                origin: SymbolOrigin::StandardLibrary(symbol.header()),
                arity: 0,
                policy: DependencyPolicy::Import(super::CImportKind::Standard(symbol.header())),
                dependency: None,
                source: SourceRef::logical(["c", "standard", symbol.spelling()]),
            })
            .collect(),
            callables: vec![],
            runtime_callables: vec![],
            fields: vec![],
            constructors: vec![],
            methods: vec![],
            helpers: vec![],
        }
    }

    fn package_symbol_catalogue(
        &self,
        package: &TargetAstPackage<Self>,
    ) -> Result<SymbolCatalogue<Self>, Vec<portable_diagnostics::Diagnostic>> {
        super::dependency_symbols::catalogue(package)
    }

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

    fn identifier_from_candidate(
        &self,
        candidate: &str,
        _: &CNamespace,
    ) -> Result<CIdentifier, AstViolation> {
        // A reserved generated prefix prevents preprocessor/library collisions
        // in every C namespace. The public poly_score ABI remains unchanged.
        let name = if candidate.starts_with("poly_") {
            candidate.to_owned()
        } else {
            format!("poly_{candidate}")
        };
        CIdentifier::new(&name).map_err(|error| violation(error.to_string()))
    }
    fn identifier_key(&self, value: &CIdentifier) -> String {
        value.as_str().into()
    }
    fn is_public(&self, visibility: &CVisibility) -> bool {
        *visibility == CVisibility::Exported
    }
    fn type_namespace(&self, _: &CSharedTypeKind) -> CNamespace {
        CNamespace::Tag
    }
    fn type_namespace_from_known(&self, _: &CStdType) -> CNamespace {
        CNamespace::Ordinary
    }
    fn callable_namespace(&self) -> CNamespace {
        CNamespace::Ordinary
    }
    fn member_namespace(&self) -> CNamespace {
        CNamespace::Ordinary
    }
    fn value_namespace(&self) -> CNamespace {
        CNamespace::Ordinary
    }
    fn known_call_expression(
        &self,
        value: CUnavailable,
        _: CInvocation,
        _: Vec<TargetExprId>,
    ) -> CUnavailable {
        match value {}
    }
    fn known_constructor_expression(
        &self,
        value: CUnavailable,
        _: Vec<TargetExprId>,
    ) -> CUnavailable {
        match value {}
    }
    fn known_method_expression(
        &self,
        value: CUnavailable,
        _: TargetExprId,
        _: Vec<TargetExprId>,
    ) -> CUnavailable {
        match value {}
    }
    fn expression_references(&self, value: &CUnavailable) -> Vec<TargetSymbolRef<Self>> {
        match *value {}
    }
    fn statement_references(&self, value: &CUnavailable) -> Vec<TargetSymbolRef<Self>> {
        match *value {}
    }

    fn file_item_roots(&self, unit: &CProjectedUnit) -> FileItemRoots<Self> {
        FileItemRoots {
            declarations: unit.declarations(),
            expressions: vec![],
            statements: vec![],
            symbols: unit.symbols(),
        }
    }
    fn file_standard_libraries(&self, file: &portable_codegen::TargetFile<Self>) -> Vec<CHeader> {
        file.items()
            .iter()
            .flat_map(|unit| unit.data.standard_libraries.iter().copied())
            .collect()
    }
    fn resolve_standard_library_import(
        &self,
        library: &CHeader,
    ) -> Result<Self::ImportKind, AstViolation> {
        match library {
            CHeader::Float | CHeader::Math => Ok(super::CImportKind::Standard(*library)),
            _ => Err(violation("unnamed C library is outside the closed profile")),
        }
    }
    fn resolve_module(&self, file: &CFileRef) -> Result<CFileRef, AstViolation> {
        Ok(file.clone())
    }
    fn file_requirements(
        &self,
        file: &portable_codegen::TargetFile<Self>,
    ) -> Vec<portable_codegen::TargetFileRequirement<Self>> {
        let [unit] = file.items() else {
            return vec![];
        };
        let registry = unit.projection.registry.registrations();
        match registry.source_package() {
            Some(package) if file.module().key().role == crate::ast::CFileRole::GeneratedSource => {
                vec![portable_codegen::TargetFileRequirement::new(
                    package.header().clone(),
                    package.header().key().path.clone(),
                )]
            }
            _ => vec![],
        }
    }

    fn resolve_file_import(
        &self,
        source: &portable_codegen::TargetFile<Self>,
        destination: &portable_codegen::TargetFile<Self>,
    ) -> Result<Option<Self::ImportKind>, AstViolation> {
        let [unit] = source.items() else {
            return Err(violation(
                "C file import requires one authenticated source unit",
            ));
        };
        for file in [source, destination] {
            if file.path() != &file.module().key().path
                || file.placement() != &file.module().key().role
            {
                return Err(violation(
                    "C file import metadata differs from registered identity",
                ));
            }
        }
        super::CGeneratedHeader::resolve(
            unit.projection.registry.registrations(),
            source.module(),
            destination.module(),
        )
        .map(|header| Some(super::CImportKind::Generated(header)))
    }
    fn resolve_file_item(
        &self,
        _: &TargetAstPackage<Self>,
        unit: &CProjectedUnit,
        references: &ResolvedReferenceMap<Self>,
    ) -> Result<CResolvedUnit, AstViolation> {
        let names = unit
            .symbols()
            .into_iter()
            .map(|symbol| {
                let name = references
                    .get(&symbol)
                    .ok_or_else(|| violation("C symbol has no linker-owned binding"))?;
                Ok((symbol, name.clone()))
            })
            .collect::<Result<BTreeMap<_, _>, AstViolation>>()?;
        let spelling = super::resolved_names::CResolvedNames::new(&unit.data.bindings, &names)?;
        Ok(CResolvedUnit {
            unit: unit.clone(),
            names,
            spelling,
        })
    }
    fn verify_resolved_file_item(&self, item: &CResolvedUnit) -> Vec<AstViolation> {
        match super::resolved_names::CResolvedNames::new(&item.unit.data.bindings, &item.names) {
            Ok(expected) if expected == item.spelling => {}
            _ => {
                return vec![violation(
                    "C spelling map differs from linker-owned bindings",
                )];
            }
        }
        let expected: BTreeSet<_> = item.unit.symbols().into_iter().collect();
        if let Err(error) = super::dependency_exports::verify_unit(item) {
            return vec![error];
        }
        if expected != item.names.keys().cloned().collect() {
            return vec![violation("C resolved unit lacks its exact typed name map")];
        }
        for (symbol, name) in &item.names {
            if let TargetSymbolRef::KnownType(standard) = symbol
                && !matches!(name, ResolvedReference::Imported { binding, .. } if binding.as_str() == standard.spelling())
            {
                return vec![violation("C standard typedef cannot be import-aliased")];
            }
        }
        vec![]
    }
    fn verify_resolved_file(
        &self,
        file: &LinkedFile<Self>,
        _: &TargetAstContext<'_, Self>,
    ) -> Vec<AstViolation> {
        let [item] = file.items() else {
            return vec![violation("C resolved file needs one compilation unit")];
        };
        let identity = item.unit.data.source.identity();
        if file.module() != identity
            || file.path() != &identity.key().path
            || file.placement() != &identity.key().role
        {
            vec![violation(
                "C resolved file identity differs from the registered source",
            )]
        } else {
            vec![]
        }
    }
}
