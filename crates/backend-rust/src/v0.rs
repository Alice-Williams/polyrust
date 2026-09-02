use std::collections::{BTreeMap, BTreeSet};

use portable_check::v0::{Capability, CheckedProgram};
use portable_codegen::{
    Backend, BackendDescriptor, BackendError, BackendOptions, BackendVersion, CapabilitySupport,
    DeclaredDependency, Document as CodeDocument, FileGroup, FileGroupId, FinalNewline,
    ImportGroup, ImportSet, InjectedHelper, IrVersionRange, LanguageFile, LanguageFragment,
    LanguagePackage, LanguagePlugin, LanguageRenderer, LanguageSourceFile, OptionsSchema,
    OutputManifest, RawText, RenderOptions, RuntimeHelper, RuntimeHelperGraph, SourceFileRole,
    TargetId, TextFileRole, generate_with_plugin, render, validate_backend_capability,
};
use portable_ir::v0::{
    Block, ConstantExpression, Declaration, EnumVariant, ExpectedOutcome, Expression, Intrinsic,
    IrVersion, MatchArm, MethodDispatch, NodeId, Parameter, Pattern, Statement, TestInvocation,
    TypeRef, TypedValue, Value,
};

pub struct RustBackend;

impl Backend for RustBackend {
    fn descriptor(&self) -> BackendDescriptor {
        BackendDescriptor {
            target: TargetId::parse("org.polyrust.rust").expect("static target ID is valid"),
            display_name: "Rust".to_owned(),
            backend_version: BackendVersion::new(0, 1, 0),
            supported_ir: IrVersionRange::exact(IrVersion::CURRENT),
        }
    }

    fn support(&self, capability: Capability) -> CapabilitySupport {
        match capability {
            Capability::CheckedIntegerArithmetic => CapabilitySupport::Helper {
                helper: "polyrust.runtime.checked-integers.v0".to_owned(),
            },
            Capability::UnicodeScalar => CapabilitySupport::Helper {
                helper: "polyrust.runtime.unicode-scalars.v0".to_owned(),
            },
            Capability::ImmutableList => CapabilitySupport::Helper {
                helper: "polyrust.runtime.immutable-list.v0".to_owned(),
            },
            Capability::Bytes
            | Capability::InterfaceDispatch
            | Capability::F64
            | Capability::Option
            | Capability::Result
            | Capability::WrappingIntegerArithmetic
            | Capability::BoundedIteration => CapabilitySupport::Native,
            Capability::FirstClassInterfaceValues => CapabilitySupport::Unsupported {
                reason: "first-class interface values require the M34A-12 typed Rust backend"
                    .to_owned(),
            },
        }
    }

    fn options_schema(&self) -> OptionsSchema {
        BTreeMap::new()
    }

    fn generate(
        &self,
        program: &CheckedProgram,
        options: &BackendOptions,
    ) -> Result<OutputManifest, BackendError> {
        generate_with_plugin(self, program, options)
    }
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct RustImport {
    kind: RustImportKind,
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
enum RustImportKind {
    Module { name: String, test_only: bool },
    Use { path: String, public: bool },
}

impl RustImport {
    pub fn module(name: &str, test_only: bool) -> Result<Self, String> {
        if !rust_path_segment(name) {
            return Err(format!("invalid Rust module name {name:?}"));
        }
        Ok(Self {
            kind: RustImportKind::Module {
                name: name.to_owned(),
                test_only,
            },
        })
    }

    pub fn use_path(path: &str, public: bool) -> Result<Self, String> {
        let segments = path.split("::").collect::<Vec<_>>();
        let valid = !segments.is_empty()
            && segments.iter().enumerate().all(|(index, segment)| {
                (*segment == "*" && index + 1 == segments.len())
                    || matches!(*segment, "crate" | "self" | "super")
                    || rust_path_segment(segment)
            });
        if !valid {
            return Err(format!("invalid Rust use path {path:?}"));
        }
        Ok(Self {
            kind: RustImportKind::Use {
                path: path.to_owned(),
                public,
            },
        })
    }
}

fn rust_path_segment(segment: &str) -> bool {
    let mut characters = segment.chars();
    characters
        .next()
        .is_some_and(|first| first == '_' || first.is_ascii_alphabetic())
        && characters.all(|character| character == '_' || character.is_ascii_alphanumeric())
}

#[doc(hidden)]
pub struct RustRenderer;

impl LanguageRenderer<RustImport> for RustRenderer {
    fn render_imports(&self, imports: &ImportSet<RustImport>) -> Result<CodeDocument, String> {
        let mut lines = Vec::new();
        for (_, imports) in imports.groups() {
            for import in imports {
                match &import.kind {
                    RustImportKind::Module { name, test_only } => {
                        if *test_only {
                            lines.push("#[cfg(test)]".to_owned());
                        }
                        lines.push(format!("mod {name};"));
                    }
                    RustImportKind::Use { path, public } => {
                        lines.push(format!("{}use {path};", if *public { "pub " } else { "" }))
                    }
                }
            }
        }
        Ok(CodeDocument::raw_text(RawText::new(lines.join("\n"))))
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
struct RustCode {
    text: String,
    imports: BTreeSet<(ImportGroup, RustImport)>,
    helper_roots: BTreeSet<String>,
}

impl RustCode {
    fn new(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            ..Self::default()
        }
    }

    fn with_import(mut self, import: RustImport) -> Self {
        self.imports.insert((rust_import_group(), import));
        self
    }

    fn with_helper_root(mut self, helper: impl Into<String>) -> Self {
        self.helper_roots.insert(helper.into());
        self
    }

    fn sequence(fragments: impl IntoIterator<Item = Self>) -> Self {
        fragments
            .into_iter()
            .fold(Self::default(), |mut combined, fragment| {
                combined.text.push_str(&fragment.text);
                combined.imports.extend(fragment.imports);
                combined.helper_roots.extend(fragment.helper_roots);
                combined
            })
    }

    fn joined(fragments: impl IntoIterator<Item = Self>, separator: &str) -> Self {
        let mut fragments = fragments.into_iter();
        let Some(first) = fragments.next() else {
            return Self::default();
        };
        fragments.fold(first, |mut combined, fragment| {
            combined.text.push_str(separator);
            combined.text.push_str(&fragment.text);
            combined.imports.extend(fragment.imports);
            combined.helper_roots.extend(fragment.helper_roots);
            combined
        })
    }

    fn map_text(mut self, map: impl FnOnce(String) -> String) -> Self {
        self.text = map(self.text);
        self
    }

    fn with_text_from(mut self, dependencies: impl IntoIterator<Item = Self>) -> Self {
        for dependency in dependencies {
            self.imports.extend(dependency.imports);
            self.helper_roots.extend(dependency.helper_roots);
        }
        self
    }

    fn into_fragment(self) -> LanguageFragment<RustImport> {
        let mut fragment = LanguageFragment::new(CodeDocument::raw_text(RawText::new(self.text)));
        for (group, import) in self.imports {
            fragment.require_import(group, import);
        }
        for helper in self.helper_roots {
            fragment = fragment.with_helper_root(helper);
        }
        fragment
    }
}

impl std::fmt::Display for RustCode {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(&self.text)
    }
}

impl LanguagePlugin for RustBackend {
    type Import = RustImport;
    type Renderer = RustRenderer;

    fn translate(
        &self,
        program: &CheckedProgram,
        options: &BackendOptions,
    ) -> Result<LanguagePackage<Self::Import>, BackendError> {
        let _ = options;
        validate_backend_capability(self, program, Capability::FirstClassInterfaceValues)?;
        let generator = Generator::new(program);
        let cargo = format!(
            "[package]\nname = \"{}\"\nversion = \"0.1.0\"\nedition = \"2024\"\npublish = false\n\n[lib]\npath = \"src/lib.rs\"\n",
            package_name(&program.module().name)
        );
        let helpers = program
            .capabilities()
            .program()
            .iter()
            .filter_map(|capability| match self.support(*capability) {
                CapabilitySupport::Helper { helper } => Some(InjectedHelper {
                    id: helper,
                    capability: format!("{capability:?}"),
                    files: vec!["src/polyrust_runtime.rs".to_owned()],
                }),
                CapabilitySupport::Native | CapabilitySupport::Unsupported { .. } => None,
            })
            .collect();
        LanguagePackage::new(
            vec![
                FileGroup::new(
                    rust_group("metadata")?,
                    vec![LanguageFile::text(
                        "Cargo.toml",
                        TextFileRole::Metadata,
                        cargo,
                    )],
                )
                .map_err(rust_generation_error)?,
                FileGroup::new(
                    rust_group("runtime")?,
                    vec![LanguageFile::source(rust_runtime_file(program)?)],
                )
                .map_err(rust_generation_error)?,
                FileGroup::new(
                    rust_group("source")?,
                    vec![LanguageFile::source(generator.source_file()?)],
                )
                .map_err(rust_generation_error)?,
                FileGroup::new(
                    rust_group("tests")?,
                    vec![LanguageFile::source(rust_conformance_file())],
                )
                .map_err(rust_generation_error)?,
            ],
            Vec::<DeclaredDependency>::new(),
            helpers,
        )
        .map_err(rust_generation_error)
    }

    fn renderer(&self) -> Self::Renderer {
        RustRenderer
    }
}

fn rust_generation_error(error: impl std::fmt::Display) -> BackendError {
    BackendError::Generation {
        message: error.to_string(),
    }
}

fn rust_group(name: &str) -> Result<FileGroupId, BackendError> {
    FileGroupId::parse(name).map_err(rust_generation_error)
}

fn rust_import_group() -> ImportGroup {
    ImportGroup::new(10, "modules-and-uses").expect("static import group is valid")
}

fn rust_runtime_file(
    program: &CheckedProgram,
) -> Result<LanguageSourceFile<RustImport>, BackendError> {
    let (graph, mut roots) = rust_runtime_helper_graph()?;
    for (operation, root) in [
        (Intrinsic::StringReplaceMany, "feature.string-replace-many"),
        (Intrinsic::BytesReplaceAll, "feature.bytes-replace-all"),
        (
            Intrinsic::StringTruncateUtf8Bytes,
            "feature.string-truncate-utf8",
        ),
    ] {
        if portable_ir::v0::module_uses_intrinsic(program.module(), |used| used == operation) {
            roots.push(root.to_owned());
        }
    }
    if portable_ir::v0::module_uses_intrinsic(program.module(), |operation| {
        matches!(
            operation,
            Intrinsic::IntShiftLeftChecked | Intrinsic::IntShiftRightChecked
        )
    }) {
        roots.push("feature.checked-shift".to_owned());
    }

    let mut file = LanguageSourceFile::new("src/polyrust_runtime.rs", SourceFileRole::Runtime);
    file.set_body(graph.resolve(&roots).map_err(rust_generation_error)?);
    Ok(file)
}

fn rust_runtime_helper_graph() -> Result<(RuntimeHelperGraph<RustImport>, Vec<String>), BackendError>
{
    const BEGIN: &str = "// POLYRUST-BEGIN ";
    const END: &str = "// POLYRUST-END ";

    let mut helpers = Vec::new();
    let mut common_roots = Vec::new();
    let mut common_index = 0_u16;
    let mut order = 0_u16;
    let mut active: Option<String> = None;
    let mut source = String::new();
    let emit = |id: String,
                source: &mut String,
                order: &mut u16,
                helpers: &mut Vec<RuntimeHelper<RustImport>>| {
        if source.trim().is_empty() {
            source.clear();
            return false;
        }
        helpers.push(RuntimeHelper::new(
            id,
            *order,
            RustCode::new(std::mem::take(source)).into_fragment(),
        ));
        *order = order
            .checked_add(1)
            .expect("Rust runtime helper order fits u16");
        true
    };

    for line in RUNTIME.split_inclusive('\n') {
        let marker = line.trim().trim_end_matches('\r');
        if let Some(id) = marker.strip_prefix(BEGIN) {
            if active.is_some() {
                return Err(rust_generation_error(format!(
                    "nested Rust runtime helper marker {id:?}"
                )));
            }
            let common_id = format!("runtime.common.{common_index:03}");
            if emit(common_id.clone(), &mut source, &mut order, &mut helpers) {
                common_roots.push(common_id);
                common_index += 1;
            }
            active = Some(id.to_owned());
        } else if let Some(id) = marker.strip_prefix(END) {
            let Some(open) = active.take() else {
                return Err(rust_generation_error(format!(
                    "unmatched Rust runtime helper end marker {id:?}"
                )));
            };
            if open != id {
                return Err(rust_generation_error(format!(
                    "Rust runtime helper marker {open:?} closed by {id:?}"
                )));
            }
            if !emit(open, &mut source, &mut order, &mut helpers) {
                return Err(rust_generation_error(format!(
                    "empty Rust runtime helper {id:?}"
                )));
            }
        } else {
            source.push_str(line);
        }
    }
    if let Some(open) = active {
        return Err(rust_generation_error(format!(
            "unclosed Rust runtime helper marker {open:?}"
        )));
    }
    let common_id = format!("runtime.common.{common_index:03}");
    if emit(common_id.clone(), &mut source, &mut order, &mut helpers) {
        common_roots.push(common_id);
    }

    for (index, (id, dependency)) in [
        ("feature.string-replace-many", "string-replace-many"),
        ("feature.bytes-replace-all", "bytes-replace-all"),
        ("feature.string-truncate-utf8", "string-truncate-utf8"),
        ("feature.checked-shift", "checked-shift"),
    ]
    .into_iter()
    .enumerate()
    {
        helpers.push(RuntimeHelper::new(
            id,
            u16::MAX - 8 + u16::try_from(index).expect("feature count fits u16"),
            RustCode::default()
                .with_helper_root(dependency)
                .into_fragment(),
        ));
    }
    Ok((
        RuntimeHelperGraph::new(helpers).map_err(rust_generation_error)?,
        common_roots,
    ))
}

fn rust_conformance_file() -> LanguageSourceFile<RustImport> {
    let mut file = LanguageSourceFile::new("src/conformance.rs", SourceFileRole::Conformance);
    file.set_body(
        render_conformance()
            .with_import(
                RustImport::use_path("super::*", false).expect("static Rust use path is valid"),
            )
            .into_fragment(),
    );
    file
}

struct Generator<'a> {
    program: &'a CheckedProgram,
    declarations: BTreeMap<NodeId, &'a Declaration>,
}

impl<'a> Generator<'a> {
    fn new(program: &'a CheckedProgram) -> Self {
        Self {
            program,
            declarations: program
                .module()
                .declarations
                .iter()
                .map(|declaration| (declaration.header().node.id, declaration))
                .collect(),
        }
    }

