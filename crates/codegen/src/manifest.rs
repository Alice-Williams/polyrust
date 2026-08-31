use std::collections::{BTreeMap, BTreeSet};

use portable_diagnostics::{Diagnostic, DiagnosticCode, SourceRef};
use serde::Serialize;

const RESERVED_PATHS: [&str; 2] = [".polyrust-manifest.json", ".polyrust/manifest.json"];

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(tag = "kind", content = "data", rename_all = "snake_case")]
pub enum OutputContents {
    Text(String),
    Bytes(Vec<u8>),
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct OutputFile {
    path: String,
    contents: OutputContents,
}

impl OutputFile {
    pub fn text(path: impl Into<String>, contents: impl Into<String>) -> Self {
        Self {
            path: path.into(),
            contents: OutputContents::Text(contents.into()),
        }
    }

    pub fn bytes(path: impl Into<String>, contents: impl Into<Vec<u8>>) -> Self {
        Self {
            path: path.into(),
            contents: OutputContents::Bytes(contents.into()),
        }
    }

    pub fn path(&self) -> &str {
        &self.path
    }

    pub fn contents(&self) -> &OutputContents {
        &self.contents
    }
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize)]
pub struct DeclaredDependency {
    pub ecosystem: String,
    pub name: String,
    pub requirement: String,
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize)]
pub struct InjectedHelper {
    pub id: String,
    pub capability: String,
    pub files: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct OutputManifest {
    files: Vec<OutputFile>,
    dependencies: Vec<DeclaredDependency>,
    helpers: Vec<InjectedHelper>,
}

impl OutputManifest {
    /// Builds and validates a complete in-memory artifact tree.
    ///
    /// ```
    /// use portable_codegen::{OutputFile, OutputManifest};
    ///
    /// let manifest = OutputManifest::new(
    ///     vec![OutputFile::text("src/generated.rs", "pub const OK: bool = true;\n")],
    ///     vec![],
    ///     vec![],
    /// )?;
    /// assert_eq!(manifest.files()[0].path(), "src/generated.rs");
    /// # Ok::<(), Vec<portable_diagnostics::Diagnostic>>(())
    /// ```
    pub fn new(
        mut files: Vec<OutputFile>,
        mut dependencies: Vec<DeclaredDependency>,
        mut helpers: Vec<InjectedHelper>,
    ) -> Result<Self, Vec<Diagnostic>> {
        let mut diagnostics = validate_paths(files.iter().map(OutputFile::path));
        for helper in &helpers {
            diagnostics.extend(validate_paths(helper.files.iter().map(String::as_str)));
        }
        portable_diagnostics::sort_diagnostics(&mut diagnostics);
        if !diagnostics.is_empty() {
            return Err(diagnostics);
        }
        files.sort_by(|left, right| left.path.cmp(&right.path));
        dependencies.sort();
        helpers.iter_mut().for_each(|helper| helper.files.sort());
        helpers.sort();
        Ok(Self {
            files,
            dependencies,
            helpers,
        })
    }

    pub fn files(&self) -> &[OutputFile] {
        &self.files
    }

    pub fn dependencies(&self) -> &[DeclaredDependency] {
        &self.dependencies
    }

    pub fn helpers(&self) -> &[InjectedHelper] {
        &self.helpers
    }

    pub fn file(&self, path: &str) -> Option<&OutputFile> {
        self.files
            .binary_search_by_key(&path, |file| file.path.as_str())
            .ok()
            .map(|index| &self.files[index])
    }

    pub fn canonical_json(&self) -> String {
        serde_json::to_string(self).expect("manifest model always serializes")
    }
}

fn validate_paths<'a>(paths: impl Iterator<Item = &'a str>) -> Vec<Diagnostic> {
    let mut diagnostics = Vec::new();
    let mut exact = BTreeSet::new();
    let mut folded = BTreeMap::<String, String>::new();
    for path in paths {
        if let Some(reason) = invalid_path_reason(path) {
            diagnostics.push(path_diagnostic(path, reason));
            continue;
        }
        if !exact.insert(path.to_owned()) {
            diagnostics.push(path_diagnostic(path, "duplicate output path"));
        }
        let case_folded = path.to_lowercase();
        if let Some(first) = folded.insert(case_folded, path.to_owned())
            && first != path
        {
            diagnostics.push(path_diagnostic(
                path,
                format!("case-fold collision with {first:?}"),
            ));
        }
    }
    diagnostics
}

