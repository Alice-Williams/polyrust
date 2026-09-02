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

/// Roles whose contents must be assembled from closed target-language units.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum SourceFileRole {
    Source,
    Runtime,
    Test,
    Conformance,
    NegativeTest,
}

impl From<SourceFileRole> for FileRole {
    fn from(role: SourceFileRole) -> Self {
        match role {
            SourceFileRole::Source => Self::Source,
            SourceFileRole::Runtime => Self::Runtime,
            SourceFileRole::Test => Self::Test,
            SourceFileRole::Conformance => Self::Conformance,
            SourceFileRole::NegativeTest => Self::NegativeTest,
        }
    }
}

/// Roles allowed to bypass target-language rendering because they are not
/// generated source code.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum TextFileRole {
    Metadata,
    Documentation,
    Asset,
}

impl From<TextFileRole> for FileRole {
    fn from(role: TextFileRole) -> Self {
        match role {
            TextFileRole::Metadata => Self::Metadata,
            TextFileRole::Documentation => Self::Documentation,
            TextFileRole::Asset => Self::Asset,
        }
    }
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

/// The smallest dependency-complete target-language mapping.
///
/// A fragment keeps target syntax, structured imports, and runtime-helper roots
/// inseparable while mappings are composed. Transforming or joining fragments
/// preserves every requirement by construction.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LanguageFragment<I: Ord> {
    document: Document,
    imports: ImportSet<I>,
    helper_roots: BTreeSet<String>,
}

impl<I: Ord> LanguageFragment<I> {
    pub fn new(document: Document) -> Self {
        Self {
            document,
            imports: ImportSet::default(),
            helper_roots: BTreeSet::new(),
        }
    }

    pub fn with_import(mut self, group: ImportGroup, import: I) -> Self {
        self.imports.require(group, import);
        self
    }

    pub fn require_import(&mut self, group: ImportGroup, import: I) -> bool {
        self.imports.require(group, import)
    }

    pub fn with_helper_root(mut self, helper: impl Into<String>) -> Self {
        self.helper_roots.insert(helper.into());
        self
    }

    pub fn imports(&self) -> &ImportSet<I> {
        &self.imports
    }

    pub fn helper_roots(&self) -> &BTreeSet<String> {
        &self.helper_roots
    }

    pub fn map_document(self, map: impl FnOnce(Document) -> Document) -> Self {
        Self {
            document: map(self.document),
            imports: self.imports,
            helper_roots: self.helper_roots,
        }
    }

    pub fn indent(self, spaces: usize) -> Self {
        self.map_document(|document| document.indent(spaces))
    }

    pub fn group(self) -> Self {
        self.map_document(Document::group)
    }

    pub fn sequence(fragments: impl IntoIterator<Item = Self>) -> Self {
        let mut documents = Vec::new();
        let mut imports = ImportSet::default();
        let mut helper_roots = BTreeSet::new();
        for fragment in fragments {
            documents.push(fragment.document);
            for (group, group_imports) in fragment.imports.groups {
                imports
                    .groups
                    .entry(group)
                    .or_default()
                    .extend(group_imports);
            }
            helper_roots.extend(fragment.helper_roots);
        }
        Self {
            document: Document::concat(documents),
            imports,
            helper_roots,
        }
    }

    pub fn optional(fragment: Option<Self>) -> Self {
        fragment.unwrap_or_else(|| Self::new(Document::empty()))
    }

    pub fn into_unit(self) -> LanguageUnit<I> {
        LanguageUnit { fragment: self }
    }

    fn document(&self) -> &Document {
        &self.document
    }
}

impl<I: Clone + Ord> LanguageFragment<I> {
    pub fn joined(separator: Self, fragments: impl IntoIterator<Item = Self>) -> Self {
        let mut fragments = fragments.into_iter();
        let Some(first) = fragments.next() else {
            return Self::new(Document::empty());
        };
        let mut parts = vec![first];
        for fragment in fragments {
            parts.push(separator.clone());
            parts.push(fragment);
        }
        Self::sequence(parts)
    }
}

