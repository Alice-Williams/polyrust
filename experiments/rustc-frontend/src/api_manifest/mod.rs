//! Bidirectional compiler-binding/checked-C metadata; never parsed output code.
#[cfg(public_constant_ast_probe)]
#[path = "../../test/public_constant_manifest.rs"]
pub(crate) mod constant_contract;
mod constant_export_serialization;
mod constant_exports;
mod constant_import_serialization;
mod constant_imports;
mod constants;
#[cfg(public_package_contract)]
#[path = "../../test/public_package_manifest_mutations.rs"]
pub(crate) mod contract;
#[cfg(constant_export_manifest_probe)]
#[path = "../../test/constant_export_manifest.rs"]
pub(crate) mod export_contract;
mod function_results;
mod import_serialization;
mod imports;
#[cfg(c_graph_inventory_contract)]
#[path = "../../test/c_import_manifest_contract.rs"]
pub(crate) mod inventory_contract;
mod serialization;
mod source_constant_values;
mod source_types;
mod system_libraries;
#[cfg(truncation_ast_probe)]
#[path = "../../test/truncation_manifest.rs"]
pub(crate) mod truncation_contract;
use portable_backend_c::{
    ast::*,
    dialect::{CDialect, c_defined_functions},
};
use portable_codegen::{
    RenderReadyPackage, RustCrateExports, RustDeclarationId, RustExportNamespace, RustExportTarget,
    RustSourceNode,
};
use std::{
    collections::{BTreeMap, BTreeSet},
    sync::Arc,
};

#[derive(Clone, Debug, PartialEq, Eq)]
struct Function {
    reference: CFunctionRef,
    name: CIdentifier,
    implementation: CFileRef,
    linkage: CLinkage,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct ApiManifest {
    source_types: Option<portable_codegen::RustSourceTypes>,
    exports: Arc<RustCrateExports>,
    system_libraries: BTreeSet<portable_backend_c::dialect::CSystemLibrary>,
    header: CFileRef,
    implementation: CFileRef,
    functions: BTreeMap<RustDeclarationId, Function>,
    imports: imports::ExpectedImports,
    constant_imports: constant_imports::Expected,
    used_constant_imports: constant_imports::Expected,
    foreign_constants: Vec<portable_backend_c::dialect::CForeignConstantExport>,
    constants: BTreeMap<RustDeclarationId, constants::Constant>,
}

impl ApiManifest {
    /// Recheck the retained descriptive inventory against its exact owning API.
    pub(crate) fn verify_owner(
        &self,
        api: &portable_backend_c::dialect::CDependencyApi,
    ) -> Result<(), String> {
        if Some(self.exports.root) != api.source_root() {
            return Err("bundle manifest owner disagrees".into());
        }
        if api
            .dependencies()?
            .iter()
            .any(|owner| owner.source_root().is_none())
        {
            return Err(
                "C source manifest does not yet support canonical type dependencies".into(),
            );
        }
        let expected = self
            .functions
            .iter()
            .map(|(id, function)| (*id, function.reference.clone()))
            .collect();
        let constants = self
            .constants
            .iter()
            .map(|(id, constant)| (*id, (constant.reference.clone(), constant.value)))
            .collect();
        if self
            != &Self::with_all_bindings(
                api.package(),
                self.exports.clone(),
                &expected,
                &self.imports,
                &constants,
                &self.constant_imports,
            )?
            .copy_source_types(api.package(), self.source_types.as_ref())?
        {
            return Err("bundle manifest owner inventory disagrees".into());
        }
        Ok(())
    }

    pub(crate) fn bundle_bound(&self) -> Result<usize, String> {
        self.encoded_bound()
    }

    pub(crate) fn new(
        package: &RenderReadyPackage<CDialect>,
        exports: Arc<RustCrateExports>,
        expected: &BTreeMap<RustDeclarationId, CFunctionRef>,
    ) -> Result<Self, String> {
        Self::with_imports(package, exports, expected, &BTreeMap::new())
    }

