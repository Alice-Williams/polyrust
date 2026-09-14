//! CLI publication contract, not an alternative C renderer.
use portable_backend_c::dialect::{CDialect, CStructuralRenderer};
use portable_codegen::{OutputContents, RenderReadyPackage, render_certified_package};
pub(crate) mod bundle;
mod bundle_budget;
mod publication;

pub(super) fn publish_bundle(
    path: &std::path::Path,
    graph: &crate::c_graph::CheckedGraph,
) -> Result<(), String> {
    bundle::publish(path, graph)
}

pub(super) fn publish(
    path: &std::path::Path,
    program: &crate::extract::Program,
) -> Result<(), String> {
    if let Some(manifest) = &program.manifest {
        let rendered = render_certified_package(&CStructuralRenderer, &program.package)
            .map_err(|errors| format!("certified package rendering: {errors:?}"))?;
        let mut files = Vec::new();
        for file in rendered.files() {
            let OutputContents::Text(text) = file.contents() else {
                return Err("C package requires text".into());
            };
            files.push((file.path().to_owned(), text.clone()));
        }
        files.push(("api.json".into(), manifest.canonical_json()?));
        publication::new_directory(path, &files)
    } else {
        let text = source(&program.package)?;
        std::fs::write(path, text).map_err(|error| error.to_string())
    }
}

pub(super) fn source(package: &RenderReadyPackage<CDialect>) -> Result<String, String> {
    let rendered = render_certified_package(&CStructuralRenderer, package)
        .map_err(|errors| format!("certified rendering: {errors:?}"))?;
    let [file] = rendered.files() else {
        return Err("the experimental CLI requires exactly one C output file".into());
    };
    let OutputContents::Text(text) = file.contents() else {
        return Err("the experimental CLI requires a textual C compilation unit".into());
    };
    Ok(text.clone())
}
