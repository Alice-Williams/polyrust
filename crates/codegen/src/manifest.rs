use std::collections::{BTreeMap, BTreeSet};

use portable_diagnostics::{Diagnostic, DiagnosticCode, SourceRef};
use serde::Serialize;

const RESERVED_PATHS: [&str; 2] = [".polyrust-manifest.json", ".polyrust/manifest.json"];
const MAX_MANIFEST_FILES: usize = 4_096;
const MAX_OUTPUT_FILE_BYTES: usize = 8 * 1024 * 1024;
const MAX_OUTPUT_PACKAGE_BYTES: usize = 64 * 1024 * 1024;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum OutputFileRole {
    PublicApiSource,
    ImplementationSource,
    RuntimeSource,
    NativeTestSource,
    ConformanceSource,
    NegativeTestSource,
    Metadata,
    Documentation,
    Asset,
    DerivedJavaScript,
    LegacyUnclassified,
}

impl OutputFileRole {
    pub(crate) const fn from_source_role(role: crate::SourceRole) -> Self {
        match role {
            crate::SourceRole::PublicApi => Self::PublicApiSource,
            crate::SourceRole::Implementation => Self::ImplementationSource,
            crate::SourceRole::Runtime => Self::RuntimeSource,
            crate::SourceRole::NativeTest => Self::NativeTestSource,
            crate::SourceRole::Conformance => Self::ConformanceSource,
            crate::SourceRole::NegativeTest => Self::NegativeTestSource,
        }
    }

    const fn expected_media(self) -> Option<OutputMediaType> {
        match self {
            Self::Asset => Some(OutputMediaType::Binary),
            Self::LegacyUnclassified => None,
            Self::PublicApiSource
            | Self::ImplementationSource
            | Self::RuntimeSource
            | Self::NativeTestSource
            | Self::ConformanceSource
            | Self::NegativeTestSource
            | Self::Metadata
            | Self::Documentation
            | Self::DerivedJavaScript => Some(OutputMediaType::Utf8Text),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum OutputMediaType {
    Utf8Text,
    Binary,
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize)]
pub struct ContentHash(String);

impl ContentHash {
    pub fn as_str(&self) -> &str {
        &self.0
    }

    fn of(contents: &OutputContents) -> Self {
        let bytes = match contents {
            OutputContents::Text(text) => text.as_bytes(),
            OutputContents::Bytes(bytes) => bytes,
        };
        Self::of_bytes(bytes)
    }

    fn of_bytes(bytes: &[u8]) -> Self {
        let mut value = 0xcbf2_9ce4_8422_2325_u64;
        for byte in bytes {
            value ^= u64::from(*byte);
            value = value.wrapping_mul(0x0000_0100_0000_01b3);
        }
        Self(format!("fnv1a64:{value:016x}"))
    }
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize)]
#[serde(tag = "kind", content = "value", rename_all = "snake_case")]
pub enum ManifestOptionValue {
    Boolean(bool),
    Integer(i64),
    Text(String),
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct ManifestGeneration {
    target: String,
    backend_version: String,
    ir_version: String,
    options: BTreeMap<String, ManifestOptionValue>,
}

impl ManifestGeneration {
    pub fn target(&self) -> &str {
        &self.target
    }

    pub fn backend_version(&self) -> &str {
        &self.backend_version
    }

    pub fn ir_version(&self) -> &str {
        &self.ir_version
    }

    pub fn options(&self) -> &BTreeMap<String, ManifestOptionValue> {
        &self.options
    }

