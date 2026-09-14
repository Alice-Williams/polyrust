//! Fixed-arity graph records, converted immediately to validated descriptor types.
use super::{
    CrateDescription, CrateGraph, InputMapping, MAX_CRATES, MAX_EDGES, MAX_TOTAL_INPUTS, charge,
};

pub(super) const MAX_ARGUMENT_BYTES: usize = 32 * 1024 * 1024;
pub(super) const MAX_ARGUMENTS: usize = 2 + MAX_CRATES * 6 + (MAX_EDGES + MAX_TOTAL_INPUTS) * 3;

pub(super) fn read(arguments: &[String]) -> Result<CrateGraph, String> {
    if arguments.len() > MAX_ARGUMENTS {
        return Err("crate graph argument count budget exceeded".into());
    }
    let mut bytes = 0;
    for argument in arguments {
        charge(&mut bytes, argument.len(), MAX_ARGUMENT_BYTES)?;
    }
    let mut fields = arguments.iter();
    if fields.next().map(String::as_str) != Some("--root") {
        return Err("crate graph requires --root KEY first".into());
    }
    let root = value(&mut fields)?;
    let mut descriptions = Vec::new();
    let mut current: Option<CrateDescription> = None;
    let mut input_count = 0;
    let mut edge_count = 0;
    while let Some(flag) = fields.next() {
        match flag.as_str() {
            "--crate" => {
                if let Some(previous) = current.take() {
                    descriptions.push(previous);
                }
                if descriptions.len() >= MAX_CRATES {
                    return Err("crate graph requires at most 1024 crate records".into());
                }
                let name = value(&mut fields)?;
                let key = value(&mut fields)?;
                let physical = value(&mut fields)?;
                let logical = value(&mut fields)?;
                let metadata = value(&mut fields)?;
                charge(&mut input_count, 1, MAX_TOTAL_INPUTS)?;
                current = Some(CrateDescription::new(
                    name,
                    key,
                    InputMapping::new(physical, logical)?,
                    metadata,
                )?);
            }
            "--input" => {
                let description = current
                    .take()
                    .ok_or("--input requires a preceding --crate")?;
                let physical = value(&mut fields)?;
                let logical = value(&mut fields)?;
                charge(&mut input_count, 1, MAX_TOTAL_INPUTS)?;
                current = Some(description.with_input(InputMapping::new(physical, logical)?)?);
            }
            "--dependency" => {
                let description = current
                    .take()
                    .ok_or("--dependency requires a preceding --crate")?;
                let alias = value(&mut fields)?;
                let key = value(&mut fields)?;
                charge(&mut edge_count, 1, MAX_EDGES)?;
                current = Some(description.with_dependency(alias, key)?);
            }
            _ => return Err(format!("unsupported crate graph option: {flag}")),
        }
    }
    if let Some(last) = current {
        descriptions.push(last);
    }
    CrateGraph::new(root, descriptions)
}

fn value<'a>(fields: &mut impl Iterator<Item = &'a String>) -> Result<&'a str, String> {
    fields
        .next()
        .map(String::as_str)
        .ok_or("incomplete crate graph record".into())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parser_bounds_are_checked_before_record_decoding() {
        assert!(
            read(&vec![String::new(); MAX_ARGUMENTS + 1])
                .unwrap_err()
                .contains("argument count")
        );
        assert!(
            read(&["x".repeat(MAX_ARGUMENT_BYTES + 1)])
                .unwrap_err()
                .contains("budget exceeded")
        );
        let mut count = MAX_ARGUMENT_BYTES - 1;
        charge(&mut count, 1, MAX_ARGUMENT_BYTES).unwrap();
        assert!(charge(&mut count, 1, MAX_ARGUMENT_BYTES).is_err());
    }
}
