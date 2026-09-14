//! Exercise the production per-file descriptor resolver, not a probe-only mapper.
use super::{configuration::Configuration, inputs::DeclaredInputs};
use portable_rustc_configuration::graph::{CrateDescription, InputMapping, ResolvedInputs};
use std::path::Path;

pub(super) fn arguments(
    configuration: &Configuration,
    inputs: &DeclaredInputs,
    sysroot: &str,
    output: &str,
) -> Result<Vec<String>, String> {
    let (name, key) = configuration
        .explicit_identity()
        .ok_or("probe needs explicit identity")?;
    let root = Path::new(inputs.root());
    let base = root.parent().ok_or("missing root directory")?;
    let logical_root = root
        .file_name()
        .and_then(|name| name.to_str())
        .ok_or("non-UTF-8 root name")?;
    // Analyze-only probes use '-' but stop before emission. The descriptor's
    // artifact name is not opened by source resolution or used as authority.
    let metadata = if output == "-" {
        "unused-analysis.rmeta"
    } else {
        output
    };
    let mut description = CrateDescription::new(
        name,
        key,
        InputMapping::new(inputs.root(), logical_root)?,
        metadata,
    )?;
    let (pairs, []) = configuration.declared_inputs().as_chunks::<2>() else {
        return Err("incomplete declared input pair".into());
    };
    for pair in pairs {
        let physical = Path::new(&pair[1])
            .canonicalize()
            .map_err(|error| error.to_string())?;
        let logical = physical
            .strip_prefix(base)
            .map_err(|_| "probe fixture input is outside its declared base")?
            .to_str()
            .ok_or("non-UTF-8 logical input")?;
        description = description.with_input(InputMapping::new(&pair[1], logical)?)?;
    }
    let resolved = ResolvedInputs::load(&description)?;
    if resolved.root() != inputs.root() {
        return Err("probe root changed during declared input resolution".into());
    }
    resolved.metadata_arguments(sysroot, output)
}
