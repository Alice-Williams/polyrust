//! Version-seven result metadata preserves the object/result distinction.
use super::{ApiManifest, import_serialization::scalar, serialization::quote};
use portable_backend_c::ast::{CFunctionType, CReturnType};

pub(super) fn result(ty: &CReturnType) -> Result<&'static str, String> {
    match ty {
        CReturnType::Void => Ok("unit"),
        CReturnType::Value(value) => scalar(value.declared_type()),
    }
}
pub(super) fn signature(text: &mut String, signature: &CFunctionType) -> Result<(), String> {
    text.push_str(",\"return\":");
    text.push_str(&quote(result(signature.return_type())?));
    text.push_str(",\"parameters\":[");
    for (index, parameter) in signature.parameters().iter().enumerate() {
        if index != 0 {
            text.push(',');
        }
        text.push_str(&quote(scalar(parameter.declared_type())?));
    }
    text.push(']');
    Ok(())
}
impl ApiManifest {
    pub(super) fn has_unit_results(&self) -> bool {
        self.functions.values().any(|function| {
            matches!(
                function.reference.signature().return_type(),
                CReturnType::Void
            )
        }) || self
            .imports
            .values()
            .any(|(_, proof)| matches!(proof.signature().return_type(), CReturnType::Void))
    }
}
