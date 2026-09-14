//! Owned complete-graph inventory; defining keys are descriptive, not authority.
use portable_backend_java::dialect::JavaDependencyApi;
use portable_codegen::RustDeclarationId;
use std::collections::BTreeMap;

pub(crate) struct CheckedCrate {
    pub(super) api: JavaDependencyApi,
    pub(super) key: String,
}

/// Certificates retain source/export/docs and exact used-owner inventories.
/// Serialized manifest projection belongs to publication, never reconstruction.
pub(crate) struct CheckedGraph {
    pub(super) root: RustDeclarationId,
    pub(super) crates: BTreeMap<RustDeclarationId, CheckedCrate>,
}

impl CheckedGraph {
    pub(super) fn from_checked(
        root_key: &str,
        checked: BTreeMap<String, CheckedCrate>,
    ) -> Result<Self, String> {
        let root = checked
            .get(root_key)
            .ok_or("checked Java root missing")?
            .api
            .root();
        if checked.iter().any(|(key, item)| key != &item.key) {
            return Err("checked Java defining-key inventory differs".into());
        }
        let count = checked.len();
        let crates: BTreeMap<_, _> = checked
            .into_values()
            .map(|item| (item.api.root(), item))
            .collect();
        if crates.len() != count {
            return Err("duplicate checked Java owner".into());
        }
        Ok(Self { root, crates })
    }
}
