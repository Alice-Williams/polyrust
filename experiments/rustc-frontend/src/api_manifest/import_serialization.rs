//! Version-two descriptive import records; certificate authority is never encoded.
use super::{
    ApiManifest,
    serialization::{identity, quote},
};
use portable_backend_c::ast::*;
use std::fmt::Write;

pub(super) fn scalar(ty: &CObjectType) -> Result<&'static str, String> {
    match ty.kind() {
        CObjectTypeKind::Scalar(CScalarType::I32) => Ok("i32"),
        CObjectTypeKind::Scalar(CScalarType::U32) => Ok("u32"),
        CObjectTypeKind::Scalar(CScalarType::I64) => Ok("i64"),
        CObjectTypeKind::Scalar(CScalarType::Bool) => Ok("bool"),
        CObjectTypeKind::Scalar(CScalarType::F64) => Ok("f64"),
        _ => Err("import manifest requires an admitted certified scalar signature".into()),
    }
}

impl ApiManifest {
    pub(super) fn write_imports(&self, text: &mut String) -> Result<(), String> {
        text.push_str(",\"imports\":[");
        for (index, (id, (_, proof))) in self.imports.iter().enumerate() {
            if index != 0 {
                text.push(',');
            }
            write!(text,
                "{{\"id\":{},\"owner\":{},\"header\":{},\"symbol\":{},\"return\":{},\"parameters\":[",
                identity(*id), identity(proof.package_identity().source_root().ok_or("C source inventory does not yet support canonical type owners")?),
                quote(proof.public_header().include_path()), quote(proof.symbol().as_str()),
                quote(super::function_results::result(proof.signature().return_type())?)).unwrap();
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
