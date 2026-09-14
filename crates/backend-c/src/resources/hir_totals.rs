//! Checked additive package budgets; maxima never reset at file boundaries.
use super::{Measurements, add};

pub(super) fn sum<'a>(
    files: impl IntoIterator<Item = &'a Measurements>,
) -> Result<Measurements, String> {
    let mut total = Measurements::default();
    for file in files {
        add(&mut total.nodes, file.nodes)?;
        add(&mut total.comment_bytes, file.comment_bytes)?;
        add(&mut total.diagnostic_bytes, file.diagnostic_bytes)?;
        add(&mut total.automatic_bytes, file.automatic_bytes)?;
        add(&mut total.automatic_objects, file.automatic_objects)?;
        add(&mut total.value_bytes, file.value_bytes)?;
        add(&mut total.source_bound, file.source_bound)?;
        total.depth = total.depth.max(file.depth);
        total.max_parameters = total.max_parameters.max(file.max_parameters);
        total.max_fields = total.max_fields.max(file.max_fields);
        total.max_identifier_bytes = total.max_identifier_bytes.max(file.max_identifier_bytes);
        total.max_diagnostic_bytes = total.max_diagnostic_bytes.max(file.max_diagnostic_bytes);
    }
    Ok(total)
}

pub(super) fn source_bound(measured: &Measurements) -> Result<u64, String> {
    // Every syntax node pays for two longest identifiers, punctuation, and
    // maximal indentation. Comments and escaped diagnostic bytes are additive.
    let per_node = (measured.max_identifier_bytes as u64)
        .checked_mul(2)
        .and_then(|v| v.checked_add(64))
        .and_then(|v| v.checked_add((measured.depth as u64).checked_mul(4)?))
        .ok_or("C source bound overflow")?;
    measured
        .nodes
        .checked_mul(per_node)
        .and_then(|v| v.checked_add(measured.comment_bytes))
        .and_then(|v| v.checked_add(measured.diagnostic_bytes.checked_mul(6)?))
        .and_then(|v| v.checked_add(128))
        .ok_or_else(|| "C source bound overflow".into())
}
