//! Entry admission after rustc analysis; all C lowering uses backend-c types.
use crate::{api_manifest::ApiManifest, c_lower::Selection, configuration::Mode};
use portable_backend_c::dialect::{CDialect, project_c_package};
use portable_codegen::{
    RenderReadyPackage, TargetLinker, certify_resolved_package, verify_unresolved_package,
};
use rustc_hir::def::DefKind;
use rustc_middle::ty::TyCtxt;

pub(crate) struct Program {
    pub(crate) package: RenderReadyPackage<CDialect>,
    pub(crate) manifest: Option<ApiManifest>,
}

fn selected_entry(tcx: TyCtxt<'_>) -> Result<Selection, String> {
    let roots: Vec<_> = tcx
        .hir_body_owners()
        .filter(|id| {
            tcx.def_kind(*id) == DefKind::Fn && tcx.item_name(id.to_def_id()).as_str() == "score"
        })
        .collect();
    if roots.len() != 1 {
        return Err("expected exactly one function named score".into());
    }
    Ok(Selection::Entry(roots[0]))
}

pub(crate) fn program(tcx: TyCtxt<'_>, mode: Mode) -> Result<Program, String> {
    let selection = match mode {
        Mode::Entry => selected_entry(tcx)?,
        Mode::PublicPackage => Selection::PublicApi,
    };
    let lowered = crate::c_lower::lower(tcx, selection)?;
    certify(lowered, mode)
}

pub(crate) fn dependency_program(
    tcx: TyCtxt<'_>,
    lookup: &crate::c_lower::ForeignLookup<'_>,
) -> Result<Program, String> {
    certify(
        crate::c_lower::assembly::lower(tcx, Selection::PublicApi, Some(lookup))?,
        Mode::PublicPackage,
    )
}

fn certify(lowered: crate::c_lower::LoweredPackage, mode: Mode) -> Result<Program, String> {
    let package = project_c_package(lowered.registry, lowered.sources)
        .map_err(|errors| format!("C projection: {errors:?}"))?;
    let verified = verify_unresolved_package(&CDialect, package)
        .map_err(|errors| format!("C verification: {errors:?}"))?;
    let linked = TargetLinker::new(CDialect)
        .link_ast(&verified)
        .map_err(|errors| format!("C linking: {errors:?}"))?;
    let package = certify_resolved_package(&CDialect, linked)
        .map_err(|errors| format!("C certification: {errors:?}"))?;
    let manifest = match mode {
        Mode::Entry => None,
        Mode::PublicPackage => {
            let manifest = if !lowered.constants.is_empty() {
                let manifest = ApiManifest::with_constants(
                    &package,
                    lowered.exports.clone(),
                    &lowered.functions,
                    &lowered.imports,
                    &lowered.constants,
                )?;
                manifest.verify_constants(
                    &package,
                    lowered.exports.clone(),
                    &lowered.functions,
                    &lowered.imports,
                    &lowered.constants,
                )?;
                #[cfg(public_constant_ast_probe)]
                crate::api_manifest::constant_contract::check(
                    &package,
                    &manifest,
                    &lowered.functions,
                    &lowered.constants,
                );
                manifest
            } else if lowered.imports.is_empty() {
                let manifest =
                    ApiManifest::new(&package, lowered.exports.clone(), &lowered.functions)?;
                manifest.verify(&package, lowered.exports, &lowered.functions)?;
                manifest
            } else {
                let manifest = ApiManifest::with_imports(
                    &package,
                    lowered.exports.clone(),
                    &lowered.functions,
                    &lowered.imports,
                )?;
                manifest.verify_imports(
                    &package,
                    lowered.exports,
                    &lowered.functions,
                    &lowered.imports,
                )?;
                manifest
            };
            #[cfg(c_graph_inventory_contract)]
            crate::api_manifest::inventory_contract::check(
                &package,
                &manifest,
                &lowered.functions,
                &lowered.imports,
                &lowered.constants,
            );
            Some(manifest)
        }
    };
    Ok(Program { package, manifest })
}