    fn source_file(&self) -> Result<LanguageSourceFile<RustImport>, BackendError> {
        let mut file = LanguageSourceFile::new("src/lib.rs", SourceFileRole::Source);
        file.set_preamble(
            RustCode::new(
                "#![forbid(unsafe_code)]\n#![allow(non_camel_case_types, non_snake_case, non_upper_case_globals)]\n#![allow(clippy::unnecessary_wraps)]\n\n// Generated by PolyRust from checked IR v0.",
            )
            .into_fragment(),
        );
        let dependencies = RustCode::default()
            .with_import(
                RustImport::module("polyrust_runtime", false).expect("static Rust module is valid"),
            )
            .with_import(
                RustImport::use_path("polyrust_runtime::*", true)
                    .expect("static Rust use path is valid"),
            )
            .with_import(
                RustImport::module("conformance", true).expect("static Rust module is valid"),
            );
        let mut declaration_fragments = Vec::new();

        let mut declarations: Vec<_> = self.program.module().declarations.iter().collect();
        declarations.sort_by_key(|declaration| declaration.header().node.id);
        for declaration in declarations {
            let mut source = String::new();
            let mut requirements = Vec::new();
            match declaration {
                Declaration::Constant(constant) => {
                    let documentation = self.documentation(&constant.header.documentation, 0);
                    source.push_str(&documentation.text);
                    requirements.push(documentation);
                    let ty = self.ty(&constant.ty);
                    let body = self.constant_body(&constant.value, 0);
                    source.push_str(&format!(
                        "{}fn {}() -> PolyResult<{}> {}\n\n",
                        visibility(constant.header.visibility),
                        value_name(&constant.header.name),
                        ty,
                        body,
                    ));
                    requirements.extend([ty, body]);
                }
                Declaration::Alias(alias) => {
                    let documentation = self.documentation(&alias.header.documentation, 0);
                    source.push_str(&documentation.text);
                    requirements.push(documentation);
                    let target = self.ty(&alias.target);
                    source.push_str(&format!(
                        "{}type {} = {};\n\n",
                        visibility(alias.header.visibility),
                        type_name(&alias.header.name),
                        target,
                    ));
                    requirements.push(target);
                }
                Declaration::Record(record) => {
                    let documentation = self.documentation(&record.header.documentation, 0);
                    source.push_str(&documentation.text);
                    requirements.push(documentation);
                    source.push_str("#[derive(Clone, Debug, PartialEq)]\n");
                    source.push_str(&format!(
                        "{}struct {} {{\n",
                        visibility(record.header.visibility),
                        type_name(&record.header.name)
                    ));
                    for field in &record.fields {
                        let documentation = self.documentation(&field.header.documentation, 1);
                        source.push_str(&documentation.text);
                        requirements.push(documentation);
                        let ty = self.ty(&field.ty);
                        source.push_str(&format!(
                            "    pub {}: {},\n",
                            value_name(&field.header.name),
                            ty
                        ));
                        requirements.push(ty);
                    }
                    source.push_str("}\n\n");
                }
                Declaration::Enum(enumeration) => {
                    let documentation = self.documentation(&enumeration.header.documentation, 0);
                    source.push_str(&documentation.text);
                    requirements.push(documentation);
                    source.push_str("#[derive(Clone, Debug, PartialEq)]\n");
                    source.push_str(&format!(
                        "{}enum {} {{\n",
                        visibility(enumeration.header.visibility),
                        type_name(&enumeration.header.name)
                    ));
                    for variant in &enumeration.variants {
                        let documentation = self.documentation(&variant.header.documentation, 1);
                        source.push_str(&documentation.text);
                        requirements.push(documentation);
                        source.push_str(&format!("    {}", type_name(&variant.header.name)));
                        if variant.fields.is_empty() {
                            source.push_str(",\n");
                        } else {
                            source.push_str(" {\n");
                            for field in &variant.fields {
                                let ty = self.ty(&field.ty);
                                source.push_str(&format!(
                                    "        {}: {},\n",
                                    value_name(&field.header.name),
                                    ty
                                ));
                                requirements.push(ty);
                            }
                            source.push_str("    },\n");
                        }
                    }
                    source.push_str("}\n\n");
                }
                Declaration::Interface(interface) => {
                    let documentation = self.documentation(&interface.header.documentation, 0);
                    source.push_str(&documentation.text);
                    requirements.push(documentation);
                    source.push_str(&format!(
                        "{}trait {} {{\n",
                        visibility(interface.header.visibility),
                        type_name(&interface.header.name)
                    ));
                    for method in &interface.methods {
                        let documentation = self.documentation(&method.header.documentation, 1);
                        source.push_str(&documentation.text);
                        requirements.push(documentation);
                        let parameters = self.parameters(&method.parameters, true);
                        let return_type = self.ty(&method.return_type);
                        source.push_str(&format!(
                            "    fn {}(&self{}) -> PolyResult<{}>;\n",
                            value_name(&method.header.name),
                            parameters,
                            return_type
                        ));
                        requirements.extend([parameters, return_type]);
                    }
                    source.push_str("}\n\n");
                }
                Declaration::Implementation(implementation) => {
                    let interface = self.declaration_name(implementation.interface);
                    let record = self.declaration_name(implementation.record);
                    source.push_str(&format!(
                        "impl {} for {} {{\n",
                        type_name(interface),
                        type_name(record)
                    ));
                    for method in &implementation.methods {
                        let documentation = self.documentation(&method.header.documentation, 1);
                        source.push_str(&documentation.text);
                        requirements.push(documentation);
                        let parameters = self.parameters(&method.parameters, true);
                        let return_type = self.ty(&method.return_type);
                        let body = self.block(&method.body, 1);
                        source.push_str(&format!(
                            "    fn {}(&self{}) -> PolyResult<{}> {}\n",
                            value_name(&method.header.name),
                            parameters,
                            return_type,
                            body,
                        ));
                        requirements.extend([parameters, return_type, body]);
                    }
                    source.push_str("}\n\n");
                }
                Declaration::Function(function) => {
                    let documentation = self.documentation(&function.header.documentation, 0);
                    source.push_str(&documentation.text);
                    requirements.push(documentation);
                    let parameters = self.parameters(&function.parameters, false);
                    let return_type = self.ty(&function.return_type);
                    let body = self.block(&function.body, 0);
                    source.push_str(&format!(
                        "{}fn {}({}) -> PolyResult<{}> {}\n\n",
                        visibility(function.header.visibility),
                        value_name(&function.header.name),
                        parameters,
                        return_type,
                        body,
                    ));
                    requirements.extend([parameters, return_type, body]);
                }
                Declaration::Test(_) => {}
            }
            declaration_fragments.push(RustCode::new(source).with_text_from(requirements));
        }
        let tests = self.tests();
        let source = RustCode::sequence([RustCode::sequence(declaration_fragments), tests]);
        let document = CodeDocument::raw_text(RawText::new(&source.text));
        let body = render(
            &document,
            RenderOptions {
                final_newline: FinalNewline::Always,
                ..RenderOptions::default()
            },
        )
        .map_err(|error| BackendError::Generation {
            message: format!("Rust document rendering failed: {error}"),
        })?;
        file.set_body(
            RustCode::sequence([dependencies, RustCode::new(body).with_text_from([source])])
                .into_fragment(),
        );
        Ok(file)
    }

