//! Reconstruct dependency API membership from the immutable certificate.
mod constants;
pub(super) mod structs;
use super::super::{CDialect, CGeneratedHeader, c_defined_constants, c_defined_functions};
use crate::ast::*;
use portable_codegen::{
    RenderReadyPackage, RustDeclarationId, RustExportNamespace, RustExportTarget, RustSourceNode,
    RustVisibility,
};
use std::collections::{BTreeMap, BTreeSet};

pub(super) struct Inventory {
    pub root: RustDeclarationId,
    pub header: CGeneratedHeader,
    pub implementation: CFileRef,
    pub functions: BTreeMap<RustDeclarationId, (CFunctionRef, CIdentifier)>,
    pub constants: BTreeMap<RustDeclarationId, Constant>,
    pub foreign_constants: Vec<super::CForeignConstantExport>,
    pub structs: BTreeMap<CStructRef, super::structs::StructExport>,
    pub foreign_structs: Vec<super::CDependencyStruct>,
}

pub(super) struct Constant {
    pub object: CObjectRef,
    pub symbol: CIdentifier,
    pub value: CScalarConstantValue,
    pub read_type: CObjectType,
}

fn scalar(ty: &CObjectType) -> bool {
    matches!(
        ty.kind(),
        CObjectTypeKind::Scalar(
            CScalarType::I32
                | CScalarType::I64
                | CScalarType::U32
                | CScalarType::Bool
                | CScalarType::F64
        )
    )
}

fn signature(registry: &CRegistry, function: &CFunctionRef) -> bool {
    let admitted = |ty: &CObjectType| {
        scalar(ty) || crate::ownership::value_transport::scalar_result(Some(registry), ty)
    };
    let result = match function.signature().return_type() {
        CReturnType::Void => true,
        CReturnType::Value(value) => admitted(value.declared_type()),
    };
    result
        && function
            .signature()
            .parameters()
            .iter()
            .all(|value| admitted(value.declared_type()))
}

