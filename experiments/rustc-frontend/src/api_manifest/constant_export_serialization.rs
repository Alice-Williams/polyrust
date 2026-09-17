//! Schema six retains each alias without copying the defining C object.
use super::{
    ApiManifest, constants,
    serialization::{identity, quote},
};
use std::fmt::Write;

impl ApiManifest {
    pub(super) fn write_constant_exports(&self, text: &mut String) -> Result<(), String> {
        text.push_str(",\"constant_exports\":[");
        for (position, binding) in self.foreign_constants.iter().enumerate() {
            if position != 0 {
                text.push(',');
            }
            let proof = binding.dependency();
            let (ty, value) = constants::scalar(proof.value())?;
            write!(text,
                "{{\"module\":{},\"namespace\":\"value\",\"name\":{},\"id\":{},\"owner\":{},\"header\":{},\"symbol\":{},\"type\":{},\"value\":{},\"readonly\":true}}",
                identity(binding.module()), quote(&binding.name().name), identity(proof.declaration()),
                identity(proof.package_identity().root()), quote(proof.public_header().include_path()),
                quote(proof.symbol().as_str()), quote(ty), value).unwrap();
        }
        text.push(']');
        Ok(())
    }
    pub(super) fn constant_export_bounds(
        &self,
    ) -> impl Iterator<Item = Result<usize, String>> + '_ {
        self.foreign_constants.iter().map(|binding| {
            binding
                .name()
                .name
                .len()
                .checked_add(binding.dependency().symbol().as_str().len())
                .and_then(|bytes| {
                    bytes.checked_add(binding.dependency().public_header().include_path().len())
                })
                .and_then(|bytes| bytes.checked_mul(6))
                .and_then(|bytes| bytes.checked_add(768))
                .ok_or_else(|| "API constant export byte overflow".into())
        })
    }
}
