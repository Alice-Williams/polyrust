//! One complete certified/reserved bundle, then one atomic filesystem transaction.
use super::CheckedGraph;
use portable_directory_publication::{PathPolicy, TreeLimits};
use portable_java_bundle::{Owner, PreparedBundle};
use std::path::Path;

pub(super) fn publish(destination: &Path, graph: &CheckedGraph) -> Result<(), String> {
    let owners: Vec<_> = graph
        .crates
        .values()
        .map(|member| Owner {
            key: &member.key,
            api: &member.api,
        })
        .collect();
    let prepared = PreparedBundle::new(graph.root, &owners)?;
    let output = prepared.render()?;
    let count = output.owner_count();
    if !(1..=1024).contains(&count) || output.files().len() != 2 * count + 1 {
        return Err("Java production bundle inventory differs".into());
    }
    portable_directory_publication::publish(
        destination,
        output.files(),
        PathPolicy::RelativeTree(TreeLimits {
            files: 2 * count + 1,
            directories: count + 6,
            depth: 7,
            path_bytes: 256,
            bytes: usize::try_from(prepared.reserved_bytes())
                .map_err(|_| "Java bundle byte bound does not fit host")?,
        }),
    )
}
