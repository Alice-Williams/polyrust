use std::{
    collections::{BTreeMap, BTreeSet},
    fmt::Debug,
};

use handlebars::Handlebars;
use portable_diagnostics::{Diagnostic, DiagnosticCode, SourceRef, sort_diagnostics};
use serde::Serialize;

use crate::{
    LinkedFile, LinkedTargetPackage, LinkerDialect, RenderedFile, RenderedPackage, TargetArtifact,
    verify_linked_package,
};

const MAX_RENDERED_FILE_BYTES: usize = 8 * 1024 * 1024;
const MAX_RENDERED_PACKAGE_BYTES: usize = 64 * 1024 * 1024;

pub trait CertifiedTemplateId: Clone + Debug + Eq + Ord + Send + Sync + 'static {
    fn all() -> &'static [Self];
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EmbeddedTemplate<I: CertifiedTemplateId> {
    id: I,
    source: &'static str,
    fields: &'static [&'static str],
}

impl<I: CertifiedTemplateId> EmbeddedTemplate<I> {
    pub const fn new(id: I, source: &'static str, fields: &'static [&'static str]) -> Self {
        Self { id, source, fields }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum CertifiedRenderError {
    DuplicateTemplate,
    MissingTemplate,
    InvalidTemplate,
    MissingPartial,
    ForbiddenHelper,
    MissingField,
    ExtraField,
    Serialization,
    Rendering,
    InvalidEncoding,
    FileTooLarge,
    PackageTooLarge,
    UnsupportedArtifact,
}

pub struct CertifiedTemplateEngine<I: CertifiedTemplateId> {
    registry: Handlebars<'static>,
    templates: BTreeMap<I, RegisteredTemplate>,
}

struct RegisteredTemplate {
    internal_name: String,
    fields: BTreeSet<String>,
}

#[derive(Debug)]
struct RenderFailure {
    kind: CertifiedRenderError,
    message: String,
}

impl<I: CertifiedTemplateId> CertifiedTemplateEngine<I> {
    fn new(target: &str, definitions: Vec<EmbeddedTemplate<I>>) -> Result<Self, Vec<Diagnostic>> {
        let source = SourceRef::logical(["resolved-renderer", target, "registry"]);
        let mut diagnostics = Vec::new();
        let mut supplied = BTreeMap::new();
        for definition in definitions {
            if supplied.insert(definition.id.clone(), definition).is_some() {
                diagnostics.push(registry_error(
                    CertifiedRenderError::DuplicateTemplate,
                    "template ID is registered more than once",
                    source.clone(),
                ));
            }
        }
        let expected = I::all().iter().cloned().collect::<BTreeSet<_>>();
        let actual = supplied.keys().cloned().collect::<BTreeSet<_>>();
        if expected.len() != I::all().len() {
            diagnostics.push(registry_error(
                CertifiedRenderError::DuplicateTemplate,
                "closed template ID inventory contains a duplicate",
                source.clone(),
            ));
        }
        if expected.is_empty() {
            diagnostics.push(registry_error(
                CertifiedRenderError::MissingTemplate,
                "closed template ID inventory is empty",
                source.clone(),
            ));
        }
        if expected != actual {
            diagnostics.push(registry_error(
                CertifiedRenderError::MissingTemplate,
                "registered template IDs do not exactly match the closed template enum",
                source.clone(),
            ));
        }

        let mut registry = Handlebars::new();
        registry.set_strict_mode(true);
        registry.set_dev_mode(false);
        registry.register_escape_fn(handlebars::no_escape);
        let mut templates = BTreeMap::new();
        for (index, id) in I::all().iter().enumerate() {
            let Some(definition) = supplied.get(id) else {
                continue;
            };
            let declared_fields = definition
                .fields
                .iter()
                .map(|field| (*field).to_owned())
                .collect::<BTreeSet<_>>();
            if declared_fields.len() != definition.fields.len()
                || declared_fields.iter().any(|field| !valid_field(field))
            {
                diagnostics.push(registry_error(
                    CertifiedRenderError::InvalidTemplate,
                    "template field contract contains duplicate or invalid names",
                    source.clone(),
                ));
                continue;
            }
            match template_fields(definition.source) {
                Ok(referenced_fields) if referenced_fields == declared_fields => {}
                Ok(_) => {
                    diagnostics.push(registry_error(
                        CertifiedRenderError::InvalidTemplate,
                        "template references do not exactly match its declared field contract",
                        source.clone(),
                    ));
                    continue;
                }
                Err(kind) => {
                    diagnostics.push(registry_error(
                        kind,
                        "template violates the certified presentation grammar",
                        source.clone(),
                    ));
                    continue;
                }
            }
            let internal_name = format!("__polyrust_certified_{index}");
            if let Err(error) = registry.register_template_string(&internal_name, definition.source)
            {
                diagnostics.push(registry_error(
                    CertifiedRenderError::InvalidTemplate,
                    &format!("Handlebars rejected embedded certified template {id:?}: {error}"),
                    source.clone(),
                ));
                continue;
            }
            templates.insert(
                id.clone(),
                RegisteredTemplate {
                    internal_name,
                    fields: declared_fields,
                },
            );
        }
        sort_diagnostics(&mut diagnostics);
        if diagnostics.is_empty() {
            Ok(Self {
                registry,
                templates,
            })
        } else {
            Err(diagnostics)
        }
    }

    pub fn render<V: Serialize, D: LinkerDialect<TemplateId = I>>(
        &mut self,
        id: &I,
        view: &V,
        target: &str,
        file: &LinkedFile<D>,
    ) -> Result<String, Vec<Diagnostic>> {
        self.render_value(id, view).map_err(|failure| {
            vec![render_error(
                failure.kind,
                target,
                id,
                file,
                &failure.message,
                file.source().clone(),
            )]
        })
    }

    fn render_value<V: Serialize>(&self, id: &I, view: &V) -> Result<String, RenderFailure> {
        let Some(template) = self.templates.get(id) else {
            return Err(RenderFailure {
                kind: CertifiedRenderError::MissingTemplate,
                message: "template ID is not registered".to_owned(),
            });
        };
        let value = serde_json::to_value(view).map_err(|_| RenderFailure {
            kind: CertifiedRenderError::Serialization,
            message: "typed render view could not be serialized".to_owned(),
        })?;
        let Some(object) = value.as_object() else {
            return Err(RenderFailure {
                kind: CertifiedRenderError::Serialization,
                message: "top-level render view must be a typed struct/map".to_owned(),
            });
        };
        let actual = object.keys().cloned().collect::<BTreeSet<_>>();
        if let Some(field) = template.fields.difference(&actual).next() {
            return Err(RenderFailure {
                kind: CertifiedRenderError::MissingField,
                message: format!("render view is missing required field {field:?}"),
            });
        }
        if let Some(field) = actual.difference(&template.fields).next() {
            return Err(RenderFailure {
                kind: CertifiedRenderError::ExtraField,
                message: format!("render view contains undeclared field {field:?}"),
            });
        }
        self.registry
            .render(&template.internal_name, &value)
            .map_err(|_| RenderFailure {
                kind: CertifiedRenderError::Rendering,
                message: "strict Handlebars rendering failed".to_owned(),
            })
    }
}

pub trait ResolvedTemplateRenderer<D>: Send + Sync + 'static
where
    D: LinkerDialect,
    D::TemplateId: CertifiedTemplateId,
{
    type FileView: Serialize;

    fn target_name(&self) -> &'static str;
    fn templates(&self) -> Vec<EmbeddedTemplate<D::TemplateId>>;
    fn build_file_view(
        &self,
        package: &LinkedTargetPackage<D>,
        file: &LinkedFile<D>,
        templates: &mut CertifiedTemplateEngine<D::TemplateId>,
    ) -> Result<Self::FileView, Vec<Diagnostic>>;
}

/// Certified rendering accepts a linked package, never unresolved target AST.
///
/// ```compile_fail
/// use portable_codegen::{
///     CertifiedTemplateId, LinkerDialect, ResolvedTemplateRenderer,
///     TargetAstPackage, render_linked_package,
/// };
///
/// fn cannot_render_unresolved<D, R>(
///     renderer: &R,
///     package: &TargetAstPackage<D>,
/// ) where
///     D: LinkerDialect,
///     D::TemplateId: CertifiedTemplateId,
///     R: ResolvedTemplateRenderer<D>,
/// {
///     let _ = render_linked_package(renderer, package);
/// }
/// ```
pub fn render_linked_package<D, R>(
    renderer: &R,
    package: &LinkedTargetPackage<D>,
) -> Result<RenderedPackage, Vec<Diagnostic>>
where
    D: LinkerDialect,
    D::TemplateId: CertifiedTemplateId,
    R: ResolvedTemplateRenderer<D>,
{
    verify_linked_package(package)?;
    let target = renderer.target_name();
    let mut templates = CertifiedTemplateEngine::new(target, renderer.templates())?;
    let mut diagnostics = Vec::new();
    let mut files = Vec::new();
    let mut package_bytes = 0usize;
    let mut previous_path: Option<&str> = None;
    for file in package.files() {
        if previous_path.is_some_and(|previous| previous >= file.path().as_str()) {
            diagnostics.push(render_error(
                CertifiedRenderError::InvalidEncoding,
                target,
                file.template(),
                file,
                "resolved files are not in deterministic path order",
                file.source().clone(),
            ));
        }
        previous_path = Some(file.path().as_str());
        let view = match renderer.build_file_view(package, file, &mut templates) {
            Ok(view) => view,
            Err(mut errors) => {
                diagnostics.append(&mut errors);
                continue;
            }
        };
        match templates.render(file.template(), &view, target, file) {
            Ok(output) => match canonical_source(output) {
                Ok(output) => {
                    if output.len() > MAX_RENDERED_FILE_BYTES {
                        diagnostics.push(render_error(
                            CertifiedRenderError::FileTooLarge,
                            target,
                            file.template(),
                            file,
                            "rendered source file exceeds the certified size limit",
                            file.source().clone(),
                        ));
                        continue;
                    }
                    package_bytes = package_bytes.saturating_add(output.len());
                    files.push(RenderedFile::source(
                        file.path().as_str(),
                        file.role(),
                        output,
                    ));
                }
                Err(kind) => diagnostics.push(render_error(
                    kind,
                    target,
                    file.template(),
                    file,
                    "rendered source is not canonical UTF-8/LF text",
                    file.source().clone(),
                )),
            },
            Err(mut errors) => diagnostics.append(&mut errors),
        }
    }
    for artifact in package.artifacts() {
        match artifact {
            TargetArtifact::Documentation { path, contents, .. } => {
                match canonical_source(contents.clone()) {
                    Ok(contents) if contents.len() <= MAX_RENDERED_FILE_BYTES => {
                        package_bytes = package_bytes.saturating_add(contents.len());
                        files.push(RenderedFile::documentation(path.as_str(), contents));
                    }
                    Ok(_) => diagnostics.push(artifact_error(
                        CertifiedRenderError::FileTooLarge,
                        target,
                        artifact,
                        "documentation artifact exceeds the certified size limit",
                    )),
                    Err(kind) => diagnostics.push(artifact_error(
                        kind,
                        target,
                        artifact,
                        "documentation artifact is not canonical UTF-8/LF text",
                    )),
                }
            }
            TargetArtifact::Asset { path, contents, .. } => {
                if contents.len() > MAX_RENDERED_FILE_BYTES {
                    diagnostics.push(artifact_error(
                        CertifiedRenderError::FileTooLarge,
                        target,
                        artifact,
                        "binary artifact exceeds the certified size limit",
                    ));
                } else {
                    package_bytes = package_bytes.saturating_add(contents.len());
                    files.push(RenderedFile::asset(path.as_str(), contents.clone()));
                }
            }
            TargetArtifact::Metadata { .. } => diagnostics.push(artifact_error(
                CertifiedRenderError::UnsupportedArtifact,
                target,
                artifact,
                "typed metadata requires its language renderer/template migration",
            )),
            TargetArtifact::DerivedJavaScript { .. } => diagnostics.push(artifact_error(
                CertifiedRenderError::UnsupportedArtifact,
                target,
                artifact,
                "derived JavaScript requires the pinned TypeScript compiler phase",
            )),
        }
    }
    if package_bytes > MAX_RENDERED_PACKAGE_BYTES {
        diagnostics.push(registry_error(
            CertifiedRenderError::PackageTooLarge,
            "rendered package exceeds the certified total size limit",
            SourceRef::logical(["resolved-renderer", target, "package"]),
        ));
    }
    sort_diagnostics(&mut diagnostics);
    if diagnostics.is_empty() {
        files.sort_by(|left, right| left.path().cmp(right.path()));
        Ok(RenderedPackage::new(
            files,
            package.manifest_dependencies(),
            package.manifest_helpers(),
        ))
    } else {
        Err(diagnostics)
    }
}

fn canonical_source(mut output: String) -> Result<String, CertifiedRenderError> {
    if output.contains('\r') || output.contains('\0') {
        return Err(CertifiedRenderError::InvalidEncoding);
    }
    while output.ends_with('\n') {
        output.pop();
    }
    output.push('\n');
    Ok(output)
}

fn template_fields(source: &str) -> Result<BTreeSet<String>, CertifiedRenderError> {
    if source.contains('\r') || source.contains('\0') || source.contains("{{{") {
        return Err(CertifiedRenderError::InvalidTemplate);
    }
    let mut fields = BTreeSet::new();
    let mut remaining = source;
    while let Some(start) = remaining.find("{{") {
        remaining = &remaining[start + 2..];
        let Some(end) = remaining.find("}}") else {
            return Err(CertifiedRenderError::InvalidTemplate);
        };
        let tag = remaining[..end].trim();
        remaining = &remaining[end + 2..];
        if tag.is_empty()
            || tag.starts_with('!')
            || tag == "else"
            || matches!(tag, "/each" | "/if" | "/unless" | "/with")
        {
            continue;
        }
        if tag.starts_with('>') {
            return Err(CertifiedRenderError::MissingPartial);
        }
        let field = if let Some(block) = tag.strip_prefix('#') {
            let mut parts = block.split_whitespace();
            let Some(kind) = parts.next() else {
                return Err(CertifiedRenderError::InvalidTemplate);
            };
            if !matches!(kind, "each" | "if" | "unless" | "with") {
                return Err(CertifiedRenderError::ForbiddenHelper);
            }
            let Some(field) = parts.next() else {
                return Err(CertifiedRenderError::InvalidTemplate);
            };
            if parts.next().is_some() {
                return Err(CertifiedRenderError::ForbiddenHelper);
            }
            field
        } else {
            if tag.split_whitespace().count() != 1 {
                return Err(CertifiedRenderError::ForbiddenHelper);
            }
            tag
        };
        if let Some(root) = root_field(field)? {
            fields.insert(root.to_owned());
        }
    }
    Ok(fields)
}

fn root_field(field: &str) -> Result<Option<&str>, CertifiedRenderError> {
    if matches!(field, "this" | ".") || field.starts_with('@') {
        return Ok(None);
    }
    if field.starts_with("../") || field.contains('[') || field.contains(']') {
        return Err(CertifiedRenderError::InvalidTemplate);
    }
    let root = field.split('.').next().unwrap_or_default();
    if valid_field(root) {
        Ok(Some(root))
    } else {
        Err(CertifiedRenderError::InvalidTemplate)
    }
}

fn valid_field(field: &str) -> bool {
    !field.is_empty()
        && field
            .chars()
            .all(|character| character == '_' || character.is_ascii_alphanumeric())
        && !field.as_bytes()[0].is_ascii_digit()
}

fn registry_error(kind: CertifiedRenderError, message: &str, source: SourceRef) -> Diagnostic {
    Diagnostic::error(
        DiagnosticCode::InvalidStructure,
        format!("certified renderer {kind:?}: {message}"),
        source,
    )
}

fn render_error<D: LinkerDialect>(
    kind: CertifiedRenderError,
    target: &str,
    template: &D::TemplateId,
    file: &LinkedFile<D>,
    message: &str,
    source: SourceRef,
) -> Diagnostic {
    Diagnostic::error(
        DiagnosticCode::InvalidStructure,
        format!(
            "certified renderer {kind:?}; target={target:?}; template={template:?}; role={:?}; path={:?}: {message}",
            file.role(),
            file.path().as_str()
        ),
        source,
    )
}

fn artifact_error(
    kind: CertifiedRenderError,
    target: &str,
    artifact: &TargetArtifact,
    message: &str,
) -> Diagnostic {
    Diagnostic::error(
        DiagnosticCode::InvalidStructure,
        format!(
            "certified renderer {kind:?}; target={target:?}; artifact_role={:?}; path={:?}: {message}",
            artifact.group_role(),
            artifact.path().as_str()
        ),
        artifact.source().clone(),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
    enum TemplateId {
        File,
        Lines,
    }

    impl CertifiedTemplateId for TemplateId {
        fn all() -> &'static [Self] {
            const ALL: &[TemplateId] = &[TemplateId::File, TemplateId::Lines];
            ALL
        }
    }

    #[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
    enum DuplicateInventoryId {
        File,
    }

    impl CertifiedTemplateId for DuplicateInventoryId {
        fn all() -> &'static [Self] {
            const ALL: &[DuplicateInventoryId] =
                &[DuplicateInventoryId::File, DuplicateInventoryId::File];
            ALL
        }
    }

    #[derive(Serialize)]
    struct FileView {
        declaration: String,
    }

    #[derive(Serialize)]
    struct MissingView {
        other: String,
    }

    #[derive(Serialize)]
    struct ExtraView {
        declaration: String,
        extra: String,
    }

    #[derive(Serialize)]
    struct LinesView {
        lines: Vec<String>,
    }

    fn definitions() -> Vec<EmbeddedTemplate<TemplateId>> {
        vec![
            EmbeddedTemplate::new(TemplateId::File, "{{declaration}}", &["declaration"]),
            EmbeddedTemplate::new(
                TemplateId::Lines,
                "{{#each lines}}{{this}}\n{{/each}}",
                &["lines"],
            ),
        ]
    }

    #[test]
    fn registry_is_strict_complete_and_deterministic() {
        let first = CertifiedTemplateEngine::new("test", definitions()).unwrap();
        let second = CertifiedTemplateEngine::new("test", definitions()).unwrap();
        assert_eq!(
            first.templates.keys().collect::<Vec<_>>(),
            second.templates.keys().collect::<Vec<_>>()
        );
        assert!(first.registry.strict_mode());
        assert!(!first.registry.dev_mode());
        assert_eq!(
            serde_json::to_value(FileView {
                declaration: "record Value;".to_owned()
            })
            .unwrap()["declaration"],
            "record Value;"
        );
        let view = FileView {
            declaration: "left < right".to_owned(),
        };
        let rendered = first.render_value(&TemplateId::File, &view).unwrap();
        assert_eq!(rendered, "left < right");
        assert_eq!(
            rendered,
            first.render_value(&TemplateId::File, &view).unwrap()
        );
        assert_eq!(
            first
                .render_value(
                    &TemplateId::Lines,
                    &LinesView {
                        lines: vec!["left".to_owned(), "right".to_owned()],
                    },
                )
                .unwrap(),
            "left\nright\n"
        );
        assert_eq!(
            rendered,
            first.render_value(&TemplateId::File, &view).unwrap()
        );
    }

    #[test]
    fn duplicate_missing_field_contract_helper_and_partial_faults_are_rejected() {
        let mut duplicate = definitions();
        duplicate.push(duplicate[0].clone());
        assert!(CertifiedTemplateEngine::new("test", duplicate).is_err());

        let missing = vec![definitions()[0].clone()];
        assert!(CertifiedTemplateEngine::new("test", missing).is_err());

        assert!(
            CertifiedTemplateEngine::new(
                "test",
                vec![EmbeddedTemplate::new(
                    DuplicateInventoryId::File,
                    "{{declaration}}",
                    &["declaration"],
                )],
            )
            .is_err()
        );

        let engine = CertifiedTemplateEngine::new("test", definitions()).unwrap();
        assert_eq!(
            engine
                .render_value(
                    &TemplateId::File,
                    &MissingView {
                        other: String::new(),
                    },
                )
                .unwrap_err()
                .kind,
            CertifiedRenderError::MissingField
        );
        assert_eq!(
            engine
                .render_value(
                    &TemplateId::File,
                    &ExtraView {
                        declaration: String::new(),
                        extra: String::new(),
                    },
                )
                .unwrap_err()
                .kind,
            CertifiedRenderError::ExtraField
        );

        for definition in [
            EmbeddedTemplate::new(TemplateId::File, "{{missing}}", &["declared"]),
            EmbeddedTemplate::new(TemplateId::File, "{{semantic_helper value}}", &["value"]),
            EmbeddedTemplate::new(TemplateId::File, "{{> missing}}", &[]),
        ] {
            let result =
                CertifiedTemplateEngine::new("test", vec![definition, definitions()[1].clone()]);
            assert!(result.is_err());
        }
    }

    #[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
    enum Precedence {
        Sum,
        Product,
        Call,
        Atom,
    }

    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    enum Associativity {
        Left,
        Right,
        None,
    }

    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    enum OperandSide {
        Left,
        Right,
    }

    fn child(
        parent: Precedence,
        child: Precedence,
        associativity: Associativity,
        side: OperandSide,
        spelling: &str,
    ) -> String {
        let equal_needs_parentheses = parent == child
            && match associativity {
                Associativity::Left => side == OperandSide::Right,
                Associativity::Right => side == OperandSide::Left,
                Associativity::None => true,
            };
        if child < parent || equal_needs_parentheses {
            format!("({spelling})")
        } else {
            spelling.to_owned()
        }
    }

    fn escaped_identifier(value: &str) -> String {
        let mut result = String::new();
        for character in value.chars() {
            if character == '_' || character.is_ascii_alphanumeric() {
                result.push(character);
            } else {
                result.push_str(&format!("_u{:04x}", u32::from(character)));
            }
        }
        if result.as_bytes().first().is_some_and(u8::is_ascii_digit) {
            result.insert(0, '_');
        }
        result
    }

    fn escaped_literal(value: &str) -> String {
        value
            .replace('\\', "\\\\")
            .replace('"', "\\\"")
            .replace('\n', "\\n")
            .replace('\r', "\\r")
            .replace('\0', "\\0")
    }

    fn escaped_documentation(value: &str) -> String {
        value
            .replace('\0', "\u{fffd}")
            .replace('\r', "")
            .replace("*/", "* /")
    }

    fn escaped_comment(value: &str) -> String {
        escaped_documentation(value).replace('\n', " ")
    }

    #[test]
    fn precedence_and_escaping_are_decided_before_templates() {
        for parent in [
            Precedence::Sum,
            Precedence::Product,
            Precedence::Call,
            Precedence::Atom,
        ] {
            for nested in [
                Precedence::Sum,
                Precedence::Product,
                Precedence::Call,
                Precedence::Atom,
            ] {
                for associativity in [
                    Associativity::Left,
                    Associativity::Right,
                    Associativity::None,
                ] {
                    for side in [OperandSide::Left, OperandSide::Right] {
                        let rendered = child(parent, nested, associativity, side, "value");
                        let equal_needs_parentheses = parent == nested
                            && match associativity {
                                Associativity::Left => side == OperandSide::Right,
                                Associativity::Right => side == OperandSide::Left,
                                Associativity::None => true,
                            };
                        assert_eq!(
                            rendered.starts_with('('),
                            nested < parent || equal_needs_parentheses
                        );
                    }
                }
            }
        }
        assert_eq!(
            child(
                Precedence::Sum,
                Precedence::Sum,
                Associativity::Left,
                OperandSide::Right,
                "a + b",
            ),
            "(a + b)"
        );
        assert_eq!(
            child(
                Precedence::Sum,
                Precedence::Sum,
                Associativity::Right,
                OperandSide::Left,
                "a + b",
            ),
            "(a + b)"
        );
        assert_eq!(
            child(
                Precedence::Sum,
                Precedence::Sum,
                Associativity::None,
                OperandSide::Left,
                "a + b",
            ),
            "(a + b)"
        );
        for (input, expected) in [
            ("name", "name"),
            ("9bad-name", "_9bad_u002dname"),
            ("snowman☃", "snowman_u2603"),
        ] {
            assert_eq!(escaped_identifier(input), expected);
        }
        for (input, expected) in [
            ("plain", "plain"),
            ("quote=\"", "quote=\\\""),
            ("slash=\\", "slash=\\\\"),
            ("newline=\n", "newline=\\n"),
            ("nul=\0", "nul=\\0"),
        ] {
            assert_eq!(escaped_literal(input), expected);
        }
        assert_eq!(
            escaped_documentation("close */\r nul=\0"),
            "close * / nul=�"
        );
        assert_eq!(escaped_comment("close */\r\nnext\0"), "close * / next�");
    }

    #[test]
    fn newline_policy_is_canonical_and_rejects_injected_control_bytes() {
        assert_eq!(canonical_source("value".to_owned()).unwrap(), "value\n");
        assert_eq!(canonical_source("value\n\n".to_owned()).unwrap(), "value\n");
        assert!(canonical_source("bad\r\n".to_owned()).is_err());
        assert!(canonical_source("bad\0".to_owned()).is_err());
    }
}