    pub(crate) fn with_imports(
        package: &RenderReadyPackage<CDialect>,
        exports: Arc<RustCrateExports>,
        expected: &BTreeMap<RustDeclarationId, CFunctionRef>,
        imported: &imports::ExpectedImports,
    ) -> Result<Self, String> {
        Self::with_constants(package, exports, expected, imported, &BTreeMap::new())
    }

    pub(crate) fn with_constants(
        package: &RenderReadyPackage<CDialect>,
        exports: Arc<RustCrateExports>,
        expected: &BTreeMap<RustDeclarationId, CFunctionRef>,
        imported: &imports::ExpectedImports,
        expected_constants: &constants::ExpectedConstants,
    ) -> Result<Self, String> {
        Self::with_all_bindings(
            package,
            exports,
            expected,
            imported,
            expected_constants,
            &BTreeMap::new(),
        )
    }

    pub(crate) fn with_all_bindings(
        package: &RenderReadyPackage<CDialect>,
        exports: Arc<RustCrateExports>,
        expected: &BTreeMap<RustDeclarationId, CFunctionRef>,
        imported: &imports::ExpectedImports,
        expected_constants: &constants::ExpectedConstants,
        constant_imports: &constant_imports::Expected,
    ) -> Result<Self, String> {
        if portable_backend_c::dialect::c_canonical_type_package(package).is_some() {
            return Err("C source manifest does not support canonical type owners".into());
        }
        if portable_backend_c::dialect::c_dependency_packages(package)?
            .iter()
            .any(|owner| owner.source_root().is_none())
        {
            return Err(
                "C source manifest does not yet support canonical type dependencies".into(),
            );
        }
        if constant_imports.keys().any(|id| {
            expected.contains_key(id)
                || imported.contains_key(id)
                || expected_constants.contains_key(id)
        }) {
            return Err("API constant import and owned/callable identities overlap".into());
        }
        if expected_constants
            .keys()
            .any(|id| expected.contains_key(id))
        {
            return Err("API constant and function identity overlap".into());
        }
        let files = package.ast().files();
        if files.len() != 2 {
            return Err("API manifest requires a certified public pair".into());
        }
        let file = |role| -> Result<CFileRef, String> {
            files
                .iter()
                .find(|file| file.module().key().role == role)
                .map(|file| file.module().clone())
                .ok_or("missing API manifest file role".into())
        };
        let header = file(CFileRole::GeneratedPublicHeader)?;
        let implementation = file(CFileRole::GeneratedSource)?;
        for (file, extension) in [(&header, "h"), (&implementation, "c")] {
            if file.key().path.as_str()
                != format!("polyrust_{:016x}.{extension}", exports.root.crate_id)
            {
                return Err("API manifest file is not owned by the expected source crate".into());
            }
        }
        let foreign_constants = constant_exports::collect(package, &exports)?;
        let foreign: BTreeMap<_, _> = foreign_constants
            .iter()
            .map(|binding| {
                (
                    (binding.module(), binding.name()),
                    binding.dependency().declaration(),
                )
            })
            .collect();
        let mut public = BTreeSet::new();
        for (module, bindings) in &exports.modules {
            for (name, target) in bindings {
                match target {
                    RustExportTarget::Module(id)
                        if name.namespace == RustExportNamespace::Type
                            && id.crate_id == exports.root.crate_id
                            && exports.modules.contains_key(id) => {}
                    RustExportTarget::Declaration(id)
                        if name.namespace == RustExportNamespace::Value
                            && id.crate_id == exports.root.crate_id
                            && (expected.contains_key(id)
                                || expected_constants.contains_key(id)) =>
                    {
                        public.insert(*id);
                    }
                    RustExportTarget::Declaration(id)
                        if foreign.get(&(*module, name)) == Some(id) => {}
                    _ => return Err("API manifest contains an unmapped compiler export".into()),
                }
            }
        }
        if public.is_empty() && foreign_constants.is_empty() {
            return Err("API manifest has no public declarations".into());
        }
        let mut functions = BTreeMap::new();
        for definition in c_defined_functions(package) {
            let reference = definition.function();
            let CGeneratedOrigin::RustSource(origin) = &reference.key().origin else {
                return Err("API manifest definition lacks compiler provenance".into());
            };
            let id = origin.declaration;
            let exported = public.contains(&id);
            let primary = if exported { &header } else { &implementation };
            let linkage = if exported {
                CLinkage::External
            } else {
                CLinkage::Internal
            };
            if origin.node != RustSourceNode::Declaration
                || id.crate_id != exports.root.crate_id
                || origin.crate_exports != exports
                || origin.externally_reachable != exported
                || reference.file() != primary
                || definition.implementation() != &implementation
                || definition.linkage() != linkage
                || expected.get(&id) != Some(reference)
            {
                return Err("API manifest compiler/function/file/linkage mapping disagrees".into());
            }
            let value = Function {
                reference: reference.clone(),
                name: definition.name().clone(),
                implementation: definition.implementation().clone(),
                linkage,
            };
            if functions.insert(id, value).is_some() {
                return Err("duplicate API definition identity".into());
            }
        }
        if functions.len() != expected.len()
            || !public
                .iter()
                .all(|id| functions.contains_key(id) || expected_constants.contains_key(id))
        {
            return Err("API manifest misses a source or target function".into());
        }
        let constants = constants::collect(
            package,
            &exports,
            &header,
            &implementation,
            expected_constants,
            &public,
        )?;
        let constant_imports = constant_imports::collect(package, exports.root, constant_imports)?;
        let used_constant_imports = portable_backend_c::dialect::c_used_imported_constants(package)
            .map(|imported| {
                (
                    imported.dependency().declaration(),
                    (imported.object().clone(), imported.dependency().clone()),
                )
            })
            .collect();
        let manifest = Self {
            source_types: None,
            system_libraries: portable_backend_c::dialect::c_system_libraries(package)?,
            foreign_constants,
            used_constant_imports,
            constants,
            imports: imports::collect(package, exports.root, imported)?,
            constant_imports,
            exports,
            header,
            implementation,
            functions,
        };
        manifest.encoded_bound()?;
        Ok(manifest)
    }