/// A closed target-language file section.
///
/// Units can only be created from dependency-complete fragments. They expose no
/// document or dependency mutation path, so a file cannot be repaired after a
/// mapping has discarded its requirements.
///
/// ```compile_fail
/// use portable_codegen::{Document, LanguageUnit};
/// let _ = LanguageUnit::<String>::new(Document::empty());
/// ```
///
/// ```compile_fail
/// use portable_codegen::{Document, ImportGroup, LanguageFragment};
/// let mut unit = LanguageFragment::<String>::new(Document::empty()).into_unit();
/// unit.require_import(ImportGroup::new(10, "standard").unwrap(), "value".into());
/// ```
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LanguageUnit<I: Ord> {
    fragment: LanguageFragment<I>,
}

impl<I: Ord> LanguageUnit<I> {
    pub fn imports(&self) -> &ImportSet<I> {
        self.fragment.imports()
    }

    pub fn helper_roots(&self) -> &BTreeSet<String> {
        self.fragment.helper_roots()
    }

    fn document(&self) -> &Document {
        self.fragment.document()
    }
}

impl<I: Ord> From<LanguageFragment<I>> for LanguageUnit<I> {
    fn from(fragment: LanguageFragment<I>) -> Self {
        Self { fragment }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RuntimeHelper<I: Ord> {
    id: String,
    order: u16,
    fragment: LanguageFragment<I>,
}

impl<I: Ord> RuntimeHelper<I> {
    pub fn new(id: impl Into<String>, order: u16, fragment: LanguageFragment<I>) -> Self {
        Self {
            id: id.into(),
            order,
            fragment,
        }
    }

    pub fn id(&self) -> &str {
        &self.id
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RuntimeHelperGraph<I: Ord> {
    helpers: BTreeMap<String, RuntimeHelper<I>>,
}

impl<I: Ord> RuntimeHelperGraph<I> {
    pub fn new(
        helpers: impl IntoIterator<Item = RuntimeHelper<I>>,
    ) -> Result<Self, HelperGraphError> {
        let mut indexed = BTreeMap::new();
        for helper in helpers {
            if helper.id.is_empty() {
                return Err(HelperGraphError::InvalidId(helper.id));
            }
            let id = helper.id.clone();
            if indexed.insert(id.clone(), helper).is_some() {
                return Err(HelperGraphError::DuplicateId(id));
            }
        }
        Ok(Self { helpers: indexed })
    }
}

impl<I: Clone + Ord> RuntimeHelperGraph<I> {
    pub fn resolve(
        &self,
        roots: impl IntoIterator<Item = impl AsRef<str>>,
    ) -> Result<LanguageFragment<I>, HelperGraphError> {
        let mut selected = BTreeSet::new();
        let mut pending = roots
            .into_iter()
            .map(|root| (root.as_ref().to_owned(), None::<String>))
            .collect::<Vec<_>>();
        while let Some((id, required_by)) = pending.pop() {
            let Some(helper) = self.helpers.get(&id) else {
                return Err(HelperGraphError::MissingId { id, required_by });
            };
            if !selected.insert(id.clone()) {
                continue;
            }
            pending.extend(
                helper
                    .fragment
                    .helper_roots()
                    .iter()
                    .rev()
                    .map(|dependency| (dependency.clone(), Some(id.clone()))),
            );
        }

        let mut dependents = BTreeMap::<String, BTreeSet<String>>::new();
        let mut indegree = BTreeMap::<String, usize>::new();
        for id in &selected {
            indegree.insert(id.clone(), 0);
        }
        for id in &selected {
            let helper = &self.helpers[id];
            for dependency in helper.fragment.helper_roots() {
                if !selected.contains(dependency) {
                    return Err(HelperGraphError::MissingId {
                        id: dependency.clone(),
                        required_by: Some(id.clone()),
                    });
                }
                dependents
                    .entry(dependency.clone())
                    .or_default()
                    .insert(id.clone());
                *indegree.get_mut(id).expect("selected helper has indegree") += 1;
            }
        }

        let mut ready = BTreeSet::<(u16, String)>::new();
        for (id, degree) in &indegree {
            if *degree == 0 {
                ready.insert((self.helpers[id].order, id.clone()));
            }
        }
        let mut ordered = Vec::new();
        while let Some((order, id)) = ready.pop_first() {
            let _ = order;
            ordered.push(id.clone());
            for dependent in dependents.get(&id).into_iter().flatten() {
                let degree = indegree
                    .get_mut(dependent)
                    .expect("dependent helper has indegree");
                *degree -= 1;
                if *degree == 0 {
                    ready.insert((self.helpers[dependent].order, dependent.clone()));
                }
            }
        }
        if ordered.len() != selected.len() {
            let cycle = indegree
                .into_iter()
                .filter_map(|(id, degree)| (degree != 0).then_some(id))
                .collect();
            return Err(HelperGraphError::Cycle(cycle));
        }

        let mut fragment = LanguageFragment::sequence(
            ordered
                .into_iter()
                .map(|id| self.helpers[&id].fragment.clone()),
        );
        fragment.helper_roots.clear();
        Ok(fragment)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum HelperGraphError {
    InvalidId(String),
    DuplicateId(String),
    MissingId {
        id: String,
        required_by: Option<String>,
    },
    Cycle(Vec<String>),
}

impl fmt::Display for HelperGraphError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidId(id) => write!(formatter, "invalid empty runtime helper ID {id:?}"),
            Self::DuplicateId(id) => write!(formatter, "duplicate runtime helper ID {id:?}"),
            Self::MissingId { id, required_by } => match required_by {
                Some(required_by) => write!(
                    formatter,
                    "runtime helper {required_by:?} requires missing helper {id:?}"
                ),
                None => write!(formatter, "missing runtime helper root {id:?}"),
            },
            Self::Cycle(ids) => write!(
                formatter,
                "runtime helper dependency cycle among {}",
                ids.iter()
                    .map(|id| format!("{id:?}"))
                    .collect::<Vec<_>>()
                    .join(", ")
            ),
        }
    }
}

impl std::error::Error for HelperGraphError {}

/// A source file after target translation but before syntax rendering.
///
/// Non-source roles cannot enter this API:
///
/// ```compile_fail
/// use portable_codegen::{LanguageSourceFile, TextFileRole};
/// let _ = LanguageSourceFile::<String>::new("README.md", TextFileRole::Documentation);
/// ```
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LanguageSourceFile<I: Ord> {
    path: String,
    role: SourceFileRole,
    preamble: Option<LanguageUnit<I>>,
    body: Option<LanguageUnit<I>>,
    epilogue: Option<LanguageUnit<I>>,
    render_options: RenderOptions,
}

impl<I: Ord> LanguageSourceFile<I> {
    pub fn new(path: impl Into<String>, role: SourceFileRole) -> Self {
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
        self.role.into()
    }

    pub fn set_preamble(&mut self, unit: impl Into<LanguageUnit<I>>) {
        self.preamble = Some(unit.into());
    }

    pub fn set_body(&mut self, unit: impl Into<LanguageUnit<I>>) {
        self.body = Some(unit.into());
    }

    pub fn set_epilogue(&mut self, unit: impl Into<LanguageUnit<I>>) {
        self.epilogue = Some(unit.into());
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

/// A role-safe generated package file.
///
/// Source-bearing variants are private and can only be created from a closed
/// `LanguageSourceFile`. Text constructors accept only non-source roles.
///
/// ```compile_fail
/// use portable_codegen::{FileRole, LanguageFile};
/// let _ = LanguageFile::<String>::text("src/lib.rs", FileRole::Source, "source");
/// ```
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LanguageFile<I: Ord> {
    kind: LanguageFileKind<I>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum LanguageFileKind<I: Ord> {
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
        Self {
            kind: LanguageFileKind::Source(file),
        }
    }

    pub fn text(path: impl Into<String>, role: TextFileRole, contents: impl Into<String>) -> Self {
        Self {
            kind: LanguageFileKind::Text {
                path: path.into(),
                role: role.into(),
                contents: contents.into(),
            },
        }
    }

    pub fn bytes(path: impl Into<String>, contents: impl Into<Vec<u8>>) -> Self {
        Self {
            kind: LanguageFileKind::Bytes {
                path: path.into(),
                role: FileRole::Asset,
                contents: contents.into(),
            },
        }
    }

    pub fn path(&self) -> &str {
        match &self.kind {
            LanguageFileKind::Source(file) => file.path(),
            LanguageFileKind::Text { path, .. } | LanguageFileKind::Bytes { path, .. } => path,
        }
    }

    pub fn role(&self) -> FileRole {
        match &self.kind {
            LanguageFileKind::Source(file) => file.role(),
            LanguageFileKind::Text { role, .. } | LanguageFileKind::Bytes { role, .. } => *role,
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
/// use portable_codegen::{ImportGroup, LanguageSourceFile, SourceFileRole};
///
/// let mut file = LanguageSourceFile::<String>::new("src/example", SourceFileRole::Source);
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
            match &file.kind {
                LanguageFileKind::Source(file) => {
                    output.push(OutputFile::text(
                        file.path(),
                        render_source(file, renderer)?,
                    ));
                }
                LanguageFileKind::Text { path, contents, .. } => {
                    output.push(OutputFile::text(path, contents));
                }
                LanguageFileKind::Bytes { path, contents, .. } => {
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

    fn fragment(
        text: &'static str,
        import: &'static str,
        helper: &'static str,
    ) -> LanguageFragment<TestImport> {
        LanguageFragment::new(Document::raw_text(RawText::new(text)))
            .with_import(import_group(10, "standard"), TestImport(import))
            .with_helper_root(helper)
    }

    #[test]
    fn fragment_composition_is_associative_and_preserves_requirements() {
        let alpha = fragment("a", "alpha", "helper.alpha");
        let beta = fragment("b", "beta", "helper.beta");
        let gamma = fragment("c", "alpha", "helper.alpha");
        let left = LanguageFragment::sequence([
            LanguageFragment::sequence([alpha.clone(), beta.clone()]),
            gamma.clone(),
        ]);
        let right = LanguageFragment::sequence([alpha, LanguageFragment::sequence([beta, gamma])]);
        assert_eq!(left, right);
        assert_eq!(left.imports().len(), 2);
        assert_eq!(
            left.helper_roots()
                .iter()
                .map(String::as_str)
                .collect::<Vec<_>>(),
            ["helper.alpha", "helper.beta"]
        );
        assert_eq!(
            render(left.document(), RenderOptions::default()).unwrap(),
            "abc\n"
        );
    }

    #[test]
    fn optional_joined_grouped_and_indented_fragments_keep_dependencies() {
        let separator = fragment("|", "separator", "helper.separator");
        let joined = LanguageFragment::joined(
            separator,
            [
                fragment("a", "alpha", "helper.alpha"),
                fragment("b", "beta", "helper.beta"),
            ],
        )
        .indent(2)
        .group();
        assert_eq!(joined.imports().len(), 3);
        assert_eq!(joined.helper_roots().len(), 3);
        assert_eq!(
            render(joined.document(), RenderOptions::default()).unwrap(),
            "a|b\n"
        );

        let empty = LanguageFragment::<TestImport>::optional(None);
        assert!(empty.imports().is_empty());
        assert!(empty.helper_roots().is_empty());
        assert_eq!(
            render(empty.document(), RenderOptions::default()).unwrap(),
            "\n"
        );

        let unit = joined.into_unit();
        assert_eq!(unit.imports().len(), 3);
        assert_eq!(unit.helper_roots().len(), 3);
    }

    #[test]
    fn helper_graph_resolves_stable_dependency_order_and_deduplicates() {
        let dependency = RuntimeHelper::new(
            "dependency",
            5,
            LanguageFragment::new(Document::raw_text(RawText::new("a")))
                .with_import(import_group(10, "standard"), TestImport("alpha")),
        );
        let root = RuntimeHelper::new("root", 10, fragment("b", "beta", "dependency"));
        let independent = RuntimeHelper::new(
            "independent",
            20,
            LanguageFragment::new(Document::raw_text(RawText::new("c"))),
        );
        let graph = RuntimeHelperGraph::new([root, independent, dependency]).unwrap();
        let resolved = graph.resolve(["root", "independent", "root"]).unwrap();
        assert_eq!(
            render(resolved.document(), RenderOptions::default()).unwrap(),
            "abc\n"
        );
        assert_eq!(resolved.imports().len(), 2);
        assert!(resolved.helper_roots().is_empty());
    }

    #[test]
    fn helper_graph_rejects_invalid_duplicate_missing_and_cyclic_nodes() {
        let empty =
            RuntimeHelper::<TestImport>::new("", 0, LanguageFragment::new(Document::empty()));
        assert!(matches!(
            RuntimeHelperGraph::new([empty]),
            Err(HelperGraphError::InvalidId(_))
        ));

        let one =
            RuntimeHelper::<TestImport>::new("same", 0, LanguageFragment::new(Document::empty()));
        let two = one.clone();
        assert!(matches!(
            RuntimeHelperGraph::new([one, two]),
            Err(HelperGraphError::DuplicateId(id)) if id == "same"
        ));

        let graph = RuntimeHelperGraph::<TestImport>::new([]).unwrap();
        assert!(matches!(
            graph.resolve(["missing"]),
            Err(HelperGraphError::MissingId { id, required_by: None }) if id == "missing"
        ));

        let missing_dependency = RuntimeHelper::<TestImport>::new(
            "root",
            0,
            LanguageFragment::new(Document::empty()).with_helper_root("missing"),
        );
        let graph = RuntimeHelperGraph::new([missing_dependency]).unwrap();
        assert!(matches!(
            graph.resolve(["root"]),
            Err(HelperGraphError::MissingId { id, required_by: Some(parent) })
                if id == "missing" && parent == "root"
        ));

        let left = RuntimeHelper::<TestImport>::new(
            "left",
            0,
            LanguageFragment::new(Document::empty()).with_helper_root("right"),
        );
        let right = RuntimeHelper::<TestImport>::new(
            "right",
            0,
            LanguageFragment::new(Document::empty()).with_helper_root("left"),
        );
        let graph = RuntimeHelperGraph::new([left, right]).unwrap();
        assert!(matches!(
            graph.resolve(["left"]),
            Err(HelperGraphError::Cycle(ids)) if ids == ["left", "right"]
        ));
    }

    #[test]
    fn groups_files_and_imports_are_sorted_and_deduplicated() {
        let mut source = LanguageSourceFile::new("src/lib.test", SourceFileRole::Source);
        let mut preamble = LanguageFragment::new(Document::raw_text(RawText::new("// preamble")));
        assert!(preamble.require_import(import_group(10, "standard"), TestImport("zeta")));
        let mut body = LanguageFragment::new(Document::raw_text(RawText::new("body")));
        assert!(body.require_import(import_group(10, "standard"), TestImport("alpha")));
        assert!(!body.require_import(import_group(10, "standard"), TestImport("alpha")));
        let mut epilogue = LanguageFragment::new(Document::empty());
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
                TextFileRole::Metadata,
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
        let mut source =
            LanguageSourceFile::<TestImport>::new("src/empty.test", SourceFileRole::Source);
        source.set_body(LanguageFragment::new(Document::raw_text(RawText::new(
            "body",
        ))));
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
    fn file_constructors_preserve_disjoint_source_and_raw_roles() {
        let source = LanguageFile::<TestImport>::source(LanguageSourceFile::new(
            "src/lib.test",
            SourceFileRole::Source,
        ));
        let metadata = LanguageFile::<TestImport>::text(
            "project.toml",
            TextFileRole::Metadata,
            "name = 'example'",
        );
        let asset = LanguageFile::<TestImport>::bytes("icon.bin", [0_u8, 1_u8]);
        assert_eq!(source.role(), FileRole::Source);
        assert_eq!(metadata.role(), FileRole::Metadata);
        assert_eq!(asset.role(), FileRole::Asset);
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
            vec![LanguageFile::text("same", TextFileRole::Metadata, "a")],
        )
        .unwrap();
        let second = FileGroup::<TestImport>::new(
            id("second"),
            vec![LanguageFile::text("same", TextFileRole::Metadata, "b")],
        )
        .unwrap();
        assert!(matches!(
            LanguagePackage::new(vec![first, second], vec![], vec![]),
            Err(LanguagePackageError::DuplicatePath { .. })
        ));
        let one = FileGroup::<TestImport>::new(
            id("same"),
            vec![LanguageFile::text("one", TextFileRole::Metadata, "a")],
        )
        .unwrap();
        let two = FileGroup::<TestImport>::new(
            id("same"),
            vec![LanguageFile::text("two", TextFileRole::Metadata, "b")],
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
        let mut source = LanguageSourceFile::new("src/lib.test", SourceFileRole::Source);
        let mut body = LanguageFragment::new(Document::empty());
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
