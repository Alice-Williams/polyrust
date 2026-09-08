//! Non-executable documentation, normalized before it reaches a renderer.

use std::fmt::Write;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CComment(String);

impl CComment {
    pub fn new(text: &str) -> Self {
        let normalized = text.replace("\r\n", "\n").replace('\r', "\n");
        let mut safe = String::new();
        for byte in normalized.bytes() {
            match byte {
                b'\n' => safe.push('\n'),
                b' '..=b'~' if byte != b'\\' && byte != b'?' => safe.push(char::from(byte)),
                _ => write!(safe, "[0x{byte:02X}]").expect("writing to a string cannot fail"),
            }
        }
        Self(safe.replace("*/", "* /"))
    }

    pub fn text(&self) -> &str {
        &self.0
    }
}

/// Diagnostic bytes, never source-string tokens. The renderer owns quoting
/// and exact byte escapes, including NUL, quotes, trigraphs and backslashes.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CAssertDiagnostic(Vec<u8>);

impl CAssertDiagnostic {
    pub fn new(bytes: impl Into<Vec<u8>>) -> Self {
        Self(bytes.into())
    }
    pub fn bytes(&self) -> &[u8] {
        &self.0
    }
}
