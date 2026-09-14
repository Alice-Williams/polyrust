//! One structural traversal, with separate pre-serialization and encoding sinks.
use crate::budget::Budget;
use portable_codegen::RustDeclarationId;
use std::fmt::Write;

pub(crate) trait Sink {
    fn fixed(&mut self, syntax: &'static str) -> Result<(), String>;
    fn string(&mut self, value: &str) -> Result<(), String>;
    fn number(&mut self, value: usize) -> Result<(), String>;
    fn id(&mut self, value: RustDeclarationId) -> Result<(), String>;
}

#[derive(Default)]
pub(crate) struct Reservation(pub(crate) Budget);
impl Sink for Reservation {
    fn fixed(&mut self, syntax: &'static str) -> Result<(), String> {
        self.0.add(syntax.len() as u64)
    }
    fn string(&mut self, value: &str) -> Result<(), String> {
        let bytes = (value.len() as u64)
            .checked_mul(6)
            .and_then(|n| n.checked_add(2))
            .ok_or("JSON string bound overflow")?;
        self.0.add(bytes)
    }
    fn number(&mut self, _: usize) -> Result<(), String> {
        self.0.add(20)
    }
    fn id(&mut self, _: RustDeclarationId) -> Result<(), String> {
        self.0.add(35)
    }
}

pub(crate) struct Encoder {
    output: String,
    limit: u64,
}
impl Encoder {
    pub(crate) fn new(limit: u64) -> Self {
        Self {
            output: String::new(),
            limit,
        }
    }
    fn append(&mut self, value: &str) -> Result<(), String> {
        let next = (self.output.len() as u64)
            .checked_add(value.len() as u64)
            .ok_or("JSON encoding overflow")?;
        if next > self.limit {
            return Err("JSON exceeds reservation".into());
        }
        self.output.push_str(value);
        Ok(())
    }
    pub(crate) fn finish(self) -> String {
        self.output
    }
}
impl Sink for Encoder {
    fn fixed(&mut self, syntax: &'static str) -> Result<(), String> {
        self.append(syntax)
    }
    fn string(&mut self, value: &str) -> Result<(), String> {
        self.append("\"")?;
        for ch in value.chars() {
            match ch {
                '"' => self.append("\\\"")?,
                '\\' => self.append("\\\\")?,
                '\u{0}'..='\u{1f}' => {
                    let mut escape = String::with_capacity(6);
                    write!(escape, "\\u{:04x}", ch as u32).unwrap();
                    self.append(&escape)?;
                }
                _ => {
                    self.append(ch.encode_utf8(&mut [0; 4]))?;
                }
            }
        }
        self.append("\"")
    }
    fn number(&mut self, value: usize) -> Result<(), String> {
        self.append(&value.to_string())
    }
    fn id(&mut self, value: RustDeclarationId) -> Result<(), String> {
        self.append(&format!(
            "\"{:016x}:{:016x}\"",
            value.crate_id, value.definition_path_hash
        ))
    }
}
