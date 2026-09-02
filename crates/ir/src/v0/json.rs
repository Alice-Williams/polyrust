use std::fmt;

use serde_json::Value as JsonValue;

use super::{Document, IrVersion, StructuralError, validate_structure};

/// Result of explicitly migrating the final `contract`-named v0 schema.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LegacyInterfaceMigration {
    pub source_version: IrVersion,
    pub target_version: IrVersion,
    pub document: Document,
}

/// Configurable defenses for parsing untrusted `.poly.json` input.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ReadLimits {
    pub max_bytes: usize,
    pub max_depth: usize,
    /// Maximum JSON structural values after decoding.
    pub max_nodes: usize,
    /// Maximum UTF-8 bytes in one string value or object key.
    pub max_string_bytes: usize,
}

impl Default for ReadLimits {
    fn default() -> Self {
        Self {
            max_bytes: 8 * 1024 * 1024,
            max_depth: 128,
            max_nodes: 1_000_000,
            max_string_bytes: 1024 * 1024,
        }
    }
}

/// Stable error categories ready for M03 diagnostic-code assignment.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum JsonErrorKind {
    TotalBytesLimit,
    DepthLimit,
    NodeLimit,
    StringLimit,
    InvalidJson,
    UnknownField,
    UnsupportedVersion,
    InvalidStructure,
}

/// Structured canonical JSON read/write failure.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct JsonError {
    pub kind: JsonErrorKind,
    pub message: String,
}

impl JsonError {
    fn new(kind: JsonErrorKind, message: impl Into<String>) -> Self {
        Self {
            kind,
            message: message.into(),
        }
    }
}

impl fmt::Display for JsonError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.message)
    }
}

impl std::error::Error for JsonError {}

/// Serializes a structurally valid current-generation document as compact,
/// deterministic UTF-8 JSON.
pub fn to_canonical_json(document: &Document) -> Result<Vec<u8>, JsonError> {
    ensure_current(document.ir_version)?;
    validate_structure(document).map_err(structure_error)?;
    let mut canonical = document.clone();
    canonical
        .module
        .declarations
        .sort_by_key(|declaration| declaration.header().node.id);
    serde_json::to_vec(&canonical)
        .map_err(|error| JsonError::new(JsonErrorKind::InvalidJson, error.to_string()))
}

/// Reads a document using conservative default resource limits.
pub fn from_json(bytes: &[u8]) -> Result<Document, JsonError> {
    from_json_with_limits(bytes, ReadLimits::default())
}

/// Reads untrusted JSON with explicit total-byte, depth, structural-value, and
/// string limits. Serde's strict schema rejects unknown fields and variants.
pub fn from_json_with_limits(bytes: &[u8], limits: ReadLimits) -> Result<Document, JsonError> {
    let value = parse_limited_value(bytes, limits)?;
    let version = json_version(&value)?;
    if version == IrVersion::LEGACY_CONTRACTS {
        return migrate_legacy_contracts_value(value).map(|migration| migration.document);
    }
    ensure_current(version)?;
    decode_current(value)
}

/// Explicitly migrates serialized v0.1 `contract` declarations, references,
/// dispatches, and implementation fields into the current interface schema.
pub fn migrate_legacy_v0_contracts(bytes: &[u8]) -> Result<LegacyInterfaceMigration, JsonError> {
    migrate_legacy_v0_contracts_with_limits(bytes, ReadLimits::default())
}

pub fn migrate_legacy_v0_contracts_with_limits(
    bytes: &[u8],
    limits: ReadLimits,
) -> Result<LegacyInterfaceMigration, JsonError> {
    let value = parse_limited_value(bytes, limits)?;
    let version = json_version(&value)?;
    if version != IrVersion::LEGACY_CONTRACTS {
        return Err(JsonError::new(
            JsonErrorKind::UnsupportedVersion,
            format!(
                "legacy interface migration requires IR version {}; found {version}",
                IrVersion::LEGACY_CONTRACTS
            ),
        ));
    }
    migrate_legacy_contracts_value(value)
}

fn parse_limited_value(bytes: &[u8], limits: ReadLimits) -> Result<JsonValue, JsonError> {
    if bytes.len() > limits.max_bytes {
        return Err(JsonError::new(
            JsonErrorKind::TotalBytesLimit,
            format!(
                "IR input is {} bytes; limit is {}",
                bytes.len(),
                limits.max_bytes
            ),
        ));
    }

    let value: JsonValue = serde_json::from_slice(bytes).map_err(classify_serde_error)?;
    let mut node_count = 0;
    measure_json(&value, 1, &mut node_count, limits)?;
    Ok(value)
}