    fn documentation(&self, paragraphs: &[String], indent: usize) -> RustCode {
        let prefix = "    ".repeat(indent);
        let mut output = String::new();
        for paragraph in paragraphs {
            for line in paragraph.lines() {
                output.push_str(&format!("{prefix}/// {line}\n"));
            }
        }
        RustCode::new(output)
    }

    fn parameters(&self, parameters: &[Parameter], leading_comma: bool) -> RustCode {
        let rendered = RustCode::joined(
            parameters.iter().map(|parameter| {
                let ty = match &parameter.ty {
                    TypeRef::Interface(id) => {
                        RustCode::new(format!("&dyn {}", type_name(self.declaration_name(*id))))
                    }
                    other => self.ty(other),
                };
                ty.map_text(|ty| format!("{}: {ty}", value_name(&parameter.header.name)))
            }),
            ", ",
        );
        if leading_comma {
            rendered.map_text(|rendered| {
                if rendered.is_empty() {
                    rendered
                } else {
                    format!(", {rendered}")
                }
            })
        } else {
            rendered
        }
    }

    fn ty(&self, ty: &TypeRef) -> RustCode {
        match ty {
            TypeRef::Unit => RustCode::new("()"),
            TypeRef::Bool => RustCode::new("bool"),
            TypeRef::I32 => RustCode::new("i32"),
            TypeRef::I64 => RustCode::new("i64"),
            TypeRef::F64 => RustCode::new("f64"),
            TypeRef::Char => RustCode::new("char"),
            TypeRef::String => RustCode::new("String"),
            TypeRef::Bytes => RustCode::new("Vec<u8>"),
            TypeRef::List(element) => self
                .ty(element)
                .map_text(|element| format!("Vec<{element}>")),
            TypeRef::Option(inner) => self.ty(inner).map_text(|inner| format!("Option<{inner}>")),
            TypeRef::Result { ok, error } => {
                let ok = self.ty(ok);
                let error = self.ty(error);
                RustCode::new(format!("Result<{ok}, {error}>")).with_text_from([ok, error])
            }
            TypeRef::Named(id) | TypeRef::Interface(id) => {
                RustCode::new(type_name(self.declaration_name(*id)))
            }
        }
    }

    fn declaration_name(&self, id: NodeId) -> &str {
        self.declarations
            .get(&id)
            .map(|declaration| declaration.header().name.as_str())
            .unwrap_or("MissingDeclaration")
    }

    fn declaration(&self, id: NodeId) -> Option<&Declaration> {
        self.declarations.get(&id).copied()
    }

    fn field_name(&self, id: NodeId) -> &str {
        for declaration in self.declarations.values() {
            match declaration {
                Declaration::Record(record) => {
                    if let Some(field) = record
                        .fields
                        .iter()
                        .find(|field| field.header.node.id == id)
                    {
                        return &field.header.name;
                    }
                }
                Declaration::Enum(enumeration) => {
                    for variant in &enumeration.variants {
                        if let Some(field) = variant
                            .fields
                            .iter()
                            .find(|field| field.header.node.id == id)
                        {
                            return &field.header.name;
                        }
                    }
                }
                _ => {}
            }
        }
        "missing_field"
    }

    fn variant(&self, id: NodeId) -> Option<(&str, &EnumVariant)> {
        self.declarations.values().find_map(|declaration| {
            let Declaration::Enum(enumeration) = declaration else {
                return None;
            };
            enumeration
                .variants
                .iter()
                .find(|variant| variant.header.node.id == id)
                .map(|variant| (enumeration.header.name.as_str(), variant))
        })
    }
}

fn visibility(visibility: portable_ir::v0::Visibility) -> &'static str {
    match visibility {
        portable_ir::v0::Visibility::Public => "pub ",
        portable_ir::v0::Visibility::Package => "pub(crate) ",
    }
}

fn package_name(module: &str) -> String {
    format!("polyrust-generated-{}", module.replace('_', "-"))
}

fn type_name(name: &str) -> String {
    rust_identifier(name)
}

fn value_name(name: &str) -> String {
    rust_identifier(name)
}

fn rust_identifier(name: &str) -> String {
    const KEYWORDS: &[&str] = &[
        "as", "break", "const", "continue", "crate", "else", "enum", "extern", "false", "fn",
        "for", "if", "impl", "in", "let", "loop", "match", "mod", "move", "mut", "pub", "ref",
        "return", "static", "struct", "trait", "true", "type", "unsafe", "use", "where", "while",
        "async", "await", "dyn", "abstract", "become", "box", "do", "final", "macro", "override",
        "priv", "typeof", "unsized", "virtual", "yield", "try", "gen",
    ];
    match name {
        "self" | "Self" | "super" | "crate" => format!("{name}_"),
        _ if KEYWORDS.contains(&name) => format!("r#{name}"),
        _ => name.to_owned(),
    }
}

fn test_name(name: &str, id: NodeId) -> String {
    let mut result = String::new();
    for character in name.chars() {
        if character.is_ascii_alphanumeric() {
            result.push(character.to_ascii_lowercase());
        } else if !result.ends_with('_') {
            result.push('_');
        }
    }
    format!("{}_n{}", result.trim_matches('_'), id.0)
}

