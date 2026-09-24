//! Whole-graph ownership and resource preflight followed by structural rendering.
use crate::c_graph::CheckedGraph;
use portable_backend_c::{
    ast::CLinkage,
    dialect::{
        CStructuralRenderer, c_defined_constants, c_defined_functions, c_imported_constants,
        c_imported_functions, c_output_byte_bound,
    },
};
use portable_codegen::{OutputContents, RustDeclarationId, render_certified_package};
use std::{collections::BTreeSet, fmt::Write, path::Path};

fn identity(id: RustDeclarationId) -> String {
    format!("\"{:016x}:{:016x}\"", id.crate_id, id.definition_path_hash)
}

pub(crate) fn preflight(graph: &CheckedGraph) -> Result<(BTreeSet<String>, u64), String> {
    let count = graph.crates().len();
    if count == 0 || count > 1024 || !graph.crates().contains_key(&graph.root()) {
        return Err("bundle has invalid member/root inventory".into());
    }
    let mut names = BTreeSet::from(["bundle.json".to_owned()]);
    let mut symbols = BTreeSet::new();
    let mut budget = super::bundle_budget::BundleBudget::new(count)?;
    for (root, member) in graph.crates() {
        let api = member.api();
        if Some(*root) != api.source_root() {
            return Err("bundle owner identity disagrees".into());
        }
        member.manifest().verify_owner(api)?;
        for extension in ["h", "c", "api.json"] {
            if !names.insert(format!("polyrust_{:016x}.{extension}", root.crate_id)) {
                return Err("bundle output collision".into());
            }
        }
        #[cfg(c_graph_constant_collision)]
        if let Some(constant) = c_defined_constants(api.package()).next() {
            symbols.insert(constant.name().clone());
        }
        for function in c_defined_functions(api.package()) {
            if function.linkage() == CLinkage::External && !symbols.insert(function.name().clone())
            {
                return Err("bundle has duplicate external definitions".into());
            }
        }
        for constant in c_defined_constants(api.package()) {
            if constant.linkage() == CLinkage::External && !symbols.insert(constant.name().clone())
            {
                return Err("bundle has duplicate external definitions".into());
            }
        }
        for imported in c_imported_functions(api.package()) {
            let proof = imported.dependency();
            let owner = graph
                .crates()
                .get(
                    &proof
                        .package_identity()
                        .source_root()
                        .ok_or("C source inventory does not yet support canonical type owners")?,
                )
                .ok_or("bundle is missing an imported owner")?;
            if owner.api().function(proof.declaration()) != Some(proof) {
                return Err("bundle import differs from exact member certificate".into());
            }
        }
        for imported in c_imported_constants(api.package()) {
            let proof = imported.dependency();
            let owner = graph
                .crates()
                .get(
                    &proof
                        .package_identity()
                        .source_root()
                        .ok_or("C source inventory does not yet support canonical type owners")?,
                )
                .ok_or("bundle is missing a constant imported owner")?;
            if owner.api().constant(proof.declaration()) != Some(proof) {
                return Err("bundle constant import differs from exact member certificate".into());
            }
        }
        let manifest_bound = member.manifest().bundle_bound()? as u64;
        budget.include(c_output_byte_bound(api.package())?, manifest_bound)?;
    }
    Ok((names, budget.bytes()))
}

pub(super) fn publish(path: &Path, graph: &CheckedGraph) -> Result<(), String> {
    let (names, bound) = preflight(graph)?;
    let count = graph.crates().len();
    let mut files = Vec::with_capacity(3 * count + 1);
    let mut index = format!(
        "{{\"schema_version\":1,\"root\":{},\"members\":[",
        identity(graph.root())
    );
    for (position, (root, member)) in graph.crates().iter().enumerate() {
        let manifest_name = format!("polyrust_{:016x}.api.json", root.crate_id);
        if position != 0 {
            index.push(',');
        }
        write!(
            index,
            "{{\"root\":{},\"manifest\":\"{}\"}}",
            identity(*root),
            manifest_name
        )
        .unwrap();
        let output = render_certified_package(&CStructuralRenderer, member.api().package())
            .map_err(|errors| format!("bundle rendering: {errors:?}"))?;
        for file in output.files() {
            let OutputContents::Text(text) = file.contents() else {
                return Err("C bundle requires text".into());
            };
            files.push((file.path().to_owned(), text.clone()));
        }
        files.push((manifest_name, member.manifest().bundle_json()?));
    }
    index.push_str("]}\n");
    files.push(("bundle.json".into(), index));
    let actual: BTreeSet<_> = files.iter().map(|(name, _)| name.clone()).collect();
    let bytes = files
        .iter()
        .try_fold(0u64, |sum, (_, text)| sum.checked_add(text.len() as u64))
        .ok_or("bundle rendered byte overflow")?;
    if files.len() != names.len() || actual != names || bytes > bound {
        return Err("rendered bundle differs from preflight inventory/bounds".into());
    }
    super::publication::bundle_directory(path, &files, count)
}