fn invalid_path_reason(path: &str) -> Option<String> {
    if path.is_empty() {
        return Some("path is empty".into());
    }
    if path.starts_with('/') || path.starts_with('\\') {
        return Some("path is rooted".into());
    }
    if path.contains('\\') {
        return Some("backslash separators are not normalized".into());
    }
    if path
        .bytes()
        .any(|byte| byte == 0 || byte < 0x20 || byte == 0x7f)
    {
        return Some("path contains control text".into());
    }
    if path.len() >= 2 && path.as_bytes()[0].is_ascii_alphabetic() && path.as_bytes()[1] == b':' {
        return Some("path has a Windows drive prefix".into());
    }
    let segments: Vec<&str> = path.split('/').collect();
    if segments.iter().any(|segment| segment.is_empty()) {
        return Some("path contains an empty segment".into());
    }
    if segments
        .iter()
        .any(|segment| *segment == "." || *segment == "..")
    {
        return Some("path contains a traversal segment".into());
    }
    if segments
        .iter()
        .any(|segment| segment.ends_with(' ') || segment.ends_with('.'))
    {
        return Some("path has a Windows-normalized trailing character".into());
    }
    if segments.iter().any(|segment| windows_reserved(segment)) {
        return Some("path contains a Windows reserved device name".into());
    }
    let folded = path.to_lowercase();
    if RESERVED_PATHS.contains(&folded.as_str()) || folded.starts_with(".polyrust/") {
        return Some("path is reserved for PolyRust metadata".into());
    }
    None
}

fn windows_reserved(segment: &str) -> bool {
    let stem = segment
        .split('.')
        .next()
        .unwrap_or_default()
        .to_ascii_lowercase();
    matches!(stem.as_str(), "con" | "prn" | "aux" | "nul")
        || stem
            .strip_prefix("com")
            .or_else(|| stem.strip_prefix("lpt"))
            .is_some_and(|number| {
                matches!(number, "1" | "2" | "3" | "4" | "5" | "6" | "7" | "8" | "9")
            })
}

fn path_diagnostic(path: &str, reason: impl Into<String>) -> Diagnostic {
    Diagnostic::error(
        DiagnosticCode::UnsafeOutputPath,
        format!("unsafe output path {path:?}: {}", reason.into()),
        SourceRef::logical(["output_manifest", path]),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn valid_manifest_is_sorted_and_serializes_deterministically() {
        let manifest = OutputManifest::new(
            vec![
                OutputFile::bytes("z/data.bin", [0, 1, 255]),
                OutputFile::text("a/source.rs", "fn main() {}\n"),
            ],
            vec![
                DeclaredDependency {
                    ecosystem: "cargo".into(),
                    name: "zeta".into(),
                    requirement: "1".into(),
                },
                DeclaredDependency {
                    ecosystem: "cargo".into(),
                    name: "alpha".into(),
                    requirement: "1".into(),
                },
            ],
            vec![InjectedHelper {
                id: "unicode".into(),
                capability: "UnicodeScalar".into(),
                files: vec!["z/data.bin".into(), "a/source.rs".into()],
            }],
        )
        .unwrap();
        assert_eq!(manifest.files()[0].path(), "a/source.rs");
        assert_eq!(manifest.dependencies()[0].name, "alpha");
        assert_eq!(manifest.helpers()[0].files[0], "a/source.rs");
        assert_eq!(manifest.canonical_json(), manifest.canonical_json());
        assert!(manifest.file("z/data.bin").is_some());
    }

    #[test]
    fn malicious_and_ambiguous_path_corpus_is_rejected() {
        let corpus = [
            "",
            "../escape",
            "a/../escape",
            "./source.rs",
            "/rooted",
            "C:/drive",
            r"C:\drive",
            r"\\server\share",
            r"a\b",
            "a//b",
            "a/CON.txt",
            "a/lpt9",
            "a/trailing.",
            "a/trailing ",
            ".polyrust-manifest.json",
            ".polyrust/private",
            "a/\u{0}b",
            "a/\u{1f}b",
        ];
        for path in corpus {
            let diagnostics =
                OutputManifest::new(vec![OutputFile::text(path, "x")], vec![], vec![]).unwrap_err();
            assert_eq!(
                diagnostics[0].code,
                DiagnosticCode::UnsafeOutputPath,
                "{path:?}"
            );
        }
    }

    #[test]
    fn duplicates_ascii_case_and_unicode_case_collisions_are_rejected() {
        for paths in [
            vec!["a.rs", "a.rs"],
            vec!["Readme.md", "README.md"],
            vec!["unicode/É.txt", "unicode/é.txt"],
        ] {
            let files = paths
                .into_iter()
                .map(|path| OutputFile::text(path, "x"))
                .collect();
            assert!(OutputManifest::new(files, vec![], vec![]).is_err());
        }
    }

    #[test]
    fn manifest_construction_is_in_memory_only() {
        let sentinel = ".polyrust-m08-must-not-exist";
        assert!(!std::path::Path::new(sentinel).exists());
        let _manifest = OutputManifest::new(
            vec![OutputFile::text(sentinel, "not written")],
            vec![],
            vec![],
        )
        .unwrap();
        assert!(!std::path::Path::new(sentinel).exists());
    }
}