    pub(crate) fn verify_all_bindings(
        &self,
        package: &RenderReadyPackage<CDialect>,
        exports: Arc<RustCrateExports>,
        expected: &BTreeMap<RustDeclarationId, CFunctionRef>,
        imported: &imports::ExpectedImports,
        constants: &constants::ExpectedConstants,
        constant_imports: &constant_imports::Expected,
    ) -> Result<(), String> {
        if self
            != &Self::with_all_bindings(
                package,
                exports,
                expected,
                imported,
                constants,
                constant_imports,
            )?
            .copy_source_types(package, self.source_types.as_ref())?
        {
            return Err(
                "API manifest differs from exact compiler/target import reconstruction".into(),
            );
        }
        Ok(())
    }

    pub(crate) fn verify_constants(
        &self,
        package: &RenderReadyPackage<CDialect>,
        exports: Arc<RustCrateExports>,
        expected: &BTreeMap<RustDeclarationId, CFunctionRef>,
        imported: &imports::ExpectedImports,
        constants: &constants::ExpectedConstants,
    ) -> Result<(), String> {
        self.verify_all_bindings(
            package,
            exports,
            expected,
            imported,
            constants,
            &BTreeMap::new(),
        )
    }

    pub(crate) fn verify(
        &self,
        package: &RenderReadyPackage<CDialect>,
        exports: Arc<RustCrateExports>,
        expected: &BTreeMap<RustDeclarationId, CFunctionRef>,
    ) -> Result<(), String> {
        self.verify_imports(package, exports, expected, &BTreeMap::new())
    }

    pub(crate) fn verify_imports(
        &self,
        package: &RenderReadyPackage<CDialect>,
        exports: Arc<RustCrateExports>,
        expected: &BTreeMap<RustDeclarationId, CFunctionRef>,
        imported: &imports::ExpectedImports,
    ) -> Result<(), String> {
        self.verify_constants(package, exports, expected, imported, &BTreeMap::new())
    }
}
