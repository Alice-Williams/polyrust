//! Non-executable Java documentation payloads, not preformatted source code.
use std::fmt::Write;

/// Attribute text encoded for a structurally delimited Java documentation comment.
/// Construction does not authenticate the attribute's source or certify a file.
///
/// Unencoded payload construction is not a public API:
/// ```compile_fail
/// use portable_backend_java::ast::JavaDocComment;
/// let unsafe_payload = JavaDocComment(String::from("*/"));
/// ```
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct JavaDocComment(String);

impl JavaDocComment {
    pub fn new(attribute: &str) -> Self {
        let normalized = attribute.replace("\r\n", "\n").replace('\r', "\n");
        let mut encoded = String::new();
        for character in normalized.chars() {
            match character {
                '\n' => encoded.push('\n'),
                '&' => encoded.push_str("&amp;"),
                '<' => encoded.push_str("&lt;"),
                '>' => encoded.push_str("&gt;"),
                '\\' => encoded.push_str("&#92;"),
                '*' => encoded.push_str("&#42;"),
                '@' => encoded.push_str("&#64;"),
                value
                    if value.is_control()
                        || matches!(value, '\u{2028}'..='\u{202e}' | '\u{2066}'..='\u{2069}') =>
                {
                    write!(encoded, "&#x{:X};", u32::from(value))
                        .expect("writing to a String cannot fail");
                }
                value => encoded.push(value),
            }
        }
        Self(encoded)
    }

    pub fn text(&self) -> &str {
        &self.0
    }

    pub fn encoded_len(&self) -> usize {
        self.0.len()
    }
}

#[cfg(test)]
#[path = "../tests/doc_comments.rs"]
mod tests;