fn decode_current(value: JsonValue) -> Result<Document, JsonError> {
    let document: Document = serde_json::from_value(value).map_err(classify_serde_error)?;
    validate_structure(&document).map_err(structure_error)?;
    Ok(document)
}

fn ensure_current(version: IrVersion) -> Result<(), JsonError> {
    if version == IrVersion::CURRENT {
        Ok(())
    } else {
        Err(JsonError::new(
            JsonErrorKind::UnsupportedVersion,
            format!(
                "IR version {version} is incompatible with reader {}",
                IrVersion::CURRENT
            ),
        ))
    }
}

fn json_version(value: &JsonValue) -> Result<IrVersion, JsonError> {
    value
        .get("ir_version")
        .and_then(JsonValue::as_str)
        .ok_or_else(|| JsonError::new(JsonErrorKind::InvalidJson, "missing string ir_version"))?
        .parse::<IrVersion>()
        .map_err(|error| JsonError::new(JsonErrorKind::InvalidJson, error.to_string()))
}

fn migrate_legacy_contracts_value(
    mut value: JsonValue,
) -> Result<LegacyInterfaceMigration, JsonError> {
    migrate_contract_terms(&mut value);
    let object = value.as_object_mut().ok_or_else(|| {
        JsonError::new(
            JsonErrorKind::InvalidJson,
            "IR document must be a JSON object",
        )
    })?;
    object.insert(
        "ir_version".to_owned(),
        JsonValue::String(IrVersion::CURRENT.to_string()),
    );
    Ok(LegacyInterfaceMigration {
        source_version: IrVersion::LEGACY_CONTRACTS,
        target_version: IrVersion::CURRENT,
        document: decode_current(value)?,
    })
}

fn migrate_contract_terms(value: &mut JsonValue) {
    match value {
        JsonValue::Object(object) => {
            if object.get("kind").and_then(JsonValue::as_str) == Some("contract") {
                object.insert("kind".to_owned(), JsonValue::String("interface".to_owned()));
            }
            for (legacy, current) in [
                ("contract", "interface"),
                ("contract_method", "interface_method"),
            ] {
                if let Some(value) = object.remove(legacy) {
                    object.insert(current.to_owned(), value);
                }
            }
            for child in object.values_mut() {
                migrate_contract_terms(child);
            }
        }
        JsonValue::Array(values) => {
            for child in values {
                migrate_contract_terms(child);
            }
        }
        JsonValue::Null | JsonValue::Bool(_) | JsonValue::Number(_) | JsonValue::String(_) => {}
    }
}

fn structure_error(error: StructuralError) -> JsonError {
    JsonError::new(JsonErrorKind::InvalidStructure, error.to_string())
}

fn classify_serde_error(error: serde_json::Error) -> JsonError {
    let message = error.to_string();
    let kind = if message.contains("unknown field") || message.contains("unknown variant") {
        JsonErrorKind::UnknownField
    } else {
        JsonErrorKind::InvalidJson
    };
    JsonError::new(kind, message)
}

fn measure_json(
    value: &JsonValue,
    depth: usize,
    node_count: &mut usize,
    limits: ReadLimits,
) -> Result<(), JsonError> {
    if depth > limits.max_depth {
        return Err(JsonError::new(
            JsonErrorKind::DepthLimit,
            format!("IR JSON depth exceeds limit {}", limits.max_depth),
        ));
    }
    *node_count = node_count.saturating_add(1);
    if *node_count > limits.max_nodes {
        return Err(JsonError::new(
            JsonErrorKind::NodeLimit,
            format!("IR JSON node count exceeds limit {}", limits.max_nodes),
        ));
    }

    match value {
        JsonValue::Object(object) => {
            for (key, child) in object {
                check_string(key, limits)?;
                measure_json(child, depth + 1, node_count, limits)?;
            }
        }
        JsonValue::Array(values) => {
            for child in values {
                measure_json(child, depth + 1, node_count, limits)?;
            }
        }
        JsonValue::String(text) => check_string(text, limits)?,
        JsonValue::Null | JsonValue::Bool(_) | JsonValue::Number(_) => {}
    }
    Ok(())
}

fn check_string(text: &str, limits: ReadLimits) -> Result<(), JsonError> {
    if text.len() > limits.max_string_bytes {
        Err(JsonError::new(
            JsonErrorKind::StringLimit,
            format!(
                "IR string is {} bytes; limit is {}",
                text.len(),
                limits.max_string_bytes
            ),
        ))
    } else {
        Ok(())
    }
}
