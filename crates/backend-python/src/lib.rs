//! Typed Python 3.13 generation from checked portable IR v0.

#![forbid(unsafe_code)]

use std::collections::{BTreeMap, BTreeSet};

use portable_check::v0::{Capability, CheckedProgram};
use portable_codegen::{
    Backend, BackendDescriptor, BackendError, BackendOptions, BackendVersion, CapabilitySupport,
    DeclaredDependency, Document as CodeDocument, FileGroup, FileGroupId, ImportGroup, ImportSet,
    InjectedHelper, IrVersionRange, LanguageFile, LanguageFragment, LanguagePackage,
    LanguagePlugin, LanguageRenderer, LanguageSourceFile, OptionsSchema, OutputManifest, RawText,
    RuntimeHelper, RuntimeHelperGraph, SourceFileRole, TargetId, TextFileRole,
    generate_with_plugin,
};
use portable_ir::v0::{Declaration, Intrinsic, IrVersion, NodeId, TypeRef};

const RUNTIME: &str = include_str!("runtime.py");

pub struct PythonBackend;

impl Backend for PythonBackend {
    fn descriptor(&self) -> BackendDescriptor {
        BackendDescriptor {
            target: TargetId::parse("org.polyrust.python").expect("valid target"),
            display_name: "Python".into(),
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
enum PythonImportKind {
    Future(&'static str),
    Module(&'static str),
    From {
        module: &'static str,
        name: &'static str,
    },
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
#[doc(hidden)]
pub struct PythonImport {
    kind: PythonImportKind,
}

impl PythonImport {
    fn future(name: &'static str) -> Result<Self, String> {
        validate_python_name(name)?;
        Ok(Self {
            kind: PythonImportKind::Future(name),
        })
    }

    fn module(module: &'static str) -> Result<Self, String> {
        validate_python_module(module, false)?;
        Ok(Self {
            kind: PythonImportKind::Module(module),
        })
    }

    fn from(module: &'static str, name: &'static str) -> Result<Self, String> {
        validate_python_module(module, true)?;
        validate_python_name(name)?;
        Ok(Self {
            kind: PythonImportKind::From { module, name },
        })
    }
}

fn validate_python_module(module: &str, allow_relative: bool) -> Result<(), String> {
    if !allow_relative && module.starts_with('.') {
        return Err(format!(
            "relative Python module is invalid for direct import {module:?}"
        ));
    }
    let absolute = module.trim_start_matches('.');
    if absolute.is_empty()
        || absolute
            .split('.')
            .any(|part| validate_python_name(part).is_err())
    {
        return Err(format!("invalid Python module name {module:?}"));
    }
    Ok(())
}

fn validate_python_name(name: &str) -> Result<(), String> {
    let mut characters = name.chars();
    if !matches!(characters.next(), Some(first) if first == '_' || first.is_ascii_alphabetic())
        || !characters.all(|character| character == '_' || character.is_ascii_alphanumeric())
    {
        return Err(format!("invalid Python import name {name:?}"));
    }
    Ok(())
}

#[doc(hidden)]
pub struct PythonRenderer;

impl LanguageRenderer<PythonImport> for PythonRenderer {
    fn render_imports(&self, imports: &ImportSet<PythonImport>) -> Result<CodeDocument, String> {
        let mut rendered_groups = Vec::new();
        for (_, imports) in imports.groups() {
            let mut futures = BTreeSet::new();
            let mut modules = BTreeSet::new();
            let mut from = BTreeMap::<&str, BTreeSet<&str>>::new();
            for import in imports {
                match &import.kind {
                    PythonImportKind::Future(name) => {
                        futures.insert(*name);
                    }
                    PythonImportKind::Module(module) => {
                        modules.insert(*module);
                    }
                    PythonImportKind::From { module, name } => {
                        from.entry(*module).or_default().insert(*name);
                    }
                }
            }
            let mut lines = Vec::new();
            if !futures.is_empty() {
                lines.push(format!(
                    "from __future__ import {}",
                    futures.into_iter().collect::<Vec<_>>().join(", ")
                ));
            }
            lines.extend(modules.into_iter().map(|module| format!("import {module}")));
            lines.extend(from.into_iter().map(|(module, names)| {
                format!(
                    "from {module} import {}",
                    names.into_iter().collect::<Vec<_>>().join(", ")
                )
            }));
            if !lines.is_empty() {
                rendered_groups.push(lines.join("\n"));
            }
        }
        Ok(CodeDocument::raw_text(RawText::new(
            rendered_groups.join("\n\n"),
        )))
    }
}

impl LanguagePlugin for PythonBackend {
    type Import = PythonImport;
    type Renderer = PythonRenderer;

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
                    files: vec!["src/generated_polyrust/runtime.py".into()],
                }),
                CapabilitySupport::Native | CapabilitySupport::Unsupported { .. } => None,
            })
            .collect();
        LanguagePackage::new(
            vec![
                FileGroup::new(
                    file_group("metadata")?,
                    vec![LanguageFile::text(
                        "pyproject.toml",
                        TextFileRole::Metadata,
                        PYPROJECT,
                    )],
                )
                .map_err(generation_error)?,
                FileGroup::new(
                    file_group("runtime")?,
                    vec![LanguageFile::source(runtime_file(program)?)],
                )
                .map_err(generation_error)?,
                FileGroup::new(
                    file_group("source")?,
                    vec![LanguageFile::source(generator.module_file()?)],
                )
                .map_err(generation_error)?,
                FileGroup::new(
                    file_group("tests")?,
                    vec![
                        LanguageFile::source(generator.tests_file()),
                        LanguageFile::source(conformance_file()),
                    ],
                )
                .map_err(generation_error)?,
                FileGroup::new(
                    file_group("negative-tests")?,
                    vec![LanguageFile::source(type_negative_file())],
                )
                .map_err(generation_error)?,
            ],
            Vec::<DeclaredDependency>::new(),
            helpers,
        )
        .map_err(generation_error)
    }

    fn renderer(&self) -> Self::Renderer {
        PythonRenderer
    }
}

fn generation_error(error: impl std::fmt::Display) -> BackendError {
    BackendError::Generation {
        message: error.to_string(),
    }
}

fn file_group(name: &str) -> Result<FileGroupId, BackendError> {
    FileGroupId::parse(name).map_err(generation_error)
}

fn future_group() -> ImportGroup {
    ImportGroup::new(0, "future").expect("static import group is valid")
}

fn standard_group() -> ImportGroup {
    ImportGroup::new(10, "standard-library").expect("static import group is valid")
}

fn local_group() -> ImportGroup {
    ImportGroup::new(20, "local-package").expect("static import group is valid")
}

fn python_future(name: &'static str) -> PythonImport {
    PythonImport::future(name).expect("static Python future import is valid")
}

fn python_module(module: &'static str) -> PythonImport {
    PythonImport::module(module).expect("static Python module import is valid")
}

fn python_from(module: &'static str, name: &'static str) -> PythonImport {
    PythonImport::from(module, name).expect("static Python from import is valid")
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct PythonCode {
    text: String,
    imports: BTreeSet<(ImportGroup, PythonImport)>,
}

impl PythonCode {
    fn text(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            imports: BTreeSet::new(),
        }
    }

    fn with_import(mut self, group: ImportGroup, import: PythonImport) -> Self {
        self.imports.insert((group, import));
        self
    }

    fn with_future(self, name: &'static str) -> Self {
        self.with_import(future_group(), python_future(name))
    }

    fn with_module(self, module: &'static str) -> Self {
        self.with_import(standard_group(), python_module(module))
    }

    fn with_from(self, group: ImportGroup, module: &'static str, name: &'static str) -> Self {
        self.with_import(group, python_from(module, name))
    }

    fn with_callable_imports(self) -> Self {
        self.with_future("annotations")
            .with_from(standard_group(), "typing", "cast")
            .with_from(local_group(), ".runtime", "PolyResult")
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

    fn dependency_text(&mut self, dependency: Self) -> String {
        self.imports.extend(dependency.imports);
        dependency.text
    }

    fn into_fragment(self) -> LanguageFragment<PythonImport> {
        let mut fragment = LanguageFragment::new(CodeDocument::raw_text(RawText::new(self.text)));
        for (group, import) in self.imports {
            fragment.require_import(group, import);
        }
        fragment
    }
}

fn runtime_file(
    program: &CheckedProgram,
) -> Result<LanguageSourceFile<PythonImport>, BackendError> {
    let (graph, mut roots) = python_runtime_helper_graph()?;
    if program.capabilities().program().contains(&Capability::F64) {
        roots.push("feature.f64".to_owned());
    }
    if portable_ir::v0::module_uses_intrinsic(program.module(), |operation| {
        operation == Intrinsic::StringUtf16Length
    }) {
        roots.push("feature.string-utf16-length".to_owned());
    }
    if portable_ir::v0::module_uses_intrinsic(program.module(), |operation| {
        operation == Intrinsic::ListIndexOf
    }) {
        roots.push("feature.list-index-of".to_owned());
    }
    let mut file =
        LanguageSourceFile::new("src/generated_polyrust/runtime.py", SourceFileRole::Runtime);
    file.set_body(graph.resolve(&roots).map_err(generation_error)?);
    Ok(file)
}

fn python_runtime_helper_graph()
-> Result<(RuntimeHelperGraph<PythonImport>, Vec<String>), BackendError> {
    const BEGIN: &str = "# POLYRUST-BEGIN ";
    const END: &str = "# POLYRUST-END ";
    let mut helpers = Vec::new();
    let mut common_roots = Vec::new();
    let mut common_index = 0_u16;
    let mut order = 0_u16;
    let mut active: Option<String> = None;
    let mut source = String::new();
    let emit = |id: String,
                source: &mut String,
                order: &mut u16,
                helpers: &mut Vec<RuntimeHelper<PythonImport>>| {
        if source.trim().is_empty() {
            source.clear();
            return false;
        }
        helpers.push(RuntimeHelper::new(
            id.clone(),
            *order,
            python_runtime_fragment(&id, std::mem::take(source)),
        ));
        *order = order
            .checked_add(1)
            .expect("Python runtime helper order fits u16");
        true
    };
    for line in RUNTIME.split_inclusive('\n') {
        let marker = line.trim().trim_end_matches('\r');
        if let Some(id) = marker.strip_prefix(BEGIN) {
            if active.is_some() {
                return Err(generation_error(format!(
                    "nested Python runtime helper marker {id:?}"
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
                return Err(generation_error(format!(
                    "unmatched Python runtime helper end marker {id:?}"
                )));
            };
            if open != id {
                return Err(generation_error(format!(
                    "Python runtime helper marker {open:?} closed by {id:?}"
                )));
            }
            if !emit(open, &mut source, &mut order, &mut helpers) {
                return Err(generation_error(format!(
                    "empty Python runtime helper {id:?}"
                )));
            }
        } else {
            source.push_str(line);
        }
    }
    if let Some(open) = active {
        return Err(generation_error(format!(
            "unclosed Python runtime helper marker {open:?}"
        )));
    }
    let common_id = format!("runtime.common.{common_index:03}");
    if emit(common_id.clone(), &mut source, &mut order, &mut helpers) {
        common_roots.push(common_id);
    }
    let f64 = [
        "f64-portable-equality",
        "f64-functions",
        "f64-decode",
        "f64-intrinsics",
    ]
    .into_iter()
    .fold(
        LanguageFragment::new(CodeDocument::empty()),
        |fragment, dependency| fragment.with_helper_root(dependency),
    );
    helpers.push(RuntimeHelper::new("feature.f64", u16::MAX, f64));
    helpers.push(RuntimeHelper::new(
        "feature.string-utf16-length",
        u16::MAX - 1,
        LanguageFragment::new(CodeDocument::empty()).with_helper_root("case.string-utf16-length"),
    ));
    helpers.push(RuntimeHelper::new(
        "feature.list-index-of",
        u16::MAX - 2,
        LanguageFragment::new(CodeDocument::empty()).with_helper_root("case.list-index-of"),
    ));
    Ok((
        RuntimeHelperGraph::new(helpers).map_err(generation_error)?,
        common_roots,
    ))
}

fn python_runtime_fragment(id: &str, source: String) -> LanguageFragment<PythonImport> {
    let mut fragment = LanguageFragment::new(CodeDocument::raw_text(RawText::new(source)));
    let imports: &[(ImportGroup, PythonImport)] = match id {
        "runtime.common.000" => &[
            (future_group(), python_future("annotations")),
            (standard_group(), python_from("dataclasses", "dataclass")),
            (standard_group(), python_from("types", "MappingProxyType")),
            (standard_group(), python_from("typing", "Any")),
            (standard_group(), python_from("typing", "Generic")),
            (standard_group(), python_from("typing", "TypeVar")),
        ],
        "runtime.common.001" => &[(standard_group(), python_from("types", "MappingProxyType"))],
        "runtime.common.002" => &[(standard_group(), python_from("typing", "Any"))],
        "runtime.common.003" | "runtime.common.004" => &[
            (standard_group(), python_from("types", "MappingProxyType")),
            (standard_group(), python_from("typing", "Any")),
        ],
        "f64-portable-equality" => &[
            (standard_group(), python_module("math")),
            (standard_group(), python_module("struct")),
        ],
        "f64-functions" | "f64-intrinsics" => &[(standard_group(), python_module("math"))],
        "f64-decode" => &[(standard_group(), python_module("struct"))],
        _ => &[],
    };
    for (group, import) in imports {
        fragment.require_import(group.clone(), import.clone());
    }
    fragment
}

fn conformance_file() -> LanguageSourceFile<PythonImport> {
    let mut file =
        LanguageSourceFile::new("tests/test_conformance.py", SourceFileRole::Conformance);
    let mut body = LanguageFragment::new(CodeDocument::raw_text(RawText::new(CONFORMANCE_BODY)));
    for name in ["checked_i32", "checked_i64", "scalar_length", "wrapping"] {
        body.require_import(
            local_group(),
            python_from("generated_polyrust.runtime", name),
        );
    }
    file.set_body(body);
    file
}

fn type_negative_file() -> LanguageSourceFile<PythonImport> {
    let mut file =
        LanguageSourceFile::new("negative/invalid_option.py", SourceFileRole::NegativeTest);
    let mut body = LanguageFragment::new(CodeDocument::raw_text(RawText::new(TYPE_NEGATIVE_BODY)));
    body.require_import(
        local_group(),
        python_from("generated_polyrust.runtime", "PolyOption"),
    );
    file.set_body(body);
    file
}

struct Generator<'a> {
    program: &'a CheckedProgram,
    names: BTreeMap<NodeId, String>,
}

impl<'a> Generator<'a> {
    fn new(program: &'a CheckedProgram) -> Self {
        let names = program
            .module()
            .declarations
            .iter()
            .map(|declaration| {
                (
                    declaration.header().node.id,
                    declaration.header().name.clone(),
                )
            })
            .collect();
        Self { program, names }
    }

    fn module_file(&self) -> Result<LanguageSourceFile<PythonImport>, BackendError> {
        let document =
            portable_ir::v0::to_canonical_json(self.program.document()).map_err(|error| {
                BackendError::Generation {
                    message: format!("cannot serialize checked IR: {error}"),
                }
            })?;
        let document = String::from_utf8(document).expect("canonical JSON is UTF-8");
        let literal = serde_json::to_string(&document).expect("JSON text string serializes");
        let mut file =
            LanguageSourceFile::new("src/generated_polyrust/__init__.py", SourceFileRole::Source);
        file.set_preamble(LanguageFragment::new(CodeDocument::raw_text(RawText::new(
            "# Generated by PolyRust from checked IR v0.",
        ))));
        let base = PythonCode::text(format!("_runtime = Runtime(json.loads({literal}))\n\n"))
            .with_module("json")
            .with_from(local_group(), ".runtime", "Runtime");
        let mut declarations: Vec<_> = self.program.module().declarations.iter().collect();
        declarations.sort_by_key(|declaration| declaration.header().node.id);
        let tests: Vec<_> = self.program.module().declarations.iter().filter_map(|declaration| if let Declaration::Test(test) = declaration { Some(serde_json::json!({"invocation": test.invocation, "expected": test.expected})) } else { None }).collect();
        let test_support = (!tests.is_empty()).then(|| self.test_support(&tests).into_fragment());
        file.set_body(LanguageFragment::sequence(
            std::iter::once(base.into_fragment())
                .chain(
                    declarations
                        .into_iter()
                        .map(|declaration| self.declaration(declaration).into_fragment()),
                )
                .chain(test_support),
        ));
        Ok(file)
    }

    fn test_support(&self, tests: &[serde_json::Value]) -> PythonCode {
        let tests = serde_json::to_string(&serde_json::to_string(tests).expect("tests serialize"))
            .expect("test JSON literal");
        PythonCode::text(format!("_TESTS: list[dict[str, object]] = json.loads({tests})\n\ndef _run_test(index: int) -> tuple[PolyResult[object], object, bool]:\n    test = _TESTS[index]\n    invocation = cast(dict[str, object], test[\"invocation\"])\n    data = cast(dict[str, object], invocation[\"data\"])\n    arguments = tuple(_runtime.decode(cast(dict[str, object], item)) for item in cast(list[object], data[\"arguments\"]))\n    if invocation[\"kind\"] == \"function\":\n        actual = _runtime.invoke(cast(int, data[\"function\"]), arguments)\n    else:\n        actual = _runtime.invoke_method(cast(int, data[\"implementation\"]), cast(int, data[\"method\"]), _runtime.decode(cast(dict[str, object], data[\"receiver\"])), arguments)\n    expected = cast(dict[str, object], test[\"expected\"])\n    return actual, _runtime.decode(cast(dict[str, object], expected[\"data\"])), expected[\"kind\"] == \"error\"\n"))
            .with_future("annotations")
            .with_from(standard_group(), "typing", "cast")
            .with_from(local_group(), ".runtime", "PolyResult")
    }

    fn declaration(&self, declaration: &Declaration) -> PythonCode {
        let mut output = PythonCode::text("");
        match declaration {
            Declaration::Alias(item) => {
                output = output.with_future("annotations").with_from(
                    standard_group(),
                    "typing",
                    "TypeAlias",
                );
                let target = output.dependency_text(self.ty(&item.target));
                output.text.push_str(&format!(
                    "{}: TypeAlias = {target}\n\n",
                    type_name(&item.header.name)
                ));
            }
            Declaration::Record(item) => {
                output = output
                    .with_future("annotations")
                    .with_from(standard_group(), "dataclasses", "dataclass")
                    .with_from(standard_group(), "dataclasses", "field");
                output
                    .text
                    .push_str("@dataclass(frozen=True, slots=True)\n");
                output.text.push_str(&format!(
                    "class {}:\n    __poly_decl__: int = field(default={}, init=False, repr=False)\n",
                    type_name(&item.header.name),
                    item.header.node.id.0
                ));
                if item.fields.is_empty() {
                    output.text.push_str("    pass\n");
                } else {
                    for field in &item.fields {
                        let ty = output.dependency_text(self.ty(&field.ty));
                        output
                            .text
                            .push_str(&format!("    {}: {ty}\n", value_name(&field.header.name)));
                    }
                }
                for implementation in self.implementations(item.header.node.id) {
                    if !implementation.methods.is_empty() {
                        output = output.with_callable_imports();
                    }
                    for method in &implementation.methods {
                        let parameters =
                            output.dependency_text(self.parameters(&method.parameters));
                        let return_type = output.dependency_text(self.ty(&method.return_type));
                        output.text.push_str(&format!(
                            "\n    def {}(self, {}) -> PolyResult[{return_type}]:\n        return cast(PolyResult[{return_type}], _runtime.invoke_method({}, {}, self, ({})))\n",
                            value_name(&method.header.name),
                            parameters,
                            implementation.header.node.id.0,
                            method.header.node.id.0,
                            tuple_arguments(&method.parameters)
                        ));
                    }
                }
                output.text.push('\n');
            }
            Declaration::Enum(item) => {
                output = output
                    .with_future("annotations")
                    .with_from(standard_group(), "dataclasses", "dataclass")
                    .with_from(standard_group(), "dataclasses", "field")
                    .with_from(standard_group(), "typing", "TypeAlias");
                let mut names = Vec::new();
                for variant in &item.variants {
                    let name = format!(
                        "{}{}",
                        type_name(&item.header.name),
                        type_name(&variant.header.name)
                    );
                    names.push(name.clone());
                    output
                        .text
                        .push_str("@dataclass(frozen=True, slots=True)\n");
                    output.text.push_str(&format!(
                        "class {name}:\n    tag: str = field(default={:?}, init=False)\n",
                        variant.header.name
                    ));
                    if variant.fields.is_empty() {
                        output.text.push_str("    pass\n");
                    } else {
                        for field in &variant.fields {
                            let ty = output.dependency_text(self.ty(&field.ty));
                            output.text.push_str(&format!(
                                "    {}: {ty}\n",
                                value_name(&field.header.name)
                            ));
                        }
                    }
                    output.text.push('\n');
                }
                output.text.push_str(&format!(
                    "{}: TypeAlias = {}\n\n",
                    type_name(&item.header.name),
                    names.join(" | ")
                ));
            }
            Declaration::Contract(item) => {
                output = output
                    .with_future("annotations")
                    .with_from(standard_group(), "typing", "Protocol")
                    .with_from(local_group(), ".runtime", "PolyResult");
                output.text.push_str(&format!(
                    "class {}(Protocol):\n",
                    type_name(&item.header.name)
                ));
                if item.methods.is_empty() {
                    output.text.push_str("    pass\n");
                } else {
                    for method in &item.methods {
                        let parameters =
                            output.dependency_text(self.parameters(&method.parameters));
                        let return_type = output.dependency_text(self.ty(&method.return_type));
                        output.text.push_str(&format!(
                            "    def {}(self, {}) -> PolyResult[{return_type}]: ...\n",
                            value_name(&method.header.name),
                            parameters
                        ));
                    }
                }
                output.text.push('\n');
            }
            Declaration::Constant(item) => {
                output = output.with_callable_imports();
                let ty = output.dependency_text(self.ty(&item.ty));
                output.text.push_str(&format!(
                    "def {}() -> PolyResult[{ty}]:\n    return cast(PolyResult[{ty}], _runtime.read_constant({}))\n\n",
                    value_name(&item.header.name),
                    item.header.node.id.0
                ));
            }
            Declaration::Function(item) => {
                output = output.with_callable_imports();
                let parameters = output.dependency_text(self.parameters(&item.parameters));
                let return_type = output.dependency_text(self.ty(&item.return_type));
                output.text.push_str(&format!(
                    "def {}({parameters}) -> PolyResult[{return_type}]:\n    return cast(PolyResult[{return_type}], _runtime.invoke({}, ({})))\n\n",
                    value_name(&item.header.name),
                    item.header.node.id.0,
                    tuple_arguments(&item.parameters)
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
            .filter_map(|declaration| match declaration {
                Declaration::Implementation(item) if item.record == record => Some(item),
                _ => None,
            })
            .collect()
    }
    fn parameters(&self, parameters: &[portable_ir::v0::Parameter]) -> PythonCode {
        PythonCode::joined(
            ", ",
            parameters.iter().map(|parameter| {
                self.ty(&parameter.ty)
                    .map_text(|ty| format!("{}: {ty}", value_name(&parameter.header.name)))
            }),
        )
    }
    fn ty(&self, ty: &TypeRef) -> PythonCode {
        match ty {
            TypeRef::Unit => PythonCode::text("None"),
            TypeRef::Bool => PythonCode::text("bool"),
            TypeRef::I32 | TypeRef::I64 => PythonCode::text("int"),
            TypeRef::F64 => PythonCode::text("float"),
            TypeRef::Char | TypeRef::String => PythonCode::text("str"),
            TypeRef::Bytes => PythonCode::text("bytes"),
            TypeRef::List(inner) => self
                .ty(inner)
                .map_text(|inner| format!("tuple[{inner}, ...]")),
            TypeRef::Option(inner) => self
                .ty(inner)
                .map_text(|inner| format!("PolyOption[{inner}]"))
                .with_from(local_group(), ".runtime", "PolyOption"),
            TypeRef::Result { ok, error } => {
                PythonCode::joined(", ", [self.ty(ok), self.ty(error)])
                    .map_text(|types| format!("PolyValueResult[{types}]"))
                    .with_from(local_group(), ".runtime", "PolyValueResult")
            }
            TypeRef::Named(id) | TypeRef::Contract(id) => PythonCode::text(type_name(
                self.names.get(id).map(String::as_str).unwrap_or("Unknown"),
            )),
        }
    }
    fn tests_file(&self) -> LanguageSourceFile<PythonImport> {
        let mut file = LanguageSourceFile::new("tests/test_generated.py", SourceFileRole::Test);
        file.set_body(LanguageFragment::sequence(
            self.program
                .module()
                .declarations
                .iter()
                .filter_map(|declaration| match declaration {
                    Declaration::Test(test) => Some(test),
                    _ => None,
                })
                .enumerate()
                .map(|(index, test)| {
                    PythonCode::text(format!("def test_{}() -> None:\n    actual, expected, expects_error = _run_test({index})\n    assert actual.ok is not expects_error\n    if actual.ok:\n        assert portable_test_equal(actual.value, expected)\n\n", value_name(&test.header.name)))
                        .with_from(local_group(), "generated_polyrust", "_run_test")
                        .with_from(
                            local_group(),
                            "generated_polyrust.runtime",
                            "portable_test_equal",
                        )
                        .into_fragment()
                }),
        ));
        file
    }
}

fn tuple_arguments(parameters: &[portable_ir::v0::Parameter]) -> String {
    match parameters {
        [] => String::new(),
        [one] => format!("{},", value_name(&one.header.name)),
        many => many
            .iter()
            .map(|parameter| value_name(&parameter.header.name))
            .collect::<Vec<_>>()
            .join(", "),
    }
}
fn type_name(name: &str) -> String {
    identifier(name)
}
fn value_name(name: &str) -> String {
    identifier(name)
}
fn identifier(name: &str) -> String {
    const KEYWORDS: &[&str] = &[
        "False", "None", "True", "and", "as", "assert", "async", "await", "break", "class",
        "continue", "def", "del", "elif", "else", "except", "finally", "for", "from", "global",
        "if", "import", "in", "is", "lambda", "nonlocal", "not", "or", "pass", "raise", "return",
        "try", "while", "with", "yield", "match", "case",
    ];
    if KEYWORDS.contains(&name) {
        format!("{name}_")
    } else {
        name.to_owned()
    }
}

const PYPROJECT: &str = "[project]\nname = \"generated-polyrust-package\"\nversion = \"0.1.0\"\nrequires-python = \">=3.13\"\ndependencies = []\n\n[tool.pytest.ini_options]\npythonpath = [\"src\"]\ntestpaths = [\"tests\"]\n\n[tool.ruff]\ntarget-version = \"py313\"\nline-length = 120\n\n[tool.ruff.lint]\nselect = [\"E4\", \"E7\", \"E9\", \"F\"]\nignore = [\"E701\", \"E702\"]\n";
const CONFORMANCE_BODY: &str = "def test_twenty_semantic_vectors() -> None:\n    source = (1,)\n    appended = source + (2,)\n    vectors = (\n        checked_i32(0).ok, checked_i32(2**31 - 1).ok, checked_i32(-(2**31)).ok, not checked_i32(2**31).ok, not checked_i32(-(2**31) - 1).ok,\n        checked_i64(0).ok, checked_i64(2**63 - 1).ok, checked_i64(-(2**63)).ok, not checked_i64(2**63).ok, not checked_i64(-(2**63) - 1).ok,\n        wrapping(2**31, 32) == -(2**31), wrapping(-(2**31) - 1, 32) == 2**31 - 1, wrapping(2**63, 64) == -(2**63), wrapping(-(2**63) - 1, 64) == 2**63 - 1,\n        scalar_length(\"a\").ok, scalar_length(\"😀\").value == 1, not scalar_length(\"\\ud800\").ok, len(appended) == 2, id(appended) != id(source), float(\"-0.0\").hex().startswith(\"-\"),\n    )\n    assert len(vectors) == 20\n    assert all(vectors)\n";
const TYPE_NEGATIVE_BODY: &str = "invalid: PolyOption[int] = PolyOption(tag=1)\n";

#[cfg(test)]
mod tests {
    use super::*;
    use portable_ir::v0::{
        Block, DeclarationHeader, Document as IrDocument, Expression, F64Bits, FunctionDeclaration,
        Intrinsic, Module, NodeMeta, SourceRef, Value, Visibility,
    };
    #[test]
    fn keyword_and_types() {
        assert_eq!(identifier("class"), "class_");
        assert_eq!(Generator::new(&fixture()).ty(&TypeRef::I64).text, "int");
    }
    #[test]
    fn deterministic_strict_manifest() {
        let checked = fixture();
        let first = PythonBackend
            .generate(&checked, &BackendOptions::default())
            .unwrap();
        let second = PythonBackend
            .generate(&checked, &BackendOptions::default())
            .unwrap();
        assert_eq!(first.canonical_json(), second.canonical_json());
        assert!(PYPROJECT.contains("py313"));
    }
    #[test]
    fn generated_module_imports_follow_mapped_constructs_and_runtime_imports_are_merged() {
        let manifest = PythonBackend
            .generate(&fixture(), &BackendOptions::default())
            .unwrap();
        let generated = match manifest
            .file("src/generated_polyrust/__init__.py")
            .unwrap()
            .contents()
        {
            portable_codegen::OutputContents::Text(text) => text,
            portable_codegen::OutputContents::Bytes(_) => panic!("Python source must be text"),
        };
        assert!(generated.contains("from dataclasses import dataclass, field"));
        assert!(!generated.contains("# noqa: F401"));
        assert!(generated.contains("import json"));
        assert!(generated.contains("from typing import Protocol, cast"));

        let empty_manifest = PythonBackend
            .generate(&empty_fixture(), &BackendOptions::default())
            .unwrap();
        let empty = match empty_manifest
            .file("src/generated_polyrust/__init__.py")
            .unwrap()
            .contents()
        {
            portable_codegen::OutputContents::Text(text) => text,
            portable_codegen::OutputContents::Bytes(_) => panic!("Python source must be text"),
        };
        assert!(!empty.contains("from __future__ import"));
        assert!(!empty.contains("from dataclasses import"));
        assert!(!empty.contains("from typing import"));
        assert_eq!(empty.matches("import json").count(), 1);
        assert_eq!(empty.matches("from .runtime import Runtime").count(), 1);

        let runtime = match manifest
            .file("src/generated_polyrust/runtime.py")
            .unwrap()
            .contents()
        {
            portable_codegen::OutputContents::Text(text) => text,
            portable_codegen::OutputContents::Bytes(_) => panic!("Python runtime must be text"),
        };
        assert!(runtime.contains("from dataclasses import dataclass"));
        assert!(runtime.contains("from typing import Any, Generic, TypeVar"));
        assert_eq!(runtime.matches("from dataclasses import").count(), 1);
        assert!(!runtime.contains("import math"));
        assert!(!runtime.contains("import struct"));
        assert!(!runtime.contains("POLYRUST-BEGIN"));
        assert!(!runtime.contains("POLYRUST-END"));
    }

    #[test]
    fn python_import_data_is_validated_before_rendering() {
        for module in ["json", "generated_polyrust.runtime"] {
            assert!(PythonImport::module(module).is_ok(), "rejected {module:?}");
        }
        for invalid in [
            "",
            ".",
            "typing..extensions",
            "import os",
            "typing.*",
            "typing/extension",
        ] {
            assert!(
                PythonImport::module(invalid).is_err(),
                "accepted {invalid:?}"
            );
        }
        assert!(PythonImport::from("typing", "Protocol").is_ok());
        assert!(PythonImport::from(".runtime", "PolyResult").is_ok());
        assert!(PythonImport::module(".runtime").is_err());
        assert!(PythonImport::from("typing", "Protocol, cast").is_err());
    }

    #[test]
    fn python_type_fragments_propagate_nested_runtime_imports() {
        let checked = fixture();
        let code = Generator::new(&checked).ty(&TypeRef::Result {
            ok: Box::new(TypeRef::Option(Box::new(TypeRef::I64))),
            error: Box::new(TypeRef::String),
        });
        let imports = code
            .imports
            .iter()
            .map(|(_, import)| match &import.kind {
                PythonImportKind::From { module, name } => format!("{module}:{name}"),
                PythonImportKind::Future(name) => format!("future:{name}"),
                PythonImportKind::Module(module) => format!("module:{module}"),
            })
            .collect::<BTreeSet<_>>();
        assert_eq!(
            imports,
            BTreeSet::from([
                ".runtime:PolyOption".to_owned(),
                ".runtime:PolyValueResult".to_owned(),
            ])
        );
    }

    #[test]
    fn checked_f64_program_selects_python_math_struct_closure() {
        let manifest = PythonBackend
            .generate(&f64_fixture(), &BackendOptions::default())
            .unwrap();
        let runtime = generated_text(&manifest, "src/generated_polyrust/runtime.py");
        assert_eq!(runtime.matches("import math").count(), 1);
        assert_eq!(runtime.matches("import struct").count(), 1);
        assert!(runtime.contains("def float_div"));
        assert!(runtime.contains("struct.unpack"));
        assert!(!runtime.contains("POLYRUST-BEGIN"));
    }

    fn generated_text<'a>(manifest: &'a OutputManifest, path: &str) -> &'a str {
        match manifest.file(path).unwrap().contents() {
            portable_codegen::OutputContents::Text(text) => text,
            portable_codegen::OutputContents::Bytes(_) => panic!("Python source must be text"),
        }
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

    fn f64_fixture() -> CheckedProgram {
        let source = |id| SourceRef::logical([format!("python-f64-{id}")]);
        let node = |id| NodeMeta::new(NodeId::new(id), source(id));
        portable_check::v0::check_program(IrDocument::new(
            IrVersion::CURRENT,
            Module {
                name: "python_f64".to_owned(),
                declarations: vec![Declaration::Function(FunctionDeclaration {
                    header: DeclarationHeader {
                        node: node(1),
                        name: "negate".to_owned(),
                        visibility: Visibility::Public,
                        documentation: vec![],
                    },
                    parameters: vec![],
                    return_type: TypeRef::F64,
                    body: Block {
                        node: node(4),
                        statements: vec![],
                        result: Some(Box::new(Expression::Intrinsic {
                            node: node(3),
                            operation: Intrinsic::FloatNeg,
                            arguments: vec![Expression::Literal {
                                node: node(2),
                                value: Value::F64(F64Bits::from_f64(1.5)),
                            }],
                        })),
                    },
                })],
            },
        ))
        .unwrap()
    }
}
