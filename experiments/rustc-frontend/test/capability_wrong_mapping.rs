use super::{Builder, CEntrySignatures, CScalarComparisons};

#[allow(dead_code)]
fn must_not_compile() {
    Builder::new().literal_values(CScalarComparisons);
    Builder::new().direct_calls(CScalarComparisons);
    Builder::new().function_signatures(CEntrySignatures);
}
