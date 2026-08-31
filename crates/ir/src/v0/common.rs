use std::{collections::BTreeMap, fmt, str::FromStr};

use serde::{Deserialize, Deserializer, Serialize, Serializer, de};

/// Semantic version of the serialized IR schema.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct IrVersion {
    /// Breaking schema generation.
    pub major: u16,
    /// Backward-compatible feature generation.
    pub minor: u16,
    /// Schema clarification/fix generation.
    pub patch: u16,
}

impl IrVersion {
    /// The schema emitted by this crate.
    pub const CURRENT: Self = Self::new(0, 1, 0);

    /// Creates a semantic IR version.
    pub const fn new(major: u16, minor: u16, patch: u16) -> Self {
        Self {
            major,
            minor,
            patch,
        }
    }

    /// Returns whether this reader supports the version's major generation.
    pub const fn is_compatible_with(self, reader: Self) -> bool {
        self.major == reader.major
    }
}

impl fmt::Display for IrVersion {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}.{}.{}", self.major, self.minor, self.patch)
    }
}

/// Error returned when semantic-version text is not exactly `major.minor.patch`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct IrVersionParseError {
    /// Original rejected input.
    pub input: String,
}

impl fmt::Display for IrVersionParseError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "invalid IR version {:?}; expected major.minor.patch",
            self.input
        )
    }
}

impl std::error::Error for IrVersionParseError {}

impl FromStr for IrVersion {
    type Err = IrVersionParseError;

    fn from_str(input: &str) -> Result<Self, Self::Err> {
        let invalid = || IrVersionParseError {
            input: input.to_owned(),
        };
        let mut components = input.split('.');
        let major = components.next().ok_or_else(invalid)?;
        let minor = components.next().ok_or_else(invalid)?;
        let patch = components.next().ok_or_else(invalid)?;
        if components.next().is_some()
            || [major, minor, patch].iter().any(|component| {
                component.is_empty() || (component.len() > 1 && component.starts_with('0'))
            })
        {
            return Err(invalid());
        }
        Ok(Self::new(
            major.parse().map_err(|_| invalid())?,
            minor.parse().map_err(|_| invalid())?,
            patch.parse().map_err(|_| invalid())?,
        ))
    }
}

impl Serialize for IrVersion {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.collect_str(self)
    }
}

impl<'de> Deserialize<'de> for IrVersion {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let text = String::deserialize(deserializer)?;
        text.parse().map_err(de::Error::custom)
    }
}

/// Stable document-local identity for a syntax node.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct NodeId(pub u64);

impl NodeId {
    /// Creates a node ID. Structural validation rejects zero and duplicates.
    pub const fn new(value: u64) -> Self {
        Self(value)
    }
}

/// Origin of an unchecked syntax node.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(
    tag = "kind",
    content = "data",
    rename_all = "snake_case",
    deny_unknown_fields
)]
pub enum SourceRef {
    /// Half-open UTF-8 byte range in a logical input file.
    File(FileSpan),
    /// Builder/front-end path independent of a physical file.
    Logical(LogicalSource),
}

impl SourceRef {
    /// Creates a logical builder path.
    pub fn logical(segments: impl IntoIterator<Item = impl Into<String>>) -> Self {
        Self::Logical(LogicalSource {
            segments: segments.into_iter().map(Into::into).collect(),
        })
    }
}

/// Half-open byte span in a logical source file.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FileSpan {
    /// Portable input label; canonical documents must not use absolute paths.
    pub file: String,
    /// Inclusive UTF-8 byte offset.
    pub start: u64,
    /// Exclusive UTF-8 byte offset.
    pub end: u64,
}

/// Builder/front-end path for nodes without a physical source file.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LogicalSource {
    /// Ordered path segments, for example `module(example)` and `record(User)`.
    pub segments: Vec<String>,
}

/// Identity and source carried by every syntax node.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NodeMeta {
    /// Stable document-local node identity.
    pub id: NodeId,
    /// Original source or builder path.
    pub source: SourceRef,
}

impl NodeMeta {
    /// Creates node metadata.
    pub const fn new(id: NodeId, source: SourceRef) -> Self {
        Self { id, source }
    }
}

/// Public API visibility portable across required targets.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Visibility {
    /// Visible outside the generated package/module.
    Public,
    /// Visible only within the generated package/module.
    Package,
}

/// Shared declaration metadata.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DeclarationHeader {
    /// Node identity and source.
    pub node: NodeMeta,
    /// Portable identifier before resolution.
    pub name: String,
    /// Portable visibility.
    pub visibility: Visibility,
    /// Non-semantic documentation paragraphs.
    pub documentation: Vec<String>,
}

/// Shared metadata for fields, variants, parameters, and methods.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MemberHeader {
    /// Node identity and source.
    pub node: NodeMeta,
    /// Portable member identifier before resolution.
    pub name: String,
    /// Non-semantic documentation paragraphs.
    pub documentation: Vec<String>,
}

/// Complete v0 portable type syntax.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(
    tag = "kind",
    content = "data",
    rename_all = "snake_case",
    deny_unknown_fields
)]
pub enum TypeRef {
    Unit,
    Bool,
    I32,
    I64,
    F64,
    Char,
    String,
    Bytes,
    List(Box<TypeRef>),
    Option(Box<TypeRef>),
    Result {
        ok: Box<TypeRef>,
        error: Box<TypeRef>,
    },
    /// Record, enum, or alias declaration reference.
    Named(NodeId),
    /// Restricted contract parameter view.
    Contract(NodeId),
}

/// Exact IEEE-754 binary64 bits, preserving NaNs and negative zero.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct F64Bits(pub u64);

impl F64Bits {
    /// Captures the exact representation of a Rust `f64`.
    pub fn from_f64(value: f64) -> Self {
        Self(value.to_bits())
    }

    /// Reconstructs the represented IEEE-754 value.
    pub fn to_f64(self) -> f64 {
        f64::from_bits(self.0)
    }
}

/// Canonical portable value used by constants and test expectations.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(
    tag = "kind",
    content = "data",
    rename_all = "snake_case",
    deny_unknown_fields
)]
pub enum Value {
    Unit,
    Bool(bool),
    I32(i32),
    I64(i64),
    F64(F64Bits),
    Char(char),
    String(String),
    Bytes(Vec<u8>),
    List(Vec<Value>),
    None,
    Some(Box<Value>),
    Ok(Box<Value>),
    Err(Box<Value>),
    Record {
        declaration: NodeId,
        fields: Vec<ValueField>,
    },
    Enum {
        declaration: NodeId,
        variant: NodeId,
        fields: Vec<ValueField>,
    },
}

/// Field value keyed by a resolved declaration identity.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ValueField {
    pub field: NodeId,
    pub value: Value,
}

/// Value paired with its explicit portable type.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TypedValue {
    pub ty: TypeRef,
    pub value: Value,
}

/// Root of a versioned `.poly.json` document.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Document {
    pub ir_version: IrVersion,
    pub module: super::Module,
    /// Deterministically ordered non-semantic producer metadata.
    pub metadata: BTreeMap<String, String>,
}

impl Document {
    /// Creates a document with empty producer metadata.
    pub fn new(ir_version: IrVersion, module: super::Module) -> Self {
        Self {
            ir_version,
            module,
            metadata: BTreeMap::new(),
        }
    }
}
