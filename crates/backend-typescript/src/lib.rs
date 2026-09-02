//! Strict ESM TypeScript generation from checked portable IR v0.

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
use portable_ir::v0::{Declaration, Intrinsic, IrVersion, NodeId, TypeRef, Visibility};

const RUNTIME: &str = include_str!("runtime.ts");
const JAVASCRIPT_RUNTIME: &str = include_str!("runtime.js");

pub struct TypeScriptBackend;
pub struct JavaScriptBackend;

fn support(capability: Capability) -> CapabilitySupport {
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

impl Backend for TypeScriptBackend {
    fn descriptor(&self) -> BackendDescriptor {
        BackendDescriptor {
            target: TargetId::parse("org.polyrust.typescript").expect("static target ID is valid"),
            display_name: "TypeScript".to_owned(),
            backend_version: BackendVersion::new(0, 1, 0),
            supported_ir: IrVersionRange::exact(IrVersion::CURRENT),
        }
    }

    fn support(&self, capability: Capability) -> CapabilitySupport {
        support(capability)
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

impl Backend for JavaScriptBackend {
    fn descriptor(&self) -> BackendDescriptor {
        BackendDescriptor {
            target: TargetId::parse("org.polyrust.javascript").expect("static target ID is valid"),
            display_name: "JavaScript".to_owned(),
            backend_version: BackendVersion::new(0, 1, 0),
            supported_ir: IrVersionRange::exact(IrVersion::CURRENT),
        }
    }

    fn support(&self, capability: Capability) -> CapabilitySupport {
        support(capability)
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
pub struct EcmaImport {
    kind: EcmaImportKind,
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
enum EcmaImportKind {
    Default {
        module: String,
        name: String,
    },
    Named {
        module: String,
        name: String,
        type_only: bool,
    },
    ExportAll {
        module: String,
    },
}

impl EcmaImport {
    pub fn default(module: &str, name: &str) -> Result<Self, String> {
        Ok(Self {
            kind: EcmaImportKind::Default {
                module: ecma_module(module)?,
                name: ecma_symbol(name)?,
            },
        })
    }

    pub fn named(module: &str, name: &str, type_only: bool) -> Result<Self, String> {
        Ok(Self {
            kind: EcmaImportKind::Named {
                module: ecma_module(module)?,
                name: ecma_symbol(name)?,
                type_only,
            },
        })
    }

    pub fn export_all(module: &str) -> Result<Self, String> {
        Ok(Self {
            kind: EcmaImportKind::ExportAll {
                module: ecma_module(module)?,
            },
        })
    }

    fn is_type_only(&self) -> bool {
        matches!(
            self.kind,
            EcmaImportKind::Named {
                type_only: true,
                ..
            }
        )
    }
}

fn ecma_module(module: &str) -> Result<String, String> {
    let valid = !module.is_empty()
        && !module.starts_with('/')
        && !module.ends_with('/')
        && !module.contains("//")
        && module.chars().all(|character| {
            character.is_ascii_alphanumeric()
                || matches!(character, '@' | '_' | '-' | '.' | '/' | ':')
        });
    if valid {
        Ok(module.to_owned())
    } else {
        Err(format!("invalid ECMAScript module specifier {module:?}"))
    }
}

fn ecma_symbol(name: &str) -> Result<String, String> {
    let mut characters = name.chars();
    let valid = characters
        .next()
        .is_some_and(|first| first == '_' || first == '$' || first.is_ascii_alphabetic())
        && characters.all(|character| {
            character == '_' || character == '$' || character.is_ascii_alphanumeric()
        });
    if valid {
        Ok(name.to_owned())
    } else {
        Err(format!("invalid ECMAScript import symbol {name:?}"))
    }
}

#[doc(hidden)]
pub struct EcmaRenderer;

impl LanguageRenderer<EcmaImport> for EcmaRenderer {
    fn render_imports(&self, imports: &ImportSet<EcmaImport>) -> Result<CodeDocument, String> {
        let mut groups = Vec::new();
        for (_, imports) in imports.groups() {
            let mut defaults = Vec::new();
            let mut exports = Vec::new();
            let mut named = BTreeMap::<&str, (BTreeSet<&str>, BTreeSet<&str>)>::new();
            for import in imports {
                match &import.kind {
                    EcmaImportKind::Default { module, name } => {
                        defaults.push(format!("import {name} from {module:?};"));
                    }
                    EcmaImportKind::Named {
                        module,
                        name,
                        type_only,
                    } => {
                        let names = named.entry(module.as_str()).or_default();
                        if *type_only {
                            names.1.insert(name.as_str());
                        } else {
                            names.0.insert(name.as_str());
                        }
                    }
                    EcmaImportKind::ExportAll { module } => {
                        exports.push(format!("export * from {module:?};"));
                    }
                }
            }
            let mut lines = defaults;
            for (module, (values, types)) in named {
                let mut names = values.into_iter().map(str::to_owned).collect::<Vec<_>>();
                names.extend(types.into_iter().map(|name| format!("type {name}")));
                lines.push(format!(
                    "import {{ {} }} from {module:?};",
                    names.join(", ")
                ));
            }
            lines.extend(exports);
            if !lines.is_empty() {
                groups.push(lines.join("\n"));
            }
        }
        Ok(CodeDocument::raw_text(RawText::new(groups.join("\n\n"))))
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
struct EcmaCode {
    typescript: String,
    javascript: String,
    imports: BTreeSet<(ImportGroup, EcmaImport)>,
}

impl EcmaCode {
    fn paired(typescript: impl Into<String>, javascript: impl Into<String>) -> Self {
        Self {
            typescript: typescript.into(),
            javascript: javascript.into(),
            imports: BTreeSet::new(),
        }
    }

    fn same(text: impl Into<String>) -> Self {
        let text = text.into();
        Self::paired(text.clone(), text)
    }

    fn typescript(text: impl Into<String>) -> Self {
        Self::paired(text, String::new())
    }

    fn with_default(mut self, group: ImportGroup, module: &str, name: &str) -> Self {
        self.imports.insert((
            group,
            EcmaImport::default(module, name).expect("static ECMAScript import is valid"),
        ));
        self
    }

    fn with_named(mut self, module: &str, name: &str, type_only: bool) -> Self {
        self.imports.insert((
            local_import_group(),
            EcmaImport::named(module, name, type_only).expect("static ECMAScript import is valid"),
        ));
        self
    }

    fn with_export_all(mut self, module: &str) -> Self {
        self.imports.insert((
            export_group(),
            EcmaImport::export_all(module).expect("static ECMAScript export is valid"),
        ));
        self
    }

    fn sequence(fragments: impl IntoIterator<Item = Self>) -> Self {
        fragments
            .into_iter()
            .fold(Self::default(), |mut combined, fragment| {
                combined.typescript.push_str(&fragment.typescript);
                combined.javascript.push_str(&fragment.javascript);
                combined.imports.extend(fragment.imports);
                combined
            })
    }

    fn joined(fragments: impl IntoIterator<Item = Self>, separator: &str) -> Self {
        let mut fragments = fragments.into_iter();
        let Some(first) = fragments.next() else {
            return Self::default();
        };
        fragments.fold(first, |mut combined, fragment| {
            combined.typescript.push_str(separator);
            combined.typescript.push_str(&fragment.typescript);
            combined.javascript.push_str(separator);
            combined.javascript.push_str(&fragment.javascript);
            combined.imports.extend(fragment.imports);
            combined
        })
    }

    fn into_fragment(self, javascript: bool) -> LanguageFragment<EcmaImport> {
        let text = if javascript {
            self.javascript
        } else {
            self.typescript
        };
        let mut fragment = LanguageFragment::new(CodeDocument::raw_text(RawText::new(text)));
        for (group, dependency) in self.imports {
            if !javascript || !dependency.is_type_only() {
                fragment.require_import(group, dependency);
            }
        }
        fragment
    }
}

impl LanguagePlugin for TypeScriptBackend {
    type Import = EcmaImport;
    type Renderer = EcmaRenderer;

    fn translate(
        &self,
        program: &CheckedProgram,
        _options: &BackendOptions,
    ) -> Result<LanguagePackage<Self::Import>, BackendError> {
        ecma_package(self, program, false)
    }

    fn renderer(&self) -> Self::Renderer {
        EcmaRenderer
    }
}

impl LanguagePlugin for JavaScriptBackend {
    type Import = EcmaImport;
    type Renderer = EcmaRenderer;

    fn translate(
        &self,
        program: &CheckedProgram,
        _options: &BackendOptions,
    ) -> Result<LanguagePackage<Self::Import>, BackendError> {
        ecma_package(self, program, true)
    }

    fn renderer(&self) -> Self::Renderer {
        EcmaRenderer
    }
}

fn ecma_package(
    _backend: &impl Backend,
    program: &CheckedProgram,
    javascript: bool,
) -> Result<LanguagePackage<EcmaImport>, BackendError> {
    let generator = Generator::new(program);
    let runtime_path = if javascript {
        "src/runtime.js"
    } else {
        "src/runtime.ts"
    };
    let helpers = program
        .capabilities()
        .program()
        .iter()
        .filter_map(|capability| match support(*capability) {
            CapabilitySupport::Helper { helper } => Some(InjectedHelper {
                id: helper,
                capability: format!("{capability:?}"),
                files: vec![runtime_path.into()],
            }),
            CapabilitySupport::Native | CapabilitySupport::Unsupported { .. } => None,
        })
        .collect();
    let metadata = if javascript {
        vec![LanguageFile::text(
            "package.json",
            TextFileRole::Metadata,
            JAVASCRIPT_PACKAGE_JSON,
        )]
    } else {
        vec![
            LanguageFile::text("package.json", TextFileRole::Metadata, PACKAGE_JSON),
            LanguageFile::text("tsconfig.json", TextFileRole::Metadata, TSCONFIG),
        ]
    };
    let mut groups = vec![
        FileGroup::new(ecma_group("metadata")?, metadata).map_err(ecma_generation_error)?,
        FileGroup::new(
            ecma_group("runtime")?,
            vec![LanguageFile::source(ecma_runtime_file(
                program, javascript,
            )?)],
        )
        .map_err(ecma_generation_error)?,
        FileGroup::new(
            ecma_group("source")?,
            vec![LanguageFile::source(generator.index_file(javascript)?)],
        )
        .map_err(ecma_generation_error)?,
        FileGroup::new(
            ecma_group("tests")?,
            vec![
                LanguageFile::source(generator.tests_file(javascript)),
                LanguageFile::source(conformance_file(javascript)),
            ],
        )
        .map_err(ecma_generation_error)?,
    ];
    if !javascript {
        groups.push(
            FileGroup::new(
                ecma_group("type-system-tests")?,
                vec![
                    LanguageFile::source(node_shims_file()),
                    LanguageFile::source(invalid_types_file()),
                ],
            )
            .map_err(ecma_generation_error)?,
        );
    }
    LanguagePackage::new(groups, Vec::<DeclaredDependency>::new(), helpers)
        .map_err(ecma_generation_error)
}

fn ecma_generation_error(error: impl std::fmt::Display) -> BackendError {
    BackendError::Generation {
        message: error.to_string(),
    }
}

fn ecma_group(name: &str) -> Result<FileGroupId, BackendError> {
    FileGroupId::parse(name).map_err(ecma_generation_error)
}

fn node_import_group() -> ImportGroup {
    ImportGroup::new(10, "node-standard-library").expect("static import group is valid")
}

fn local_import_group() -> ImportGroup {
    ImportGroup::new(20, "local-modules").expect("static import group is valid")
}

fn export_group() -> ImportGroup {
    ImportGroup::new(30, "module-exports").expect("static import group is valid")
}

fn ecma_runtime_file(
    program: &CheckedProgram,
    javascript: bool,
) -> Result<LanguageSourceFile<EcmaImport>, BackendError> {
    let path = if javascript {
        "src/runtime.js"
    } else {
        "src/runtime.ts"
    };
    let (graph, mut roots) = ecma_runtime_helper_graph(javascript)?;
    for (operation, root) in [
        (Intrinsic::StringReplaceAll, "feature.string-replace-all"),
        (Intrinsic::BytesReplaceAll, "feature.bytes-replace-all"),
        (Intrinsic::StringReplaceMany, "feature.string-replace-many"),
        (
            Intrinsic::StringTruncateUtf8Bytes,
            "feature.string-truncate-utf8",
        ),
        (Intrinsic::StringTrimStart, "feature.string-trim-start"),
        (Intrinsic::StringTrimEnd, "feature.string-trim-end"),
        (Intrinsic::StringToUtf8, "feature.string-to-utf8"),
        (Intrinsic::StringFromUtf8Checked, "feature.string-from-utf8"),
    ] {
        if portable_ir::v0::module_uses_intrinsic(program.module(), |used| used == operation) {
            roots.push(root.to_owned());
        }
    }
    if portable_ir::v0::module_uses_intrinsic(program.module(), |operation| {
        matches!(operation, Intrinsic::BytesConcat | Intrinsic::ListConcat)
    }) {
        roots.push("feature.list-concat".to_owned());
    }
    let mut file = LanguageSourceFile::new(path, SourceFileRole::Runtime);
    file.set_body(graph.resolve(&roots).map_err(ecma_generation_error)?);
    Ok(file)
}

#[derive(Debug)]
struct RuntimeSegment {
    id: String,
    source: String,
    common: bool,
}

fn ecma_runtime_helper_graph(
    javascript: bool,
) -> Result<(RuntimeHelperGraph<EcmaImport>, Vec<String>), BackendError> {
    let typescript = parse_runtime_segments(RUNTIME, "TypeScript")?;
    let derived = parse_runtime_segments(JAVASCRIPT_RUNTIME, "JavaScript")?;
    if typescript.len() != derived.len()
        || typescript
            .iter()
            .zip(&derived)
            .any(|(left, right)| left.id != right.id || left.common != right.common)
    {
        return Err(ecma_generation_error(
            "TypeScript and derived JavaScript runtime helper layouts differ",
        ));
    }

    let mut common_roots = Vec::new();
    let mut helpers = Vec::new();
    for (order, (typescript, derived)) in typescript.into_iter().zip(derived).enumerate() {
        if typescript.common {
            common_roots.push(typescript.id.clone());
        }
        helpers.push(RuntimeHelper::new(
            typescript.id,
            u16::try_from(order).expect("ECMAScript runtime helper order fits u16"),
            EcmaCode::paired(typescript.source, derived.source).into_fragment(javascript),
        ));
    }
    let feature = |dependencies: &[&str]| {
        dependencies.iter().fold(
            LanguageFragment::new(CodeDocument::empty()),
            |fragment, dependency| fragment.with_helper_root(*dependency),
        )
    };
    for (index, (id, dependencies)) in [
        (
            "feature.string-replace-all",
            &["top.string-replace-all", "case.string-replace-all"][..],
        ),
        (
            "feature.bytes-replace-all",
            &["top.bytes-replace-all", "case.bytes-replace-all"][..],
        ),
        (
            "feature.string-replace-many",
            &["top.string-replace-many", "case.string-replace-many"][..],
        ),
        (
            "feature.string-truncate-utf8",
            &["top.string-truncate-utf8", "case.string-truncate-utf8"][..],
        ),
        (
            "feature.string-trim-start",
            &["top.string-trim-start", "case.string-trim-start"][..],
        ),
        (
            "feature.string-trim-end",
            &["top.string-trim-end", "case.string-trim-end"][..],
        ),
        (
            "feature.list-concat",
            &["top.list-concat", "case.list-concat"][..],
        ),
        ("feature.string-to-utf8", &["case.string-to-utf8"][..]),
        ("feature.string-from-utf8", &["case.string-from-utf8"][..]),
    ]
    .into_iter()
    .enumerate()
    {
        helpers.push(RuntimeHelper::new(
            id,
            u16::MAX - 16 + u16::try_from(index).expect("feature count fits u16"),
            feature(dependencies),
        ));
    }
    Ok((
        RuntimeHelperGraph::new(helpers).map_err(ecma_generation_error)?,
        common_roots,
    ))
}

fn parse_runtime_segments(
    runtime: &str,
    dialect: &str,
) -> Result<Vec<RuntimeSegment>, BackendError> {
    const BEGIN: &str = "// POLYRUST-BEGIN ";
    const END: &str = "// POLYRUST-END ";

    let mut segments = Vec::new();
    let mut common_index = 0_u16;
    let mut active: Option<String> = None;
    let mut source = String::new();
    let emit =
        |id: String, common: bool, source: &mut String, segments: &mut Vec<RuntimeSegment>| {
            if source.trim().is_empty() {
                source.clear();
                return false;
            }
            segments.push(RuntimeSegment {
                id,
                source: std::mem::take(source),
                common,
            });
            true
        };

    for line in runtime.split_inclusive('\n') {
        let marker = line.trim().trim_end_matches('\r');
        if let Some(id) = marker.strip_prefix(BEGIN) {
            if active.is_some() {
                return Err(ecma_generation_error(format!(
                    "nested {dialect} runtime helper marker {id:?}"
                )));
            }
            let common_id = format!("runtime.common.{common_index:03}");
            if emit(common_id, true, &mut source, &mut segments) {
                common_index += 1;
            }
            active = Some(id.to_owned());
        } else if let Some(id) = marker.strip_prefix(END) {
            let Some(open) = active.take() else {
                return Err(ecma_generation_error(format!(
                    "unmatched {dialect} runtime helper end marker {id:?}"
                )));
            };
            if open != id {
                return Err(ecma_generation_error(format!(
                    "{dialect} runtime helper marker {open:?} closed by {id:?}"
                )));
            }
            if !emit(open, false, &mut source, &mut segments) {
                return Err(ecma_generation_error(format!(
                    "empty {dialect} runtime helper {id:?}"
                )));
            }
        } else {
            source.push_str(line);
        }
    }
    if let Some(open) = active {
        return Err(ecma_generation_error(format!(
            "unclosed {dialect} runtime helper marker {open:?}"
        )));
    }
    let common_id = format!("runtime.common.{common_index:03}");
    let _ = emit(common_id, true, &mut source, &mut segments);
    Ok(segments)
}

fn node_shims_file() -> LanguageSourceFile<EcmaImport> {
    let mut file = LanguageSourceFile::new("src/node-shims.d.ts", SourceFileRole::Source);
    file.set_body(EcmaCode::typescript(NODE_SHIMS).into_fragment(false));
    file
}

fn conformance_file(javascript: bool) -> LanguageSourceFile<EcmaImport> {
    let path = if javascript {
        "src/conformance.test.js"
    } else {
        "src/conformance.test.ts"
    };
    let mut file = LanguageSourceFile::new(path, SourceFileRole::Conformance);
    let mut body = EcmaCode::paired(CONFORMANCE_BODY, JAVASCRIPT_CONFORMANCE_BODY)
        .with_default(node_import_group(), "node:assert/strict", "assert")
        .with_default(node_import_group(), "node:test", "test");
    for name in [
        "checkedI32",
        "checkedI64",
        "listAppend",
        "scalarLength",
        "wrappingI32",
        "wrappingI64",
    ] {
        body = body.with_named("./runtime.js", name, false);
    }
    file.set_body(body.into_fragment(javascript));
    file
}

fn invalid_types_file() -> LanguageSourceFile<EcmaImport> {
    let mut file = LanguageSourceFile::new("tests/invalid-types.ts", SourceFileRole::NegativeTest);
    file.set_body(
        EcmaCode::typescript(INVALID_TYPES_BODY)
            .with_named("../src/runtime.js", "PolyOption", true)
            .into_fragment(false),
    );
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

    fn index_file(&self, javascript: bool) -> Result<LanguageSourceFile<EcmaImport>, BackendError> {
        let mut document = serde_json::to_value(self.program.document()).map_err(|error| {
            BackendError::Generation {
                message: format!("cannot serialize checked IR: {error}"),
            }
        })?;
        stringify_wide_numbers(&mut document);
        let document = serde_json::to_string(&document).expect("checked document serializes");
        let path = if javascript {
            "src/index.js"
        } else {
            "src/index.ts"
        };
        let mut file = LanguageSourceFile::new(path, SourceFileRole::Source);
        file.set_preamble(
            EcmaCode::paired(
                "// Generated by PolyRust from checked IR v0.",
                "// Generated by PolyRust from the TypeScript target fragments.",
            )
            .into_fragment(javascript),
        );
        let initializer = EcmaCode::paired(
            format!("const runtime = new Runtime({document});\nconst castResult = <T>(value: PolyResult<unknown>): PolyResult<T> => value as PolyResult<T>;\n\n"),
            format!("const runtime = new Runtime({document});\n\n"),
        )
        .with_named("./runtime.js", "Runtime", false)
        .with_named("./runtime.js", "PolyResult", true)
        .with_export_all("./runtime.js");
        let mut declarations: Vec<_> = self.program.module().declarations.iter().collect();
        declarations.sort_by_key(|declaration| declaration.header().node.id);
        let declarations = EcmaCode::sequence(
            declarations
                .into_iter()
                .map(|declaration| self.paired_declaration(declaration)),
        );
        let invocation = EcmaCode::paired(
            "export const __invokeTest = (index: number): Readonly<{ actual: PolyResult<unknown>; expected: unknown; expectsError: boolean }> => {\n  const test = TESTS[index];\n  if (test === undefined) return { actual: { ok: false, error: { code: \"invalid_test\", message: \"unknown test\" } }, expected: undefined, expectsError: true };\n  const invocation = test.invocation;\n  const arguments_ = invocation.data.arguments.map((value: unknown) => runtime.decode(value));\n  const actual = invocation.kind === \"function\" ? runtime.invoke(invocation.data.function, arguments_) : runtime.invokeMethod(invocation.data.implementation, invocation.data.method, runtime.decode(invocation.data.receiver), arguments_);\n  return { actual, expected: runtime.decode(test.expected.data), expectsError: test.expected.kind === \"error\" };\n};\n",
            "export const __invokeTest = (index) => {\n  const test = TESTS[index];\n  if (test === undefined) return { actual: { ok: false, error: { code: \"invalid_test\", message: \"unknown test\" } }, expected: undefined, expectsError: true };\n  const invocation = test.invocation;\n  const arguments_ = invocation.data.arguments.map((value) => runtime.decode(value));\n  const actual = invocation.kind === \"function\" ? runtime.invoke(invocation.data.function, arguments_) : runtime.invokeMethod(invocation.data.implementation, invocation.data.method, runtime.decode(invocation.data.receiver), arguments_);\n  return { actual, expected: runtime.decode(test.expected.data), expectsError: test.expected.kind === \"error\" };\n};\n",
        )
        .with_named("./runtime.js", "PolyResult", true);
        let tests: Vec<_> = self.program.module().declarations.iter().filter_map(|declaration| if let Declaration::Test(test) = declaration { Some(serde_json::json!({"invocation": test.invocation, "expected": test.expected})) } else { None }).collect();
        let mut tests = serde_json::to_value(tests).expect("tests serialize");
        stringify_wide_numbers(&mut tests);
        let tests = serde_json::to_string(&tests).expect("tests serialize");
        let test_data = EcmaCode::paired(
            format!("const TESTS: readonly any[] = {tests};\n"),
            format!("const TESTS = {tests};\n"),
        );
        file.set_body(
            EcmaCode::sequence([initializer, declarations, invocation, test_data])
                .into_fragment(javascript),
        );
        Ok(file)
    }

    fn paired_declaration(&self, declaration: &Declaration) -> EcmaCode {
        match declaration {
            Declaration::Alias(item) => {
                let mut target = self.paired_ty(&item.target);
                target.typescript = format!(
                    "{}type {} = {};\n\n",
                    export(item.header.visibility),
                    type_name(&item.header.name),
                    target.typescript
                );
                target
            }
            Declaration::Record(item) => {
                let implementations: Vec<_> = self
                    .program
                    .module()
                    .declarations
                    .iter()
                    .filter_map(|candidate| match candidate {
                        Declaration::Implementation(value)
                            if value.record == item.header.node.id =>
                        {
                            Some(value)
                        }
                        _ => None,
                    })
                    .collect();
                let contracts = implementations
                    .iter()
                    .map(|implementation| type_name(self.name(implementation.contract)))
                    .collect::<Vec<_>>()
                    .join(", ");
                let mut requirements = Vec::new();
                let mut typescript = format!(
                    "{}class {}{} {{\n  public readonly __polyDecl = {};\n",
                    export(item.header.visibility),
                    type_name(&item.header.name),
                    if contracts.is_empty() {
                        String::new()
                    } else {
                        format!(" implements {contracts}")
                    },
                    item.header.node.id.0
                );
                let mut javascript = format!(
                    "{}class {} {{\n  __polyDecl = {};\n",
                    export(item.header.visibility),
                    type_name(&item.header.name),
                    item.header.node.id.0
                );
                let mut constructor_types = Vec::new();
                for field in &item.fields {
                    let field_ty = self.paired_ty(&field.ty);
                    typescript.push_str(&format!(
                        "  public readonly {}: {};\n",
                        value_name(&field.header.name),
                        field_ty.typescript
                    ));
                    constructor_types.push(format!(
                        "{}: {}",
                        value_name(&field.header.name),
                        field_ty.typescript
                    ));
                    requirements.push(field_ty);
                }
                let constructor_names = item
                    .fields
                    .iter()
                    .map(|field| value_name(&field.header.name))
                    .collect::<Vec<_>>()
                    .join(", ");
                typescript.push_str(&format!(
                    "  public constructor({}) {{\n",
                    constructor_types.join(", ")
                ));
                javascript.push_str(&format!("  constructor({constructor_names}) {{\n"));
                for field in &item.fields {
                    let assignment = format!(
                        "    this.{} = {};\n",
                        value_name(&field.header.name),
                        value_name(&field.header.name)
                    );
                    typescript.push_str(&assignment);
                    javascript.push_str(&assignment);
                }
                typescript.push_str("    Object.freeze(this);\n  }\n");
                javascript.push_str("    Object.freeze(this);\n  }\n");
                let mut has_methods = false;
                for implementation in implementations {
                    for method in &implementation.methods {
                        has_methods = true;
                        let parameters = self.paired_parameters(&method.parameters);
                        let result = self.paired_ty(&method.return_type);
                        let arguments = method
                            .parameters
                            .iter()
                            .map(|parameter| value_name(&parameter.header.name))
                            .collect::<Vec<_>>()
                            .join(", ");
                        typescript.push_str(&format!(
                            "  public {}({}): PolyResult<{}> {{ return castResult(runtime.invokeMethod({}, {}, this, [{}])); }}\n",
                            value_name(&method.header.name),
                            parameters.typescript,
                            result.typescript,
                            implementation.header.node.id.0,
                            method.header.node.id.0,
                            arguments
                        ));
                        javascript.push_str(&format!(
                            "  {}({}) {{ return runtime.invokeMethod({}, {}, this, [{}]); }}\n",
                            value_name(&method.header.name),
                            parameters.javascript,
                            implementation.header.node.id.0,
                            method.header.node.id.0,
                            arguments
                        ));
                        requirements.push(parameters);
                        requirements.push(result);
                    }
                }
                typescript.push_str("}\n\n");
                javascript.push_str("}\n\n");
                let mut code = EcmaCode::sequence(requirements);
                code.typescript = typescript;
                code.javascript = javascript;
                if has_methods {
                    code = code.with_named("./runtime.js", "PolyResult", true);
                }
                code
            }
            Declaration::Enum(item) => {
                let mut requirements = Vec::new();
                let variants = item
                    .variants
                    .iter()
                    .map(|variant| {
                        let mut fields = vec![format!("readonly tag: {:?}", variant.header.name)];
                        for field in &variant.fields {
                            let field_ty = self.paired_ty(&field.ty);
                            fields.push(format!(
                                "readonly {}: {}",
                                value_name(&field.header.name),
                                field_ty.typescript
                            ));
                            requirements.push(field_ty);
                        }
                        format!("Readonly<{{ {} }}>", fields.join("; "))
                    })
                    .collect::<Vec<_>>()
                    .join(" | ");
                let mut code = EcmaCode::sequence(requirements);
                code.typescript = format!(
                    "{}type {} = {};\n\n",
                    export(item.header.visibility),
                    type_name(&item.header.name),
                    variants
                );
                code
            }
            Declaration::Contract(item) => {
                let mut requirements = Vec::new();
                let mut typescript = format!(
                    "{}interface {} {{\n",
                    export(item.header.visibility),
                    type_name(&item.header.name)
                );
                for method in &item.methods {
                    let parameters = self.paired_parameters(&method.parameters);
                    let result = self.paired_ty(&method.return_type);
                    typescript.push_str(&format!(
                        "  {}({}): PolyResult<{}>;\n",
                        value_name(&method.header.name),
                        parameters.typescript,
                        result.typescript
                    ));
                    requirements.push(parameters);
                    requirements.push(result);
                }
                typescript.push_str("}\n\n");
                let mut code = EcmaCode::sequence(requirements);
                code.typescript = typescript;
                code.javascript.clear();
                if !item.methods.is_empty() {
                    code = code.with_named("./runtime.js", "PolyResult", true);
                }
                code
            }
            Declaration::Constant(item) => {
                let mut ty = self.paired_ty(&item.ty);
                ty.typescript = format!(
                    "{}const {} = (): PolyResult<{}> => castResult(runtime.readConstant({}));\n\n",
                    export(item.header.visibility),
                    value_name(&item.header.name),
                    ty.typescript,
                    item.header.node.id.0
                );
                ty.javascript = format!(
                    "{}const {} = () => runtime.readConstant({});\n\n",
                    export(item.header.visibility),
                    value_name(&item.header.name),
                    item.header.node.id.0
                );
                ty.with_named("./runtime.js", "PolyResult", true)
            }
            Declaration::Function(item) => {
                let parameters = self.paired_parameters(&item.parameters);
                let result = self.paired_ty(&item.return_type);
                let arguments = item
                    .parameters
                    .iter()
                    .map(|parameter| value_name(&parameter.header.name))
                    .collect::<Vec<_>>()
                    .join(", ");
                let mut code = EcmaCode::sequence([parameters.clone(), result.clone()]);
                code.typescript = format!(
                    "{}const {} = ({}): PolyResult<{}> => castResult(runtime.invoke({}, [{}]));\n\n",
                    export(item.header.visibility),
                    value_name(&item.header.name),
                    parameters.typescript,
                    result.typescript,
                    item.header.node.id.0,
                    arguments
                );
                code.javascript = format!(
                    "{}const {} = ({}) => runtime.invoke({}, [{}]);\n\n",
                    export(item.header.visibility),
                    value_name(&item.header.name),
                    parameters.javascript,
                    item.header.node.id.0,
                    arguments
                );
                code.with_named("./runtime.js", "PolyResult", true)
            }
            Declaration::Implementation(_) | Declaration::Test(_) => EcmaCode::default(),
        }
    }

    fn paired_parameters(&self, parameters: &[portable_ir::v0::Parameter]) -> EcmaCode {
        EcmaCode::joined(
            parameters.iter().map(|parameter| {
                let mut ty = self.paired_ty(&parameter.ty);
                ty.typescript =
                    format!("{}: {}", value_name(&parameter.header.name), ty.typescript);
                ty.javascript = value_name(&parameter.header.name);
                ty
            }),
            ", ",
        )
    }

    fn paired_ty(&self, ty: &TypeRef) -> EcmaCode {
        match ty {
            TypeRef::Unit => EcmaCode::typescript("undefined"),
            TypeRef::Bool => EcmaCode::typescript("boolean"),
            TypeRef::I32 | TypeRef::F64 => EcmaCode::typescript("number"),
            TypeRef::I64 => EcmaCode::typescript("bigint"),
            TypeRef::Char | TypeRef::String => EcmaCode::typescript("string"),
            TypeRef::Bytes => EcmaCode::typescript("readonly number[]"),
            TypeRef::List(inner) => {
                let mut inner = self.paired_ty(inner);
                inner.typescript = format!("readonly {}[]", parenthesize(inner.typescript));
                inner
            }
            TypeRef::Option(inner) => {
                let mut inner = self.paired_ty(inner);
                inner.typescript = format!("PolyOption<{}>", inner.typescript);
                inner.with_named("./runtime.js", "PolyOption", true)
            }
            TypeRef::Result { ok, error } => {
                let ok = self.paired_ty(ok);
                let error = self.paired_ty(error);
                let mut result = EcmaCode::sequence([ok.clone(), error.clone()]);
                result.typescript =
                    format!("PolyValueResult<{}, {}>", ok.typescript, error.typescript);
                result.with_named("./runtime.js", "PolyValueResult", true)
            }
            TypeRef::Named(id) | TypeRef::Contract(id) => {
                EcmaCode::typescript(type_name(self.name(*id)))
            }
        }
    }

    fn name(&self, id: NodeId) -> &str {
        self.names.get(&id).map(String::as_str).unwrap_or("Unknown")
    }

    fn tests_file(&self, javascript: bool) -> LanguageSourceFile<EcmaImport> {
        let path = if javascript {
            "src/generated.test.js"
        } else {
            "src/generated.test.ts"
        };
        let mut file = LanguageSourceFile::new(path, SourceFileRole::Test);
        let tests = self
            .program
            .module()
            .declarations
            .iter()
            .filter_map(|declaration| {
                if let Declaration::Test(test) = declaration {
                    Some(test)
                } else {
                    None
                }
            })
            .enumerate()
            .map(|(index, test)| {
                EcmaCode::same(format!("test({:?}, () => {{ const result = __invokeTest({index}); assert.equal(result.actual.ok, !result.expectsError); if (result.actual.ok) assert.deepEqual(result.actual.value, result.expected); }});\n", test.header.name))
                    .with_default(node_import_group(), "node:assert/strict", "assert")
                    .with_default(node_import_group(), "node:test", "test")
                    .with_named("./index.js", "__invokeTest", false)
            })
            .collect::<Vec<_>>();
        if !tests.is_empty() {
            file.set_body(EcmaCode::sequence(tests).into_fragment(javascript));
        }
        file
    }
}

fn export(visibility: Visibility) -> &'static str {
    if visibility == Visibility::Public {
        "export "
    } else {
        ""
    }
}
fn type_name(name: &str) -> String {
    identifier(name)
}
fn value_name(name: &str) -> String {
    identifier(name)
}
fn parenthesize(ty: String) -> String {
    if ty.contains(" | ") {
        format!("({ty})")
    } else {
        ty
    }
}
fn identifier(name: &str) -> String {
    const KEYWORDS: &[&str] = &[
        "break",
        "case",
        "catch",
        "class",
        "const",
        "continue",
        "debugger",
        "default",
        "delete",
        "do",
        "else",
        "enum",
        "export",
        "extends",
        "false",
        "finally",
        "for",
        "function",
        "if",
        "import",
        "in",
        "instanceof",
        "new",
        "null",
        "return",
        "super",
        "switch",
        "this",
        "throw",
        "true",
        "try",
        "typeof",
        "var",
        "void",
        "while",
        "with",
        "yield",
        "interface",
        "implements",
        "package",
        "private",
        "protected",
        "public",
        "static",
        "await",
    ];
    if KEYWORDS.contains(&name) {
        format!("{name}_")
    } else {
        name.to_owned()
    }
}

fn stringify_wide_numbers(value: &mut serde_json::Value) {
    match value {
        serde_json::Value::Array(values) => {
            for value in values {
                stringify_wide_numbers(value);
            }
        }
        serde_json::Value::Object(object) => {
            if matches!(
                object.get("kind").and_then(serde_json::Value::as_str),
                Some("i64" | "f64")
            ) && let Some(data) = object.get_mut("data")
                && data.is_number()
            {
                *data = serde_json::Value::String(data.to_string());
            }
            for value in object.values_mut() {
                stringify_wide_numbers(value);
            }
        }
        _ => {}
    }
}

const PACKAGE_JSON: &str = "{\n  \"name\": \"generated-polyrust-package\",\n  \"version\": \"0.1.0\",\n  \"private\": true,\n  \"type\": \"module\",\n  \"scripts\": {\n    \"typecheck\": \"tsc --noEmit\",\n    \"test\": \"tsc && node --test dist/*.test.js\"\n  }\n}\n";
const JAVASCRIPT_PACKAGE_JSON: &str = "{\n  \"name\": \"generated-polyrust-javascript-package\",\n  \"version\": \"0.1.0\",\n  \"private\": true,\n  \"type\": \"module\",\n  \"scripts\": {\n    \"test\": \"node --test src/*.test.js\"\n  }\n}\n";
const TSCONFIG: &str = "{\n  \"compilerOptions\": {\n    \"target\": \"ES2024\",\n    \"module\": \"NodeNext\",\n    \"moduleResolution\": \"NodeNext\",\n    \"strict\": true,\n    \"noImplicitAny\": true,\n    \"noUncheckedIndexedAccess\": true,\n    \"exactOptionalPropertyTypes\": true,\n    \"rootDir\": \"src\",\n    \"outDir\": \"dist\",\n    \"declaration\": true,\n    \"skipLibCheck\": true\n  },\n  \"include\": [\"src/**/*.ts\"],\n  \"exclude\": [\"tests\"]\n}\n";
const CONFORMANCE_BODY: &str = "test(\"20 semantic boundary vectors\", () => {\n  const astral = scalarLength(\"😀\");\n  const original: readonly number[] = [1];\n  const appended = listAppend(original, 2);\n  const vectors: readonly boolean[] = [\n    checkedI32(0).ok, checkedI32(2147483647).ok, checkedI32(-2147483648).ok, !checkedI32(2147483648).ok, !checkedI32(-2147483649).ok,\n    checkedI64(0n).ok, checkedI64(9223372036854775807n).ok, checkedI64(-9223372036854775808n).ok, !checkedI64(9223372036854775808n).ok, !checkedI64(-9223372036854775809n).ok,\n    wrappingI32(2147483648) === -2147483648, wrappingI32(-2147483649) === 2147483647, wrappingI64(9223372036854775808n) === -9223372036854775808n, wrappingI64(-9223372036854775809n) === 9223372036854775807n,\n    scalarLength(\"a\").ok, astral.ok && astral.value === 1, !scalarLength(\"\\ud800\").ok, appended.length === 2, appended !== original, Object.is(-0, -0),\n  ];\n  assert.equal(vectors.length, 20); assert.ok(vectors.every(Boolean));\n});\n";
const JAVASCRIPT_CONFORMANCE_BODY: &str = "test(\"20 semantic boundary vectors\", () => {\n  const astral = scalarLength(\"😀\");\n  const original = [1];\n  const appended = listAppend(original, 2);\n  const vectors = [\n    checkedI32(0).ok, checkedI32(2147483647).ok, checkedI32(-2147483648).ok, !checkedI32(2147483648).ok, !checkedI32(-2147483649).ok,\n    checkedI64(0n).ok, checkedI64(9223372036854775807n).ok, checkedI64(-9223372036854775808n).ok, !checkedI64(9223372036854775808n).ok, !checkedI64(-9223372036854775809n).ok,\n    wrappingI32(2147483648) === -2147483648, wrappingI32(-2147483649) === 2147483647, wrappingI64(9223372036854775808n) === -9223372036854775808n, wrappingI64(-9223372036854775809n) === 9223372036854775807n,\n    scalarLength(\"a\").ok, astral.ok && astral.value === 1, !scalarLength(\"\\ud800\").ok, appended.length === 2, appended !== original, Object.is(-0, -0),\n  ];\n  assert.equal(vectors.length, 20); assert.ok(vectors.every(Boolean));\n});\n";
const INVALID_TYPES_BODY: &str = "// @ts-expect-error invalid option tag must be rejected\nconst invalid: PolyOption<number> = { tag: \"missing\" };\nvoid invalid;\n";
const NODE_SHIMS: &str = "declare module \"node:assert/strict\" { const assert: { equal(actual: unknown, expected: unknown): void; deepEqual(actual: unknown, expected: unknown): void; ok(value: unknown): void }; export default assert; }\ndeclare module \"node:test\" { const test: (name: string, body: () => void) => void; export default test; }\n";

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn keywords_are_escaped_and_i64_is_bigint() {
        assert_eq!(identifier("class"), "class_");
        assert_eq!(
            Generator::new(&fixture())
                .paired_ty(&TypeRef::I64)
                .typescript,
            "bigint"
        );
    }
    #[test]
    fn generated_manifest_is_deterministic_and_strict() {
        let checked = fixture();
        let first = TypeScriptBackend
            .generate(&checked, &BackendOptions::default())
            .unwrap();
        let second = TypeScriptBackend
            .generate(&checked, &BackendOptions::default())
            .unwrap();
        let third = TypeScriptBackend
            .generate(&checked, &BackendOptions::default())
            .unwrap();
        assert_eq!(first.canonical_json(), second.canonical_json());
        assert_eq!(second.canonical_json(), third.canonical_json());
        let index = first
            .files()
            .iter()
            .find(|file| file.path() == "src/index.ts")
            .unwrap();
        let portable_codegen::OutputContents::Text(index) = index.contents() else {
            panic!("text")
        };
        assert!(index.contains("bigint") || index.contains("call_render"));
        assert!(TSCONFIG.contains("\"strict\": true"));
    }
    #[test]
    fn javascript_manifest_is_standalone_and_deterministic() {
        let checked = fixture();
        let first = JavaScriptBackend
            .generate(&checked, &BackendOptions::default())
            .unwrap();
        let second = JavaScriptBackend
            .generate(&checked, &BackendOptions::default())
            .unwrap();
        let third = JavaScriptBackend
            .generate(&checked, &BackendOptions::default())
            .unwrap();
        assert_eq!(first.canonical_json(), second.canonical_json());
        assert_eq!(second.canonical_json(), third.canonical_json());
        assert!(
            first
                .files()
                .iter()
                .all(|file| !file.path().ends_with(".ts"))
        );
        assert!(
            first
                .files()
                .iter()
                .any(|file| file.path() == "src/runtime.js")
        );
    }
    #[test]
    fn ecmascript_imports_are_merged_and_omitted_per_file() {
        let typescript = TypeScriptBackend
            .generate(&fixture(), &BackendOptions::default())
            .unwrap();
        let index = generated_text(&typescript, "src/index.ts");
        assert_eq!(
            index
                .matches("import { Runtime, type PolyResult } from \"./runtime.js\";")
                .count(),
            1
        );
        let conformance = generated_text(&typescript, "src/conformance.test.ts");
        assert_eq!(conformance.matches("from \"./runtime.js\";").count(), 1);

        let empty_typescript = TypeScriptBackend
            .generate(&empty_fixture(), &BackendOptions::default())
            .unwrap();
        assert!(!generated_text(&empty_typescript, "src/generated.test.ts").contains("import "));
        let empty_javascript = JavaScriptBackend
            .generate(&empty_fixture(), &BackendOptions::default())
            .unwrap();
        assert!(!generated_text(&empty_javascript, "src/generated.test.js").contains("import "));
    }

    #[test]
    fn ecmascript_import_data_is_validated_and_renderer_owned() {
        assert!(EcmaImport::default("node:test", "test").is_ok());
        assert!(EcmaImport::named("./runtime.js", "PolyResult", true).is_ok());
        assert!(EcmaImport::export_all("../runtime.js").is_ok());
        for module in ["", "/rooted", "bad module", "./bad//path", "x/"] {
            assert!(EcmaImport::default(module, "valid").is_err(), "{module}");
        }
        for symbol in ["", "9value", "bad-name", "import { x }"] {
            assert!(
                EcmaImport::named("./runtime.js", symbol, false).is_err(),
                "{symbol}"
            );
        }
    }

    #[test]
    fn nested_type_fragments_own_and_erase_type_only_imports() {
        let checked = fixture();
        let generator = Generator::new(&checked);
        let nested = generator.paired_ty(&TypeRef::Result {
            ok: Box::new(TypeRef::Option(Box::new(TypeRef::I64))),
            error: Box::new(TypeRef::String),
        });
        assert_eq!(
            nested.typescript,
            "PolyValueResult<PolyOption<bigint>, string>"
        );
        let typescript = nested.clone().into_fragment(false);
        assert_eq!(typescript.imports().len(), 2);
        let javascript = nested.into_fragment(true);
        assert!(render_runtime(&javascript).trim().is_empty());
        assert!(javascript.imports().is_empty());
    }

    #[test]
    fn contract_declarations_are_fully_erased_from_javascript() {
        let checked = fixture();
        let generator = Generator::new(&checked);
        for declaration in &checked.module().declarations {
            if matches!(declaration, Declaration::Contract(_)) {
                let contract = generator.paired_declaration(declaration);
                assert!(contract.javascript.is_empty());
                assert!(!contract.typescript.is_empty());
            }
        }
        let javascript = JavaScriptBackend
            .generate(&checked, &BackendOptions::default())
            .unwrap();
        assert!(!generated_text(&javascript, "src/index.js").contains("userexport"));
    }

    #[test]
    fn runtime_helper_matrix_is_exact_and_paired() {
        let cases = [
            (
                "feature.string-replace-all",
                "replaceAllLiteral",
                "replaceBytesAll",
            ),
            (
                "feature.bytes-replace-all",
                "replaceBytesAll",
                "replaceManyLiteral",
            ),
            (
                "feature.string-replace-many",
                "replaceManyLiteral",
                "truncateUtf8Bytes",
            ),
            (
                "feature.string-truncate-utf8",
                "truncateUtf8Bytes",
                "trimStartScalars",
            ),
            (
                "feature.string-trim-start",
                "trimStartScalars",
                "trimEndScalars",
            ),
            ("feature.string-trim-end", "trimEndScalars", "listConcat"),
            ("feature.list-concat", "listConcat", "replaceAllLiteral"),
            (
                "feature.string-to-utf8",
                "new TextEncoder",
                "new TextDecoder",
            ),
            (
                "feature.string-from-utf8",
                "new TextDecoder",
                "new TextEncoder",
            ),
        ];
        for javascript in [false, true] {
            let (graph, common) = ecma_runtime_helper_graph(javascript).unwrap();
            let minimal = render_runtime(&graph.resolve(&common).unwrap());
            assert!(!minimal.contains("POLYRUST-"));
            for token in [
                "replaceAllLiteral",
                "replaceBytesAll",
                "replaceManyLiteral",
                "truncateUtf8Bytes",
                "trimStartScalars",
                "trimEndScalars",
                "listConcat",
                "new TextEncoder",
                "new TextDecoder",
            ] {
                assert!(!minimal.contains(token), "{token} in minimal runtime");
            }
            for (root, present, absent) in cases {
                let mut roots = common.clone();
                roots.push(root.to_owned());
                let runtime = render_runtime(&graph.resolve(&roots).unwrap());
                assert!(runtime.contains(present), "{root} lacks {present}");
                assert!(!runtime.contains(absent), "{root} includes {absent}");
                assert!(!runtime.contains("POLYRUST-"));
            }
        }
    }

    fn generated_text<'a>(manifest: &'a OutputManifest, path: &str) -> &'a str {
        match manifest.file(path).unwrap().contents() {
            portable_codegen::OutputContents::Text(text) => text,
            portable_codegen::OutputContents::Bytes(_) => panic!("ECMAScript source must be text"),
        }
    }

    fn render_runtime(fragment: &LanguageFragment<EcmaImport>) -> String {
        let mut file = LanguageSourceFile::new("runtime.test.js", SourceFileRole::Runtime);
        file.set_body(fragment.clone());
        let group = FileGroup::new(
            FileGroupId::parse("test").unwrap(),
            vec![LanguageFile::source(file)],
        )
        .unwrap();
        let package =
            LanguagePackage::new(vec![group], Vec::<DeclaredDependency>::new(), Vec::new())
                .unwrap();
        let manifest = portable_codegen::render_language_package(&package, &EcmaRenderer).unwrap();
        generated_text(&manifest, "runtime.test.js").to_owned()
    }

    fn fixture() -> CheckedProgram {
        let document = portable_ir::v0::from_json(include_bytes!(
            "../../build/testdata/registration.poly.json"
        ))
        .unwrap();
        portable_check::v0::check_program(document).unwrap()
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
}
