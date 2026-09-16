use crate::{
    Owner,
    budget::Budget,
    json::{Encoder, Reservation},
    manifest::Manifest,
    projection, serialization,
};
use portable_backend_java::dialect::JavaStructuralRenderer;
use portable_codegen::{
    OutputContents, RustDeclarationId, TargetSymbolRef, render_certified_package,
};
use std::collections::{BTreeMap, BTreeSet};

/// Complete immutable reservation. Construction neither renders nor serializes.
pub struct PreparedBundle<'a> {
    root: RustDeclarationId,
    members: Vec<Manifest<'a>>,
    names: BTreeSet<String>,
    index_bound: u64,
    total_bound: u64,
}

/// Reconciled complete payload. Publication is a separate atomic transaction.
pub struct BundleOutput {
    files: Vec<(String, String)>,
    owners: usize,
}
impl BundleOutput {
    pub fn files(&self) -> &[(String, String)] {
        &self.files
    }
    pub fn owner_count(&self) -> usize {
        self.owners
    }
}

impl<'a> PreparedBundle<'a> {
    pub fn new(root: RustDeclarationId, owners: &[Owner<'a>]) -> Result<Self, String> {
        if owners.is_empty() || owners.len() > 1024 {
            return Err("Java bundle requires 1..1024 owners".into());
        }
        let mut graph = BTreeMap::new();
        let mut keys = BTreeSet::new();
        let mut crates = BTreeSet::new();
        for owner in owners {
            if graph.insert(owner.api.root(), *owner).is_some()
                || !keys.insert(owner.key)
                || !crates.insert(owner.api.root().crate_id)
            {
                return Err("duplicate Java bundle owner/key".into());
            }
        }
        if !graph.contains_key(&root) {
            return Err("Java bundle root missing".into());
        }
        for owner in graph.values() {
            for dependency in owner.api.dependencies() {
                let member = graph
                    .get(&dependency.root())
                    .ok_or("Java bundle dependency owner missing")?;
                if member.api.package_identity() != dependency {
                    return Err("Java bundle dependency owner replaced".into());
                }
            }
            for symbol in owner
                .api
                .package()
                .ast()
                .files()
                .iter()
                .flat_map(|f| f.items())
                .flat_map(|i| i.names.keys())
            {
                if let TargetSymbolRef::DependencyValue(value) = symbol {
                    let proof = value.constant();
                    let member = graph
                        .get(&proof.package_identity().root())
                        .ok_or("Java bundle constant owner missing")?;
                    if member.api.constant(proof.declaration()) != Some(proof) {
                        return Err("Java bundle constant witness replaced".into());
                    }
                }
                if let TargetSymbolRef::DependencyCallable(callable) = symbol {
                    let function = callable.function();
                    let member = graph
                        .get(&function.package_identity().root())
                        .ok_or("Java bundle callable owner missing")?;
                    if member.api.function(function.declaration()) != Some(function) {
                        return Err("Java bundle callable witness replaced".into());
                    }
                }
            }
        }
        let mut budget = Budget::default();
        let mut members = Vec::new();
        let mut names = BTreeSet::from(["bundle.json".to_owned()]);
        for owner in graph.into_values() {
            let mut manifest = projection::project(owner)?;
            let mut reservation = Reservation::default();
            serialization::owner(&mut reservation, &manifest)?;
            manifest.json_bound = reservation.0.0;
            budget.add(manifest.source_bound)?;
            budget.add(manifest.json_bound)?;
            if !names.insert(manifest.source.clone()) || !names.insert(manifest.filename.clone()) {
                return Err("Java bundle filename collision".into());
            }
            members.push(manifest);
        }
        let mut reservation = Reservation::default();
        serialization::index(&mut reservation, root, &members)?;
        let index_bound = reservation.0.0;
        budget.add(index_bound)?;
        Ok(Self {
            root,
            members,
            names,
            index_bound,
            total_bound: budget.0,
        })
    }

    pub fn reserved_bytes(&self) -> u64 {
        self.total_bound
    }

    pub fn render(&self) -> Result<BundleOutput, String> {
        let mut files = Vec::new();
        for member in &self.members {
            member.verify_owner()?;
            let rendered =
                render_certified_package(&JavaStructuralRenderer, member.owner.api.package())
                    .map_err(|errors| format!("Java bundle rendering: {errors:?}"))?;
            let [file] = rendered.files() else {
                return Err("Java owner rendered multiple files".into());
            };
            let OutputContents::Text(contents) = file.contents() else {
                return Err("Java source is not text".into());
            };
            if file.path() != member.source || contents.len() as u64 > member.source_bound {
                return Err("Java source differs from reservation".into());
            }
            files.push((member.source.clone(), contents.clone()));
            let mut encoder = Encoder::new(member.json_bound);
            serialization::owner(&mut encoder, member)?;
            files.push((member.filename.clone(), encoder.finish()));
        }
        let mut encoder = Encoder::new(self.index_bound);
        serialization::index(&mut encoder, self.root, &self.members)?;
        files.push(("bundle.json".into(), encoder.finish()));
        reconcile(&files, &self.names, self.total_bound)?;
        Ok(BundleOutput {
            files,
            owners: self.members.len(),
        })
    }
}

pub(crate) fn reconcile(
    files: &[(String, String)],
    names: &BTreeSet<String>,
    bound: u64,
) -> Result<(), String> {
    let actual: BTreeSet<_> = files.iter().map(|(name, _)| name.clone()).collect();
    let mut bytes = Budget::default();
    for (_, contents) in files {
        bytes.add(contents.len() as u64)?;
    }
    if actual != *names || files.len() != names.len() || bytes.0 > bound {
        return Err("Java bundle payload inventory/bounds differ".into());
    }
    Ok(())
}
