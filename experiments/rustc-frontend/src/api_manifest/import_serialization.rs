//! Version-two descriptive import records; certificate authority is never encoded.
use super::{
    ApiManifest,
    serialization::{identity, quote},
};
use portable_backend_c::ast::*;
use std::fmt::Write;

fn scalar(ty: &CObjectType) -> Result<&'static str, String> {
    match ty.kind() {
        CObjectTypeKind::Scalar(CScalarType::I32) => Ok("i32"),
        CObjectTypeKind::Scalar(CScalarType::Bool) => Ok("bool"),
        _ => Err("import manifest requires a certified i32/bool signature".into()),
    }
}

impl ApiManifest {
    pub(super) fn write_imports(&self, text: &mut String) -> Result<(), String> {
        text.push_str(",\"imports\":[");
        for (index, (id, (_, proof))) in self.imports.iter().enumerate() {
            if index != 0 {
                text.push(',');
            }
            let CReturnType::Value(result) = proof.signature().return_type() else {
                return Err("import manifest requires a scalar return".into());
            };
            write!(text,
                "{{\"id\":{},\"owner\":{},\"header\":{},\"symbol\":{},\"return\":{},\"parameters\":[",
                identity(*id), identity(proof.package_identity().root()),
                quote(proof.public_header().include_path()), quote(proof.symbol().as_str()),
                quote(scalar(result.declared_type())?)).unwrap();
            for (index, parameter) in proof.signature().parameters().iter().enumerate() {
                if index != 0 {
                    text.push(',');
                }
                text.push_str(&quote(scalar(parameter.declared_type())?));
            }
            text.push_str("]}");
        }
        text.push(']');
        Ok(())
    }

    pub(super) fn import_bounds(&self) -> impl Iterator<Item = Result<usize, String>> + '_ {
        self.imports.values().map(|(_, proof)| {
            proof
                .symbol()
                .as_str()
                .len()
                .checked_add(proof.public_header().include_path().len())
                .and_then(|bytes| bytes.checked_mul(6))
                .and_then(|bytes| {
                    proof
                        .signature()
                        .parameters()
                        .len()
                        .checked_mul(8)
                        .and_then(|parameters| bytes.checked_add(parameters))
                })
                .and_then(|bytes| bytes.checked_add(512))
                .ok_or_else(|| "API import byte overflow".into())
        })
    }
}
