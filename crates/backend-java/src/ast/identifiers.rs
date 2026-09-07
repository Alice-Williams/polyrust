//! Java AST: identifiers.

use portable_codegen::AstViolation;
use portable_diagnostics::DiagnosticCode;

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct JavaIdentifier(String);

impl JavaIdentifier {
    pub fn new(candidate: impl Into<String>) -> Result<Self, AstViolation> {
        let candidate = candidate.into();
        let mut chars = candidate.chars();
        let lexical = chars
            .next()
            .is_some_and(|first| first == '_' || first == '$' || first.is_ascii_alphabetic())
            && chars.all(|ch| ch == '_' || ch == '$' || ch.is_ascii_alphanumeric());
        if lexical && !JAVA_KEYWORDS.contains(&candidate.as_str()) {
            Ok(Self(candidate))
        } else {
            Err(AstViolation::new(
                DiagnosticCode::InvalidIdentifier,
                format!("invalid Java identifier {candidate:?}"),
            ))
        }
    }

    pub fn from_portable(candidate: &str) -> Self {
        let mut value = candidate
            .chars()
            .map(|ch| {
                if ch == '_' || ch == '$' || ch.is_ascii_alphanumeric() {
                    ch
                } else {
                    '_'
                }
            })
            .collect::<String>();
        if value.is_empty()
            || !value
                .chars()
                .next()
                .is_some_and(|ch| ch == '_' || ch == '$' || ch.is_ascii_alphabetic())
        {
            value.insert(0, '_');
        }
        if JAVA_KEYWORDS.contains(&value.as_str()) {
            value.push('_');
        }
        if value.starts_with("__polyrust_") {
            value.push_str("_user");
        }
        Self::new(value).expect("portable identifier normalization is valid")
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

pub(super) const JAVA_KEYWORDS: &[&str] = &[
    "abstract",
    "assert",
    "boolean",
    "break",
    "byte",
    "case",
    "catch",
    "char",
    "class",
    "const",
    "continue",
    "default",
    "do",
    "double",
    "else",
    "enum",
    "exports",
    "extends",
    "false",
    "final",
    "finally",
    "float",
    "for",
    "goto",
    "if",
    "implements",
    "import",
    "instanceof",
    "int",
    "interface",
    "long",
    "module",
    "native",
    "new",
    "non-sealed",
    "null",
    "open",
    "opens",
    "package",
    "permits",
    "private",
    "protected",
    "provides",
    "public",
    "record",
    "requires",
    "return",
    "sealed",
    "short",
    "static",
    "strictfp",
    "super",
    "switch",
    "synchronized",
    "this",
    "throw",
    "throws",
    "to",
    "transient",
    "transitive",
    "true",
    "try",
    "uses",
    "var",
    "void",
    "volatile",
    "when",
    "while",
    "with",
    "yield",
    "_",
];
