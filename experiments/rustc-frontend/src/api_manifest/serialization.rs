//! Deterministic JSON metadata spelling; no target source syntax is emitted.
use super::*;
use std::fmt::Write;

impl ApiManifest {
    pub(crate) fn canonical_json(&self) -> Result<String, String> {
        if !self.imports.is_empty() {
            return Err("standalone schema cannot omit certified imports".into());
        }
        self.encode(false)
    }

    pub(crate) fn bundle_json(&self) -> Result<String, String> {
        if !self.constants.is_empty() {
            return Err("C public constant bundle metadata is not yet admitted".into());
        }
        self.encode(true)
    }

    fn encode(&self, bundle: bool) -> Result<String, String> {
        let bound = self.encoded_bound()?;
        let mut text = format!(
            "{{\"schema_version\":{},\"root\":{},\"header\":{},\"implementation\":{},\"modules\":[",
            if bundle {
                2
            } else if self.constants.is_empty() {
                1
            } else {
                3
            },
            identity(self.exports.root),
            quote(self.header.key().path.as_str()),
            quote(self.implementation.key().path.as_str())
        );
        for (index, (module, bindings)) in self.exports.modules.iter().enumerate() {
            if index != 0 {
                text.push(',');
            }
            write!(text, "{{\"id\":{},\"bindings\":[", identity(*module)).unwrap();
            for (index, (name, target)) in bindings.iter().enumerate() {
                if index != 0 {
                    text.push(',');
                }
                let namespace = match name.namespace {
                    RustExportNamespace::Type => "type",
                    RustExportNamespace::Value => "value",
                    RustExportNamespace::Macro => "macro",
                };
                let (kind, id) = match target {
                    RustExportTarget::Module(id) => ("module", id),
                    RustExportTarget::Declaration(id) if self.constants.contains_key(id) => {
                        ("constant", id)
                    }
                    RustExportTarget::Declaration(id) => ("function", id),
                };
                write!(
                    text,
                    "{{\"namespace\":{},\"name\":{},\"kind\":{},\"target\":{}}}",
                    quote(namespace),
                    quote(&name.name),
                    quote(kind),
                    identity(*id)
                )
                .unwrap();
            }
            text.push_str("]}");
        }
        text.push_str("],\"functions\":[");
        for (index, (id, function)) in self.functions.iter().enumerate() {
            if index != 0 {
                text.push(',');
            }
            let linkage = match function.linkage {
                CLinkage::External => "external",
                CLinkage::Internal => "internal",
                CLinkage::None => return Err("API function cannot have no linkage".into()),
            };
            write!(
                text,
                "{{\"id\":{},\"symbol\":{},\"primary\":{},\"implementation\":{},\"linkage\":{}}}",
                identity(*id),
                quote(function.name.as_str()),
                quote(function.reference.file().key().path.as_str()),
                quote(function.implementation.key().path.as_str()),
                quote(linkage)
            )
            .unwrap();
        }
        text.push(']');
        if !self.constants.is_empty() {
            text.push_str(",\"constants\":[");
            for (index, (id, constant)) in self.constants.iter().enumerate() {
                if index != 0 {
                    text.push(',');
                }
                let (ty, value) = constants::scalar(&constant.value)?;
                write!(text, "{{\"id\":{},\"symbol\":{},\"primary\":{},\"implementation\":{},\"type\":{},\"value\":{},\"readonly\":true}}",
                    identity(*id), quote(constant.name.as_str()), quote(self.header.key().path.as_str()),
                    quote(self.implementation.key().path.as_str()), quote(ty), value).unwrap();
            }
            text.push(']');
        }
        if bundle {
            self.write_imports(&mut text)?;
        }
        text.push_str("}\n");
        if text.len() > bound {
            return Err("API metadata exceeded its byte estimate".into());
        }
        Ok(text)
    }

    pub(super) fn encoded_bound(&self) -> Result<usize, String> {
        let mut bytes = 512usize;
        let mut add = |value: usize| -> Result<(), String> {
            bytes = bytes
                .checked_add(value)
                .ok_or("API metadata byte overflow")?;
            if bytes > 8 * 1024 * 1024 {
                return Err("API metadata exceeds the 8 MiB policy".into());
            }
            Ok(())
        };
        for bindings in self.exports.modules.values() {
            add(128)?;
            for name in bindings.keys() {
                add(name
                    .name
                    .len()
                    .checked_mul(6)
                    .and_then(|value| value.checked_add(256))
                    .ok_or("API metadata byte overflow")?)?;
            }
        }
        for function in self.functions.values() {
            add(512 + function.name.as_str().len())?;
        }
        for constant in self.constants.values() {
            add(512 + constant.name.as_str().len())?;
        }
        for bytes in self.import_bounds() {
            add(bytes?)?;
        }
        Ok(bytes)
    }
}

pub(super) fn identity(id: RustDeclarationId) -> String {
    format!("\"{:016x}:{:016x}\"", id.crate_id, id.definition_path_hash)
}

pub(super) fn quote(value: &str) -> String {
    let mut text = String::from("\"");
    for character in value.chars() {
        match character {
            '"' => text.push_str("\\\""),
            '\\' => text.push_str("\\\\"),
            '\0'..='\u{1f}' => write!(text, "\\u{:04x}", character as u32).unwrap(),
            value => text.push(value),
        }
    }
    text.push('"');
    text
}
