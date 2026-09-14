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
        Self(safe.replace("*/", "* /").replace("/*", "/ *"))
    }

    pub fn text(&self) -> &str {
        &self.0
    }
}

/// Diagnostic bytes plus non-executable printable presentation. The original
/// bytes remain available; unsafe presentation bytes become `[0xNN]`, not C
/// numeric escapes (rejected by the pinned Clang for unevaluated messages).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CAssertDiagnostic {
    bytes: Vec<u8>,
    display: String,
}

impl CAssertDiagnostic {
    pub fn new(bytes: impl Into<Vec<u8>>) -> Self {
        let bytes = bytes.into();
        let mut display = String::new();
        for byte in &bytes {
            match byte {
                b' '..=b'~' if !matches!(byte, b'\\' | b'"' | b'?') => {
                    display.push(char::from(*byte));
                }
                _ => write!(display, "[0x{byte:02X}]").expect("writing to a string cannot fail"),
            }
        }
        Self { bytes, display }
    }
    pub fn bytes(&self) -> &[u8] {
        &self.bytes
    }
    pub fn display_text(&self) -> &str {
        &self.display
    }
}
