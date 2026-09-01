use std::collections::{BTreeMap, BTreeSet};
use std::fmt;

use portable_check::v0::CheckedProgram;

use crate::{
    BackendError, BackendOptions, DeclaredDependency, Document, FinalNewline, InjectedHelper,
    OutputFile, OutputManifest, RenderOptions, render,
};

/// Stable identity for a group of related generated files.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct FileGroupId(String);

impl FileGroupId {
    pub fn parse(value: impl Into<String>) -> Result<Self, LanguagePackageError> {
        let value = value.into();
        if value.is_empty()
            || !value.bytes().all(|byte| {
                byte.is_ascii_lowercase() || byte.is_ascii_digit() || matches!(byte, b'-' | b'.')
            })
            || value.starts_with(['-', '.'])
            || value.ends_with(['-', '.'])
        {
            return Err(LanguagePackageError::InvalidGroupId(value));
        }
        Ok(Self(value))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// Why a file exists in a generated package.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum FileRole {
    Source,
    Runtime,
    Test,
    Conformance,
    NegativeTest,
    Metadata,
    Documentation,
    Asset,
}

/// Target-owned ordering bucket for imports, such as future, standard-library,
/// package, and relative imports.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ImportGroup {
    order: u16,
    name: String,
}

impl ImportGroup {
    pub fn new(order: u16, name: impl Into<String>) -> Result<Self, LanguagePackageError> {
        let name = name.into();
        if name.is_empty()
            || !name
                .bytes()
                .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'-')
        {
            return Err(LanguagePackageError::InvalidImportGroup(name));
        }
        Ok(Self { order, name })
    }

    pub fn order(&self) -> u16 {
        self.order
    }

    pub fn name(&self) -> &str {
        &self.name
    }
}

/// Sorted, deduplicated import requirements collected during target lowering.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ImportSet<I: Ord> {
    groups: BTreeMap<ImportGroup, BTreeSet<I>>,
}

impl<I: Ord> Default for ImportSet<I> {
    fn default() -> Self {
        Self {
            groups: BTreeMap::new(),
        }
    }
}

impl<I: Ord> ImportSet<I> {
    pub fn require(&mut self, group: ImportGroup, import: I) -> bool {
        self.groups.entry(group).or_default().insert(import)
    }

    pub fn is_empty(&self) -> bool {
        self.groups.values().all(BTreeSet::is_empty)
    }

    pub fn len(&self) -> usize {
        self.groups.values().map(BTreeSet::len).sum()
    }

    pub fn groups(&self) -> impl Iterator<Item = (&ImportGroup, &BTreeSet<I>)> {
        self.groups
            .iter()
            .filter(|(_, imports)| !imports.is_empty())
    }
}

impl<I: Clone + Ord> ImportSet<I> {
    fn merge(&mut self, other: &Self) {
        for (group, imports) in &other.groups {
            self.groups
                .entry(group.clone())
                .or_default()
                .extend(imports.iter().cloned());
        }
    }
}

/// One flat target-language mapping after semantic translation.
///
/// A unit couples target syntax to every import/include/use requirement caused
/// by that syntax. Files can only collect imports by accepting units, which
/// prevents import selection from drifting into the renderer or a separate
/// static file-level list.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LanguageUnit<I: Ord> {
    document: Document,
    imports: ImportSet<I>,
}

impl<I: Ord> LanguageUnit<I> {
    pub fn new(document: Document) -> Self {
        Self {
            document,
            imports: ImportSet::default(),
        }
    }

    pub fn set_document(&mut self, document: Document) {
        self.document = document;
    }

    pub fn require_import(&mut self, group: ImportGroup, import: I) -> bool {
        self.imports.require(group, import)
    }

    pub fn imports(&self) -> &ImportSet<I> {
        &self.imports
    }

    fn document(&self) -> &Document {
        &self.document
    }
}

