use std::fmt;

use portable_ir::v0::{F64Bits, NodeId, Value, ValueField};
use serde_json::{Map, Value as JsonValue, json};

use crate::{EvaluationError, EvaluationOutcome};

const PROTOCOL: &str = "polyrust.canonical.v0";

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CanonicalDecodeError {
    message: String,
}

impl CanonicalDecodeError {
    fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

impl fmt::Display for CanonicalDecodeError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.message)
    }
}

impl std::error::Error for CanonicalDecodeError {}

pub fn encode_canonical_value(value: &Value) -> JsonValue {
    match value {
        Value::Unit => json!({"type": "unit"}),
        Value::Bool(value) => json!({"type": "bool", "value": value}),
        Value::I32(value) => json!({"type": "i32", "value": value.to_string()}),
        Value::I64(value) => json!({"type": "i64", "value": value.to_string()}),
        Value::F64(value) => {
            json!({"type": "f64", "bits": format!("{:016x}", value.0)})
        }
        Value::Char(value) => json!({"type": "char", "value": value.to_string()}),
        Value::String(value) => json!({"type": "string", "value": value}),
        Value::Bytes(value) => json!({"type": "bytes", "hex": encode_hex(value)}),
        Value::List(values) => json!({
            "type": "list",
            "values": values.iter().map(encode_canonical_value).collect::<Vec<_>>()
        }),
        Value::None => json!({"type": "option", "variant": "none"}),
        Value::Some(value) => json!({
            "type": "option",
            "variant": "some",
            "value": encode_canonical_value(value)
        }),
        Value::Ok(value) => json!({
            "type": "result",
            "variant": "ok",
            "value": encode_canonical_value(value)
        }),
        Value::Err(value) => json!({
            "type": "result",
            "variant": "err",
            "value": encode_canonical_value(value)
        }),
        Value::Record {
            declaration,
            fields,
        } => json!({
            "type": "record",
            "declaration": declaration.0.to_string(),
            "fields": encode_fields(fields)
        }),
        Value::Enum {
            declaration,
            variant,
            fields,
        } => json!({
            "type": "enum",
            "declaration": declaration.0.to_string(),
            "variant": variant.0.to_string(),
            "fields": encode_fields(fields)
        }),
    }
}

pub fn decode_canonical_value(value: &JsonValue) -> Result<Value, CanonicalDecodeError> {
    let object = object(value, "canonical value")?;
    let value_type = text_field(object, "type")?;
    match value_type {
        "unit" => Ok(Value::Unit),
        "bool" => Ok(Value::Bool(
            object
                .get("value")
                .and_then(JsonValue::as_bool)
                .ok_or_else(|| CanonicalDecodeError::new("bool.value must be boolean"))?,
        )),
        "i32" => text_field(object, "value")?
            .parse()
            .map(Value::I32)
            .map_err(|_| CanonicalDecodeError::new("i32.value is out of range")),
        "i64" => text_field(object, "value")?
            .parse()
            .map(Value::I64)
            .map_err(|_| CanonicalDecodeError::new("i64.value is out of range")),
        "f64" => {
            let bits = text_field(object, "bits")?;
            if bits.len() != 16
                || !bits
                    .bytes()
                    .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
            {
                return Err(CanonicalDecodeError::new(
                    "f64.bits must be 16 lowercase hexadecimal digits",
                ));
            }
            u64::from_str_radix(bits, 16)
                .map(F64Bits)
                .map(Value::F64)
                .map_err(|_| CanonicalDecodeError::new("f64.bits is invalid"))
        }
        "char" => {
            let text = text_field(object, "value")?;
            let mut characters = text.chars();
            let character = characters
                .next()
                .ok_or_else(|| CanonicalDecodeError::new("char.value is empty"))?;
            if characters.next().is_some() {
                return Err(CanonicalDecodeError::new(
                    "char.value must contain one Unicode scalar",
                ));
            }
            Ok(Value::Char(character))
        }
        "string" => Ok(Value::String(text_field(object, "value")?.to_owned())),
        "bytes" => decode_hex(text_field(object, "hex")?).map(Value::Bytes),
        "list" => array_field(object, "values")?
            .iter()
            .map(decode_canonical_value)
            .collect::<Result<Vec<_>, _>>()
            .map(Value::List),
        "option" => match text_field(object, "variant")? {
            "none" => Ok(Value::None),
            "some" => decode_canonical_value(required_field(object, "value")?)
                .map(Box::new)
                .map(Value::Some),
            _ => Err(CanonicalDecodeError::new("unknown option.variant")),
        },
        "result" => {
            let value = decode_canonical_value(required_field(object, "value")?)?;
            match text_field(object, "variant")? {
                "ok" => Ok(Value::Ok(Box::new(value))),
                "err" => Ok(Value::Err(Box::new(value))),
                _ => Err(CanonicalDecodeError::new("unknown result.variant")),
            }
        }
        "record" => Ok(Value::Record {
            declaration: NodeId(parse_u64_text(object, "declaration")?),
            fields: decode_fields(array_field(object, "fields")?)?,
        }),
        "enum" => Ok(Value::Enum {
            declaration: NodeId(parse_u64_text(object, "declaration")?),
            variant: NodeId(parse_u64_text(object, "variant")?),
            fields: decode_fields(array_field(object, "fields")?)?,
        }),
        _ => Err(CanonicalDecodeError::new(format!(
            "unknown canonical value type {value_type:?}"
        ))),
    }
}