impl Generator<'_> {
    fn block(&self, block: &Block, indent: usize) -> RustCode {
        let prefix = "    ".repeat(indent);
        let inner = "    ".repeat(indent + 1);
        let mut output = String::from("{\n");
        let mut dependencies = Vec::new();
        for statement in &block.statements {
            match statement {
                Statement::Let {
                    name,
                    annotation,
                    value,
                    ..
                } => {
                    let annotation = annotation.as_ref().map_or_else(RustCode::default, |ty| {
                        self.ty(ty).map_text(|ty| format!(": {ty}"))
                    });
                    let value = self.expr(value, indent + 1);
                    output.push_str(&format!(
                        "{inner}let {}{annotation} = ({})?;\n",
                        value_name(name),
                        value
                    ));
                    dependencies.push(annotation);
                    dependencies.push(value);
                }
                Statement::ForEach {
                    binding,
                    iterable,
                    body,
                    ..
                } => {
                    let iterable = self.expr(iterable, indent + 1);
                    let body = self.block(body, indent + 1);
                    output.push_str(&format!(
                        "{inner}for {} in ({})? {}\n",
                        value_name(binding),
                        iterable,
                        body
                    ));
                    dependencies.push(iterable);
                    dependencies.push(body);
                }
                Statement::Return { value, .. } => match value {
                    Some(value) => {
                        let value = self.expr(value, indent + 1);
                        output.push_str(&format!("{inner}return {value};\n"));
                        dependencies.push(value);
                    }
                    None => output.push_str(&format!("{inner}return Ok(());\n")),
                },
                Statement::Expression { value, .. } => {
                    let value = self.expr(value, indent + 1);
                    output.push_str(&format!("{inner}let _ = ({value})?;\n"));
                    dependencies.push(value);
                }
            }
        }
        match &block.result {
            Some(result) => {
                let result = self.expr(result, indent + 1);
                output.push_str(&format!("{inner}{result}\n"));
                dependencies.push(result);
            }
            None => output.push_str(&format!("{inner}Ok(())\n")),
        }
        output.push_str(&format!("{prefix}}}"));
        RustCode::new(output).with_text_from(dependencies)
    }

    fn expr(&self, expression: &Expression, indent: usize) -> RustCode {
        match expression {
            Expression::Literal { value, .. } => {
                self.value(value).map_text(|value| format!("Ok({value})"))
            }
            Expression::Local { node, name } => {
                let value = value_name(name);
                if self
                    .program
                    .expression_type(node.id)
                    .is_some_and(|ty| self.is_copy(ty))
                {
                    RustCode::new(format!("Ok({value})"))
                } else {
                    RustCode::new(format!("Ok({value}.clone())"))
                }
            }
            Expression::Constant { declaration, .. } => RustCode::new(format!(
                "{}()",
                value_name(self.declaration_name(*declaration))
            )),
            Expression::SelfValue { .. } => RustCode::new("Ok(self.clone())"),
            Expression::ConstructRecord {
                declaration,
                fields,
                ..
            } => RustCode::joined(
                fields.iter().map(|field| {
                    self.expr(&field.value, indent).map_text(|value| {
                        format!("{}: ({value})?", value_name(self.field_name(field.field)))
                    })
                }),
                ", ",
            )
            .map_text(|fields| {
                format!(
                    "Ok({} {{ {fields} }})",
                    type_name(self.declaration_name(*declaration))
                )
            }),
            Expression::ConstructEnum {
                declaration,
                variant,
                fields,
                ..
            } => {
                let variant_name = self
                    .variant(*variant)
                    .map_or("MissingVariant", |(_, variant)| {
                        variant.header.name.as_str()
                    });
                if fields.is_empty() {
                    RustCode::new(format!(
                        "Ok({}::{})",
                        type_name(self.declaration_name(*declaration)),
                        type_name(variant_name)
                    ))
                } else {
                    RustCode::joined(
                        fields.iter().map(|field| {
                            self.expr(&field.value, indent).map_text(|value| {
                                format!("{}: ({value})?", value_name(self.field_name(field.field)))
                            })
                        }),
                        ", ",
                    )
                    .map_text(|fields| {
                        format!(
                            "Ok({}::{} {{ {fields} }})",
                            type_name(self.declaration_name(*declaration)),
                            type_name(variant_name)
                        )
                    })
                }
            }
            Expression::ConstructSome { value, .. } => self
                .expr(value, indent)
                .map_text(|value| format!("Ok(Some(({value})?))")),
            Expression::ConstructNone { .. } => RustCode::new("Ok(None)"),
            Expression::ConstructOk { value, .. } => self
                .expr(value, indent)
                .map_text(|value| format!("Ok(Ok(({value})?))")),
            Expression::ConstructErr { value, .. } => self
                .expr(value, indent)
                .map_text(|value| format!("Ok(Err(({value})?))")),
            Expression::ConstructList { elements, .. } => RustCode::joined(
                elements.iter().map(|element| {
                    self.expr(element, indent)
                        .map_text(|value| format!("({value})?"))
                }),
                ", ",
            )
            .map_text(|elements| format!("Ok(vec![{elements}])")),
            Expression::CoerceInterface { .. } => unreachable!(
                "legacy Rust translation rejects first-class interface values during preflight"
            ),
            Expression::Field { base, field, .. } => {
                let copied = self.field_type(*field).is_some_and(|ty| self.is_copy(ty));
                self.expr(base, indent).map_text(|base| {
                    let access = format!("(({base})?).{}", value_name(self.field_name(*field)));
                    if copied {
                        format!("Ok({access})")
                    } else {
                        format!("Ok({access}.clone())")
                    }
                })
            }
            Expression::Call {
                function,
                arguments,
                ..
            } => self.call(
                value_name(self.declaration_name(*function)),
                None,
                arguments,
                indent,
            ),
            Expression::MethodCall {
                receiver,
                dispatch,
                arguments,
                ..
            } => self.method_call(receiver, dispatch, arguments, indent),
            Expression::Intrinsic {
                operation,
                arguments,
                ..
            } => {
                let first_type = arguments
                    .first()
                    .and_then(|argument| self.program.expression_type(argument.node().id));
                self.intrinsic(
                    *operation,
                    arguments
                        .iter()
                        .map(|argument| self.expr(argument, indent))
                        .collect(),
                    first_type,
                )
            }
            Expression::If {
                condition,
                then_block,
                else_block,
                ..
            } => {
                let condition = self.expr(condition, indent);
                let then_block = self.block(then_block, indent);
                let else_block = self.block(else_block, indent);
                RustCode::new(format!("if ({condition})? {then_block} else {else_block}"))
                    .with_text_from([condition, then_block, else_block])
            }
            Expression::Match { value, arms, .. } => {
                let value = self.expr(value, indent);
                let arms =
                    RustCode::sequence(arms.iter().map(|arm| self.match_arm(arm, indent + 1)));
                RustCode::new(format!(
                    "match ({value})? {{\n{arms}{} }}",
                    "    ".repeat(indent)
                ))
                .with_text_from([value, arms])
            }
            Expression::Block(block) => self.block(block, indent),
        }
    }

    fn call(
        &self,
        callable: String,
        receiver: Option<String>,
        arguments: &[Expression],
        indent: usize,
    ) -> RustCode {
        let mut output = String::from("{ ");
        let mut dependencies = Vec::new();
        if let Some(receiver) = receiver {
            output.push_str(&format!("let __receiver = ({receiver})?; "));
        }
        for (index, argument) in arguments.iter().enumerate() {
            let argument = self.expr(argument, indent);
            output.push_str(&format!("let __argument_{index} = ({argument})?; "));
            dependencies.push(argument);
        }
        let args = (0..arguments.len())
            .map(|index| format!("__argument_{index}"))
            .collect::<Vec<_>>()
            .join(", ");
        output.push_str(&format!("{callable}({args}) }}"));
        RustCode::new(output).with_text_from(dependencies)
    }

    fn method_call(
        &self,
        receiver: &Expression,
        dispatch: &MethodDispatch,
        arguments: &[Expression],
        indent: usize,
    ) -> RustCode {
        let receiver_result = self.expr(receiver, indent);
        let mut prefix = format!("{{ let __receiver = ({receiver_result})?; ");
        let mut dependencies = vec![receiver_result];
        for (index, argument) in arguments.iter().enumerate() {
            let argument = self.expr(argument, indent);
            prefix.push_str(&format!("let __argument_{index} = ({argument})?; "));
            dependencies.push(argument);
        }
        let args = (0..arguments.len())
            .map(|index| format!("__argument_{index}"))
            .collect::<Vec<_>>()
            .join(", ");
        let suffix = if args.is_empty() {
            String::new()
        } else {
            format!(", {args}")
        };
        let call = match dispatch {
            MethodDispatch::Interface { interface, method } => {
                let method_name = self.interface_method_name(*interface, *method);
                format!("__receiver.{}({args})", value_name(method_name))
            }
            MethodDispatch::Concrete {
                implementation,
                method,
            } => {
                let (interface, record, method_name) =
                    self.implementation_method(*implementation, *method);
                format!(
                    "<{} as {}>::{}(&__receiver{suffix})",
                    type_name(record),
                    type_name(interface),
                    value_name(method_name)
                )
            }
        };
        prefix.push_str(&call);
        prefix.push_str(" }");
        RustCode::new(prefix).with_text_from(dependencies)
    }

    fn match_arm(&self, arm: &MatchArm, indent: usize) -> RustCode {
        let pattern = self.pattern(&arm.pattern);
        let body = self.block(&arm.body, indent);
        RustCode::new(format!("{}{pattern} => {body},\n", "    ".repeat(indent)))
            .with_text_from([pattern, body])
    }

    fn pattern(&self, pattern: &Pattern) -> RustCode {
        match pattern {
            Pattern::Wildcard { .. } => RustCode::new("_"),
            Pattern::Bool { value, .. } => RustCode::new(value.to_string()),
            Pattern::None { .. } => RustCode::new("None"),
            Pattern::Some { binding, .. } => {
                RustCode::new(format!("Some({})", value_name(binding)))
            }
            Pattern::Ok { binding, .. } => RustCode::new(format!("Ok({})", value_name(binding))),
            Pattern::Err { binding, .. } => RustCode::new(format!("Err({})", value_name(binding))),
            Pattern::EnumVariant {
                declaration,
                variant,
                bindings,
                ..
            } => {
                let variant_name = self
                    .variant(*variant)
                    .map_or("MissingVariant", |(_, variant)| {
                        variant.header.name.as_str()
                    });
                if bindings.is_empty() {
                    RustCode::new(format!(
                        "{}::{}",
                        type_name(self.declaration_name(*declaration)),
                        type_name(variant_name)
                    ))
                } else {
                    RustCode::new(format!(
                        "{}::{} {{ {} }}",
                        type_name(self.declaration_name(*declaration)),
                        type_name(variant_name),
                        bindings
                            .iter()
                            .map(|binding| format!(
                                "{}: {}",
                                value_name(self.field_name(binding.field)),
                                value_name(&binding.binding)
                            ))
                            .collect::<Vec<_>>()
                            .join(", ")
                    ))
                }
            }
        }
    }

    fn is_copy(&self, ty: &TypeRef) -> bool {
        match ty {
            TypeRef::Unit
            | TypeRef::Bool
            | TypeRef::I32
            | TypeRef::I64
            | TypeRef::F64
            | TypeRef::Char
            | TypeRef::Interface(_) => true,
            TypeRef::Named(id) => match self.declaration(*id) {
                Some(Declaration::Alias(alias)) => self.is_copy(&alias.target),
                _ => false,
            },
            TypeRef::String
            | TypeRef::Bytes
            | TypeRef::List(_)
            | TypeRef::Option(_)
            | TypeRef::Result { .. } => false,
        }
    }

    fn field_type(&self, id: NodeId) -> Option<&TypeRef> {
        for declaration in self.declarations.values() {
            match declaration {
                Declaration::Record(record) => {
                    if let Some(field) = record
                        .fields
                        .iter()
                        .find(|field| field.header.node.id == id)
                    {
                        return Some(&field.ty);
                    }
                }
                Declaration::Enum(enumeration) => {
                    for variant in &enumeration.variants {
                        if let Some(field) = variant
                            .fields
                            .iter()
                            .find(|field| field.header.node.id == id)
                        {
                            return Some(&field.ty);
                        }
                    }
                }
                _ => {}
            }
        }
        None
    }

    fn interface_method_name(&self, interface: NodeId, method: NodeId) -> &str {
        let Some(Declaration::Interface(interface)) = self.declaration(interface) else {
            return "missing_method";
        };
        interface
            .methods
            .iter()
            .find(|candidate| candidate.header.node.id == method)
            .map_or("missing_method", |method| method.header.name.as_str())
    }

    fn implementation_method(&self, implementation: NodeId, method: NodeId) -> (&str, &str, &str) {
        let Some(Declaration::Implementation(implementation)) = self.declaration(implementation)
        else {
            return ("MissingInterface", "MissingRecord", "missing_method");
        };
        let method = implementation
            .methods
            .iter()
            .find(|candidate| {
                candidate.header.node.id == method || candidate.interface_method == method
            })
            .map_or("missing_method", |method| method.header.name.as_str());
        (
            self.declaration_name(implementation.interface),
            self.declaration_name(implementation.record),
            method,
        )
    }
}

