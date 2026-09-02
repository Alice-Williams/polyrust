use std::collections::{BTreeMap, BTreeSet};

use portable_check::v0::{Capability, CheckedProgram};
use portable_codegen::{
    Backend, BackendDescriptor, BackendError, BackendOptions, BackendVersion, CapabilitySupport,
    DeclaredDependency, Document as CodeDocument, FileGroup, FileGroupId, ImportSet,
    InjectedHelper, IrVersionRange, LanguageFile, LanguageFragment, LanguagePackage,
    LanguagePlugin, LanguageRenderer, LanguageSourceFile, OptionsSchema, OutputManifest, RawText,
    RuntimeHelper, RuntimeHelperGraph, SourceFileRole, TargetId, TextFileRole,
    generate_with_plugin,
};
use portable_ir::v0::{
    Declaration, ExpectedOutcome, Intrinsic, IrVersion, NodeId, TestInvocation, TypeRef,
    TypedValue, Value,
};

const RUNTIME: &str = include_str!("runtime.go");

pub struct GoV0Backend;

impl Backend for GoV0Backend {
    fn descriptor(&self) -> BackendDescriptor {
        BackendDescriptor {
            target: TargetId::parse("org.polyrust.go").expect("valid target"),
            display_name: "Go".into(),
            backend_version: BackendVersion::new(0, 1, 0),
            supported_ir: IrVersionRange::exact(IrVersion::CURRENT),
        }
    }
    fn support(&self, capability: Capability) -> CapabilitySupport {
        match capability {
            Capability::CheckedIntegerArithmetic => CapabilitySupport::Helper {
                helper: "polyrust.runtime.checked-integers.v0".into(),
            },
            Capability::UnicodeScalar => CapabilitySupport::Helper {
                helper: "polyrust.runtime.unicode-scalars.v0".into(),
            },
            Capability::ImmutableList => CapabilitySupport::Helper {
                helper: "polyrust.runtime.immutable-list.v0".into(),
            },
            Capability::Bytes
            | Capability::ContractDispatch
            | Capability::F64
            | Capability::Option
            | Capability::Result
            | Capability::WrappingIntegerArithmetic
            | Capability::BoundedIteration => CapabilitySupport::Native,
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
#[doc(hidden)]
pub struct GoImport {
    path: &'static str,
}

impl GoImport {
    fn parse(path: &'static str) -> Result<Self, String> {
        if path.is_empty()
            || path.starts_with('/')
            || path.ends_with('/')
            || path.contains("//")
            || path.split('/').any(|part| {
                part.is_empty()
                    || matches!(part, "." | "..")
                    || !part.bytes().all(|byte| {
                        byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.' | b'~')
                    })
            })
        {
            return Err(format!("invalid Go import path {path:?}"));
        }
        Ok(Self { path })
    }
}

#[doc(hidden)]
pub struct GoRenderer;

impl LanguageRenderer<GoImport> for GoRenderer {
    fn render_imports(&self, imports: &ImportSet<GoImport>) -> Result<CodeDocument, String> {
        let names = imports
            .groups()
            .flat_map(|(_, imports)| imports.iter())
            .map(|import| format!("\t{:?}", import.path))
            .collect::<Vec<_>>();
        let rendered = match names.as_slice() {
            [one] => format!("import {}", one.trim()),
            _ => format!("import (\n{}\n)", names.join("\n")),
        };
        Ok(CodeDocument::raw_text(RawText::new(rendered)))
    }
}

impl LanguagePlugin for GoV0Backend {
    type Import = GoImport;
    type Renderer = GoRenderer;

    fn translate(
        &self,
        program: &CheckedProgram,
        _options: &BackendOptions,
    ) -> Result<LanguagePackage<Self::Import>, BackendError> {
        let generator = Generator::new(program);
        let helpers = program
            .capabilities()
            .program()
            .iter()
            .filter_map(|capability| match self.support(*capability) {
                CapabilitySupport::Helper { helper } => Some(InjectedHelper {
                    id: helper,
                    capability: format!("{capability:?}"),
                    files: vec!["runtime.go".into()],
                }),
                CapabilitySupport::Native | CapabilitySupport::Unsupported { .. } => None,
            })
            .collect();
        LanguagePackage::new(
            vec![
                FileGroup::new(
                    go_group("metadata")?,
                    vec![LanguageFile::text(
                        "go.mod",
                        TextFileRole::Metadata,
                        "module generated.polyrust/package\n\ngo 1.25.0\n",
                    )],
                )
                .map_err(go_generation_error)?,
                FileGroup::new(
                    go_group("runtime")?,
                    vec![LanguageFile::source(go_runtime_file(program)?)],
                )
                .map_err(go_generation_error)?,
                FileGroup::new(
                    go_group("source")?,
                    vec![LanguageFile::source(generator.source_file()?)],
                )
                .map_err(go_generation_error)?,
                FileGroup::new(
                    go_group("tests")?,
                    vec![
                        LanguageFile::source(generator.tests_file()),
                        LanguageFile::source(go_conformance_file()),
                    ],
                )
                .map_err(go_generation_error)?,
            ],
            Vec::<DeclaredDependency>::new(),
            helpers,
        )
        .map_err(go_generation_error)
    }

    fn renderer(&self) -> Self::Renderer {
        GoRenderer
    }
}

fn go_generation_error(error: impl std::fmt::Display) -> BackendError {
    BackendError::Generation {
        message: error.to_string(),
    }
}

fn go_group(name: &str) -> Result<FileGroupId, BackendError> {
    FileGroupId::parse(name).map_err(go_generation_error)
}

fn go_import_group() -> portable_codegen::ImportGroup {
    portable_codegen::ImportGroup::new(10, "standard-library")
        .expect("static import group is valid")
}

fn go_import(path: &'static str) -> GoImport {
    GoImport::parse(path).expect("static Go import path is valid")
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct GoCode {
    text: String,
    imports: BTreeSet<GoImport>,
}

impl GoCode {
    fn text(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            imports: BTreeSet::new(),
        }
    }

    fn with_import(mut self, path: &'static str) -> Self {
        self.imports.insert(go_import(path));
        self
    }

    fn joined(separator: &str, parts: impl IntoIterator<Item = Self>) -> Self {
        let mut output = Self::text("");
        for part in parts {
            if !output.text.is_empty() {
                output.text.push_str(separator);
            }
            output.text.push_str(&part.text);
            output.imports.extend(part.imports);
        }
        output
    }

    fn map_text(mut self, mapping: impl FnOnce(String) -> String) -> Self {
        self.text = mapping(self.text);
        self
    }

    fn with_dependencies(text: String, dependencies: impl IntoIterator<Item = Self>) -> Self {
        let mut output = Self::text(text);
        for dependency in dependencies {
            output.imports.extend(dependency.imports);
        }
        output
    }

    fn dependency_text(&mut self, dependency: Self) -> String {
        self.imports.extend(dependency.imports);
        dependency.text
    }

    fn push_code(&mut self, code: Self) {
        self.text.push_str(&code.text);
        self.imports.extend(code.imports);
    }

    fn into_fragment(self) -> LanguageFragment<GoImport> {
        let mut fragment = LanguageFragment::new(CodeDocument::raw_text(RawText::new(self.text)));
        for import in self.imports {
            fragment.require_import(go_import_group(), import);
        }
        fragment
    }
}

fn go_preamble(generated: bool) -> CodeDocument {
    let text = if generated {
        "// Code generated by PolyRust. DO NOT EDIT.\npackage generated"
    } else {
        "package generated"
    };
    CodeDocument::raw_text(RawText::new(text))
}

fn go_runtime_file(program: &CheckedProgram) -> Result<LanguageSourceFile<GoImport>, BackendError> {
    let (graph, mut roots) = go_runtime_helper_graph()?;
    if program
        .capabilities()
        .program()
        .contains(&Capability::CheckedIntegerArithmetic)
    {
        roots.push("feature.checked-integers".to_owned());
    }
    if program.capabilities().program().contains(&Capability::F64) {
        roots.push("feature.f64".to_owned());
    }
    if portable_ir::v0::module_uses_intrinsic(program.module(), |operation| {
        operation == Intrinsic::BytesReplaceAll
    }) {
        roots.push("feature.bytes-replace".to_owned());
    }
    for (operation, root) in [
        (Intrinsic::StringScalarLength, "feature.utf8-scalar"),
        (Intrinsic::StringReplaceMany, "feature.replace-many"),
        (Intrinsic::StringTruncateUtf8Bytes, "feature.truncate-utf8"),
        (Intrinsic::StringFromUtf8Checked, "feature.from-utf8"),
    ] {
        if portable_ir::v0::module_uses_intrinsic(program.module(), |used| used == operation) {
            roots.push(root.to_owned());
        }
    }

    let mut file = LanguageSourceFile::new("runtime.go", SourceFileRole::Runtime);
    file.set_preamble(LanguageFragment::new(go_preamble(true)));
    file.set_body(graph.resolve(&roots).map_err(go_generation_error)?);
    Ok(file)
}

fn go_runtime_helper_graph() -> Result<(RuntimeHelperGraph<GoImport>, Vec<String>), BackendError> {
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
                helpers: &mut Vec<RuntimeHelper<GoImport>>| {
        if source.trim().is_empty() {
            source.clear();
            return false;
        }
        helpers.push(RuntimeHelper::new(
            id.clone(),
            *order,
            go_runtime_fragment(&id, std::mem::take(source)),
        ));
        *order = order
            .checked_add(1)
            .expect("Go runtime helper order fits u16");
        true
    };

    for line in RUNTIME.split_inclusive('\n') {
        let marker = line.trim().trim_end_matches('\r');
        if let Some(id) = marker.strip_prefix(BEGIN) {
            if active.is_some() {
                return Err(go_generation_error(format!(
                    "nested Go runtime helper marker {id:?}"
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
                return Err(go_generation_error(format!(
                    "unmatched Go runtime helper end marker {id:?}"
                )));
            };
            if open != id {
                return Err(go_generation_error(format!(
                    "Go runtime helper marker {open:?} closed by {id:?}"
                )));
            }
            if !emit(open, &mut source, &mut order, &mut helpers) {
                return Err(go_generation_error(format!(
                    "empty Go runtime helper {id:?}"
                )));
            }
        } else {
            source.push_str(line);
        }
    }
    if let Some(open) = active {
        return Err(go_generation_error(format!(
            "unclosed Go runtime helper marker {open:?}"
        )));
    }
    let common_id = format!("runtime.common.{common_index:03}");
    if emit(common_id.clone(), &mut source, &mut order, &mut helpers) {
        common_roots.push(common_id);
    }

    let feature = |dependencies: &[&str]| {
        dependencies.iter().fold(
            LanguageFragment::new(CodeDocument::empty()),
            |fragment, dependency| fragment.with_helper_root(*dependency),
        )
    };
    helpers.push(RuntimeHelper::new(
        "feature.bytes-replace",
        u16::MAX - 2,
        feature(&["bytes-replace-function", "bytes-replace-case"]),
    ));
    helpers.push(RuntimeHelper::new(
        "feature.checked-integers",
        u16::MAX - 5,
        feature(&[
            "math-narrow-case",
            "math-i32-checked-cases",
            "math-i64-checked-case",
            "math-checked32",
        ]),
    ));
    helpers.push(RuntimeHelper::new(
        "feature.f64",
        u16::MAX - 4,
        feature(&[
            "math-unsigned-number",
            "math-value-case",
            "math-float-dispatch",
            "math-float-equality",
        ]),
    ));
    helpers.push(RuntimeHelper::new(
        "feature.utf8-scalar",
        u16::MAX - 3,
        feature(&["utf8-scalar-case"]),
    ));
    helpers.push(RuntimeHelper::new(
        "feature.replace-many",
        u16::MAX - 2,
        feature(&["utf8-replace-many-case", "utf8-replace-many-function"]),
    ));
    helpers.push(RuntimeHelper::new(
        "feature.truncate-utf8",
        u16::MAX - 1,
        feature(&["utf8-truncate-case", "utf8-truncate-function"]),
    ));
    helpers.push(RuntimeHelper::new(
        "feature.from-utf8",
        u16::MAX,
        feature(&["utf8-from-bytes-case"]),
    ));
    Ok((
        RuntimeHelperGraph::new(helpers).map_err(go_generation_error)?,
        common_roots,
    ))
}

fn go_runtime_fragment(id: &str, source: String) -> LanguageFragment<GoImport> {
    let mut fragment = LanguageFragment::new(CodeDocument::raw_text(RawText::new(source)));
    let imports: &[&'static str] = match id {
        "runtime.common.001" => &["encoding/json", "strings"],
        "runtime.common.004" | "runtime.common.005" => &["strings"],
        "bytes-replace-function" => &["bytes"],
        "math-unsigned-number" => &["encoding/json", "strconv"],
        "math-value-case"
        | "math-narrow-case"
        | "math-i64-checked-case"
        | "math-float-dispatch"
        | "math-checked32"
        | "math-float-equality" => &["math"],
        "utf8-scalar-case" | "utf8-from-bytes-case" | "utf8-truncate-function" => &["unicode/utf8"],
        "utf8-replace-many-function" => &["strings", "unicode/utf8"],
        _ => &[],
    };
    for import in imports {
        fragment.require_import(go_import_group(), go_import(import));
    }
    fragment
}

fn go_conformance_file() -> LanguageSourceFile<GoImport> {
    let mut file = LanguageSourceFile::new("conformance_test.go", SourceFileRole::Conformance);
    file.set_preamble(LanguageFragment::new(go_preamble(false)));
    let mut body = LanguageFragment::new(CodeDocument::raw_text(RawText::new(CONFORMANCE_BODY)));
    body.require_import(go_import_group(), go_import("math"));
    body.require_import(go_import_group(), go_import("testing"));
    file.set_body(body);
    file
}

struct Generator<'a> {
    program: &'a CheckedProgram,
    names: BTreeMap<NodeId, String>,
}

impl<'a> Generator<'a> {
    fn new(program: &'a CheckedProgram) -> Self {
        Self {
            program,
            names: program
                .module()
                .declarations
                .iter()
                .map(|item| (item.header().node.id, item.header().name.clone()))
                .collect(),
        }
    }
    fn source_file(&self) -> Result<LanguageSourceFile<GoImport>, BackendError> {
        let document =
            portable_ir::v0::to_canonical_json(self.program.document()).map_err(|error| {
                BackendError::Generation {
                    message: format!("cannot serialize checked IR: {error}"),
                }
            })?;
        let document = String::from_utf8(document).expect("canonical JSON is UTF-8");
        let mut file = LanguageSourceFile::new("generated.go", SourceFileRole::Source);
        file.set_preamble(LanguageFragment::new(CodeDocument::raw_text(RawText::new(
            "// Code generated by PolyRust from checked IR v0. DO NOT EDIT.\npackage generated",
        ))));
        let base = GoCode::text(format!(
            "var generatedRuntime = newRuntime({})\n\n",
            go_string(&document)
        ));
        let mut declarations: Vec<_> = self.program.module().declarations.iter().collect();
        declarations.sort_by_key(|item| item.header().node.id);
        file.set_body(LanguageFragment::sequence(
            std::iter::once(base.into_fragment()).chain(
                declarations
                    .into_iter()
                    .map(|declaration| self.declaration(declaration).into_fragment()),
            ),
        ));
        Ok(file)
    }
    fn declaration(&self, declaration: &Declaration) -> GoCode {
        let mut output = GoCode::text("");
        match declaration {
            Declaration::Alias(item) => {
                let target = output.dependency_text(self.ty(&item.target));
                output.text.push_str(&format!(
                    "type {} = {target}\n\n",
                    exported(&item.header.name),
                ));
            }
            Declaration::Record(item) => {
                output
                    .text
                    .push_str(&format!("type {} struct {{\n", exported(&item.header.name)));
                for member in &item.fields {
                    let ty = output.dependency_text(self.ty(&member.ty));
                    output
                        .text
                        .push_str(&format!("\t{} {}\n", exported(&member.header.name), ty));
                }
                output.text.push_str("}\n\n");
                output.text.push_str(&format!("func (value {}) polyValue() map[string]any {{ return map[string]any{{\"__polyDecl\": int64({})", exported(&item.header.name), item.header.node.id.0));
                for member in &item.fields {
                    output.text.push_str(&format!(
                        ", {:?}: value.{}",
                        member.header.name,
                        exported(&member.header.name)
                    ));
                }
                output.text.push_str("} }\n\n");
                output.push_code(self.record_result_converter(item));
                for implementation in self.implementations(item.header.node.id) {
                    output.text.push_str(&format!(
                        "var _ {} = {}{{}}\n\n",
                        exported(self.name(implementation.contract)),
                        exported(&item.header.name)
                    ));
                    for method in &implementation.methods {
                        let call = format!(
                            "generatedRuntime.invokeMethod({}, {}, value, []any{{{}}})",
                            implementation.header.node.id.0,
                            method.header.node.id.0,
                            args(&method.parameters)
                        );
                        let parameters =
                            output.dependency_text(self.parameters(&method.parameters));
                        let return_type = output.dependency_text(self.ty(&method.return_type));
                        let converted =
                            output.dependency_text(self.convert_result(&method.return_type, &call));
                        output.text.push_str(&format!(
                            "func (value {}) {}({}) PolyResult[{}] {{ return {} }}\n\n",
                            exported(&item.header.name),
                            exported(&method.header.name),
                            parameters,
                            return_type,
                            converted,
                        ));
                    }
                }
            }
            Declaration::Enum(item) => {
                let mut variants = Vec::new();
                for variant in &item.variants {
                    let name = format!(
                        "{}{}",
                        exported(&item.header.name),
                        exported(&variant.header.name)
                    );
                    variants.push(name.clone());
                    output
                        .text
                        .push_str(&format!("type {name} struct {{\n\tTag string\n"));
                    for member in &variant.fields {
                        let ty = output.dependency_text(self.ty(&member.ty));
                        output.text.push_str(&format!(
                            "\t{} {}\n",
                            exported(&member.header.name),
                            ty
                        ));
                    }
                    output.text.push_str("}\n\n");
                }
                output.text.push_str(&format!(
                    "type {} interface {{ is{}() }}\n",
                    exported(&item.header.name),
                    exported(&item.header.name)
                ));
                for variant in variants {
                    output.text.push_str(&format!(
                        "func ({variant}) is{}() {{}}\n",
                        exported(&item.header.name)
                    ));
                }
                output.text.push('\n');
            }
            Declaration::Contract(item) => {
                output.text.push_str(&format!(
                    "type {} interface {{\n\tpolyValue() map[string]any\n",
                    exported(&item.header.name)
                ));
                for method in &item.methods {
                    let parameters = output.dependency_text(self.parameters(&method.parameters));
                    let return_type = output.dependency_text(self.ty(&method.return_type));
                    output.text.push_str(&format!(
                        "\t{}({}) PolyResult[{}]\n",
                        exported(&method.header.name),
                        parameters,
                        return_type,
                    ));
                }
                output.text.push_str("}\n\n");
            }
            Declaration::Constant(item) => {
                let call = format!("generatedRuntime.constant({})", item.header.node.id.0);
                let ty = output.dependency_text(self.ty(&item.ty));
                let converted = output.dependency_text(self.convert_result(&item.ty, &call));
                output.text.push_str(&format!(
                    "func {}() PolyResult[{}] {{ return {} }}\n\n",
                    exported(&item.header.name),
                    ty,
                    converted,
                ));
            }
            Declaration::Function(item) => {
                let call = format!(
                    "generatedRuntime.invoke({}, []any{{{}}})",
                    item.header.node.id.0,
                    args(&item.parameters)
                );
                let parameters = output.dependency_text(self.parameters(&item.parameters));
                let return_type = output.dependency_text(self.ty(&item.return_type));
                let converted =
                    output.dependency_text(self.convert_result(&item.return_type, &call));
                output.text.push_str(&format!(
                    "func {}({}) PolyResult[{}] {{ return {} }}\n\n",
                    exported(&item.header.name),
                    parameters,
                    return_type,
                    converted,
                ));
            }
            Declaration::Implementation(_) | Declaration::Test(_) => {}
        }
        output
    }
    fn implementations(&self, record: NodeId) -> Vec<&portable_ir::v0::ImplementationDeclaration> {
        self.program
            .module()
            .declarations
            .iter()
            .filter_map(|item| match item {
                Declaration::Implementation(value) if value.record == record => Some(value),
                _ => None,
            })
            .collect()
    }
    fn record_result_converter(&self, record: &portable_ir::v0::RecordDeclaration) -> GoCode {
        let name = exported(&record.header.name);
        let mut fields = GoCode::text("");
        for field in &record.fields {
            let ty = fields.dependency_text(self.ty(&field.ty));
            fields.text.push_str(&format!(
                "\t\t{}: value[{:?}].({}),\n",
                exported(&field.header.name),
                field.header.name,
                ty,
            ));
        }
        fields.map_text(|fields| format!(
            "func polyResult{name}(result PolyResult[any]) PolyResult[{name}] {{\n\tif !result.Ok {{ return polyFail[{name}](result.Error.Code, result.Error.Message) }}\n\tvalue, ok := result.Value.(map[string]any)\n\tif !ok {{ return polyFail[{name}](\"internal_type\", \"checked record result type mismatch\") }}\n\treturn polyOk({name}{{\n{fields}\t}})\n}}\n\n"
        ))
    }
    fn convert_result(&self, ty: &TypeRef, call: &str) -> GoCode {
        if let TypeRef::Named(id) = ty
            && matches!(self.declaration_by_id(*id), Some(Declaration::Record(_)))
        {
            return GoCode::text(format!("polyResult{}({call})", exported(self.name(*id))));
        }
        self.ty(ty)
            .map_text(|ty| format!("castResult[{ty}]({call})"))
    }
    fn declaration_by_id(&self, id: NodeId) -> Option<&Declaration> {
        self.program
            .module()
            .declarations
            .iter()
            .find(|declaration| declaration.header().node.id == id)
    }
    fn parameters(&self, parameters: &[portable_ir::v0::Parameter]) -> GoCode {
        GoCode::joined(
            ", ",
            parameters.iter().map(|item| {
                self.ty(&item.ty)
                    .map_text(|ty| format!("{} {ty}", local(&item.header.name)))
            }),
        )
    }
    fn ty(&self, ty: &TypeRef) -> GoCode {
        match ty {
            TypeRef::Unit => GoCode::text("struct{}"),
            TypeRef::Bool => GoCode::text("bool"),
            TypeRef::I32 => GoCode::text("int32"),
            TypeRef::I64 => GoCode::text("int64"),
            TypeRef::F64 => GoCode::text("float64"),
            TypeRef::Char => GoCode::text("rune"),
            TypeRef::String => GoCode::text("string"),
            TypeRef::Bytes => GoCode::text("PolyBytes"),
            TypeRef::List(inner) => self
                .ty(inner)
                .map_text(|inner| format!("PolyList[{inner}]")),
            TypeRef::Option(inner) => self
                .ty(inner)
                .map_text(|inner| format!("PolyOption[{inner}]")),
            TypeRef::Result { ok, error } => {
                let ok = self.ty(ok);
                let error = self.ty(error);
                let text = format!("PolyValueResult[{}, {}]", ok.text, error.text);
                GoCode::with_dependencies(text, [ok, error])
            }
            TypeRef::Named(id) | TypeRef::Contract(id) => GoCode::text(exported(self.name(*id))),
        }
    }
    fn name(&self, id: NodeId) -> &str {
        self.names.get(&id).map(String::as_str).unwrap_or("Unknown")
    }
    fn tests_file(&self) -> LanguageSourceFile<GoImport> {
        let mut file = LanguageSourceFile::new("generated_test.go", SourceFileRole::Test);
        file.set_preamble(LanguageFragment::new(CodeDocument::raw_text(RawText::new(
            "// Code generated from portable tests. DO NOT EDIT.\npackage generated",
        ))));
        file.set_body(LanguageFragment::sequence(
            self.program
                .module()
                .declarations
                .iter()
                .filter_map(|declaration| match declaration {
                    Declaration::Test(test) => Some(self.test(test).into_fragment()),
                    _ => None,
                }),
        ));
        file
    }

    fn test(&self, test: &portable_ir::v0::TestDeclaration) -> GoCode {
        let call = match &test.invocation {
            TestInvocation::Function {
                function,
                arguments,
            } => GoCode::joined(", ", arguments.iter().map(|value| self.value(value)))
                .map_text(|arguments| format!("{}({arguments})", exported(self.name(*function)))),
            TestInvocation::Method {
                implementation,
                method,
                receiver,
                arguments,
            } => {
                let receiver = self.value(receiver);
                let arguments =
                    GoCode::joined(", ", arguments.iter().map(|value| self.value(value)));
                let text = format!(
                    "{}.{}({})",
                    receiver.text,
                    exported(self.method_name(*implementation, *method)),
                    arguments.text
                );
                GoCode::with_dependencies(text, [receiver, arguments])
            }
        };
        let output = match &test.expected {
            ExpectedOutcome::Value(expected) => {
                let expected = self.value(expected);
                let text = format!(
                    "func Test{}(t *testing.T) {{\n\tgot := {}\n\tif !got.Ok || !testEqual(got.Value, {}) {{ t.Fatalf(\"unexpected result: %#v\", got) }}\n}}\n\n",
                    exported(&test.header.name),
                    call.text,
                    expected.text,
                );
                GoCode::with_dependencies(text, [call, expected])
            }
            ExpectedOutcome::Error(_) => {
                let text = format!(
                    "func Test{}(t *testing.T) {{\n\tgot := {}\n\tif got.Ok {{ t.Fatalf(\"expected error: %#v\", got) }}\n}}\n\n",
                    exported(&test.header.name),
                    call.text,
                );
                GoCode::with_dependencies(text, [call])
            }
        };
        output.with_import("testing")
    }
    fn method_name(&self, implementation: NodeId, method: NodeId) -> &str {
        self.program
            .module()
            .declarations
            .iter()
            .find_map(|item| match item {
                Declaration::Implementation(value) if value.header.node.id == implementation => {
                    value
                        .methods
                        .iter()
                        .find(|item| {
                            item.header.node.id == method || item.contract_method == method
                        })
                        .map(|item| item.header.name.as_str())
                }
                _ => None,
            })
            .unwrap_or("Unknown")
    }
    fn value(&self, typed: &TypedValue) -> GoCode {
        self.raw_value(&typed.value, &typed.ty)
    }
    fn raw_value(&self, value: &Value, ty: &TypeRef) -> GoCode {
        match (value, ty) {
            (Value::Unit, _) => GoCode::text("struct{}{}"),
            (Value::Bool(value), _) => GoCode::text(value.to_string()),
            (Value::I32(value), _) => GoCode::text(format!("int32({value})")),
            (Value::I64(value), _) => GoCode::text(format!("int64({value})")),
            (Value::F64(value), _) => {
                GoCode::text(format!("math.Float64frombits(0x{:016x})", value.0))
                    .with_import("math")
            }
            (Value::String(value), _) => GoCode::text(go_string(value)),
            (Value::Bytes(values), _) => GoCode::text(format!(
                "NewPolyBytes({})",
                values
                    .iter()
                    .map(|value| format!("0x{value:02x}"))
                    .collect::<Vec<_>>()
                    .join(", ")
            )),
            (
                Value::Record {
                    declaration,
                    fields,
                },
                _,
            ) => {
                let Declaration::Record(record) = self
                    .program
                    .module()
                    .declarations
                    .iter()
                    .find(|item| item.header().node.id == *declaration)
                    .expect("checked record")
                else {
                    unreachable!()
                };
                GoCode::joined(
                    ", ",
                    fields.iter().map(|field| {
                        let member = record
                            .fields
                            .iter()
                            .find(|item| item.header.node.id == field.field)
                            .expect("checked field");
                        self.raw_value(&field.value, &member.ty)
                            .map_text(|value| format!("{}: {value}", exported(&member.header.name)))
                    }),
                )
                .map_text(|fields| format!("{}{{{fields}}}", exported(&record.header.name)))
            }
            _ => GoCode::text("nil"),
        }
    }
}

fn args(parameters: &[portable_ir::v0::Parameter]) -> String {
    parameters
        .iter()
        .map(|item| local(&item.header.name))
        .collect::<Vec<_>>()
        .join(", ")
}
fn exported(name: &str) -> String {
    let mut result = String::new();
    for word in name
        .split(|value: char| !value.is_ascii_alphanumeric())
        .filter(|word| !word.is_empty())
    {
        let mut chars = word.chars();
        if let Some(first) = chars.next() {
            result.push(first.to_ascii_uppercase());
            result.extend(chars);
        }
    }
    if result.is_empty() {
        "Generated".into()
    } else {
        result
    }
}
fn local(name: &str) -> String {
    let value = exported(name);
    let mut chars = value.chars();
    let first = chars.next().unwrap_or('v').to_ascii_lowercase();
    let identifier = format!("{first}{}", chars.collect::<String>());
    if is_go_reserved(&identifier) {
        format!("{identifier}_")
    } else {
        identifier
    }
}

fn is_go_reserved(identifier: &str) -> bool {
    matches!(
        identifier,
        "any"
            | "append"
            | "bool"
            | "break"
            | "byte"
            | "cap"
            | "case"
            | "chan"
            | "clear"
            | "close"
            | "comparable"
            | "complex"
            | "complex64"
            | "complex128"
            | "const"
            | "continue"
            | "copy"
            | "default"
            | "defer"
            | "delete"
            | "else"
            | "error"
            | "fallthrough"
            | "false"
            | "float32"
            | "float64"
            | "for"
            | "func"
            | "go"
            | "goto"
            | "if"
            | "imag"
            | "import"
            | "int"
            | "int8"
            | "int16"
            | "int32"
            | "int64"
            | "interface"
            | "iota"
            | "len"
            | "make"
            | "map"
            | "max"
            | "min"
            | "new"
            | "nil"
            | "package"
            | "panic"
            | "print"
            | "println"
            | "range"
            | "real"
            | "recover"
            | "return"
            | "rune"
            | "select"
            | "string"
            | "struct"
            | "switch"
            | "true"
            | "type"
            | "uint"
            | "uint8"
            | "uint16"
            | "uint32"
            | "uint64"
            | "uintptr"
            | "var"
    )
}
fn go_string(value: &str) -> String {
    let mut output = String::from("\"");
    for character in value.chars() {
        match character {
            '\\' => output.push_str("\\\\"),
            '"' => output.push_str("\\\""),
            '\n' => output.push_str("\\n"),
            '\r' => output.push_str("\\r"),
            '\t' => output.push_str("\\t"),
            character if character.is_control() || character == '\u{feff}' => {
                output.push_str(&format!("\\u{:04x}", character as u32));
            }
            character => output.push(character),
        }
    }
    output.push('"');
    output
}

const CONFORMANCE_BODY: &str = "func TestFifteenSemanticVectors(t *testing.T) {\n source := NewPolyList(int32(1)); appended := source.append(2)\n wide32 := uint32(2147483648); wide64 := uint64(9223372036854775808)\n vectors := []bool{\n  int64(0) == 0, int64(9223372036854775807) > 0, int64(-9223372036854775807-1) < 0,\n  int32(wide32) == -2147483648, int64(wide64) == -9223372036854775807-1,\n  len([]rune(\"a\")) == 1, len([]rune(\"😀\")) == 1, NewPolyBytes(1,2).Values()[1] == 2,\n  source.Len() == 1, appended.Len() == 2, len(source.Values()) == 1, len(appended.Values()) == 2,\n  PolyOption[int]{Tag:\"none\"}.Tag == \"none\", PolyOption[int]{Tag:\"some\",Value:0}.Tag == \"some\", math.Signbit(math.Copysign(0,-1)),\n }\n if len(vectors)!=15 { t.Fatal(len(vectors)) }; for index,value:=range vectors { if !value { t.Fatal(index) } }\n}\n";

#[cfg(test)]
mod tests {
    use super::*;
    use portable_ir::v0::{
        Block, DeclarationHeader, Document as IrDocument, Expression, F64Bits, FunctionDeclaration,
        Module, NodeMeta, SourceRef, ValueField, Visibility,
    };
    #[test]
    fn deterministic_and_i64() {
        let checked = fixture();
        let first = GoV0Backend
            .generate(&checked, &BackendOptions::default())
            .unwrap();
        let second = GoV0Backend
            .generate(&checked, &BackendOptions::default())
            .unwrap();
        assert_eq!(first.canonical_json(), second.canonical_json());
        assert_eq!(Generator::new(&checked).ty(&TypeRef::I64).text, "int64");
    }
    #[test]
    fn strings_use_only_valid_go_escapes_and_preserve_unicode() {
        assert_eq!(
            go_string("\0\u{8}\n\r\t\\\"\u{301}\u{feff}🦄"),
            "\"\\u0000\\u0008\\n\\r\\t\\\\\\\"\u{301}\\ufeff🦄\""
        );
    }

    #[test]
    fn local_names_do_not_shadow_go_keywords_or_predeclared_identifiers() {
        assert_eq!(local("string"), "string_");
        assert_eq!(local("range"), "range_");
        assert_eq!(local("ordinary_name"), "ordinaryName");
    }

    #[test]
    fn imports_are_collected_per_go_file() {
        let manifest = GoV0Backend
            .generate(&fixture(), &BackendOptions::default())
            .unwrap();
        let tests = generated_text(&manifest, "generated_test.go");
        assert!(has_go_import(tests, "testing"));
        let runtime = generated_text(&manifest, "runtime.go");
        assert!(has_go_import(runtime, "encoding/json"));
        for optional in ["bytes", "math", "strconv", "unicode/utf8"] {
            assert!(!has_go_import(runtime, optional));
        }
        assert!(!runtime.contains("POLYRUST-BEGIN"));
        assert!(!runtime.contains("POLYRUST-END"));

        let empty = GoV0Backend
            .generate(&empty_fixture(), &BackendOptions::default())
            .unwrap();
        let empty_tests = generated_text(&empty, "generated_test.go");
        assert!(!empty_tests.contains("import"));
        assert!(empty_tests.contains("package generated"));
        let empty_runtime = generated_text(&empty, "runtime.go");
        assert!(has_go_import(empty_runtime, "encoding/json"));
        assert!(has_go_import(empty_runtime, "strings"));
        for optional in ["bytes", "math", "strconv", "unicode/utf8"] {
            assert!(!has_go_import(empty_runtime, optional));
        }
    }

    #[test]
    fn validated_go_import_paths_reject_rendered_or_ambiguous_data() {
        for valid in ["math", "encoding/json", "example.com/module-v2/pkg"] {
            assert!(GoImport::parse(valid).is_ok(), "rejected {valid:?}");
        }
        for invalid in [
            "",
            "/math",
            "math/",
            "encoding//json",
            "encoding/../json",
            "import math",
            "\"math\"",
        ] {
            assert!(GoImport::parse(invalid).is_err(), "accepted {invalid:?}");
        }
    }

    #[test]
    fn checked_go_features_select_exact_runtime_import_closures() {
        let cases = [
            (
                Intrinsic::IntAddChecked,
                ["math"].as_slice(),
                ["bytes", "strconv", "unicode/utf8"].as_slice(),
            ),
            (
                Intrinsic::FloatNeg,
                ["math", "strconv"].as_slice(),
                ["bytes", "unicode/utf8"].as_slice(),
            ),
            (
                Intrinsic::StringScalarLength,
                ["unicode/utf8"].as_slice(),
                ["bytes", "math", "strconv"].as_slice(),
            ),
            (
                Intrinsic::BytesReplaceAll,
                ["bytes"].as_slice(),
                ["math", "strconv", "unicode/utf8"].as_slice(),
            ),
            (
                Intrinsic::StringToUtf8,
                [].as_slice(),
                ["bytes", "math", "strconv", "unicode/utf8"].as_slice(),
            ),
        ];
        for (operation, required, forbidden) in cases {
            let manifest = GoV0Backend
                .generate(&intrinsic_fixture(operation), &BackendOptions::default())
                .unwrap();
            let runtime = generated_text(&manifest, "runtime.go");
            for path in required {
                assert!(has_go_import(runtime, path), "{operation:?} missing {path}");
            }
            for path in forbidden {
                assert!(
                    !has_go_import(runtime, path),
                    "{operation:?} unexpectedly imported {path}"
                );
            }
        }
    }

    #[test]
    fn nested_go_value_fragments_propagate_math_without_a_repair_scan() {
        let checked = fixture();
        let generator = Generator::new(&checked);
        let code = generator.raw_value(
            &Value::Record {
                declaration: NodeId::new(1),
                fields: vec![ValueField {
                    field: NodeId::new(2),
                    value: Value::F64(F64Bits::from_f64(-0.0)),
                }],
            },
            &TypeRef::Named(NodeId::new(1)),
        );
        assert_eq!(code.imports, BTreeSet::from([go_import("math")]));
    }

    fn generated_text<'a>(manifest: &'a OutputManifest, path: &str) -> &'a str {
        match manifest.file(path).unwrap().contents() {
            portable_codegen::OutputContents::Text(text) => text,
            portable_codegen::OutputContents::Bytes(_) => panic!("Go source must be text"),
        }
    }

    fn has_go_import(source: &str, path: &str) -> bool {
        let quoted = format!("{path:?}");
        let single = format!("import {quoted}");
        source
            .lines()
            .map(str::trim)
            .any(|line| line == quoted || line == single)
    }

    fn fixture() -> CheckedProgram {
        portable_check::v0::check_program(
            portable_ir::v0::from_json(include_bytes!(
                "../../build/testdata/registration.poly.json"
            ))
            .unwrap(),
        )
        .unwrap()
    }

    fn empty_fixture() -> CheckedProgram {
        portable_check::v0::check_program(
            portable_ir::v0::from_json(
                br#"{"ir_version":"0.1.0","module":{"name":"empty","declarations":[]},"metadata":{}}"#,
            )
            .unwrap(),
        )
        .unwrap()
    }

    fn intrinsic_fixture(operation: Intrinsic) -> CheckedProgram {
        let source = |id| SourceRef::logical([format!("go-runtime-feature-{id}")]);
        let node = |id| NodeMeta::new(NodeId::new(id), source(id));
        let (values, return_type) = match operation {
            Intrinsic::IntAddChecked => (vec![Value::I32(20), Value::I32(22)], TypeRef::I32),
            Intrinsic::FloatNeg => (vec![Value::F64(F64Bits::from_f64(1.5))], TypeRef::F64),
            Intrinsic::StringScalarLength => {
                (vec![Value::String("hello".to_owned())], TypeRef::I64)
            }
            Intrinsic::BytesReplaceAll => (
                vec![
                    Value::Bytes(vec![1, 2]),
                    Value::Bytes(vec![1]),
                    Value::Bytes(vec![3]),
                ],
                TypeRef::Bytes,
            ),
            Intrinsic::StringToUtf8 => (vec![Value::String("hello".to_owned())], TypeRef::Bytes),
            _ => panic!("unsupported Go runtime feature fixture"),
        };
        let arguments = values
            .into_iter()
            .enumerate()
            .map(|(index, value)| Expression::Literal {
                node: node(index as u64 + 2),
                value,
            })
            .collect();
        portable_check::v0::check_program(IrDocument::new(
            IrVersion::CURRENT,
            Module {
                name: "go_runtime_feature".to_owned(),
                declarations: vec![Declaration::Function(FunctionDeclaration {
                    header: DeclarationHeader {
                        node: node(1),
                        name: "feature".to_owned(),
                        visibility: Visibility::Public,
                        documentation: vec![],
                    },
                    parameters: vec![],
                    return_type,
                    body: Block {
                        node: node(101),
                        statements: vec![],
                        result: Some(Box::new(Expression::Intrinsic {
                            node: node(100),
                            operation,
                            arguments,
                        })),
                    },
                })],
            },
        ))
        .unwrap()
    }
}
