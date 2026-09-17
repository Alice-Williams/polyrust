use crate::{Owner, manifest::Manifest};
use portable_backend_java::{
    ast::{JavaFileItem, JavaType},
    dialect::{JavaSourceDescriptionKind as Kind, JavaSourceTarget},
};
use portable_codegen::{
    RustCrateExports, RustDeclarationId, RustExportNamespace, RustExportTarget, RustModuleAncestry,
    RustModuleDocumentation,
};
use std::collections::{BTreeMap, BTreeSet};

pub(crate) fn exports(
    api: &portable_backend_java::dialect::JavaDependencyApi,
) -> Result<&RustCrateExports, String> {
    for file in api.package().ast().files() {
        for item in file.items() {
            if let JavaFileItem::Type {
                source_package: Some(source),
                ..
            } = &item.item
            {
                return Ok(source.exports().as_ref());
            }
        }
    }
    api.source_descriptions()?
        .first()
        .map(|description| description.source().crate_exports.as_ref())
        .ok_or("Java owner has no certified source-package graph".into())
}

pub(crate) fn project(owner: Owner<'_>) -> Result<Manifest<'_>, String> {
    let api = owner.api;
    let root = api.root();
    let source_bound = api.source_byte_bound()?;
    let declarations = api.source_descriptions()?;
    let exports = exports(api)?;
    let foreign_constants = crate::constant_exports::collect(api)?;
    let foreign_bindings: BTreeMap<_, _> = foreign_constants
        .iter()
        .map(|export| {
            (
                (export.module(), export.name()),
                export.dependency().declaration(),
            )
        })
        .collect();
    if exports.root != root {
        return Err("Java export root disagrees".into());
    }
    let mut modules = BTreeMap::new();
    let mut seen = BTreeSet::new();
    for chain in exports
        .module_ancestries
        .values()
        .chain(declarations.iter().map(|d| &d.source().module_ancestors))
    {
        add_ancestry(chain, &mut seen, &mut modules)?;
    }
    let mut ids = BTreeSet::new();
    let mut public = BTreeSet::new();
    let mut export_allocations = BTreeSet::from([exports as *const _]);
    for description in &declarations {
        let source = description.source();
        if !ids.insert(source.declaration)
            || source.declaration.crate_id != root.crate_id
            || !modules.contains_key(&source.module)
        {
            return Err("Java declaration inventory disagrees".into());
        }
        if export_allocations.insert(source.crate_exports.as_ref() as *const _)
            && source.crate_exports.as_ref() != exports
        {
            return Err("Java declaration export inventory disagrees".into());
        }
        match description.kind() {
            Kind::Function { parameters, result } => {
                scalar(result)?;
                for parameter in parameters {
                    scalar(&parameter.ty)?;
                }
                if source.externally_reachable {
                    let function = api
                        .function(source.declaration)
                        .ok_or("public Java function missing")?;
                    if (!std::ptr::eq(function.source(), source) && function.source() != source)
                        || description.target() != JavaSourceTarget::Declaration(function.path())
                        || &function.signature().result != result
                        || !parameters
                            .iter()
                            .map(|p| &p.ty)
                            .eq(function.signature().parameters.iter())
                    {
                        return Err("Java function description differs from owner witness".into());
                    }
                    public.insert(source.declaration);
                } else if api.function(source.declaration).is_some() {
                    return Err("private Java description grants a callable".into());
                }
            }
            Kind::Constant { ty, value } => {
                scalar(ty)?;
                let constant = api
                    .constant(source.declaration)
                    .ok_or("public Java constant missing")?;
                if !source.externally_reachable
                    || (!std::ptr::eq(constant.source(), source) && constant.source() != source)
                    || description.target() != JavaSourceTarget::Declaration(constant.path())
                    || constant.ty() != ty
                    || constant.value() != value
                {
                    return Err("Java constant description differs from owner witness".into());
                }
                public.insert(source.declaration);
            }
            Kind::Record => {}
            Kind::Field { ty, .. } => {
                scalar(ty)?;
            }
        }
    }
    if public
        != api
            .functions()
            .map(|f| f.declaration())
            .chain(api.constants().map(|c| c.declaration()))
            .collect()
    {
        return Err("Java public declaration inventory differs".into());
    }
    for (id, module) in &modules {
        if id.crate_id != root.crate_id || module.parent.is_some_and(|p| !modules.contains_key(&p))
        {
            return Err("Java module ancestry is incomplete".into());
        }
    }
    for (module, bindings) in &exports.modules {
        if !modules.contains_key(module) {
            return Err("Java export module metadata missing".into());
        }
        for (name, target) in bindings {
            if name.namespace == RustExportNamespace::Macro {
                return Err("unsupported macro export".into());
            }
            match target {
                RustExportTarget::Module(id)
                    if id.crate_id == root.crate_id && modules.contains_key(id) => {}
                RustExportTarget::Declaration(id) if ids.contains(id) => {}
                RustExportTarget::Declaration(id)
                    if foreign_bindings.get(&(*module, name)) == Some(id) => {}
                _ => return Err("Java export target is outside retained inventory".into()),
            }
        }
    }
    Ok(Manifest {
        owner,
        source: format!(
            "src/main/java/org/polyrust/generated/r{:016x}/Generated.java",
            root.crate_id
        ),
        filename: format!("polyrust_{:016x}.api.json", root.crate_id),
        declarations,
        foreign_constants,
        modules,
        exports,
        source_bound,
        json_bound: 0,
    })
}

fn add_ancestry<'a>(
    chain: &'a RustModuleAncestry,
    seen: &mut BTreeSet<(*const std::sync::Arc<RustModuleDocumentation>, usize)>,
    modules: &mut BTreeMap<RustDeclarationId, &'a RustModuleDocumentation>,
) -> Result<(), String> {
    // Pointer identity only skips repeated traversal of immutable shared ancestry.
    // Stable IDs and content equality, never addresses, determine output.
    if !seen.insert((chain.as_ptr(), chain.len())) {
        return Ok(());
    }
    for module in chain.iter() {
        if let Some(previous) = modules.insert(module.declaration, module.as_ref())
            && !std::ptr::eq(previous, module.as_ref())
            && previous != module.as_ref()
        {
            return Err("conflicting Java module metadata".into());
        }
    }
    Ok(())
}

pub(crate) fn scalar(ty: &JavaType) -> Result<&'static str, String> {
    use portable_backend_java::ast::JavaPrimitive;
    if *ty == JavaType::primitive(JavaPrimitive::Int) {
        Ok("i32")
    } else if *ty == JavaType::primitive(JavaPrimitive::Long) {
        Ok("i64")
    } else if *ty == JavaType::primitive(JavaPrimitive::Boolean) {
        Ok("bool")
    } else {
        Err("unsupported Java manifest scalar".into())
    }
}