impl Generator<'_> {
    fn constant_body(&self, expression: &ConstantExpression, _indent: usize) -> RustCode {
        self.constant_expr(expression)
            .map_text(|expression| format!("{{ {expression} }}"))
    }

    fn constant_expr(&self, expression: &ConstantExpression) -> RustCode {
        match expression {
            ConstantExpression::Literal { value, .. } => {
                self.value(value).map_text(|value| format!("Ok({value})"))
            }
            ConstantExpression::Reference { declaration, .. } => RustCode::new(format!(
                "{}()",
                value_name(self.declaration_name(*declaration))
            )),
            ConstantExpression::Record {
                declaration,
                fields,
                ..
            } => RustCode::joined(
                fields.iter().map(|field| {
                    self.constant_expr(&field.value).map_text(|value| {
                        format!("{}: ({value})?", value_name(self.field_name(field.field)))
                    })
                }),
                ", ",
            )
            .map_text(|fields| {
                format!(
                    "Ok({} {{ {fields} }})",
                    type_name(self.declaration_name(*declaration))
                )
            }),
            ConstantExpression::Enum {
                declaration,
                variant,
                fields,
                ..
            } => {
                let variant_name = self
                    .variant(*variant)
                    .map_or("MissingVariant", |(_, variant)| {
                        variant.header.name.as_str()
                    });
                if fields.is_empty() {
                    RustCode::new(format!(
                        "Ok({}::{})",
                        type_name(self.declaration_name(*declaration)),
                        type_name(variant_name)
                    ))
                } else {
                    RustCode::joined(
                        fields.iter().map(|field| {
                            self.constant_expr(&field.value).map_text(|value| {
                                format!("{}: ({value})?", value_name(self.field_name(field.field)))
                            })
                        }),
                        ", ",
                    )
                    .map_text(|fields| {
                        format!(
                            "Ok({}::{} {{ {fields} }})",
                            type_name(self.declaration_name(*declaration)),
                            type_name(variant_name)
                        )
                    })
                }
            }
            ConstantExpression::Some { value, .. } => self
                .constant_expr(value)
                .map_text(|value| format!("Ok(Some(({value})?))")),
            ConstantExpression::None { .. } => RustCode::new("Ok(None)"),
            ConstantExpression::Ok { value, .. } => self
                .constant_expr(value)
                .map_text(|value| format!("Ok(Ok(({value})?))")),
            ConstantExpression::Err { value, .. } => self
                .constant_expr(value)
                .map_text(|value| format!("Ok(Err(({value})?))")),
            ConstantExpression::List { elements, .. } => RustCode::joined(
                elements.iter().map(|element| {
                    self.constant_expr(element)
                        .map_text(|value| format!("({value})?"))
                }),
                ", ",
            )
            .map_text(|elements| format!("Ok(vec![{elements}])")),
            ConstantExpression::Intrinsic {
                operation,
                arguments,
                ..
            } => {
                let first_type = arguments
                    .first()
                    .and_then(|argument| self.constant_type(argument));
                self.intrinsic(
                    *operation,
                    arguments
                        .iter()
                        .map(|argument| self.constant_expr(argument))
                        .collect(),
                    first_type.as_ref(),
                )
            }
        }
    }

    fn intrinsic(
        &self,
        operation: Intrinsic,
        arguments: Vec<RustCode>,
        first_type: Option<&TypeRef>,
    ) -> RustCode {
        if operation == Intrinsic::BoolAnd {
            return RustCode::new(format!(
                "{{ let __argument_0 = ({})?; if !__argument_0 {{ Ok(false) }} else {{ {} }} }}",
                arguments[0], arguments[1]
            ))
            .with_text_from(arguments);
        }
        if operation == Intrinsic::BoolOr {
            return RustCode::new(format!(
                "{{ let __argument_0 = ({})?; if __argument_0 {{ Ok(true) }} else {{ {} }} }}",
                arguments[0], arguments[1]
            ))
            .with_text_from(arguments);
        }
        let mut output = String::from("{ ");
        for (index, argument) in arguments.iter().enumerate() {
            output.push_str(&format!("let __argument_{index} = ({argument})?; "));
        }
        let a = "__argument_0";
        let b = "__argument_1";
        let c = "__argument_2";
        let width = match first_type {
            Some(TypeRef::I32) => 32,
            _ => 64,
        };
        let expression = match operation {
            Intrinsic::BoolNot => format!("Ok(!{a})"),
            Intrinsic::BoolAnd | Intrinsic::BoolOr => unreachable!(),
            Intrinsic::Equal => format!("Ok({a} == {b})"),
            Intrinsic::NotEqual => format!("Ok({a} != {b})"),
            Intrinsic::Less => format!("Ok({a} < {b})"),
            Intrinsic::LessEqual => format!("Ok({a} <= {b})"),
            Intrinsic::Greater => format!("Ok({a} > {b})"),
            Intrinsic::GreaterEqual => format!("Ok({a} >= {b})"),
            Intrinsic::IntNegChecked => format!(
                "{a}.checked_neg().ok_or(PolyRuntimeError::CheckedOverflow {{ operation: \"neg\" }})"
            ),
            Intrinsic::IntAddChecked => format!(
                "{a}.checked_add({b}).ok_or(PolyRuntimeError::CheckedOverflow {{ operation: \"add\" }})"
            ),
            Intrinsic::IntSubChecked => format!(
                "{a}.checked_sub({b}).ok_or(PolyRuntimeError::CheckedOverflow {{ operation: \"sub\" }})"
            ),
            Intrinsic::IntMulChecked => format!(
                "{a}.checked_mul({b}).ok_or(PolyRuntimeError::CheckedOverflow {{ operation: \"mul\" }})"
            ),
            Intrinsic::IntDivChecked => format!(
                "if {b} == 0 {{ Err(PolyRuntimeError::DivisionByZero) }} else {{ {a}.checked_div({b}).ok_or(PolyRuntimeError::CheckedOverflow {{ operation: \"div\" }}) }}"
            ),
            Intrinsic::IntRemChecked => format!(
                "if {b} == 0 {{ Err(PolyRuntimeError::RemainderByZero) }} else {{ {a}.checked_rem({b}).ok_or(PolyRuntimeError::CheckedOverflow {{ operation: \"rem\" }}) }}"
            ),
            Intrinsic::IntNegWrapping => format!("Ok({a}.wrapping_neg())"),
            Intrinsic::IntAddWrapping => format!("Ok({a}.wrapping_add({b}))"),
            Intrinsic::IntSubWrapping => format!("Ok({a}.wrapping_sub({b}))"),
            Intrinsic::IntMulWrapping => format!("Ok({a}.wrapping_mul({b}))"),
            Intrinsic::IntBitNot => format!("Ok(!{a})"),
            Intrinsic::IntBitAnd => format!("Ok({a} & {b})"),
            Intrinsic::IntBitOr => format!("Ok({a} | {b})"),
            Intrinsic::IntBitXor => format!("Ok({a} ^ {b})"),
            Intrinsic::IntShiftLeftChecked => format!("_poly_shift_left({a}, {b} as i64, {width})"),
            Intrinsic::IntShiftRightChecked => {
                format!("_poly_shift_right({a}, {b} as i64, {width})")
            }
            Intrinsic::FloatNeg => format!("Ok(-{a})"),
            Intrinsic::FloatTrunc => format!("Ok({a}.trunc())"),
            Intrinsic::FloatIsNaN => format!("Ok({a}.is_nan())"),
            Intrinsic::FloatIsNegativeZero => {
                format!("Ok({a}.to_bits() == (-0.0_f64).to_bits())")
            }
            Intrinsic::FloatAbs => {
                format!("Ok(f64::from_bits({a}.to_bits() & 0x7fff_ffff_ffff_ffff))")
            }
            Intrinsic::FloatAdd => format!("Ok({a} + {b})"),
            Intrinsic::FloatSub => format!("Ok({a} - {b})"),
            Intrinsic::FloatMul => format!("Ok({a} * {b})"),
            Intrinsic::FloatDiv => format!("Ok({a} / {b})"),
            Intrinsic::FloatRemTrunc => format!("Ok({a} % {b})"),
            Intrinsic::StringConcat => {
                format!("{{ let mut value = {a}; value.push_str(&{b}); Ok(value) }}")
            }
            Intrinsic::StringScalarLength => format!("Ok({a}.chars().count() as i64)"),
            Intrinsic::StringUtf16Length => format!("Ok({a}.encode_utf16().count() as i64)"),
            Intrinsic::StringIndexOfLiteral => format!(
                "Ok({a}.find({b}.as_str()).map(|byte_index| {a}[..byte_index].chars().count() as i64))"
            ),
            Intrinsic::StringSliceScalars => format!(
                "{{ let scalar_len = {a}.chars().count() as i64; let start = {b}.clamp(0, scalar_len); let end = {c}.clamp(0, scalar_len); if start >= end {{ Ok(String::new()) }} else {{ Ok({a}.chars().skip(start as usize).take((end - start) as usize).collect::<String>()) }} }}"
            ),
            Intrinsic::StringIsEmpty => format!("Ok({a}.is_empty())"),
            Intrinsic::StringContains => format!("Ok({a}.contains({b}.as_str()))"),
            Intrinsic::StringStartsWith => format!("Ok({a}.starts_with({b}.as_str()))"),
            Intrinsic::StringStripPrefix => {
                format!("Ok({a}.strip_prefix({b}.as_str()).unwrap_or({a}.as_str()).to_owned())")
            }
            Intrinsic::StringEndsWith => format!("Ok({a}.ends_with({b}.as_str()))"),
            Intrinsic::StringReplaceAll => {
                format!("Ok({a}.replace({b}.as_str(), {c}.as_str()))")
            }
            Intrinsic::StringReplaceMany => {
                let mappings = (1..arguments.len())
                    .step_by(2)
                    .map(|index| {
                        format!(
                            "(__argument_{index}.as_str(), __argument_{}.as_str())",
                            index + 1
                        )
                    })
                    .collect::<Vec<_>>()
                    .join(", ");
                format!("_poly_string_replace_many({a}, &[{mappings}])")
            }
            Intrinsic::StringTruncateUtf8Bytes => {
                format!("_poly_string_truncate_utf8_bytes({a}, {b})")
            }
            Intrinsic::StringTrimStart => {
                format!(
                    "Ok({a}.trim_start_matches(|character| {b}.contains(character)).to_owned())"
                )
            }
            Intrinsic::StringTrimEnd => {
                format!("Ok({a}.trim_end_matches(|character| {b}.contains(character)).to_owned())")
            }
            Intrinsic::BytesConcat | Intrinsic::ListConcat => {
                format!("{{ let mut value = {a}; value.extend({b}); Ok(value) }}")
            }
            Intrinsic::BytesReplaceAll => format!("_poly_bytes_replace_all({a}, {b}, {c})"),
            Intrinsic::BytesLength | Intrinsic::ListLength => format!("Ok({a}.len() as i64)"),
            Intrinsic::BytesIsEmpty | Intrinsic::ListIsEmpty => format!("Ok({a}.is_empty())"),
            Intrinsic::ListGetChecked => format!("_poly_list_get({a}, {b})"),
            Intrinsic::ListAppend => {
                format!("{{ let mut value = {a}; value.push({b}); Ok(value) }}")
            }
            Intrinsic::ListContains => format!("Ok({a}.contains(&{b}))"),
            Intrinsic::ListIndexOf => {
                format!("Ok({a}.iter().position(|item| item == &{b}).map(|index| index as i64))")
            }
            Intrinsic::OptionIsSome => format!("Ok({a}.is_some())"),
            Intrinsic::OptionIsNone => format!("Ok({a}.is_none())"),
            Intrinsic::OptionUnwrapOr => format!("Ok({a}.unwrap_or({b}))"),
            Intrinsic::ResultIsOk => format!("Ok({a}.is_ok())"),
            Intrinsic::ResultIsErr => format!("Ok({a}.is_err())"),
            Intrinsic::WidenI32ToI64 => format!("Ok(i64::from({a}))"),
            Intrinsic::NarrowI64ToI32Checked => format!(
                "i32::try_from({a}).map_err(|_| PolyRuntimeError::NarrowingOutOfRange {{ value: {a} }})"
            ),
            Intrinsic::StringToUtf8 => format!("Ok({a}.into_bytes())"),
            Intrinsic::StringFromUtf8Checked => {
                format!("String::from_utf8({a}).map_err(|_| PolyRuntimeError::InvalidUtf8)")
            }
        };
        output.push_str(&expression);
        output.push_str(" }");
        let mut code = RustCode::new(output).with_text_from(arguments);
        let helper = match operation {
            Intrinsic::StringReplaceMany => Some("feature.string-replace-many"),
            Intrinsic::BytesReplaceAll => Some("feature.bytes-replace-all"),
            Intrinsic::StringTruncateUtf8Bytes => Some("feature.string-truncate-utf8"),
            Intrinsic::IntShiftLeftChecked | Intrinsic::IntShiftRightChecked => {
                Some("feature.checked-shift")
            }
            _ => None,
        };
        if let Some(helper) = helper {
            code = code.with_helper_root(helper);
        }
        code
    }

    fn constant_type(&self, expression: &ConstantExpression) -> Option<TypeRef> {
        match expression {
            ConstantExpression::Literal { value, .. } => self.value_type(value),
            ConstantExpression::Reference { declaration, .. } => {
                let Some(Declaration::Constant(constant)) = self.declaration(*declaration) else {
                    return None;
                };
                Some(constant.ty.clone())
            }
            ConstantExpression::Record { declaration, .. }
            | ConstantExpression::Enum { declaration, .. } => Some(TypeRef::Named(*declaration)),
            ConstantExpression::Some { value, .. } => self
                .constant_type(value)
                .map(|ty| TypeRef::Option(Box::new(ty))),
            ConstantExpression::None { inner_type, .. } => {
                Some(TypeRef::Option(Box::new(inner_type.clone())))
            }
            ConstantExpression::Ok {
                value, error_type, ..
            } => self.constant_type(value).map(|ok| TypeRef::Result {
                ok: Box::new(ok),
                error: Box::new(error_type.clone()),
            }),
            ConstantExpression::Err { value, ok_type, .. } => {
                self.constant_type(value).map(|error| TypeRef::Result {
                    ok: Box::new(ok_type.clone()),
                    error: Box::new(error),
                })
            }
            ConstantExpression::List { element_type, .. } => {
                Some(TypeRef::List(Box::new(element_type.clone())))
            }
            ConstantExpression::Intrinsic {
                operation,
                arguments,
                ..
            } => {
                let first = arguments
                    .first()
                    .and_then(|argument| self.constant_type(argument))?;
                match operation {
                    Intrinsic::BoolNot
                    | Intrinsic::BoolAnd
                    | Intrinsic::BoolOr
                    | Intrinsic::Equal
                    | Intrinsic::NotEqual
                    | Intrinsic::Less
                    | Intrinsic::LessEqual
                    | Intrinsic::Greater
                    | Intrinsic::GreaterEqual
                    | Intrinsic::StringIsEmpty
                    | Intrinsic::StringContains
                    | Intrinsic::StringStartsWith
                    | Intrinsic::StringEndsWith
                    | Intrinsic::BytesIsEmpty
                    | Intrinsic::ListIsEmpty
                    | Intrinsic::ListContains
                    | Intrinsic::OptionIsSome
                    | Intrinsic::OptionIsNone
                    | Intrinsic::ResultIsOk
                    | Intrinsic::ResultIsErr => Some(TypeRef::Bool),
                    Intrinsic::StringScalarLength
                    | Intrinsic::StringUtf16Length
                    | Intrinsic::BytesLength
                    | Intrinsic::ListLength
                    | Intrinsic::WidenI32ToI64 => Some(TypeRef::I64),
                    Intrinsic::NarrowI64ToI32Checked => Some(TypeRef::I32),
                    Intrinsic::StringToUtf8 | Intrinsic::BytesReplaceAll => Some(TypeRef::Bytes),
                    Intrinsic::FloatAbs
                    | Intrinsic::StringFromUtf8Checked
                    | Intrinsic::StringConcat
                    | Intrinsic::StringReplaceAll
                    | Intrinsic::StringReplaceMany
                    | Intrinsic::StringSliceScalars
                    | Intrinsic::StringTruncateUtf8Bytes
                    | Intrinsic::StringStripPrefix
                    | Intrinsic::StringTrimStart
                    | Intrinsic::StringTrimEnd => Some(TypeRef::String),
                    Intrinsic::ListGetChecked => match first {
                        TypeRef::List(element) => Some(*element),
                        _ => None,
                    },
                    Intrinsic::StringIndexOfLiteral | Intrinsic::ListIndexOf => {
                        Some(TypeRef::Option(Box::new(TypeRef::I64)))
                    }
                    Intrinsic::OptionUnwrapOr => match first {
                        TypeRef::Option(inner) => Some(*inner),
                        _ => None,
                    },
                    _ => Some(first),
                }
            }
        }
    }

    fn value_type(&self, value: &Value) -> Option<TypeRef> {
        match value {
            Value::Unit => Some(TypeRef::Unit),
            Value::Bool(_) => Some(TypeRef::Bool),
            Value::I32(_) => Some(TypeRef::I32),
            Value::I64(_) => Some(TypeRef::I64),
            Value::F64(_) => Some(TypeRef::F64),
            Value::Char(_) => Some(TypeRef::Char),
            Value::String(_) => Some(TypeRef::String),
            Value::Bytes(_) => Some(TypeRef::Bytes),
            Value::List(values) => values
                .first()
                .and_then(|value| self.value_type(value))
                .map(|ty| TypeRef::List(Box::new(ty))),
            Value::Some(value) => self
                .value_type(value)
                .map(|ty| TypeRef::Option(Box::new(ty))),
            Value::Ok(_) | Value::Err(_) | Value::None => None,
            Value::Record { declaration, .. } | Value::Enum { declaration, .. } => {
                Some(TypeRef::Named(*declaration))
            }
        }
    }

    fn value(&self, value: &Value) -> RustCode {
        match value {
            Value::Unit => RustCode::new("()"),
            Value::Bool(value) => RustCode::new(value.to_string()),
            Value::I32(value) => RustCode::new(format!("{value}_i32")),
            Value::I64(value) => RustCode::new(format!("{value}_i64")),
            Value::F64(value) => RustCode::new(format!("f64::from_bits(0x{:016x})", value.0)),
            Value::Char(value) => RustCode::new(format!("{value:?}")),
            Value::String(value) => RustCode::new(format!("String::from({value:?})")),
            Value::Bytes(value) => RustCode::new(format!(
                "vec![{}]",
                value
                    .iter()
                    .map(|byte| format!("0x{byte:02x}_u8"))
                    .collect::<Vec<_>>()
                    .join(", ")
            )),
            Value::List(values) => {
                RustCode::joined(values.iter().map(|value| self.value(value)), ", ")
                    .map_text(|values| format!("vec![{values}]"))
            }
            Value::None => RustCode::new("None"),
            Value::Some(value) => self.value(value).map_text(|value| format!("Some({value})")),
            Value::Ok(value) => self.value(value).map_text(|value| format!("Ok({value})")),
            Value::Err(value) => self.value(value).map_text(|value| format!("Err({value})")),
            Value::Record {
                declaration,
                fields,
            } => RustCode::joined(
                fields.iter().map(|field| {
                    self.value(&field.value).map_text(|value| {
                        format!("{}: {value}", value_name(self.field_name(field.field)))
                    })
                }),
                ", ",
            )
            .map_text(|fields| {
                format!(
                    "{} {{ {fields} }}",
                    type_name(self.declaration_name(*declaration))
                )
            }),
            Value::Enum {
                declaration,
                variant,
                fields,
            } => {
                let variant_name = self
                    .variant(*variant)
                    .map_or("MissingVariant", |(_, variant)| {
                        variant.header.name.as_str()
                    });
                if fields.is_empty() {
                    RustCode::new(format!(
                        "{}::{}",
                        type_name(self.declaration_name(*declaration)),
                        type_name(variant_name)
                    ))
                } else {
                    RustCode::joined(
                        fields.iter().map(|field| {
                            self.value(&field.value).map_text(|value| {
                                format!("{}: {value}", value_name(self.field_name(field.field)))
                            })
                        }),
                        ", ",
                    )
                    .map_text(|fields| {
                        format!(
                            "{}::{} {{ {fields} }}",
                            type_name(self.declaration_name(*declaration)),
                            type_name(variant_name)
                        )
                    })
                }
            }
        }
    }
}

