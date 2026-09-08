//! Validated C spellings. Namespace/catalogue collision checks remain contextual.

use super::keywords::CKeyword;

/// A C17 identifier in the generator's conservative ASCII naming subset.
///
/// Reserved names cannot be introduced by directly forging a wrapper:
///
/// ```compile_fail
/// use portable_backend_c::ast::CIdentifier;
/// let forged = CIdentifier("not an identifier".to_owned());
/// ```
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct CIdentifier(String);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CReservedMacro {
    Bool,
    True,
    False,
    Null,
}

impl CReservedMacro {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Bool => "bool",
            Self::True => "true",
            Self::False => "false",
            Self::Null => "NULL",
        }
    }

    fn from_spelling(value: &str) -> Option<Self> {
        [Self::Bool, Self::True, Self::False, Self::Null]
            .into_iter()
            .find(|name| name.as_str() == value)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CNameError {
    Empty,
    InvalidStart,
    InvalidContinuation { byte_offset: usize },
    Keyword(CKeyword),
    ImplementationReserved,
    StandardMacro(CReservedMacro),
}

impl CIdentifier {
    pub fn new(value: &str) -> Result<Self, CNameError> {
        let Some(first) = value.bytes().next() else {
            return Err(CNameError::Empty);
        };
        if !first.is_ascii_alphabetic() && first != b'_' {
            return Err(CNameError::InvalidStart);
        }
        if let Some((byte_offset, _)) = value
            .bytes()
            .enumerate()
            .skip(1)
            .find(|(_, byte)| !byte.is_ascii_alphanumeric() && *byte != b'_')
        {
            return Err(CNameError::InvalidContinuation { byte_offset });
        }
        if let Some(keyword) = CKeyword::from_spelling(value) {
            return Err(CNameError::Keyword(keyword));
        }
        // Exclude every leading underscore so allocation is safe at either
        // file or block scope. This does not ban such names in portable input.
        if first == b'_' {
            return Err(CNameError::ImplementationReserved);
        }
        if let Some(name) = CReservedMacro::from_spelling(value) {
            return Err(CNameError::StandardMacro(name));
        }
        Ok(Self(value.to_owned()))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for CNameError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Empty => f.write_str("empty C identifier"),
            Self::InvalidStart => f.write_str("C identifier must start with an ASCII letter"),
            Self::InvalidContinuation { byte_offset } => {
                write!(f, "invalid C identifier byte at offset {byte_offset}")
            }
            Self::Keyword(keyword) => write!(f, "reserved C17 keyword: {}", keyword.as_str()),
            Self::ImplementationReserved => f.write_str("implementation-reserved C identifier"),
            Self::StandardMacro(name) => write!(f, "reserved C macro: {}", name.as_str()),
        }
    }
}

impl std::error::Error for CNameError {}
