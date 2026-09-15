//! Reconstruct dependency API membership from the immutable certificate.
mod constants;
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
}

pub(super) struct Constant {
    pub object: CObjectRef,
    pub symbol: CIdentifier,
    pub value: CLiteral,
    pub read_type: CObjectType,
}

fn scalar(ty: &CObjectType) -> bool {
    matches!(
        ty.kind(),
        CObjectTypeKind::Scalar(CScalarType::I32 | CScalarType::I64 | CScalarType::Bool)
    )
}

fn signature(function: &CFunctionRef) -> bool {
    matches!(function.signature().return_type(), CReturnType::Value(value) if scalar(value.declared_type()))
        && function
            .signature()
            .parameters()
            .iter()
            .all(|value| scalar(value.declared_type()))
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
    let implementation = file(CFileRole::GeneratedSource)?;
    let projection = &implementation.items()[0].unit.projection;
    let header = CGeneratedHeader::resolve(
        projection.registry.registrations(),
        implementation.module(),
        header.module(),
    )
    .map_err(|error| error.message)?;
    let first = c_defined_functions(package)
        .map(|definition| &definition.function().key().origin)
        .chain(c_defined_constants(package).map(|definition| &definition.object().key().origin))
        .next()
        .ok_or("C dependency API has no definitions")?;
    let CGeneratedOrigin::RustSource(first_origin) = first else {
        return Err("C dependency API requires Rust-source definition provenance".into());
    };
    let exports = &first_origin.crate_exports;
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
    let mut public = BTreeSet::new();
    for bindings in exports.modules.values() {
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
                _ => {
                    return Err(
                        "C dependency export has no supported local function/constant mapping"
                            .into(),
                    );
                }
            }
        }
    }
    if public.is_empty() {
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
            if !signature(function) || !closed.contains(function) {
                return Err(
                    "C dependency function lacks the closed i32/i64/bool scalar-call proof".into(),
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
    Ok(Inventory {
        root,
        header,
        implementation: implementation.module().clone(),
        functions,
        constants,
    })
}