impl Generator<'_> {
    fn tests(&self) -> RustCode {
        let mut output = String::new();
        let mut dependencies = Vec::new();
        let mut tests: Vec<_> = self
            .program
            .module()
            .declarations
            .iter()
            .filter_map(|declaration| match declaration {
                Declaration::Test(test) => Some(test),
                _ => None,
            })
            .collect();
        tests.sort_by_key(|test| test.header.node.id);
        for test in tests {
            output.push_str("#[cfg(test)]\n");
            output.push_str("    #[test]\n");
            output.push_str(&format!(
                "    fn {}() {{\n",
                test_name(&test.header.name, test.header.node.id)
            ));
            let (call, return_type) = match &test.invocation {
                TestInvocation::Function {
                    function,
                    arguments,
                } => {
                    let parameters: &[Parameter] = match self.declaration(*function) {
                        Some(Declaration::Function(function)) => &function.parameters,
                        _ => &[],
                    };
                    let setup = self.test_arguments(arguments);
                    output.push_str(&setup.text);
                    dependencies.push(setup);
                    let argument_list = self.test_argument_list(arguments, parameters);
                    let call = format!(
                        "{}({argument_list})",
                        value_name(self.declaration_name(*function))
                    );
                    dependencies.push(argument_list);
                    (
                        call,
                        match self.declaration(*function) {
                            Some(Declaration::Function(function)) => function.return_type.clone(),
                            _ => TypeRef::Unit,
                        },
                    )
                }
                TestInvocation::Method {
                    implementation,
                    method,
                    receiver,
                    arguments,
                } => {
                    output.push_str(&format!(
                        "        let receiver = {};\n",
                        self.value(&receiver.value)
                    ));
                    let setup = self.test_arguments(arguments);
                    output.push_str(&setup.text);
                    dependencies.push(setup);
                    let (interface, record, method_name) =
                        self.implementation_method(*implementation, *method);
                    let parameters = match self.declaration(*implementation) {
                        Some(Declaration::Implementation(implementation)) => implementation
                            .methods
                            .iter()
                            .find(|candidate| {
                                candidate.header.node.id == *method
                                    || candidate.interface_method == *method
                            })
                            .map_or(&[][..], |method| method.parameters.as_slice()),
                        _ => &[],
                    };
                    let return_type = match self.declaration(*implementation) {
                        Some(Declaration::Implementation(implementation)) => implementation
                            .methods
                            .iter()
                            .find(|candidate| {
                                candidate.header.node.id == *method
                                    || candidate.interface_method == *method
                            })
                            .map_or(TypeRef::Unit, |method| method.return_type.clone()),
                        _ => TypeRef::Unit,
                    };
                    let arguments = self.test_argument_list(arguments, parameters);
                    let suffix = if arguments.text.is_empty() {
                        String::new()
                    } else {
                        format!(", {arguments}")
                    };
                    dependencies.push(arguments);
                    (
                        format!(
                            "<{} as {}>::{}(&receiver{suffix})",
                            type_name(record),
                            type_name(interface),
                            value_name(method_name)
                        ),
                        return_type,
                    )
                }
            };
            match &test.expected {
                ExpectedOutcome::Value(expected) => {
                    output.push_str(&format!(
                        "        let actual = {call}.expect(\"portable test expected a value\");\n"
                    ));
                    let assertions = self.test_assertions("actual", &return_type, &expected.value);
                    output.push_str(&assertions.text);
                    dependencies.push(assertions);
                }
                ExpectedOutcome::Error(expected) => {
                    let code = match &expected.value {
                        Value::String(code) => code.as_str(),
                        _ => "invariant_violation",
                    };
                    output.push_str(&format!(
                        "        let error = {call}.expect_err(\"portable test expected an error\");\n        assert_eq!(error.code(), {code:?});\n"
                    ));
                }
            }
            output.push_str("    }\n\n");
        }
        RustCode::new(output).with_text_from(dependencies)
    }

    fn test_arguments(&self, arguments: &[TypedValue]) -> RustCode {
        RustCode::sequence(arguments.iter().enumerate().map(|(index, argument)| {
            self.value(&argument.value)
                .map_text(|value| format!("        let argument_{index} = {value};\n"))
        }))
    }

    fn test_argument_list(&self, arguments: &[TypedValue], parameters: &[Parameter]) -> RustCode {
        RustCode::new(
            arguments
                .iter()
                .enumerate()
                .map(|(index, _)| {
                    if parameters
                        .get(index)
                        .is_some_and(|parameter| matches!(parameter.ty, TypeRef::Interface(_)))
                    {
                        format!("&argument_{index}")
                    } else {
                        format!("argument_{index}")
                    }
                })
                .collect::<Vec<_>>()
                .join(", "),
        )
    }

    fn test_assertions(&self, actual: &str, ty: &TypeRef, expected: &Value) -> RustCode {
        match (ty, expected) {
            (TypeRef::Bool, Value::Bool(true)) => {
                RustCode::new(format!("        assert!({actual});\n"))
            }
            (TypeRef::Bool, Value::Bool(false)) => {
                RustCode::new(format!("        assert!(!{actual});\n"))
            }
            (TypeRef::F64, Value::F64(value)) => {
                if f64::from_bits(value.0).is_nan() {
                    RustCode::new(format!("        assert!({actual}.is_nan());\n"))
                } else {
                    RustCode::new(format!(
                        "        assert_eq!({actual}.to_bits(), 0x{:016x});\n",
                        value.0
                    ))
                }
            }
            (TypeRef::List(_), Value::List(values)) if values.is_empty() => {
                RustCode::new(format!("        assert!({actual}.is_empty());\n"))
            }
            (
                TypeRef::Named(_),
                Value::Record {
                    declaration,
                    fields,
                },
            ) => {
                let Some(Declaration::Record(record)) = self.declaration(*declaration) else {
                    return self.value(expected).map_text(|expected| {
                        format!("        assert_eq!({actual}, {expected});\n")
                    });
                };
                RustCode::sequence(fields.iter().map(|field| {
                    let Some(member) = record
                        .fields
                        .iter()
                        .find(|member| member.header.node.id == field.field)
                    else {
                        return RustCode::default();
                    };
                    self.test_assertions(
                        &format!("{actual}.{}", value_name(&member.header.name)),
                        &member.ty,
                        &field.value,
                    )
                }))
            }
            _ => self
                .value(expected)
                .map_text(|expected| format!("        assert_eq!({actual}, {expected});\n")),
        }
    }
}