fn encode_fields(fields: &[ValueField]) -> Vec<JsonValue> {
    fields
        .iter()
        .map(|field| {
            json!({
                "field": field.field.0.to_string(),
                "value": encode_canonical_value(&field.value)
            })
        })
        .collect()
}

fn decode_fields(fields: &[JsonValue]) -> Result<Vec<ValueField>, CanonicalDecodeError> {
    fields
        .iter()
        .map(|field| {
            let field = object(field, "aggregate field")?;
            Ok(ValueField {
                field: NodeId(parse_u64_text(field, "field")?),
                value: decode_canonical_value(required_field(field, "value")?)?,
            })
        })
        .collect()
}

fn encode_hex(bytes: &[u8]) -> String {
    const DIGITS: &[u8; 16] = b"0123456789abcdef";
    let mut output = String::with_capacity(bytes.len().saturating_mul(2));
    for byte in bytes {
        output.push(char::from(DIGITS[usize::from(byte >> 4)]));
        output.push(char::from(DIGITS[usize::from(byte & 0x0f)]));
    }
    output
}

fn decode_hex(text: &str) -> Result<Vec<u8>, CanonicalDecodeError> {
    if !text.len().is_multiple_of(2)
        || !text
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
    {
        return Err(CanonicalDecodeError::new(
            "bytes.hex must be even-length lowercase hexadecimal",
        ));
    }
    let (pairs, remainder) = text.as_bytes().as_chunks::<2>();
    if !remainder.is_empty() {
        return Err(CanonicalDecodeError::new(
            "bytes.hex must contain complete byte pairs",
        ));
    }
    pairs
        .iter()
        .map(|pair| {
            let pair = std::str::from_utf8(pair.as_slice())
                .map_err(|_| CanonicalDecodeError::new("bytes.hex is not UTF-8"))?;
            u8::from_str_radix(pair, 16)
                .map_err(|_| CanonicalDecodeError::new("bytes.hex contains an invalid byte"))
        })
        .collect()
}

pub fn encode_canonical_error(error: &EvaluationError) -> JsonValue {
    let mut object = Map::new();
    object.insert(
        "code".to_owned(),
        JsonValue::String(error.code().to_owned()),
    );
    match error {
        EvaluationError::CheckedOverflow { operation } => {
            insert_text(&mut object, "operation", operation);
        }
        EvaluationError::InvalidShift { amount, width } => {
            insert_text(&mut object, "amount", &amount.to_string());
            insert_text(&mut object, "width", &width.to_string());
        }
        EvaluationError::NarrowingOutOfRange { value } => {
            insert_text(&mut object, "value", &value.to_string());
        }
        EvaluationError::IndexOutOfBounds { index, length } => {
            insert_text(&mut object, "index", &index.to_string());
            insert_text(&mut object, "length", &length.to_string());
        }
        EvaluationError::FuelExhausted { limit } => {
            insert_text(&mut object, "limit", &limit.to_string());
        }
        EvaluationError::CallDepthExceeded { limit } => {
            insert_text(&mut object, "limit", &limit.to_string());
        }
        EvaluationError::CollectionLimitExceeded { limit, requested } => {
            insert_text(&mut object, "limit", &limit.to_string());
            insert_text(&mut object, "requested", &requested.to_string());
        }
        EvaluationError::InvariantViolation { message } => {
            insert_text(&mut object, "message", message);
        }
        EvaluationError::DivisionByZero
        | EvaluationError::RemainderByZero
        | EvaluationError::InvalidUtf8 => {}
    }
    JsonValue::Object(object)
}