pub(super) fn collect(package: &RenderReadyPackage<CDialect>) -> Result<Inventory, String> {
    let files = package.ast().files();
    if files.len() != 2 {
        return Err("C dependency API requires a certified public header/source pair".into());
    }
    let file = |role| {
        files
            .iter()
            .find(|file| file.module().key().role == role)
            .ok_or("C dependency API lacks an owning public package file")
    };
    let header = file(CFileRole::GeneratedPublicHeader)?;
    // Collect every public tag, including those unused by exported signatures.
    let structs = structs::collect(package)?;
    let implementation = file(CFileRole::GeneratedSource)?;
    let projection = &implementation.items()[0].unit.projection;
    let header = CGeneratedHeader::resolve(
        projection.registry.registrations(),
        implementation.module(),
        header.module(),
    )
    .map_err(|error| error.message)?;
    let inferred = || {
        let first = c_defined_functions(package)
            .map(|definition| &definition.function().key().origin)
            .chain(c_defined_constants(package).map(|definition| &definition.object().key().origin))
            .next()
            .ok_or("C dependency API has no definitions")?;
        let CGeneratedOrigin::RustSource(first_origin) = first else {
            return Err("C dependency API requires Rust-source definition provenance".into());
        };
        Ok::<_, String>(&first_origin.crate_exports)
    };
    let exports = match projection.registry.registrations().source_package() {
        Some(source) => source.exports(),
        None => inferred()?,
    };
    let root = exports.root;
    for registration in projection.registry.registrations().inventory() {
        match &registration.key.origin {
            CGeneratedOrigin::RustSource(origin)
                if origin.declaration.crate_id == root.crate_id
                    && origin.crate_exports == *exports => {}
            CGeneratedOrigin::Synthesized(_) => {}
            _ => {
                return Err(
                    "C dependency registrations disagree on their owning source crate".into(),
                );
            }
        }
    }
    let selected = super::super::constant_exports::collect(projection.registry.registrations())?;
    let mut public = BTreeSet::new();
    for (module, bindings) in &exports.modules {
        for (name, target) in bindings {
            match target {
                RustExportTarget::Module(id)
                    if name.namespace == RustExportNamespace::Type
                        && id.crate_id == root.crate_id
                        && exports.modules.contains_key(id) => {}
                RustExportTarget::Declaration(id)
                    if name.namespace == RustExportNamespace::Value
                        && id.crate_id == root.crate_id =>
                {
                    public.insert(*id);
                }
                RustExportTarget::Declaration(id)
                    if name.namespace == RustExportNamespace::Value
                        && selected
                            .foreign
                            .get(&(*module, name.clone()))
                            .is_some_and(|export| export.dependency.declaration() == *id) => {}
                _ => {
                    return Err(
                        "C dependency export has no supported local function/constant mapping"
                            .into(),
                    );
                }
            }
        }
    }
    if projection
        .registry
        .registrations()
        .source_package()
        .is_some()
        && selected.owned != public
    {
        return Err("C selected owned export inventory differs from source graph".into());
    }
    if public.is_empty() && selected.foreign.is_empty() {
        return Err("C dependency API has no public value bindings".into());
    }
    let sources = projection
        .sources
        .iter()
        .map(|source| source.as_ref().clone())
        .collect::<Vec<_>>();
    let closed =
        crate::ownership::closed_scalar_functions(projection.registry.registrations(), &sources)
            .map_err(|error| error.to_string())?;
    let mut functions = BTreeMap::new();
    let mut definitions = BTreeSet::new();
    let mut symbols = BTreeSet::new();
    for definition in c_defined_functions(package) {
        let function = definition.function();
        let CGeneratedOrigin::RustSource(origin) = &function.key().origin else {
            return Err("C dependency definition has no Rust-source provenance".into());
        };
        let id = origin.declaration;
        let exported = public.contains(&id);
        let primary = if exported {
            header.file()
        } else {
            implementation.module()
        };
        let linkage = if exported {
            CLinkage::External
        } else {
            CLinkage::Internal
        };
        if origin.node != RustSourceNode::Declaration
            || id.crate_id != root.crate_id
            || origin.crate_exports != *exports
            || origin.externally_reachable != exported
            || (exported && origin.visibility != RustVisibility::Public)
            || function.file() != primary
            || definition.implementation() != implementation.module()
            || definition.linkage() != linkage
            || !definitions.insert(id)
        {
            return Err("C dependency function/provenance/file/linkage inventory disagrees".into());
        }
        if exported {
            if !signature(projection.registry.registrations(), function)
                || !closed.contains(function)
            {
                return Err(
                    "C dependency function lacks the closed admitted-scalar-parameter/scalar-or-void-result proof".into(),
                );
            }
            if !symbols.insert(definition.name().clone()) {
                return Err("C dependency public symbols are not distinct".into());
            }
            functions.insert(id, (function.clone(), definition.name().clone()));
        }
    }
    let constants = constants::collect(
        package,
        constants::Context {
            exports,
            public: &public,
            header: header.file(),
            implementation: implementation.module(),
        },
        &mut definitions,
        &mut symbols,
    )?;
    if functions
        .keys()
        .chain(constants.keys())
        .copied()
        .collect::<BTreeSet<_>>()
        != public
    {
        return Err("C dependency API omits a compiler public binding".into());
    }
    let foreign_structs = structs::imported_for_signatures(
        projection.registry.registrations(),
        functions.values().map(|(function, _)| function),
        &structs,
    )?;
    Ok(Inventory {
        structs,
        foreign_structs,
        root,
        header,
        implementation: implementation.module().clone(),
        functions,
        constants,
        foreign_constants: selected
            .foreign
            .into_iter()
            .map(|((module, name), export)| {
                super::CForeignConstantExport::new(module, name, export.dependency)
            })
            .collect(),
    })
}