const RUNTIME: &str = r#"#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PolyRuntimeError {
    CheckedOverflow { operation: &'static str },
    DivisionByZero,
    RemainderByZero,
    InvalidShift { amount: i64, width: u8 },
    NarrowingOutOfRange { value: i64 },
    IndexOutOfBounds { index: i64, length: u64 },
    InvalidUtf8,
}

impl PolyRuntimeError {
    pub const fn code(&self) -> &'static str {
        match self {
            Self::CheckedOverflow { .. } => "checked_overflow",
            Self::DivisionByZero => "division_by_zero",
            Self::RemainderByZero => "remainder_by_zero",
            Self::InvalidShift { .. } => "invalid_shift",
            Self::NarrowingOutOfRange { .. } => "narrowing_out_of_range",
            Self::IndexOutOfBounds { .. } => "index_out_of_bounds",
            Self::InvalidUtf8 => "invalid_utf8",
        }
    }
}

pub type PolyResult<T> = Result<T, PolyRuntimeError>;

// POLYRUST-BEGIN string-replace-many
#[doc(hidden)]
pub fn _poly_string_replace_many(
    source: String,
    mappings: &[(&str, &str)],
) -> PolyResult<String> {
    let mut output = String::new();
    let mut offset = 0;
    loop {
        let remaining = &source[offset..];
        if let Some((needle, replacement)) = mappings
            .iter()
            .find(|(needle, _)| remaining.starts_with(*needle))
        {
            output.push_str(replacement);
            if needle.is_empty() {
                let Some(character) = remaining.chars().next() else {
                    break;
                };
                let width = character.len_utf8();
                output.push_str(&remaining[..width]);
                offset += width;
            } else {
                offset += needle.len();
            }
        } else {
            let Some(character) = remaining.chars().next() else {
                break;
            };
            let width = character.len_utf8();
            output.push_str(&remaining[..width]);
            offset += width;
        }
    }
    Ok(output)
}
// POLYRUST-END string-replace-many

// POLYRUST-BEGIN bytes-replace-all
#[doc(hidden)]
pub fn _poly_bytes_replace_all(
    source: Vec<u8>,
    needle: Vec<u8>,
    replacement: Vec<u8>,
) -> PolyResult<Vec<u8>> {
    let mut output = Vec::new();
    if needle.is_empty() {
        output.extend_from_slice(&replacement);
        for byte in source {
            output.push(byte);
            output.extend_from_slice(&replacement);
        }
        return Ok(output);
    }
    let mut offset = 0;
    while offset < source.len() {
        if source[offset..].starts_with(&needle) {
            output.extend_from_slice(&replacement);
            offset += needle.len();
        } else {
            output.push(source[offset]);
            offset += 1;
        }
    }
    Ok(output)
}
// POLYRUST-END bytes-replace-all

// POLYRUST-BEGIN string-truncate-utf8
#[doc(hidden)]
pub fn _poly_string_truncate_utf8_bytes(
    source: String,
    budget: f64,
) -> PolyResult<String> {
    for (offset, character) in source.char_indices() {
        let end = offset + character.len_utf8();
        let consumed = end as f64;
        if consumed == budget {
            return Ok(source[..end].to_owned());
        }
        if consumed > budget {
            return Ok(source[..offset].to_owned());
        }
    }
    Ok(source)
}
// POLYRUST-END string-truncate-utf8

// POLYRUST-BEGIN checked-shift
#[doc(hidden)]
pub trait PolyShift: Sized {
    fn poly_checked_shl(self, amount: u32) -> Option<Self>;
    fn poly_checked_shr(self, amount: u32) -> Option<Self>;
}

impl PolyShift for i32 {
    fn poly_checked_shl(self, amount: u32) -> Option<Self> { self.checked_shl(amount) }
    fn poly_checked_shr(self, amount: u32) -> Option<Self> { self.checked_shr(amount) }
}

impl PolyShift for i64 {
    fn poly_checked_shl(self, amount: u32) -> Option<Self> { self.checked_shl(amount) }
    fn poly_checked_shr(self, amount: u32) -> Option<Self> { self.checked_shr(amount) }
}

#[doc(hidden)]
pub fn _poly_shift_left<T: PolyShift>(value: T, amount: i64, width: u8) -> PolyResult<T> {
    if amount < 0 || amount >= i64::from(width) {
        Err(PolyRuntimeError::InvalidShift { amount, width })
    } else {
        value.poly_checked_shl(amount as u32).ok_or(PolyRuntimeError::InvalidShift { amount, width })
    }
}

#[doc(hidden)]
pub fn _poly_shift_right<T: PolyShift>(value: T, amount: i64, width: u8) -> PolyResult<T> {
    if amount < 0 || amount >= i64::from(width) {
        Err(PolyRuntimeError::InvalidShift { amount, width })
    } else {
        value.poly_checked_shr(amount as u32).ok_or(PolyRuntimeError::InvalidShift { amount, width })
    }
}
// POLYRUST-END checked-shift

#[doc(hidden)]
pub fn _poly_list_get<T: Clone>(values: Vec<T>, index: i64) -> PolyResult<T> {
    let length = values.len() as u64;
    let index_usize = usize::try_from(index).map_err(|_| PolyRuntimeError::IndexOutOfBounds { index, length })?;
    values.get(index_usize).cloned().ok_or(PolyRuntimeError::IndexOutOfBounds { index, length })
}
"#;