pub fn decode_canonical_error(value: &JsonValue) -> Result<EvaluationError, CanonicalDecodeError> {
    let object = object(value, "canonical error")?;
    Ok(match text_field(object, "code")? {
        "checked_overflow" => EvaluationError::CheckedOverflow {
            operation: decode_operation(text_field(object, "operation")?)?,
        },
        "division_by_zero" => EvaluationError::DivisionByZero,
        "remainder_by_zero" => EvaluationError::RemainderByZero,
        "invalid_shift" => EvaluationError::InvalidShift {
            amount: parse_text(object, "amount")?,
            width: parse_text(object, "width")?,
        },
        "narrowing_out_of_range" => EvaluationError::NarrowingOutOfRange {
            value: parse_text(object, "value")?,
        },
        "index_out_of_bounds" => EvaluationError::IndexOutOfBounds {
            index: parse_text(object, "index")?,
            length: parse_text(object, "length")?,
        },
        "invalid_utf8" => EvaluationError::InvalidUtf8,
        "fuel_exhausted" => EvaluationError::FuelExhausted {
            limit: parse_text(object, "limit")?,
        },
        "call_depth_exceeded" => EvaluationError::CallDepthExceeded {
            limit: parse_text(object, "limit")?,
        },
        "collection_limit_exceeded" => EvaluationError::CollectionLimitExceeded {
            limit: parse_text(object, "limit")?,
            requested: parse_text(object, "requested")?,
        },
        "invariant_violation" => EvaluationError::InvariantViolation {
            message: text_field(object, "message")?.to_owned(),
        },
        code => {
            return Err(CanonicalDecodeError::new(format!(
                "unknown canonical error code {code:?}"
            )));
        }
    })
}

pub fn encode_canonical_outcome(outcome: &EvaluationOutcome) -> JsonValue {
    match outcome {
        EvaluationOutcome::Value(value) => json!({
            "protocol": PROTOCOL,
            "outcome": "value",
            "value": encode_canonical_value(value)
        }),
        EvaluationOutcome::Error(error) => json!({
            "protocol": PROTOCOL,
            "outcome": "error",
            "error": encode_canonical_error(error)
        }),
    }
}

pub fn decode_canonical_outcome(
    value: &JsonValue,
) -> Result<EvaluationOutcome, CanonicalDecodeError> {
    let object = object(value, "canonical outcome")?;
    if text_field(object, "protocol")? != PROTOCOL {
        return Err(CanonicalDecodeError::new(
            "canonical outcome protocol is unsupported",
        ));
    }
    match text_field(object, "outcome")? {
        "value" => {
            decode_canonical_value(required_field(object, "value")?).map(EvaluationOutcome::Value)
        }
        "error" => {
            decode_canonical_error(required_field(object, "error")?).map(EvaluationOutcome::Error)
        }
        _ => Err(CanonicalDecodeError::new(
            "canonical outcome must be value or error",
        )),
    }
}

fn decode_operation(operation: &str) -> Result<&'static str, CanonicalDecodeError> {
    match operation {
        "neg" => Ok("neg"),
        "add" => Ok("add"),
        "sub" => Ok("sub"),
        "mul" => Ok("mul"),
        "div" => Ok("div"),
        "rem" => Ok("rem"),
        _ => Err(CanonicalDecodeError::new(
            "checked-overflow operation is unsupported",
        )),
    }
}

fn insert_text(object: &mut Map<String, JsonValue>, key: &str, value: &str) {
    object.insert(key.to_owned(), JsonValue::String(value.to_owned()));
}

fn object<'a>(
    value: &'a JsonValue,
    description: &str,
) -> Result<&'a Map<String, JsonValue>, CanonicalDecodeError> {
    value
        .as_object()
        .ok_or_else(|| CanonicalDecodeError::new(format!("{description} must be an object")))
}

fn required_field<'a>(
    object: &'a Map<String, JsonValue>,
    field: &str,
) -> Result<&'a JsonValue, CanonicalDecodeError> {
    object
        .get(field)
        .ok_or_else(|| CanonicalDecodeError::new(format!("missing canonical field {field:?}")))
}

fn text_field<'a>(
    object: &'a Map<String, JsonValue>,
    field: &str,
) -> Result<&'a str, CanonicalDecodeError> {
    required_field(object, field)?
        .as_str()
        .ok_or_else(|| CanonicalDecodeError::new(format!("{field} must be a string")))
}

fn array_field<'a>(
    object: &'a Map<String, JsonValue>,
    field: &str,
) -> Result<&'a [JsonValue], CanonicalDecodeError> {
    required_field(object, field)?
        .as_array()
        .map(Vec::as_slice)
        .ok_or_else(|| CanonicalDecodeError::new(format!("{field} must be an array")))
}

fn parse_u64_text(
    object: &Map<String, JsonValue>,
    field: &str,
) -> Result<u64, CanonicalDecodeError> {
    parse_text(object, field)
}

fn parse_text<T>(object: &Map<String, JsonValue>, field: &str) -> Result<T, CanonicalDecodeError>
where
    T: std::str::FromStr,
{
    text_field(object, field)?
        .parse()
        .map_err(|_| CanonicalDecodeError::new(format!("{field} has an invalid integer")))
}
