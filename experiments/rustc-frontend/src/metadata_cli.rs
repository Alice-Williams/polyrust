//! Bounded nonrecursive response files for the closed declared graph protocol.
use portable_rustc_configuration::graph::CrateGraph;
use std::{fs, io::Read, path::Path};

const MAX_BYTES: u64 = CrateGraph::MAX_ARGUMENT_BYTES as u64;

pub(crate) fn parse(arguments: &[String]) -> Result<CrateGraph, String> {
    let [argument] = arguments else {
        return CrateGraph::parse(arguments);
    };
    let Some(path) = argument.strip_prefix('@') else {
        return CrateGraph::parse(arguments);
    };
    if path.is_empty() || !Path::new(path).is_file() {
        return Err("graph response file must be an existing regular file".into());
    }
    let file = fs::File::open(path).map_err(|error| error.to_string())?;
    let metadata = file.metadata().map_err(|error| error.to_string())?;
    if !metadata.is_file() || metadata.len() > MAX_BYTES {
        return Err("graph response file exceeds its regular-file/32 MiB limit".into());
    }
    let mut bytes = Vec::new();
    file.take(MAX_BYTES + 1)
        .read_to_end(&mut bytes)
        .map_err(|error| error.to_string())?;
    if bytes.len() as u64 > MAX_BYTES {
        return Err("graph response file exceeds 32 MiB".into());
    }
    let text = String::from_utf8(bytes).map_err(|_| "graph response file is not UTF-8")?;
    // One verbatim argument per LF-delimited line; never shell-unquote, trim,
    // interpolate, or recursively expand another response file.
    let arguments = text
        .strip_suffix('\n')
        .unwrap_or(&text)
        .split('\n')
        .take(CrateGraph::MAX_ARGUMENTS + 1)
        .map(str::to_owned)
        .collect::<Vec<_>>();
    CrateGraph::parse(&arguments)
}
