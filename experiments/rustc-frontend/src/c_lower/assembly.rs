//! Register the complete admitted callable inventory, then lower each body.
use super::{LoweredPackage, Result, Selection, c, capabilities, functions, origin, package};
use crate::source_admission as admission;
use capabilities::{
    EntryInput, EntrySignatures, FunctionInput, FunctionSignatures, Mapping, Supports,
};
use portable_backend_c::ast::*;
use portable_codegen::{RelativeOutputPath, RustSourceNode};
use rustc_middle::ty::TyCtxt;
use std::collections::HashMap;

pub(crate) fn lower(
    tcx: TyCtxt<'_>,
    selection: Selection,
    lookup: Option<&super::ForeignLookup<'_>>,
) -> Result<LoweredPackage> {
    let mappings = capabilities::c_bindings();
    let entry = match selection {
        Selection::Entry(root) => {
            let signature = Supports::<EntrySignatures>::mapping(&mappings)
                .lower(&mut (), EntryInput { tcx, root })?;
            if !tcx.effective_visibilities(()).is_exported(root) {
                return Err("selected entry must be externally reachable in Rust".into());
            }
            Some((root, signature))
        }
        Selection::PublicApi => None,
    };
    admission::aliases(tcx)?;
    let mut origins = origin::Cache::default();
    let exports = origins.exports(tcx)?;
    let public_inventory = if entry.is_none() {
        Some(if lookup.is_some() {
            crate::source_origin::public_api::Inventory::read_with_constant_reexports(
                tcx,
                &mut origins,
            )?
        } else {
            crate::source_origin::public_api::Inventory::read(tcx, &mut origins)?
        })
    } else {
        None
    };
    let roots = match &entry {
        Some((root, _)) => vec![*root],
        None => public_inventory
            .as_ref()
            .ok_or("missing public export inventory")?
            .declarations()
            .values()
            .filter(|declaration| {
                declaration.kind() == crate::source_origin::public_api::DeclarationKind::Function
            })
            .map(|declaration| declaration.definition())
            .collect(),
    };
    let inventory = functions::inventory(tcx, &roots, lookup)?;
    let constant_imports = public_inventory
        .as_ref()
        .map(|public| public.constant_imports(tcx, &inventory.constants))
        .transpose()?
        .unwrap_or_default();
    let mut registry = CRegistry::new();
    let header = if entry.is_none() {
        Some(c(registry.register_file(CFileKey {
            path: RelativeOutputPath::new(format!("polyrust_{:016x}.h", exports.root.crate_id))
                .map_err(|error| format!("invalid C header path: {error:?}"))?,
            role: CFileRole::GeneratedPublicHeader,
        }))?)
    } else {
        None
    };
    let file = c(registry.register_file(CFileKey {
        path: RelativeOutputPath::new(if header.is_some() {
            format!("polyrust_{:016x}.c", exports.root.crate_id)
        } else {
            "model.c".into()
        })
        .map_err(|error| format!("invalid C output path: {error:?}"))?,
        role: CFileRole::GeneratedSource,
    }))?;
    if let Some(header) = &header {
        c(registry.register_source_package(header, exports.clone()))?;
    }
    let mut functions = HashMap::new();
    for id in &inventory.owned {
        let signature = if let Some((selected, signature)) = &entry
            && id == selected
        {
            signature.clone()
        } else {
            Supports::<FunctionSignatures>::mapping(&mappings).lower(
                &mut (),
                FunctionInput {
                    tcx,
                    function: id.to_def_id(),
                },
            )?
        };
        let name = if entry.as_ref().is_some_and(|(selected, _)| id == selected) {
            "poly_score".into()
        } else if header.is_some() {
            let identity = origin::identity(tcx, id.to_def_id());
            format!(
                "fn_{:016x}_{:016x}",
                identity.crate_id, identity.definition_path_hash
            )
        } else {
            format!(
                "fn_{:016x}",
                tcx.def_path_hash(id.to_def_id()).local_hash().as_u64()
            )
        };
        let key = origin::key(
            tcx,
            &mut origins,
            id.to_def_id(),
            RustSourceNode::Declaration,
            tcx.def_span(*id),
            &name,
        )?;
        let primary = if tcx.effective_visibilities(()).is_exported(*id) {
            header.as_ref().unwrap_or(&file)
        } else {
            &file
        };
        let function = c(registry.register_function(primary, key, signature))?;
        functions.insert(*id, function);
    }
    let mut foreign_functions = HashMap::new();
    let mut imports = std::collections::BTreeMap::new();
    let mut foreign: Vec<_> = inventory.foreign.into_iter().collect();
    foreign.sort_by_key(|(_, proof)| proof.declaration());
    for (id, proof) in foreign {
        let signature = Supports::<FunctionSignatures>::mapping(&mappings)
            .lower(&mut (), FunctionInput { tcx, function: id })?;
        #[cfg(c_graph_wrong_signature)]
        let signature = crate::foreign_mutations::signature(signature);
        if proof.declaration() != origin::identity(tcx, id) || proof.signature() != &signature {
            return Err("foreign compiler identity/signature differs from C certificate".into());
        }
        let function = c(registry.import_function(proof.clone()))?;
        imports.insert(origin::identity(tcx, id), (function.clone(), proof));
        foreign_functions.insert(id, function);
    }
    let mut foreign_constants = HashMap::new();
    if header.is_some() {
        for id in &constant_imports {
            let lookup = lookup
                .ok_or("foreign public constant reads require a certified producer mapping")?;
            let proof = (lookup.constant)(*id)?;
            let input = capabilities::ConstantImportInput::read(tcx, *id)?;
            let mut registration = capabilities::ImportState { registry, proof };
            let object = Supports::<capabilities::PublicConstantImports>::mapping(&mappings)
                .lower(&mut registration, input)?;
            registry = registration.registry;
            foreign_constants.insert(*id, (object, registration.proof));
        }
    }
    let mut state = package::State {
        registry,
        file,
        mappings,
        functions,
        foreign_functions,
        header: header.clone(),
        constants: HashMap::new(),
        foreign_constants,
        records: HashMap::new(),
        declarations: Vec::new(),
        origins,
    };
    if let Some(public) = &public_inventory {
        for declaration in public.declarations().values() {
            if declaration.kind() == crate::source_origin::public_api::DeclarationKind::Constant {
                let input = capabilities::ConstantDeclarationInput::read(
                    tcx,
                    public,
                    declaration.definition().to_def_id(),
                )?;
                Supports::<capabilities::PublicConstants>::mapping(&mappings)
                    .lower(&mut state, input)?;
            }
        }
    }
    #[cfg(public_package_contract)]
    let state = package::assertions::check(tcx, state, &inventory.owned);
    let package::Bodies {
        state,
        prototypes,
        public_prototypes,
        definitions,
    } = package::lower_functions(tcx, state, &inventory.owned, header.as_ref())?;
    let (mut constant_prototypes, constant_definitions) = super::constants::files(&state)?;
    constant_prototypes.extend(public_prototypes);
    let mut items = state.declarations;
    items.extend(constant_definitions);
    items.extend(prototypes);
    items.extend(definitions);
    let source = c(c(CDeclarations::new(&state.registry, state.file.clone()))?.source_file(items))?;
    let mut files = Vec::new();
    if let Some(header) = header {
        files.push(c(
            c(CDeclarations::new(&state.registry, header))?.source_file(constant_prototypes)
        )?);
    }
    files.push(source);
    // rustc's source proof does not replace target representation checks.
    c(state.registry.check_context(&files))?;
    c(state.registry.check_constants_and_layout(&files))?;
    c(state.registry.check_sequencing_and_control(&files))?;
    c(state.registry.check_numeric_flow(&files))?;
    c(state.registry.check_index_extents(&files))?;
    c(state.registry.check_storage_paths(&files))?;
    let source_types = crate::source_origin::types::collect(
        tcx,
        exports.root,
        &inventory.owned,
        state.records.keys().copied(),
        state.constants.keys().copied(),
    )?;
    #[cfg(character_source_probe)]
    let source_types = crate::source_origin::types::probe::inspect(source_types);
    crate::source_origin::types::authenticate(
        tcx,
        &inventory.owned,
        state.records.keys().copied(),
        state.constants.keys().copied(),
        &source_types,
    )?;
    #[cfg(character_source_probe)]
    let source_types = crate::source_origin::types::probe::after_authentication(source_types);
    let functions = state
        .functions
        .into_iter()
        .map(|(id, function)| (origin::identity(tcx, id.to_def_id()), function))
        .collect();
    let constants = state
        .constants
        .into_iter()
        .map(|(id, (object, value))| {
            (
                origin::identity(tcx, id),
                (object, super::constants::value(value)),
            )
        })
        .collect();
    let constant_imports = state
        .foreign_constants
        .into_iter()
        .map(|(id, imported)| (origin::identity(tcx, id), imported))
        .collect();
    Ok(LoweredPackage {
        source_types,
        constant_imports,
        constants,
        registry: state.registry.freeze(),
        sources: files,
        exports,
        functions,
        imports,
    })
}
