//! Opaque consumer bindings retain their exact independently certified owners.
use super::{CDependencyFunction, CDialect, bindings::CBindings, diagnostic};
use crate::ast::{CFunctionRef, CRegistry};
use portable_codegen::{
    DependencyCallableSpec, DependencySpelling, LinkerDialect, SymbolCatalogue, TargetAstPackage,
};
use portable_diagnostics::{Diagnostic, SourceRef};

#[cfg(test)]
#[path = "../../tests/shared_dependency_symbols.rs"]
mod tests;

/// No name-, signature- or metadata-authored constructor is exposed.
///
/// ```compile_fail
/// use portable_backend_c::{ast::CFunctionRef, dialect::{CImportedCallable, CDependencyFunction}};
/// fn forge(function: CFunctionRef, dependency: CDependencyFunction) -> CImportedCallable {
///     CImportedCallable { function, dependency }
/// }
/// ```
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct CImportedCallable {
    function: CFunctionRef,
    dependency: CDependencyFunction,
}

impl CImportedCallable {
    pub(super) fn from_registry(
        registry: &CRegistry,
        function: &CFunctionRef,
    ) -> Result<Self, String> {
        let dependency = registry
            .imported_function(function)
            .map_err(|error| error.to_string())?
            .clone();
        Ok(Self {
            function: function.clone(),
            dependency,
        })
    }

    pub fn function(&self) -> &CFunctionRef {
        &self.function
    }
    pub fn dependency(&self) -> &CDependencyFunction {
        &self.dependency
    }

    pub(super) fn spec(&self) -> DependencyCallableSpec<CDialect> {
        let owner = self.dependency.package_identity();
        DependencyCallableSpec {
            symbol: self.clone(),
            owner: owner.clone(),
            name: self.dependency.symbol().clone(),
            signature: CBindings::default().signature(&self.function),
            spelling: DependencySpelling::FixedImport(super::CImportKind::Dependency(owner)),
            source: SourceRef::logical([
                "c",
                "certified-dependency",
                self.dependency.symbol().as_str(),
            ]),
        }
    }
}

pub(super) fn catalogue(
    package: &TargetAstPackage<CDialect>,
) -> Result<SymbolCatalogue<CDialect>, Vec<Diagnostic>> {
    let Some(unit) = package.files().next().and_then(|file| file.items().first()) else {
        return Err(diagnostic(
            "C dependency catalogue requires an original package authority",
        ));
    };
    let registry = unit.projection.registry.registrations();
    super::dependency_exports::inventory(registry).map_err(|error| diagnostic(error.message))?;
    let dependency_crates: std::collections::BTreeSet<_> = registry
        .dependency_packages()
        .map(|owner| {
            owner
                .map(|owner| owner.root().crate_id)
                .map_err(|error| diagnostic(error.to_string()))
        })
        .collect::<Result<_, _>>()?;
    for entry in registry.inventory() {
        if let crate::ast::CGeneratedOrigin::RustSource(origin) = &entry.key.origin
            && (dependency_crates.contains(&origin.declaration.crate_id)
                || dependency_crates.contains(&origin.crate_exports.root.crate_id))
        {
            return Err(diagnostic(
                "consumer crate identity overlaps an independently certified dependency",
            ));
        }
    }
    let mut catalogue = CDialect.symbol_catalogue();
    for (function, _) in registry.imported_functions() {
        catalogue.dependency_callables.push(
            CImportedCallable::from_registry(registry, function)
                .map_err(diagnostic)?
                .spec(),
        );
    }
    for (object, _) in registry.imported_constants() {
        catalogue.dependency_values.push(
            super::CImportedValue::from_registry(registry, object)
                .map_err(diagnostic)?
                .spec(),
        );
    }
    Ok(catalogue)
}