    pub(crate) fn new(
        descriptor: &crate::BackendDescriptor,
        ir_version: portable_ir::v0::IrVersion,
        options: &crate::BackendOptions,
    ) -> Self {
        let version = descriptor.backend_version;
        Self {
            target: descriptor.target.to_string(),
            backend_version: format!("{}.{}.{}", version.major, version.minor, version.patch),
            ir_version: ir_version.to_string(),
            options: options
                .iter()
                .map(|(name, value)| {
                    let value = match value {
                        crate::OptionValue::Boolean(value) => ManifestOptionValue::Boolean(*value),
                        crate::OptionValue::Integer(value) => ManifestOptionValue::Integer(*value),
                        crate::OptionValue::Text(value) => ManifestOptionValue::Text(value.clone()),
                    };
                    (name.to_owned(), value)
                })
                .collect(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(tag = "kind", content = "data", rename_all = "snake_case")]
pub enum OutputContents {
    Text(String),
    Bytes(Vec<u8>),
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct OutputFile {
    path: String,
    role: OutputFileRole,
    media_type: OutputMediaType,
    executable: bool,
    content_hash: ContentHash,
    contents: OutputContents,
}

impl OutputFile {
    pub fn text(path: impl Into<String>, contents: impl Into<String>) -> Self {
        Self::classified_text(path, OutputFileRole::LegacyUnclassified, contents)
    }

    pub fn bytes(path: impl Into<String>, contents: impl Into<Vec<u8>>) -> Self {
        Self::classified_bytes(path, OutputFileRole::LegacyUnclassified, contents)
    }

    pub(crate) fn classified_text(
        path: impl Into<String>,
        role: OutputFileRole,
        contents: impl Into<String>,
    ) -> Self {
        Self::classified(path.into(), role, OutputContents::Text(contents.into()))
    }

    pub(crate) fn classified_bytes(
        path: impl Into<String>,
        role: OutputFileRole,
        contents: impl Into<Vec<u8>>,
    ) -> Self {
        Self::classified(path.into(), role, OutputContents::Bytes(contents.into()))
    }

    fn classified(path: String, role: OutputFileRole, contents: OutputContents) -> Self {
        let media_type = match &contents {
            OutputContents::Text(_) => OutputMediaType::Utf8Text,
            OutputContents::Bytes(_) => OutputMediaType::Binary,
        };
        let content_hash = ContentHash::of(&contents);
        Self {
            path,
            role,
            media_type,
            executable: false,
            content_hash,
            contents,
        }
    }

    pub fn path(&self) -> &str {
        &self.path
    }

    pub fn contents(&self) -> &OutputContents {
        &self.contents
    }

    pub const fn role(&self) -> OutputFileRole {
        self.role
    }

    pub const fn media_type(&self) -> OutputMediaType {
        self.media_type
    }

    pub const fn executable(&self) -> bool {
        self.executable
    }

    pub fn content_hash(&self) -> &ContentHash {
        &self.content_hash
    }
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize)]
pub struct DeclaredDependency {
    pub ecosystem: String,
    pub name: String,
    pub requirement: String,
    pub features: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize)]
pub struct InjectedHelper {
    pub id: String,
    pub capability: String,
    pub files: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct OutputManifest {
    schema_version: u16,
    generation: Option<ManifestGeneration>,
    files: Vec<OutputFile>,
    dependencies: Vec<DeclaredDependency>,
    helpers: Vec<InjectedHelper>,
}

impl OutputManifest {
    /// Manifest construction is sealed behind the checked compiler adapter.
    ///
    /// ```compile_fail
    /// use portable_codegen::{OutputFile, OutputManifest};
    ///
    /// let _manifest = OutputManifest::new(
    ///     vec![OutputFile::text("src/generated.rs", "pub const OK: bool = true;\n")],
    ///     vec![],
    ///     vec![],
    /// );
    /// ```
    pub(crate) fn new(
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
            schema_version: 1,
            generation: None,
            files,
            dependencies,
            helpers,
        })
    }

    pub(crate) fn new_typed(
        generation: ManifestGeneration,
        files: Vec<OutputFile>,
        dependencies: Vec<DeclaredDependency>,
        helpers: Vec<InjectedHelper>,
    ) -> Result<Self, Vec<Diagnostic>> {
        let diagnostics = validate_typed_manifest(&files, &dependencies, &helpers);
        if !diagnostics.is_empty() {
            return Err(diagnostics);
        }
        Ok(Self {
            schema_version: 2,
            generation: Some(generation),
            files,
            dependencies,
            helpers,
        })
    }

    pub const fn schema_version(&self) -> u16 {
        self.schema_version
    }

    pub fn generation(&self) -> Option<&ManifestGeneration> {
        self.generation.as_ref()
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

    pub fn stable_hash(&self) -> ContentHash {
        ContentHash::of_bytes(self.canonical_json().as_bytes())
    }
}

fn validate_typed_manifest(
    files: &[OutputFile],
    dependencies: &[DeclaredDependency],
    helpers: &[InjectedHelper],
) -> Vec<Diagnostic> {
    let mut diagnostics = validate_paths(files.iter().map(OutputFile::path));
    if files.is_empty() {
        diagnostics.push(manifest_diagnostic("typed manifest contains no files"));
    }
    if files.len() > MAX_MANIFEST_FILES {
        diagnostics.push(manifest_diagnostic(format!(
            "typed manifest exceeds the {MAX_MANIFEST_FILES}-file limit"
        )));
    }
    if !strictly_increasing(files.iter().map(OutputFile::path)) {
        diagnostics.push(manifest_diagnostic(
            "typed manifest files are not in strict deterministic path order",
        ));
    }

    let mut package_bytes = 0usize;
    for file in files {
        let file_bytes = match file.contents() {
            OutputContents::Text(text) => text.len(),
            OutputContents::Bytes(bytes) => bytes.len(),
        };
        package_bytes = package_bytes.saturating_add(file_bytes);
        if file_bytes > MAX_OUTPUT_FILE_BYTES {
            diagnostics.push(file_diagnostic(
                file,
                format!("file exceeds the {MAX_OUTPUT_FILE_BYTES}-byte limit"),
            ));
        }
        if file.role == OutputFileRole::LegacyUnclassified {
            diagnostics.push(file_diagnostic(
                file,
                "typed manifest cannot contain a legacy-unclassified file",
            ));
        }
        if file.role.expected_media() != Some(file.media_type) {
            diagnostics.push(file_diagnostic(
                file,
                "file role and media type do not agree",
            ));
        }
        if file.executable {
            diagnostics.push(file_diagnostic(
                file,
                "generated output files cannot request an executable bit",
            ));
        }
        if file.content_hash != ContentHash::of(&file.contents) {
            diagnostics.push(file_diagnostic(
                file,
                "stored content hash does not match bytes",
            ));
        }
        if let OutputContents::Text(text) = &file.contents
            && (text.contains('\r')
                || text.contains('\0')
                || !text.ends_with('\n')
                || text.ends_with("\n\n"))
        {
            diagnostics.push(file_diagnostic(
                file,
                "typed text must be canonical UTF-8 with LF and exactly one final newline",
            ));
        }
    }
    if package_bytes > MAX_OUTPUT_PACKAGE_BYTES {
        diagnostics.push(manifest_diagnostic(format!(
            "typed manifest exceeds the {MAX_OUTPUT_PACKAGE_BYTES}-byte package limit"
        )));
    }

    if !strictly_increasing(dependencies.iter()) {
        diagnostics.push(manifest_diagnostic(
            "declared dependencies are not in strict deterministic order",
        ));
    }
    let mut dependency_names = BTreeSet::new();
    for dependency in dependencies {
        if !valid_manifest_atom(&dependency.ecosystem)
            || !valid_manifest_atom(&dependency.name)
            || !valid_manifest_atom(&dependency.requirement)
            || dependency
                .features
                .iter()
                .any(|feature| !valid_manifest_atom(feature))
        {
            diagnostics.push(manifest_diagnostic(
                "declared dependency contains an empty or control-bearing field",
            ));
        }
        if !strictly_increasing(dependency.features.iter()) {
            diagnostics.push(manifest_diagnostic(format!(
                "declared dependency {:?}/{:?} has a non-deterministic feature set",
                dependency.ecosystem, dependency.name
            )));
        }
        if !dependency_names.insert((&dependency.ecosystem, &dependency.name)) {
            diagnostics.push(manifest_diagnostic(format!(
                "declared dependency {:?}/{:?} appears more than once",
                dependency.ecosystem, dependency.name
            )));
        }
    }

    if !strictly_increasing(helpers.iter()) {
        diagnostics.push(manifest_diagnostic(
            "injected helper reports are not in strict deterministic order",
        ));
    }
    let output_paths = files.iter().map(OutputFile::path).collect::<BTreeSet<_>>();
    let mut helper_ids = BTreeSet::new();
    for helper in helpers {
        if !valid_manifest_atom(&helper.id) || !valid_manifest_atom(&helper.capability) {
            diagnostics.push(manifest_diagnostic(
                "injected helper contains an empty or control-bearing identity",
            ));
        }
        if !helper_ids.insert(&helper.id) {
            diagnostics.push(manifest_diagnostic(format!(
                "injected helper {:?} appears more than once",
                helper.id
            )));
        }
        if helper.files.is_empty() || !strictly_increasing(helper.files.iter()) {
            diagnostics.push(manifest_diagnostic(format!(
                "injected helper {:?} has an empty or non-deterministic file set",
                helper.id
            )));
        }
        diagnostics.extend(validate_paths(helper.files.iter().map(String::as_str)));
        for path in &helper.files {
            if !output_paths.contains(path.as_str()) {
                diagnostics.push(manifest_diagnostic(format!(
                    "injected helper {:?} refers to absent output path {path:?}",
                    helper.id
                )));
            }
        }
    }

    portable_diagnostics::sort_diagnostics(&mut diagnostics);
    diagnostics
}

fn strictly_increasing<T: Ord>(values: impl Iterator<Item = T>) -> bool {
    let mut previous = None;
    for value in values {
        if previous.as_ref().is_some_and(|previous| previous >= &value) {
            return false;
        }
        previous = Some(value);
    }
    true
}

fn valid_manifest_atom(value: &str) -> bool {
    !value.is_empty()
        && !value
            .bytes()
            .any(|byte| byte == 0 || byte < 0x20 || byte == 0x7f)
}

fn manifest_diagnostic(message: impl Into<String>) -> Diagnostic {
    Diagnostic::error(
        DiagnosticCode::InvalidStructure,
        message,
        SourceRef::logical(["typed_output_manifest"]),
    )
}

fn file_diagnostic(file: &OutputFile, message: impl Into<String>) -> Diagnostic {
    Diagnostic::error(
        DiagnosticCode::InvalidStructure,
        format!("invalid output file {:?}: {}", file.path, message.into()),
        SourceRef::logical(["typed_output_manifest", file.path.as_str()]),
    )
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

    fn typed_generation() -> ManifestGeneration {
        ManifestGeneration::new(
            &crate::BackendDescriptor {
                target: crate::TargetId::parse("org.polyrust.manifest-test").unwrap(),
                display_name: "Manifest test".to_owned(),
                backend_version: crate::BackendVersion::new(1, 2, 3),
                supported_ir: crate::IrVersionRange::exact(portable_ir::v0::IrVersion::CURRENT),
            },
            portable_ir::v0::IrVersion::CURRENT,
            &crate::BackendOptions::new(BTreeMap::from([
                ("enabled".to_owned(), crate::OptionValue::Boolean(true)),
                ("limit".to_owned(), crate::OptionValue::Integer(4)),
                (
                    "style".to_owned(),
                    crate::OptionValue::Text("strict".to_owned()),
                ),
            ])),
        )
    }

    fn typed_manifest(
        files: Vec<OutputFile>,
        dependencies: Vec<DeclaredDependency>,
        helpers: Vec<InjectedHelper>,
    ) -> Result<OutputManifest, Vec<Diagnostic>> {
        OutputManifest::new_typed(typed_generation(), files, dependencies, helpers)
    }

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
                    features: vec![],
                },
                DeclaredDependency {
                    ecosystem: "cargo".into(),
                    name: "alpha".into(),
                    requirement: "1".into(),
                    features: vec![],
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
    fn typed_manifest_records_identity_roles_media_hashes_and_options() {
        let manifest = typed_manifest(
            vec![
                OutputFile::classified_text(
                    "a/source.rs",
                    OutputFileRole::PublicApiSource,
                    "pub struct Value;\n",
                ),
                OutputFile::classified_bytes("z/data.bin", OutputFileRole::Asset, [0, 1, 255]),
            ],
            vec![],
            vec![],
        )
        .unwrap();
        assert_eq!(manifest.schema_version(), 2);
        let generation = manifest.generation().unwrap();
        assert_eq!(generation.target(), "org.polyrust.manifest-test");
        assert_eq!(generation.backend_version(), "1.2.3");
        assert_eq!(generation.ir_version(), "0.2.0");
        assert_eq!(
            generation.options().get("enabled"),
            Some(&ManifestOptionValue::Boolean(true))
        );
        assert_eq!(manifest.files()[0].role(), OutputFileRole::PublicApiSource);
        assert_eq!(manifest.files()[0].media_type(), OutputMediaType::Utf8Text);
        assert_eq!(manifest.files()[1].media_type(), OutputMediaType::Binary);
        assert!(!manifest.files()[0].executable());
        assert_eq!(
            manifest.files()[0].content_hash(),
            &ContentHash::of(manifest.files()[0].contents())
        );
        assert_eq!(manifest.canonical_json(), manifest.canonical_json());
    }

    #[test]
    fn typed_manifest_rejects_role_media_executable_hash_and_text_faults() {
        let valid = || {
            OutputFile::classified_text(
                "src/generated.rs",
                OutputFileRole::ImplementationSource,
                "pub fn generated() {}\n",
            )
        };

        let mut wrong_media = valid();
        wrong_media.role = OutputFileRole::Asset;
        assert!(typed_manifest(vec![wrong_media], vec![], vec![]).is_err());

        let mut executable = valid();
        executable.executable = true;
        assert!(typed_manifest(vec![executable], vec![], vec![]).is_err());

        let mut wrong_hash = valid();
        wrong_hash.content_hash = ContentHash("fnv1a64:0000000000000000".to_owned());
        assert!(typed_manifest(vec![wrong_hash], vec![], vec![]).is_err());

        for text in ["missing newline", "two newlines\n\n", "crlf\r\n", "nul\0\n"] {
            let file = OutputFile::classified_text(
                "src/generated.rs",
                OutputFileRole::ImplementationSource,
                text,
            );
            assert!(typed_manifest(vec![file], vec![], vec![]).is_err());
        }

        assert!(
            typed_manifest(
                vec![OutputFile::text("legacy.txt", "legacy")],
                vec![],
                vec![]
            )
            .is_err()
        );
    }

    #[test]
    fn typed_manifest_rejects_order_count_size_dependency_and_helper_faults() {
        let source = |path: String| {
            OutputFile::classified_text(path, OutputFileRole::ImplementationSource, "source\n")
        };
        assert!(
            typed_manifest(
                vec![source("z.rs".to_owned()), source("a.rs".to_owned())],
                vec![],
                vec![],
            )
            .is_err()
        );

        let too_many = (0..=MAX_MANIFEST_FILES)
            .map(|index| source(format!("src/{index:04}.rs")))
            .collect();
        assert!(typed_manifest(too_many, vec![], vec![]).is_err());

        let oversized = OutputFile::classified_bytes(
            "asset.bin",
            OutputFileRole::Asset,
            vec![0; MAX_OUTPUT_FILE_BYTES + 1],
        );
        assert!(typed_manifest(vec![oversized], vec![], vec![]).is_err());

        let dependency = DeclaredDependency {
            ecosystem: "cargo".to_owned(),
            name: "runtime".to_owned(),
            requirement: "1".to_owned(),
            features: vec![],
        };
        assert!(
            typed_manifest(
                vec![source("src/lib.rs".to_owned())],
                vec![dependency.clone(), dependency],
                vec![],
            )
            .is_err()
        );

        assert!(
            typed_manifest(
                vec![source("src/lib.rs".to_owned())],
                vec![DeclaredDependency {
                    ecosystem: "cargo".to_owned(),
                    name: "runtime".to_owned(),
                    requirement: "1".to_owned(),
                    features: vec!["z".to_owned(), "a".to_owned()],
                }],
                vec![],
            )
            .is_err()
        );

        assert!(
            typed_manifest(
                vec![source("src/lib.rs".to_owned())],
                vec![],
                vec![InjectedHelper {
                    id: "unicode".to_owned(),
                    capability: "unicode_scalar".to_owned(),
                    files: vec!["src/missing.rs".to_owned()],
                }],
            )
            .is_err()
        );
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
