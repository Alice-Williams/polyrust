//! Register the complete admitted callable inventory, then lower each body.
use super::{
    LoweredPackage, Result, Selection, c, capabilities, functions, origin, package, selection,
};
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
    let roots = match &entry {
        Some((root, _)) => vec![*root],
        None => selection::public_roots(tcx, &exports)?,
    };
    let inventory = functions::inventory(tcx, &roots, lookup)?;
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
    let state = package::State {
        registry,
        file,
        mappings,
        functions,
        foreign_functions,
        records: HashMap::new(),
        declarations: Vec::new(),
        origins,
    };
    #[cfg(public_package_contract)]
    let state = package::assertions::check(tcx, state, &inventory.owned);
    let package::Bodies {
        state,
        prototypes,
        public_prototypes,
        definitions,
    } = package::lower_functions(tcx, state, &inventory.owned, header.as_ref())?;
    let mut items = state.declarations;
    items.extend(prototypes);
    items.extend(definitions);
    let source = c(c(CDeclarations::new(&state.registry, state.file.clone()))?.source_file(items))?;
    let mut files = Vec::new();
    if let Some(header) = header {
        files.push(c(
            c(CDeclarations::new(&state.registry, header))?.source_file(public_prototypes)
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
    let functions = state
        .functions
        .into_iter()
        .map(|(id, function)| (origin::identity(tcx, id.to_def_id()), function))
        .collect();
    Ok(LoweredPackage {
        registry: state.registry.freeze(),
        sources: files,
        exports,
        functions,
        imports,
    })
}