/// A source file after target translation but before syntax rendering.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LanguageSourceFile<I: Ord> {
    path: String,
    role: FileRole,
    preamble: Option<LanguageUnit<I>>,
    body: Option<LanguageUnit<I>>,
    epilogue: Option<LanguageUnit<I>>,
    render_options: RenderOptions,
}

impl<I: Ord> LanguageSourceFile<I> {
    pub fn new(path: impl Into<String>, role: FileRole) -> Self {
        Self {
            path: path.into(),
            role,
            preamble: None,
            body: None,
            epilogue: None,
            render_options: RenderOptions::default(),
        }
    }

    pub fn path(&self) -> &str {
        &self.path
    }

    pub fn role(&self) -> FileRole {
        self.role
    }

    pub fn set_preamble(&mut self, unit: LanguageUnit<I>) {
        self.preamble = Some(unit);
    }

    pub fn set_body(&mut self, unit: LanguageUnit<I>) {
        self.body = Some(unit);
    }

    pub fn set_epilogue(&mut self, unit: LanguageUnit<I>) {
        self.epilogue = Some(unit);
    }

    pub fn set_render_options(&mut self, options: RenderOptions) {
        self.render_options = options;
    }

    fn imports(&self) -> ImportSet<I>
    where
        I: Clone,
    {
        let mut imports = ImportSet::default();
        for unit in self
            .preamble
            .iter()
            .chain(self.body.iter())
            .chain(self.epilogue.iter())
        {
            imports.merge(unit.imports());
        }
        imports
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum LanguageFile<I: Ord> {
    Source(LanguageSourceFile<I>),
    Text {
        path: String,
        role: FileRole,
        contents: String,
    },
    Bytes {
        path: String,
        role: FileRole,
        contents: Vec<u8>,
    },
}

impl<I: Ord> LanguageFile<I> {
    pub fn source(file: LanguageSourceFile<I>) -> Self {
        Self::Source(file)
    }

    pub fn text(path: impl Into<String>, role: FileRole, contents: impl Into<String>) -> Self {
        Self::Text {
            path: path.into(),
            role,
            contents: contents.into(),
        }
    }

    pub fn bytes(path: impl Into<String>, role: FileRole, contents: impl Into<Vec<u8>>) -> Self {
        Self::Bytes {
            path: path.into(),
            role,
            contents: contents.into(),
        }
    }

    pub fn path(&self) -> &str {
        match self {
            Self::Source(file) => file.path(),
            Self::Text { path, .. } | Self::Bytes { path, .. } => path,
        }
    }

    pub fn role(&self) -> FileRole {
        match self {
            Self::Source(file) => file.role(),
            Self::Text { role, .. } | Self::Bytes { role, .. } => *role,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FileGroup<I: Ord> {
    id: FileGroupId,
    files: Vec<LanguageFile<I>>,
}

impl<I: Ord> FileGroup<I> {
    pub fn new(
        id: FileGroupId,
        mut files: Vec<LanguageFile<I>>,
    ) -> Result<Self, LanguagePackageError> {
        if files.is_empty() {
            return Err(LanguagePackageError::EmptyGroup(id));
        }
        files.sort_by(|left, right| left.path().cmp(right.path()));
        Ok(Self { id, files })
    }

    pub fn id(&self) -> &FileGroupId {
        &self.id
    }

    pub fn files(&self) -> &[LanguageFile<I>] {
        &self.files
    }
}

/// Target language package IR. It is still purely in-memory and has no output
/// directory or filesystem authority.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LanguagePackage<I: Ord> {
    groups: Vec<FileGroup<I>>,
    dependencies: Vec<DeclaredDependency>,
    helpers: Vec<InjectedHelper>,
}

impl<I: Ord> LanguagePackage<I> {
    pub fn new(
        mut groups: Vec<FileGroup<I>>,
        mut dependencies: Vec<DeclaredDependency>,
        mut helpers: Vec<InjectedHelper>,
    ) -> Result<Self, LanguagePackageError> {
        groups.sort_by(|left, right| left.id.cmp(&right.id));
        for pair in groups.windows(2) {
            if pair[0].id == pair[1].id {
                return Err(LanguagePackageError::DuplicateGroup(pair[0].id.clone()));
            }
        }
        let mut paths = BTreeMap::<&str, &FileGroupId>::new();
        for group in &groups {
            for file in &group.files {
                if let Some(first_group) = paths.insert(file.path(), &group.id) {
                    return Err(LanguagePackageError::DuplicatePath {
                        path: file.path().to_owned(),
                        first_group: first_group.clone(),
                        second_group: group.id.clone(),
                    });
                }
            }
        }
        dependencies.sort();
        helpers.iter_mut().for_each(|helper| helper.files.sort());
        helpers.sort();
        Ok(Self {
            groups,
            dependencies,
            helpers,
        })
    }

    pub fn groups(&self) -> &[FileGroup<I>] {
        &self.groups
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum LanguagePackageError {
    InvalidGroupId(String),
    InvalidImportGroup(String),
    EmptyGroup(FileGroupId),
    DuplicateGroup(FileGroupId),
    DuplicatePath {
        path: String,
        first_group: FileGroupId,
        second_group: FileGroupId,
    },
    ImportRendering(String),
    DocumentRendering(String),
}

impl fmt::Display for LanguagePackageError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidGroupId(value) => write!(formatter, "invalid file group ID {value:?}"),
            Self::InvalidImportGroup(value) => {
                write!(formatter, "invalid import group name {value:?}")
            }
            Self::EmptyGroup(id) => write!(formatter, "file group {:?} is empty", id.as_str()),
            Self::DuplicateGroup(id) => {
                write!(formatter, "duplicate file group {:?}", id.as_str())
            }
            Self::DuplicatePath {
                path,
                first_group,
                second_group,
            } => write!(
                formatter,
                "duplicate language-package path {path:?} in groups {:?} and {:?}",
                first_group.as_str(),
                second_group.as_str()
            ),
            Self::ImportRendering(message) => {
                write!(formatter, "cannot render language imports: {message}")
            }
            Self::DocumentRendering(message) => {
                write!(formatter, "cannot render language document: {message}")
            }
        }
    }
}

impl std::error::Error for LanguagePackageError {}

/// Syntax-only half of a language plugin. It receives collected target import
/// IR, never checked PolyIR.
///
/// ```compile_fail
/// use portable_check::v0::CheckedProgram;
/// use portable_codegen::{Document, ImportSet, LanguageRenderer};
///
/// struct Renderer;
/// impl LanguageRenderer<String> for Renderer {
///     fn render_imports(&self, imports: &ImportSet<String>) -> Result<Document, String> {
///         let _program: &CheckedProgram = imports;
///         Ok(Document::empty())
///     }
/// }
/// ```
///
/// A source file also cannot receive imports independently from translated
/// syntax:
///
/// ```compile_fail
/// use portable_codegen::{FileRole, ImportGroup, LanguageSourceFile};
///
/// let mut file = LanguageSourceFile::<String>::new("src/example", FileRole::Source);
/// file.require_import(ImportGroup::new(10, "standard").unwrap(), "value".into());
/// ```
pub trait LanguageRenderer<I: Ord> {
    fn render_imports(&self, imports: &ImportSet<I>) -> Result<Document, String>;
}

/// Translation half of a language plugin. This is the only half that receives
/// checked PolyIR.
pub trait LanguagePlugin {
    type Import: Clone + Ord;
    type Renderer: LanguageRenderer<Self::Import>;

    fn translate(
        &self,
        program: &CheckedProgram,
        options: &BackendOptions,
    ) -> Result<LanguagePackage<Self::Import>, BackendError>;

    fn renderer(&self) -> Self::Renderer;
}

pub fn generate_with_plugin<P: LanguagePlugin>(
    plugin: &P,
    program: &CheckedProgram,
    options: &BackendOptions,
) -> Result<OutputManifest, BackendError> {
    let package = plugin.translate(program, options)?;
    render_language_package(&package, &plugin.renderer())
}

pub fn render_language_package<I: Clone + Ord>(
    package: &LanguagePackage<I>,
    renderer: &impl LanguageRenderer<I>,
) -> Result<OutputManifest, BackendError> {
    let mut output = Vec::new();
    for group in &package.groups {
        for file in &group.files {
            match file {
                LanguageFile::Source(file) => {
                    output.push(OutputFile::text(
                        file.path(),
                        render_source(file, renderer)?,
                    ));
                }
                LanguageFile::Text { path, contents, .. } => {
                    output.push(OutputFile::text(path, contents));
                }
                LanguageFile::Bytes { path, contents, .. } => {
                    output.push(OutputFile::bytes(path, contents.clone()));
                }
            }
        }
    }
    OutputManifest::new(
        output,
        package.dependencies.clone(),
        package.helpers.clone(),
    )
    .map_err(BackendError::UnsupportedCapabilities)
}

fn render_source<I: Clone + Ord>(
    file: &LanguageSourceFile<I>,
    renderer: &impl LanguageRenderer<I>,
) -> Result<String, BackendError> {
    let mut sections = Vec::new();
    for unit in file.preamble.iter() {
        sections.push(render_section(unit.document(), file.render_options)?);
    }
    let imports = file.imports();
    if !imports.is_empty() {
        let imports =
            renderer
                .render_imports(&imports)
                .map_err(|message| BackendError::Generation {
                    message: LanguagePackageError::ImportRendering(message).to_string(),
                })?;
        sections.push(render_section(&imports, file.render_options)?);
    }
    for unit in file.body.iter() {
        sections.push(render_section(unit.document(), file.render_options)?);
    }
    for unit in file.epilogue.iter() {
        sections.push(render_section(unit.document(), file.render_options)?);
    }
    sections.retain(|section| !section.is_empty());
    let mut text = sections.join("\n\n");
    match file.render_options.final_newline {
        FinalNewline::Preserve => {}
        FinalNewline::Always => {
            while text.ends_with('\n') {
                text.pop();
            }
            text.push('\n');
        }
        FinalNewline::Never => {
            while text.ends_with('\n') {
                text.pop();
            }
        }
    }
    Ok(text)
}

fn render_section(document: &Document, mut options: RenderOptions) -> Result<String, BackendError> {
    options.final_newline = FinalNewline::Never;
    render(document, options).map_err(|error| BackendError::Generation {
        message: LanguagePackageError::DocumentRendering(error.to_string()).to_string(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::RawText;

    #[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
    struct TestImport(&'static str);

    struct TestRenderer;

    impl LanguageRenderer<TestImport> for TestRenderer {
        fn render_imports(&self, imports: &ImportSet<TestImport>) -> Result<Document, String> {
            let mut lines = Vec::new();
            for (_, group) in imports.groups() {
                lines.extend(group.iter().map(|import| format!("use {};", import.0)));
            }
            Ok(Document::raw_text(RawText::new(lines.join("\n"))))
        }
    }

    fn id(value: &str) -> FileGroupId {
        FileGroupId::parse(value).unwrap()
    }

    fn import_group(order: u16, value: &str) -> ImportGroup {
        ImportGroup::new(order, value).unwrap()
    }

    #[test]
    fn groups_files_and_imports_are_sorted_and_deduplicated() {
        let mut source = LanguageSourceFile::new("src/lib.test", FileRole::Source);
        let mut preamble = LanguageUnit::new(Document::raw_text(RawText::new("// preamble")));
        assert!(preamble.require_import(import_group(10, "standard"), TestImport("zeta")));
        let mut body = LanguageUnit::new(Document::raw_text(RawText::new("body")));
        assert!(body.require_import(import_group(10, "standard"), TestImport("alpha")));
        assert!(!body.require_import(import_group(10, "standard"), TestImport("alpha")));
        let mut epilogue = LanguageUnit::new(Document::empty());
        assert!(epilogue.require_import(import_group(10, "standard"), TestImport("alpha")));
        source.set_preamble(preamble);
        source.set_body(body);
        source.set_epilogue(epilogue);
        let source_group =
            FileGroup::new(id("source"), vec![LanguageFile::source(source)]).unwrap();
        let metadata_group = FileGroup::new(
            id("metadata"),
            vec![LanguageFile::text(
                "project.toml",
                FileRole::Metadata,
                "meta\n",
            )],
        )
        .unwrap();
        let package =
            LanguagePackage::new(vec![source_group, metadata_group], vec![], vec![]).unwrap();
        assert_eq!(package.groups()[0].id().as_str(), "metadata");
        let manifest = render_language_package(&package, &TestRenderer).unwrap();
        assert_eq!(
            manifest.file("src/lib.test").unwrap().contents(),
            &crate::OutputContents::Text("// preamble\n\nuse alpha;\nuse zeta;\n\nbody\n".into())
        );
    }

    #[test]
    fn a_file_without_import_requirements_has_no_import_section() {
        let mut source = LanguageSourceFile::<TestImport>::new("src/empty.test", FileRole::Source);
        source.set_body(LanguageUnit::new(Document::raw_text(RawText::new("body"))));
        let package = LanguagePackage::new(
            vec![FileGroup::new(id("source"), vec![LanguageFile::source(source)]).unwrap()],
            vec![],
            vec![],
        )
        .unwrap();
        let manifest = render_language_package(&package, &TestRenderer).unwrap();
        assert_eq!(
            manifest.file("src/empty.test").unwrap().contents(),
            &crate::OutputContents::Text("body\n".into())
        );
    }

    #[test]
    fn invalid_empty_duplicate_groups_and_duplicate_paths_are_rejected() {
        assert!(FileGroupId::parse("Bad Group").is_err());
        assert!(ImportGroup::new(0, "Bad Group").is_err());
        assert_eq!(
            FileGroup::<TestImport>::new(id("empty"), vec![]).unwrap_err(),
            LanguagePackageError::EmptyGroup(id("empty"))
        );
        let first = FileGroup::<TestImport>::new(
            id("first"),
            vec![LanguageFile::text("same", FileRole::Metadata, "a")],
        )
        .unwrap();
        let second = FileGroup::<TestImport>::new(
            id("second"),
            vec![LanguageFile::text("same", FileRole::Metadata, "b")],
        )
        .unwrap();
        assert!(matches!(
            LanguagePackage::new(vec![first, second], vec![], vec![]),
            Err(LanguagePackageError::DuplicatePath { .. })
        ));
        let one = FileGroup::<TestImport>::new(
            id("same"),
            vec![LanguageFile::text("one", FileRole::Metadata, "a")],
        )
        .unwrap();
        let two = FileGroup::<TestImport>::new(
            id("same"),
            vec![LanguageFile::text("two", FileRole::Metadata, "b")],
        )
        .unwrap();
        assert_eq!(
            LanguagePackage::new(vec![one, two], vec![], vec![]).unwrap_err(),
            LanguagePackageError::DuplicateGroup(id("same"))
        );
    }

    struct BrokenRenderer;

    impl LanguageRenderer<TestImport> for BrokenRenderer {
        fn render_imports(&self, _imports: &ImportSet<TestImport>) -> Result<Document, String> {
            Err("unsupported import".into())
        }
    }

    #[test]
    fn renderer_failures_are_not_hidden() {
        let mut source = LanguageSourceFile::new("src/lib.test", FileRole::Source);
        let mut body = LanguageUnit::new(Document::empty());
        body.require_import(import_group(10, "standard"), TestImport("alpha"));
        source.set_body(body);
        let package = LanguagePackage::new(
            vec![FileGroup::new(id("source"), vec![LanguageFile::source(source)]).unwrap()],
            vec![],
            vec![],
        )
        .unwrap();
        let error = render_language_package(&package, &BrokenRenderer).unwrap_err();
        assert!(matches!(
            error,
            BackendError::Generation { message }
                if message.contains("unsupported import")
        ));
    }
}
