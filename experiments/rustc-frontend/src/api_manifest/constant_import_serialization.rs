//! Version-five constant references remain descriptive, never import authority.
use super::{
    ApiManifest, constants,
    serialization::{identity, quote},
};
use std::fmt::Write;

impl ApiManifest {
    pub(super) fn write_constant_imports(&self, text: &mut String) -> Result<(), String> {
        text.push_str(",\"constant_imports\":[");
        for (position, (id, (_, proof))) in self.used_constant_imports.iter().enumerate() {
            if position != 0 {
                text.push(',');
            }
            let (ty, value) = constants::certified_scalar(proof.value())?;
            write!(text,
                "{{\"id\":{},\"owner\":{},\"header\":{},\"symbol\":{},\"type\":{},\"value\":{},\"readonly\":true}}",
                identity(*id), identity(proof.package_identity().root()),
                quote(proof.public_header().include_path()), quote(proof.symbol().as_str()),
                quote(ty), value).unwrap();
        }
        text.push(']');
        Ok(())
    }
    pub(super) fn constant_import_bounds(
        &self,
    ) -> impl Iterator<Item = Result<usize, String>> + '_ {
        self.used_constant_imports.values().map(|(_, proof)| {
            proof
                .symbol()
                .as_str()
                .len()
                .checked_add(proof.public_header().include_path().len())
                .and_then(|bytes| bytes.checked_mul(6))
                .and_then(|bytes| bytes.checked_add(512))
                .ok_or_else(|| "API constant import byte overflow".into())
        })
    }
}
