use std::{collections::BTreeSet, fmt};

use serde_json::Value as JsonValue;

use super::{Document, NodeId};

/// Structural problems detectable without name resolution or type checking.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum StructuralError {
    ZeroNodeId,
    DuplicateNodeId(NodeId),
    Serialization(String),
}

impl fmt::Display for StructuralError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ZeroNodeId => write!(formatter, "node ID 0 is reserved"),
            Self::DuplicateNodeId(id) => write!(formatter, "duplicate node ID {}", id.0),
            Self::Serialization(message) => {
                write!(formatter, "cannot inspect IR structure: {message}")
            }
        }
    }
}

impl std::error::Error for StructuralError {}

/// Detects zero or duplicate syntax-node identities.
///
/// This pass intentionally does not resolve declaration references or validate
/// types; those are checker responsibilities.
pub fn validate_structure(document: &Document) -> Result<(), StructuralError> {
    let value = serde_json::to_value(document)
        .map_err(|error| StructuralError::Serialization(error.to_string()))?;
    let mut seen = BTreeSet::new();
    visit_node_metadata(&value, &mut seen)
}

fn visit_node_metadata(
    value: &JsonValue,
    seen: &mut BTreeSet<NodeId>,
) -> Result<(), StructuralError> {
    match value {
        JsonValue::Object(object) => {
            if let Some(JsonValue::Object(node)) = object.get("node")
                && let Some(JsonValue::Number(id)) = node.get("id")
                && let Some(id) = id.as_u64()
            {
                let id = NodeId(id);
                if id.0 == 0 {
                    return Err(StructuralError::ZeroNodeId);
                }
                if !seen.insert(id) {
                    return Err(StructuralError::DuplicateNodeId(id));
                }
            }
            for child in object.values() {
                visit_node_metadata(child, seen)?;
            }
        }
        JsonValue::Array(values) => {
            for child in values {
                visit_node_metadata(child, seen)?;
            }
        }
        JsonValue::Null | JsonValue::Bool(_) | JsonValue::Number(_) | JsonValue::String(_) => {}
    }
    Ok(())
}
