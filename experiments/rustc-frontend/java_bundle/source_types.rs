//! Descriptive original types, not target validity or foreign-input validation.
use crate::json::Sink;
use portable_codegen::{RustConstantValue, RustSourceTypes};

pub(crate) fn write(out: &mut impl Sink, types: Option<&RustSourceTypes>) -> Result<(), String> {
    let Some(types) = types.filter(|types| types.contains_char()) else {
        return Ok(());
    };
    out.fixed(",\"source_types\":{\"char_foreign_input_domain\":")?;
    out.string("Unicode scalar: 0..=0x10ffff excluding 0xd800..=0xdfff")?;
    out.fixed(",\"functions\":[")?;
    for (index, (id, signature)) in types.functions().iter().enumerate() {
        if index != 0 {
            out.fixed(",")?;
        }
        out.fixed("{\"id\":")?;
        out.id(*id)?;
        out.fixed(",\"result\":")?;
        out.string(signature.result.spelling())?;
        out.fixed(",\"parameters\":[")?;
        for (index, kind) in signature.parameters.iter().enumerate() {
            if index != 0 {
                out.fixed(",")?;
            }
            out.string(kind.spelling())?;
        }
        out.fixed("]}")?;
    }
    out.fixed("],\"fields\":[")?;
    for (index, (id, field)) in types.fields().iter().enumerate() {
        if index != 0 {
            out.fixed(",")?;
        }
        out.fixed("{\"id\":")?;
        out.id(*id)?;
        out.fixed(",\"owner\":")?;
        out.id(field.owner)?;
        out.fixed(",\"scalar\":")?;
        out.string(field.kind.spelling())?;
        out.fixed("}")?;
    }
    out.fixed("]")?;
    if !types.constants().is_empty() {
        out.fixed(",\"constants\":[")?;
        for (index, (id, value)) in types.constants().iter().enumerate() {
            if index != 0 {
                out.fixed(",")?;
            }
            out.fixed("{\"id\":")?;
            out.id(*id)?;
            out.fixed(",\"scalar\":")?;
            out.string(value.kind().spelling())?;
            out.fixed(",\"value\":")?;
            match value {
                RustConstantValue::Bool(value) => {
                    out.fixed(if *value { "true" } else { "false" })?
                }
                RustConstantValue::I32(value) => out.string(&value.to_string())?,
                RustConstantValue::I64(value) => out.string(&value.to_string())?,
                RustConstantValue::F64Bits(value) => out.string(&format!("0x{value:016x}"))?,
                RustConstantValue::Char(value) => out.string(&u32::from(*value).to_string())?,
            }
            out.fixed("}")?;
        }
        out.fixed("]")?;
    }
    out.fixed("}")
}