fn render_conformance() -> RustCode {
    RustCode::new(
        r#"#[test] fn vector_01_bool_not() { let result = !std::hint::black_box(true); assert!(!result); }
#[test] fn vector_02_checked_add() { assert_eq!(20_i64.checked_add(22).unwrap(), 42); }
#[test] fn vector_03_checked_overflow() { assert_eq!(i64::MAX.checked_add(1).ok_or(PolyRuntimeError::CheckedOverflow { operation: "add" }).unwrap_err().code(), "checked_overflow"); }
#[test] fn vector_04_division_by_zero() { let divisor = std::hint::black_box(0_i32); let result: PolyResult<i32> = if divisor == 0 { Err(PolyRuntimeError::DivisionByZero) } else { Ok(1 / divisor) }; assert_eq!(result.unwrap_err().code(), "division_by_zero"); }
#[test] fn vector_05_wrapping_add() { assert_eq!(i32::MAX.wrapping_add(1), i32::MIN); }
#[test] fn vector_06_unicode_astral_length() { assert_eq!("a🦀".chars().count(), 2); }
#[test] fn vector_07_unicode_combining_length() { assert_eq!("é".chars().count(), 2); }
#[test] fn vector_08_bytes_concat() { let mut value = vec![0_u8, 1]; value.extend([254, 255]); assert_eq!(value, vec![0, 1, 254, 255]); }
#[test] fn vector_09_list_get() { assert_eq!(_poly_list_get(vec![String::from("first")], 0).unwrap(), "first"); }
#[test] fn vector_10_list_oob() { assert_eq!(_poly_list_get(Vec::<i64>::new(), 0).unwrap_err().code(), "index_out_of_bounds"); }
#[test] fn vector_11_option_some() { assert!(Some(1_i64).is_some()); }
#[test] fn vector_12_option_none() { assert!(Option::<i64>::None.is_none()); }
#[test] fn vector_13_result_ok() { assert!(Result::<i64, String>::Ok(1).is_ok()); }
#[test] fn vector_14_result_err() { assert!(Result::<i64, String>::Err(String::from("error")).is_err()); }
#[test] fn vector_15_float_negative_zero() { assert_eq!((-0.0_f64).to_bits(), 0x8000000000000000); }
#[test] fn vector_16_float_nan_equality() { let left = f64::from_bits(0x7ff8000000000000); let right = f64::from_bits(0x7ff8000000000000); assert!(left != right); }
#[test] fn vector_17_narrow_out_of_range() { let result = i32::try_from(i64::MAX).map_err(|_| PolyRuntimeError::NarrowingOutOfRange { value: i64::MAX }); assert_eq!(result.unwrap_err().code(), "narrowing_out_of_range"); }
#[test] fn vector_18_invalid_utf8() { assert_eq!(String::from_utf8(std::hint::black_box(vec![0xff])).map_err(|_| PolyRuntimeError::InvalidUtf8).unwrap_err().code(), "invalid_utf8"); }
#[test] fn vector_19_string_contains() { assert!("x🦀y".contains("🦀")); }
#[test] fn vector_20_list_append_non_aliasing() { let original = vec![1_i64]; let mut appended = original.clone(); appended.push(2); assert_eq!(original, vec![1]); assert_eq!(appended, vec![1, 2]); }
"#,
    )
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use portable_codegen::{OutputContents, check_backend_contract};
    use portable_eval::Evaluator;

    use super::*;

    fn fixture() -> CheckedProgram {
        let document = portable_ir::v0::from_json(include_bytes!(
            "../../build/testdata/registration.poly.json"
        ))
        .expect("canonical fixture parses");
        portable_check::v0::check_program(document).expect("fixture checks")
    }

    fn generated_text<'a>(manifest: &'a OutputManifest, path: &str) -> &'a str {
        let file = manifest.file(path).expect("generated file exists");
        let OutputContents::Text(text) = file.contents() else {
            panic!("expected generated UTF-8 text")
        };
        text
    }

    #[test]
    fn descriptor_support_and_contract_are_complete_and_deterministic() {
        let program = fixture();
        let backend: Arc<dyn Backend> = Arc::new(RustBackend);
        assert_eq!(backend.descriptor().target.as_str(), "org.polyrust.rust");
        for capability in [
            Capability::Bytes,
            Capability::CheckedIntegerArithmetic,
            Capability::InterfaceDispatch,
            Capability::F64,
            Capability::ImmutableList,
            Capability::Option,
            Capability::Result,
            Capability::UnicodeScalar,
            Capability::WrappingIntegerArithmetic,
            Capability::BoundedIteration,
        ] {
            assert!(!matches!(
                backend.support(capability),
                CapabilitySupport::Unsupported { .. }
            ));
        }
        assert!(check_backend_contract(backend, &program, &BackendOptions::default()).is_empty());
    }

    #[test]
    fn first_class_interface_values_are_rejected_before_legacy_translation() {
        let fixture = portable_build::interface_composition_fixture();
        let program = portable_check::v0::check_program(fixture.document).unwrap();
        let error = RustBackend
            .generate(&program, &BackendOptions::default())
            .unwrap_err();
        let BackendError::UnsupportedCapabilities(diagnostics) = error else {
            panic!("first-class interface values must produce a capability diagnostic")
        };
        assert_eq!(diagnostics.len(), 1);
        assert_eq!(
            diagnostics[0].code,
            portable_diagnostics::DiagnosticCode::UnsupportedCapability
        );
        assert_eq!(diagnostics[0].target.as_deref(), Some("org.polyrust.rust"));
        assert!(diagnostics[0].message.contains("M34A-12"));
    }

    #[test]
    fn complete_fixture_generation_is_golden_and_uses_every_package_layer() {
        let program = fixture();
        assert!(
            Evaluator::new(&program)
                .run_all_tests()
                .iter()
                .all(|result| result.passed)
        );
        let first = RustBackend
            .generate(&program, &BackendOptions::default())
            .unwrap();
        let second = RustBackend
            .generate(&program, &BackendOptions::default())
            .unwrap();
        let third = RustBackend
            .generate(&program, &BackendOptions::default())
            .unwrap();
        assert_eq!(first.canonical_json(), second.canonical_json());
        assert_eq!(second.canonical_json(), third.canonical_json());
        assert_eq!(
            first
                .files()
                .iter()
                .map(|file| file.path())
                .collect::<Vec<_>>(),
            vec![
                "Cargo.toml",
                "src/conformance.rs",
                "src/lib.rs",
                "src/polyrust_runtime.rs"
            ]
        );
        let source = generated_text(&first, "src/lib.rs");
        for required in [
            "#![forbid(unsafe_code)]",
            "mod polyrust_runtime;",
            "trait ",
            "impl ",
            "PolyResult<",
            "#[cfg(test)]",
            "mod conformance;",
        ] {
            assert!(source.contains(required), "missing {required:?}");
        }
        assert_eq!(source.matches("pub use polyrust_runtime::*;").count(), 1);
        assert!(!source.contains("use super::*;"));
        let conformance = generated_text(&first, "src/conformance.rs");
        assert_eq!(conformance.matches("use super::*;").count(), 1);
        assert_eq!(conformance.matches("#[test]").count(), 20);
        assert!(conformance.contains("vector_20_list_append_non_aliasing"));
        let runtime = generated_text(&first, "src/polyrust_runtime.rs");
        assert!(runtime.contains("pub enum PolyRuntimeError"));
        assert!(!runtime.lines().any(|line| line.starts_with("use ")));
    }

    #[test]
    fn rust_imports_and_nested_types_are_validated_fragments() {
        assert!(RustImport::module("polyrust_runtime", false).is_ok());
        assert!(RustImport::use_path("polyrust_runtime::*", true).is_ok());
        assert!(RustImport::use_path("super::*", false).is_ok());
        for module in ["", "bad-name", "crate::runtime", "mod runtime"] {
            assert!(RustImport::module(module, false).is_err(), "{module}");
        }
        for path in ["", "::root", "bad-path::*", "runtime::*::item", "use x;"] {
            assert!(RustImport::use_path(path, false).is_err(), "{path}");
        }

        let program = fixture();
        let nested = Generator::new(&program).ty(&TypeRef::Result {
            ok: Box::new(TypeRef::Option(Box::new(TypeRef::List(Box::new(
                TypeRef::I64,
            ))))),
            error: Box::new(TypeRef::String),
        });
        assert_eq!(nested.text, "Result<Option<Vec<i64>>, String>");
        assert!(nested.imports.is_empty());
        assert!(nested.helper_roots.is_empty());
    }

    #[test]
    fn rust_runtime_helper_matrix_is_exact_and_mapping_local() {
        let (graph, common) = rust_runtime_helper_graph().unwrap();
        let minimal = render_runtime(&graph.resolve(&common).unwrap());
        assert!(minimal.contains("_poly_list_get"));
        assert!(!minimal.contains("POLYRUST-"));
        for token in [
            "_poly_string_replace_many",
            "_poly_bytes_replace_all",
            "_poly_string_truncate_utf8_bytes",
            "trait PolyShift",
        ] {
            assert!(!minimal.contains(token), "{token} in minimal runtime");
        }
        for (root, present, absent) in [
            (
                "feature.string-replace-many",
                "_poly_string_replace_many",
                "_poly_bytes_replace_all",
            ),
            (
                "feature.bytes-replace-all",
                "_poly_bytes_replace_all",
                "_poly_string_truncate_utf8_bytes",
            ),
            (
                "feature.string-truncate-utf8",
                "_poly_string_truncate_utf8_bytes",
                "trait PolyShift",
            ),
            (
                "feature.checked-shift",
                "trait PolyShift",
                "_poly_string_replace_many",
            ),
        ] {
            let mut roots = common.clone();
            roots.push(root.to_owned());
            let runtime = render_runtime(&graph.resolve(&roots).unwrap());
            assert!(runtime.contains(present), "{root} lacks {present}");
            assert!(!runtime.contains(absent), "{root} includes {absent}");
            assert!(!runtime.contains("POLYRUST-"));
        }

        let program = fixture();
        let generator = Generator::new(&program);
        for (operation, root) in [
            (Intrinsic::StringReplaceMany, "feature.string-replace-many"),
            (Intrinsic::BytesReplaceAll, "feature.bytes-replace-all"),
            (
                Intrinsic::StringTruncateUtf8Bytes,
                "feature.string-truncate-utf8",
            ),
            (Intrinsic::IntShiftLeftChecked, "feature.checked-shift"),
        ] {
            let code = generator.intrinsic(
                operation,
                vec![
                    RustCode::new("Ok(String::new())"),
                    RustCode::new("Ok(String::new())"),
                    RustCode::new("Ok(String::new())"),
                ],
                None,
            );
            assert!(code.helper_roots.contains(root), "{operation:?}");
        }

        let negative_zero = generator.intrinsic(
            Intrinsic::FloatIsNegativeZero,
            vec![RustCode::new("Ok(-0.0_f64)")],
            Some(&TypeRef::F64),
        );
        assert!(
            negative_zero
                .text
                .contains("to_bits() == (-0.0_f64).to_bits()")
        );
        assert!(negative_zero.imports.is_empty());
        assert!(negative_zero.helper_roots.is_empty());

        let absolute = generator.intrinsic(
            Intrinsic::FloatAbs,
            vec![RustCode::new("Ok(f64::from_bits(0xfff8_0000_0000_0123))")],
            Some(&TypeRef::F64),
        );
        assert!(absolute.text.contains("to_bits() & 0x7fff_ffff_ffff_ffff"));
        assert!(absolute.imports.is_empty());
        assert!(absolute.helper_roots.is_empty());
    }

    #[test]
    fn names_literals_and_runtime_helpers_are_target_owned_and_safe() {
        assert_eq!(rust_identifier("match"), "r#match");
        assert_eq!(rust_identifier("self"), "self_");
        assert_eq!(test_name("Astral 🦀 Case", NodeId(7)), "astral_case_n7");
        let program = fixture();
        let generator = Generator::new(&program);
        assert_eq!(
            generator.value(&Value::I64(i64::MIN)).text,
            "-9223372036854775808_i64"
        );
        assert_eq!(
            generator
                .value(&Value::F64(
                    portable_ir::v0::F64Bits(0x8000_0000_0000_0000,)
                ))
                .text,
            "f64::from_bits(0x8000000000000000)"
        );
        assert_eq!(
            generator.value(&Value::String("\"\\\n🦀".into())).text,
            "String::from(\"\\\"\\\\\\n🦀\")"
        );
        assert_eq!(
            generator.value(&Value::Bytes(vec![0, 255])).text,
            "vec![0x00_u8, 0xff_u8]"
        );
    }

    fn render_runtime(fragment: &LanguageFragment<RustImport>) -> String {
        let mut file = LanguageSourceFile::new("src/polyrust_runtime.rs", SourceFileRole::Runtime);
        file.set_body(fragment.clone());
        let group = FileGroup::new(
            FileGroupId::parse("test").unwrap(),
            vec![LanguageFile::source(file)],
        )
        .unwrap();
        let package =
            LanguagePackage::new(vec![group], Vec::<DeclaredDependency>::new(), Vec::new())
                .unwrap();
        let manifest = portable_codegen::render_language_package(&package, &RustRenderer).unwrap();
        generated_text(&manifest, "src/polyrust_runtime.rs").to_owned()
    }

    #[test]
    fn portable_boolean_expectations_use_clippy_safe_assertions() {
        let program = fixture();
        let generator = Generator::new(&program);
        assert_eq!(
            generator
                .test_assertions("actual", &TypeRef::Bool, &Value::Bool(true))
                .text,
            "        assert!(actual);\n"
        );
        assert_eq!(
            generator
                .test_assertions("actual", &TypeRef::Bool, &Value::Bool(false))
                .text,
            "        assert!(!actual);\n"
        );
    }

    #[test]
    fn generated_source_contains_no_unsafe_surface() {
        let manifest = RustBackend
            .generate(&fixture(), &BackendOptions::default())
            .unwrap();
        let unsafe_lines: Vec<_> = manifest
            .files()
            .iter()
            .filter_map(|file| match file.contents() {
                OutputContents::Text(text) => Some(text),
                OutputContents::Bytes(_) => None,
            })
            .flat_map(|text| text.lines())
            .filter(|line| line.contains("unsafe"))
            .collect();
        assert_eq!(unsafe_lines, vec!["#![forbid(unsafe_code)]"]);
    }
}
