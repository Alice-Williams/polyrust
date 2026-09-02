//! Dependency-free Java 21 generation from checked portable IR v0.

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

const RUNTIME: &str = include_str!("Runtime.java");

pub struct JavaBackend;

impl Backend for JavaBackend {
    fn descriptor(&self) -> BackendDescriptor {
        BackendDescriptor {
            target: TargetId::parse("org.polyrust.java").expect("static target ID is valid"),
            display_name: "Java".to_owned(),
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

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
enum JavaImportKind {
    Type,
    StaticMember,
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
#[doc(hidden)]
pub struct JavaImport {
    kind: JavaImportKind,
    qualified_name: &'static str,
}

impl JavaImport {
    fn type_name(qualified_name: &'static str) -> Result<Self, String> {
        Self::parse(JavaImportKind::Type, qualified_name)
    }

    #[cfg(test)]
    fn static_member(qualified_name: &'static str) -> Result<Self, String> {
        Self::parse(JavaImportKind::StaticMember, qualified_name)
    }

    fn parse(kind: JavaImportKind, qualified_name: &'static str) -> Result<Self, String> {
        let parts = qualified_name.split('.').collect::<Vec<_>>();
        if parts.len() < 2 || parts.iter().any(|part| !valid_java_name_part(part)) {
            return Err(format!(
                "invalid Java qualified import name {qualified_name:?}"
            ));
        }
        Ok(Self {
            kind,
            qualified_name,
        })
    }
}

fn valid_java_name_part(part: &str) -> bool {
    let mut bytes = part.bytes();
    matches!(bytes.next(), Some(first) if first.is_ascii_alphabetic() || matches!(first, b'_' | b'$'))
        && bytes.all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'$'))
}

#[doc(hidden)]
pub struct JavaRenderer;

impl LanguageRenderer<JavaImport> for JavaRenderer {
    fn render_imports(&self, imports: &ImportSet<JavaImport>) -> Result<CodeDocument, String> {
        let lines = imports
            .groups()
            .flat_map(|(_, imports)| imports.iter())
            .map(|import| {
                format!(
                    "import {}{};",
                    if import.kind == JavaImportKind::StaticMember {
                        "static "
                    } else {
                        ""
                    },
                    import.qualified_name
                )
            })
            .collect::<Vec<_>>();
        Ok(CodeDocument::raw_text(RawText::new(lines.join("\n"))))
    }
}

impl LanguagePlugin for JavaBackend {
    type Import = JavaImport;
    type Renderer = JavaRenderer;

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
                    files: vec!["src/main/java/org/polyrust/generated/Runtime.java".into()],
                }),
                CapabilitySupport::Native | CapabilitySupport::Unsupported { .. } => None,
            })
            .collect();
        LanguagePackage::new(
            vec![
                FileGroup::new(
                    java_group("documentation")?,
                    vec![LanguageFile::text(
                        "README.md",
                        TextFileRole::Documentation,
                        README,
                    )],
                )
                .map_err(java_generation_error)?,
                FileGroup::new(
                    java_group("runtime")?,
                    vec![LanguageFile::source(java_runtime_file(program)?)],
                )
                .map_err(java_generation_error)?,
                FileGroup::new(
                    java_group("source")?,
                    vec![LanguageFile::source(generator.module_file()?)],
                )
                .map_err(java_generation_error)?,
                FileGroup::new(
                    java_group("tests")?,
                    vec![
                        LanguageFile::source(generator.tests_file()),
                        LanguageFile::source(java_conformance_file()),
                    ],
                )
                .map_err(java_generation_error)?,
                FileGroup::new(
                    java_group("negative-tests")?,
                    vec![LanguageFile::source(java_invalid_types_file())],
                )
                .map_err(java_generation_error)?,
            ],
            Vec::<DeclaredDependency>::new(),
            helpers,
        )
        .map_err(java_generation_error)
    }

    fn renderer(&self) -> Self::Renderer {
        JavaRenderer
    }
}

fn java_generation_error(error: impl std::fmt::Display) -> BackendError {
    BackendError::Generation {
        message: error.to_string(),
    }
}

fn java_group(name: &str) -> Result<FileGroupId, BackendError> {
    FileGroupId::parse(name).map_err(java_generation_error)
}

fn java_import_group() -> ImportGroup {
    ImportGroup::new(10, "java-standard-library").expect("static import group is valid")
}

fn java_preamble(comment: Option<&str>) -> CodeDocument {
    let prefix = comment.map_or_else(String::new, |comment| format!("{comment}\n"));
    CodeDocument::raw_text(RawText::new(format!(
        "{prefix}package org.polyrust.generated;"
    )))
}

fn require_java(unit: &mut LanguageFragment<JavaImport>, import: &'static str) {
    unit.require_import(
        java_import_group(),
        JavaImport::type_name(import).expect("static Java import is valid"),
    );
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct JavaCode {
    text: String,
    imports: BTreeSet<JavaImport>,
}

impl JavaCode {
    fn text(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            imports: BTreeSet::new(),
        }
    }

    fn with_import(mut self, import: &'static str) -> Self {
        self.imports.insert(
            JavaImport::type_name(import).expect("static Java import requirement is valid"),
        );
        self
    }

    fn sequence(parts: impl IntoIterator<Item = Self>) -> Self {
        let mut output = Self::text("");
        for part in parts {
            output.text.push_str(&part.text);
            output.imports.extend(part.imports);
        }
        output
    }

    fn joined(separator: &'static str, parts: impl IntoIterator<Item = Self>) -> Self {
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

    fn into_fragment(self) -> LanguageFragment<JavaImport> {
        let mut fragment = LanguageFragment::new(CodeDocument::raw_text(RawText::new(self.text)));
        for import in self.imports {
            fragment.require_import(java_import_group(), import);
        }
        fragment
    }
}

fn java_runtime_file(
    program: &CheckedProgram,
) -> Result<LanguageSourceFile<JavaImport>, BackendError> {
    let (graph, mut roots) = java_runtime_helper_graph()?;
    if program.capabilities().program().iter().any(|capability| {
        matches!(
            capability,
            Capability::CheckedIntegerArithmetic | Capability::WrappingIntegerArithmetic
        )
    }) {
        roots.push("feature.numeric".to_owned());
    }
    if portable_ir::v0::module_uses_intrinsic(program.module(), |operation| {
        matches!(
            operation,
            Intrinsic::StringToUtf8 | Intrinsic::StringFromUtf8Checked
        )
    }) {
        roots.push("feature.utf8".to_owned());
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

    let mut file = LanguageSourceFile::new(
        "src/main/java/org/polyrust/generated/Runtime.java",
        SourceFileRole::Runtime,
    );
    file.set_preamble(LanguageFragment::new(java_preamble(Some(
        "// Generated Java 21 runtime assembled from checked-program helper roots.",
    ))));
    file.set_body(graph.resolve(&roots).map_err(java_generation_error)?);
    Ok(file)
}

fn java_runtime_helper_graph() -> Result<(RuntimeHelperGraph<JavaImport>, Vec<String>), BackendError>
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
                helpers: &mut Vec<RuntimeHelper<JavaImport>>| {
        if source.trim().is_empty() {
            source.clear();
            return false;
        }
        let fragment = java_runtime_fragment(&id, std::mem::take(source));
        helpers.push(RuntimeHelper::new(id, *order, fragment));
        *order = order.checked_add(1).expect("runtime helper order fits u16");
        true
    };

    for line in RUNTIME.split_inclusive('\n') {
        let trimmed = line.trim_end_matches(['\r', '\n']);
        if let Some(id) = trimmed.strip_prefix(BEGIN) {
            if active.is_some() {
                return Err(java_generation_error(format!(
                    "nested Java runtime helper marker {id:?}"
                )));
            }
            let common_id = format!("runtime.common.{common_index:03}");
            if emit(common_id.clone(), &mut source, &mut order, &mut helpers) {
                common_roots.push(common_id);
                common_index += 1;
            }
            active = Some(id.to_owned());
        } else if let Some(id) = trimmed.strip_prefix(END) {
            let Some(open) = active.take() else {
                return Err(java_generation_error(format!(
                    "unmatched Java runtime helper end marker {id:?}"
                )));
            };
            if open != id {
                return Err(java_generation_error(format!(
                    "Java runtime helper marker {open:?} closed by {id:?}"
                )));
            }
            if !emit(open, &mut source, &mut order, &mut helpers) {
                return Err(java_generation_error(format!(
                    "empty Java runtime helper {id:?}"
                )));
            }
        } else {
            source.push_str(line);
        }
    }
    if let Some(open) = active {
        return Err(java_generation_error(format!(
            "unclosed Java runtime helper marker {open:?}"
        )));
    }
    let common_id = format!("runtime.common.{common_index:03}");
    if emit(common_id.clone(), &mut source, &mut order, &mut helpers) {
        common_roots.push(common_id);
    }

    let numeric = LanguageFragment::new(CodeDocument::empty())
        .with_helper_root("numeric-cases-primary")
        .with_helper_root("numeric-case-narrow")
        .with_helper_root("numeric-private-methods")
        .with_helper_root("numeric-static-methods");
    helpers.push(RuntimeHelper::new("feature.numeric", u16::MAX - 1, numeric));

    let utf8 = LanguageFragment::new(CodeDocument::empty())
        .with_helper_root("utf8-cases")
        .with_helper_root("utf8-method");
    helpers.push(RuntimeHelper::new("feature.utf8", u16::MAX, utf8));
    helpers.push(RuntimeHelper::new(
        "feature.string-utf16-length",
        u16::MAX - 2,
        LanguageFragment::new(CodeDocument::empty()).with_helper_root("string-utf16-length-case"),
    ));
    helpers.push(RuntimeHelper::new(
        "feature.list-index-of",
        u16::MAX - 3,
        LanguageFragment::new(CodeDocument::empty()).with_helper_root("list-index-of-case"),
    ));

    let graph = RuntimeHelperGraph::new(helpers).map_err(java_generation_error)?;
    Ok((graph, common_roots))
}

fn java_runtime_fragment(id: &str, source: String) -> LanguageFragment<JavaImport> {
    let mut fragment = LanguageFragment::new(CodeDocument::raw_text(RawText::new(source)));
    let imports: &[&'static str] = match id {
        "runtime.common.000" => &[
            "java.util.ArrayList",
            "java.util.LinkedHashMap",
            "java.util.List",
            "java.util.Map",
            "java.util.Objects",
        ],
        "runtime.common.001" => &["java.util.List"],
        "runtime.common.002" => &[
            "java.util.ArrayList",
            "java.util.LinkedHashMap",
            "java.util.List",
            "java.util.Map",
        ],
        "runtime.common.003" => &[
            "java.util.ArrayList",
            "java.util.List",
            "java.util.Map",
            "java.util.Objects",
        ],
        "runtime.common.004" => &[
            "java.util.ArrayList",
            "java.util.LinkedHashMap",
            "java.util.List",
            "java.util.Map",
        ],
        "numeric-cases-primary"
        | "numeric-case-narrow"
        | "numeric-private-methods"
        | "numeric-static-methods" => &["java.math.BigInteger"],
        "utf8-cases" => &[
            "java.nio.charset.StandardCharsets",
            "java.util.ArrayList",
            "java.util.List",
        ],
        "utf8-method" => &[
            "java.nio.ByteBuffer",
            "java.nio.charset.CharacterCodingException",
            "java.nio.charset.CodingErrorAction",
            "java.nio.charset.StandardCharsets",
            "java.util.List",
        ],
        _ => &[],
    };
    for import in imports {
        require_java(&mut fragment, import);
    }
    fragment
}

fn java_conformance_file() -> LanguageSourceFile<JavaImport> {
    let mut file = LanguageSourceFile::new(
        "src/test/java/org/polyrust/generated/ConformanceTest.java",
        SourceFileRole::Conformance,
    );
    file.set_preamble(LanguageFragment::new(java_preamble(None)));
    let mut body = LanguageFragment::new(CodeDocument::raw_text(RawText::new(CONFORMANCE_BODY)));
    require_java(&mut body, "java.util.List");
    file.set_body(body);
    file
}

fn java_invalid_types_file() -> LanguageSourceFile<JavaImport> {
    let mut file =
        LanguageSourceFile::new("negative/InvalidTypes.java", SourceFileRole::NegativeTest);
    file.set_preamble(LanguageFragment::new(java_preamble(None)));
    file.set_body(LanguageFragment::new(CodeDocument::raw_text(RawText::new(
        INVALID_TYPES_BODY,
    ))));
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

    fn module_file(&self) -> Result<LanguageSourceFile<JavaImport>, BackendError> {
        let mut document = serde_json::to_value(self.program.document()).map_err(|error| {
            BackendError::Generation {
                message: format!("cannot serialize checked IR: {error}"),
            }
        })?;
        stringify_wide_numbers(&mut document);
        let document = serde_json::to_string(&document).expect("checked document serializes");
        let document_literal = java_string_expression(&document);
        let tests: Vec<_> = self
            .program
            .module()
            .declarations
            .iter()
            .filter_map(|declaration| {
                if let Declaration::Test(test) = declaration {
                    Some(serde_json::json!({
                        "invocation": test.invocation,
                        "expected": test.expected,
                    }))
                } else {
                    None
                }
            })
            .collect();
        let mut tests = serde_json::to_value(tests).expect("tests serialize");
        stringify_wide_numbers(&mut tests);
        let tests_literal =
            java_string_expression(&serde_json::to_string(&tests).expect("tests serialize"));
        let mut file = LanguageSourceFile::new(
            "src/main/java/org/polyrust/generated/Generated.java",
            SourceFileRole::Source,
        );
        file.set_preamble(LanguageFragment::new(java_preamble(Some(
            "// Generated by PolyRust from checked IR v0.",
        ))));
        let mut opening = LanguageFragment::new(CodeDocument::raw_text(RawText::new(format!(
            "public final class Generated {{\n\
             \x20 private static final Runtime RUNTIME = new Runtime({document_literal});\n\
             \x20 private static final List<Object> TESTS = Runtime.jsonArray({tests_literal});\n\
             \x20 private Generated() {{}}\n\n"
        ))));
        require_java(&mut opening, "java.util.List");
        let mut fragments = vec![opening];
        let mut declarations: Vec<_> = self.program.module().declarations.iter().collect();
        declarations.sort_by_key(|declaration| declaration.header().node.id);
        for declaration in declarations {
            fragments.push(self.declaration(declaration));
        }
        fragments.push(LanguageFragment::new(CodeDocument::raw_text(RawText::new(
            "  static Runtime.TestOutcome invokeTest(int index) {\n\
             \x20   return RUNTIME.invokeTest(TESTS, index);\n\
             \x20 }\n\
             }\n",
        ))));
        file.set_body(LanguageFragment::sequence(fragments));
        Ok(file)
    }

    fn declaration(&self, declaration: &Declaration) -> LanguageFragment<JavaImport> {
        let code = match declaration {
            Declaration::Alias(_) | Declaration::Implementation(_) | Declaration::Test(_) => {
                JavaCode::text("")
            }
            Declaration::Contract(item) => {
                let mut parts = vec![JavaCode::text(format!(
                    "  {}interface {} {{\n",
                    visibility(item.header.visibility),
                    type_name(&item.header.name)
                ))];
                for method in &item.methods {
                    parts.push(JavaCode::sequence([
                        JavaCode::text("    Runtime.PolyResult<"),
                        self.ty(&method.return_type),
                        JavaCode::text(format!("> {}(", value_name(&method.header.name))),
                        self.parameters(&method.parameters),
                        JavaCode::text(");\n"),
                    ]));
                }
                parts.push(JavaCode::text("  }\n\n"));
                JavaCode::sequence(parts)
            }
            Declaration::Record(item) => {
                let implementations = self.implementations(item.header.node.id);
                let contracts = implementations
                    .iter()
                    .map(|implementation| type_name(self.name(implementation.contract)))
                    .collect::<Vec<_>>();
                let mut implemented = vec!["Runtime.PolyRecord".to_owned()];
                implemented.extend(contracts);
                let fields = JavaCode::joined(
                    ", ",
                    item.fields.iter().map(|field| {
                        JavaCode::sequence([
                            self.ty(&field.ty),
                            JavaCode::text(format!(" {}", value_name(&field.header.name))),
                        ])
                    }),
                );
                let mut parts = vec![
                    JavaCode::sequence([
                        JavaCode::text(format!(
                            "  {}record {}(",
                            visibility(item.header.visibility),
                            type_name(&item.header.name)
                        )),
                        fields,
                        JavaCode::text(format!(") implements {} {{\n", implemented.join(", "))),
                    ]),
                    JavaCode::text("    @Override public Map<String, Object> polyValue() {\n"),
                    JavaCode::text("      Map<String, Object> value = new LinkedHashMap<>();\n"),
                    JavaCode::text(format!(
                        "      value.put(\"__polyDecl\", {}L);\n",
                        item.header.node.id.0
                    )),
                ];
                for field in &item.fields {
                    parts.push(JavaCode::text(format!(
                        "      value.put({}, {});\n",
                        java_string(&field.header.name),
                        value_name(&field.header.name)
                    )));
                }
                parts.push(JavaCode::text(
                    "      return Collections.unmodifiableMap(value);\n    }\n",
                ));
                for implementation in implementations {
                    for method in &implementation.methods {
                        parts.push(
                            JavaCode::sequence([
                                JavaCode::text("    @Override public Runtime.PolyResult<"),
                                self.ty(&method.return_type),
                                JavaCode::text(format!(
                                    "> {}(",
                                    value_name(&method.header.name)
                                )),
                                self.parameters(&method.parameters),
                                JavaCode::text(format!(
                                    ") {{\n      return Runtime.cast(RUNTIME.invokeMethod({}L, {}L, this, List.of({})));\n    }}\n",
                                    implementation.header.node.id.0,
                                    method.header.node.id.0,
                                    self.arguments(&method.parameters)
                                )),
                            ])
                            .with_import("java.util.List"),
                        );
                    }
                }
                parts.push(JavaCode::text("  }\n\n"));
                JavaCode::sequence(parts)
                    .with_import("java.util.Collections")
                    .with_import("java.util.LinkedHashMap")
                    .with_import("java.util.Map")
            }
            Declaration::Enum(item) => {
                let name = type_name(&item.header.name);
                let mut parts = vec![JavaCode::text(format!(
                    "  {}sealed interface {name} extends Runtime.PolyRecord permits {} {{}}\n",
                    visibility(item.header.visibility),
                    item.variants
                        .iter()
                        .map(|variant| format!("{name}{}", type_name(&variant.header.name)))
                        .collect::<Vec<_>>()
                        .join(", ")
                ))];
                for variant in &item.variants {
                    let fields = JavaCode::joined(
                        ", ",
                        variant.fields.iter().map(|field| {
                            JavaCode::sequence([
                                self.ty(&field.ty),
                                JavaCode::text(format!(" {}", value_name(&field.header.name))),
                            ])
                        }),
                    );
                    parts.extend([
                        JavaCode::sequence([
                            JavaCode::text(format!(
                                "  {}record {name}{}(",
                                visibility(item.header.visibility),
                                type_name(&variant.header.name)
                            )),
                            fields,
                            JavaCode::text(format!(") implements {name} {{\n")),
                        ]),
                        JavaCode::text("    @Override public Map<String, Object> polyValue() {\n"),
                        JavaCode::text(
                            "      Map<String, Object> value = new LinkedHashMap<>();\n",
                        ),
                        JavaCode::text(format!(
                            "      value.put(\"__polyDecl\", {}L);\n\
                         \x20     value.put(\"tag\", {});\n",
                            item.header.node.id.0,
                            java_string(&variant.header.name)
                        )),
                    ]);
                    for field in &variant.fields {
                        parts.push(JavaCode::text(format!(
                            "      value.put({}, {});\n",
                            java_string(&field.header.name),
                            value_name(&field.header.name)
                        )));
                    }
                    parts.push(JavaCode::text(
                        "      return Collections.unmodifiableMap(value);\n    }\n  }\n",
                    ));
                }
                parts.push(JavaCode::text("\n"));
                JavaCode::sequence(parts)
                    .with_import("java.util.Collections")
                    .with_import("java.util.LinkedHashMap")
                    .with_import("java.util.Map")
            }
            Declaration::Constant(item) => JavaCode::sequence([
                JavaCode::text(format!(
                    "  {}static Runtime.PolyResult<",
                    visibility(item.header.visibility)
                )),
                self.ty(&item.ty),
                JavaCode::text(format!(
                    "> {}() {{\n    return Runtime.cast(RUNTIME.readConstant({}L));\n  }}\n\n",
                    value_name(&item.header.name),
                    item.header.node.id.0
                )),
            ]),
            Declaration::Function(item) => JavaCode::sequence([
                JavaCode::text(format!(
                    "  {}static Runtime.PolyResult<",
                    visibility(item.header.visibility)
                )),
                self.ty(&item.return_type),
                JavaCode::text(format!("> {}(", value_name(&item.header.name))),
                self.parameters(&item.parameters),
                JavaCode::text(format!(
                    ") {{\n    return Runtime.cast(RUNTIME.invoke({}L, List.of({})));\n  }}\n\n",
                    item.header.node.id.0,
                    self.arguments(&item.parameters)
                )),
            ])
            .with_import("java.util.List"),
        };
        code.into_fragment()
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

    fn parameters(&self, parameters: &[portable_ir::v0::Parameter]) -> JavaCode {
        JavaCode::joined(
            ", ",
            parameters.iter().map(|parameter| {
                JavaCode::sequence([
                    self.ty(&parameter.ty),
                    JavaCode::text(format!(" {}", value_name(&parameter.header.name))),
                ])
            }),
        )
    }

    fn arguments(&self, parameters: &[portable_ir::v0::Parameter]) -> String {
        parameters
            .iter()
            .map(|parameter| value_name(&parameter.header.name))
            .collect::<Vec<_>>()
            .join(", ")
    }

    fn ty(&self, ty: &TypeRef) -> JavaCode {
        match ty {
            TypeRef::Unit => JavaCode::text("Void"),
            TypeRef::Bool => JavaCode::text("Boolean"),
            TypeRef::I32 => JavaCode::text("Integer"),
            TypeRef::I64 => JavaCode::text("Long"),
            TypeRef::F64 => JavaCode::text("Double"),
            TypeRef::Char | TypeRef::String => JavaCode::text("String"),
            TypeRef::Bytes => JavaCode::text("List<Integer>").with_import("java.util.List"),
            TypeRef::List(inner) => {
                JavaCode::sequence([JavaCode::text("List<"), self.ty(inner), JavaCode::text(">")])
                    .with_import("java.util.List")
            }
            TypeRef::Option(inner) => JavaCode::sequence([
                JavaCode::text("Runtime.PolyOption<"),
                self.ty(inner),
                JavaCode::text(">"),
            ]),
            TypeRef::Result { ok, error } => JavaCode::sequence([
                JavaCode::text("Runtime.PolyValueResult<"),
                self.ty(ok),
                JavaCode::text(", "),
                self.ty(error),
                JavaCode::text(">"),
            ]),
            TypeRef::Named(id) => self.named_ty(*id),
            TypeRef::Contract(id) => JavaCode::text(type_name(self.name(*id))),
        }
    }

    fn named_ty(&self, id: NodeId) -> JavaCode {
        self.program
            .module()
            .declarations
            .iter()
            .find_map(|declaration| match declaration {
                Declaration::Alias(item) if item.header.node.id == id => {
                    Some(self.ty(&item.target))
                }
                declaration if declaration.header().node.id == id => {
                    Some(JavaCode::text(type_name(&declaration.header().name)))
                }
                _ => None,
            })
            .unwrap_or_else(|| JavaCode::text("Object"))
    }

    fn name(&self, id: NodeId) -> &str {
        self.names.get(&id).map(String::as_str).unwrap_or("Unknown")
    }

    fn tests_file(&self) -> LanguageSourceFile<JavaImport> {
        let count = self
            .program
            .module()
            .declarations
            .iter()
            .filter(|declaration| matches!(declaration, Declaration::Test(_)))
            .count();
        let mut file = LanguageSourceFile::new(
            "src/test/java/org/polyrust/generated/GeneratedTest.java",
            SourceFileRole::Test,
        );
        file.set_preamble(LanguageFragment::new(java_preamble(Some(
            "// Generated by PolyRust from checked IR v0.",
        ))));
        file.set_body(LanguageFragment::new(CodeDocument::raw_text(RawText::new(format!(
            "public final class GeneratedTest {{\n\
             \x20 private GeneratedTest() {{}}\n\
             \x20 public static void main(String[] arguments) {{\n\
             \x20   for (int index = 0; index < {count}; index++) {{\n\
             \x20     Runtime.TestOutcome outcome = Generated.invokeTest(index);\n\
             \x20     if (outcome.actual().ok() == outcome.expectsError()) {{\n\
             \x20       throw new AssertionError(\"portable test \" + index + \" success mismatch\");\n\
             \x20     }}\n\
             \x20     if (outcome.actual().ok() && !Runtime.deepEqual(outcome.actual().value(), outcome.expected())) {{\n\
             \x20       throw new AssertionError(\"portable test \" + index + \" value mismatch\");\n\
             \x20     }}\n\
             \x20   }}\n\
             \x20 }}\n\
             }}\n"
        )))));
        file
    }
}

fn visibility(visibility: Visibility) -> &'static str {
    if visibility == Visibility::Public {
        "public "
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

fn identifier(name: &str) -> String {
    const KEYWORDS: &[&str] = &[
        "abstract",
        "assert",
        "boolean",
        "break",
        "byte",
        "case",
        "catch",
        "char",
        "class",
        "const",
        "continue",
        "default",
        "do",
        "double",
        "else",
        "enum",
        "extends",
        "final",
        "finally",
        "float",
        "for",
        "goto",
        "if",
        "implements",
        "import",
        "instanceof",
        "int",
        "interface",
        "long",
        "native",
        "new",
        "package",
        "private",
        "protected",
        "public",
        "return",
        "short",
        "static",
        "strictfp",
        "super",
        "switch",
        "synchronized",
        "this",
        "throw",
        "throws",
        "transient",
        "try",
        "void",
        "volatile",
        "while",
        "true",
        "false",
        "null",
        "record",
        "sealed",
        "permits",
        "non-sealed",
        "var",
        "yield",
    ];
    if KEYWORDS.contains(&name) {
        format!("{name}_")
    } else {
        name.to_owned()
    }
}

fn java_string(value: &str) -> String {
    let json = serde_json::to_string(value).expect("string serializes");
    json.replace('\u{2028}', "\\u2028")
        .replace('\u{2029}', "\\u2029")
}

fn java_string_expression(value: &str) -> String {
    const MAX_CHUNK_BYTES: usize = 8 * 1024;
    if value.len() <= MAX_CHUNK_BYTES {
        return java_string(value);
    }
    let mut chunks = Vec::new();
    let mut start = 0;
    while start < value.len() {
        let mut end = (start + MAX_CHUNK_BYTES).min(value.len());
        while !value.is_char_boundary(end) {
            end -= 1;
        }
        chunks.push(java_string(&value[start..end]));
        start = end;
    }
    format!("String.join(\"\", {})", chunks.join(", "))
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

const README: &str = "# Generated PolyRust Java package\n\nCompile with Java 21 or newer. The package has no third-party runtime dependencies.\n";
const INVALID_TYPES_BODY: &str = "final class InvalidTypes {\n  // Must fail: PolyOption tags are represented by a closed Java type, not strings.\n  Runtime.PolyOption<Integer> invalid = \"missing\";\n}\n";
const CONFORMANCE_BODY: &str = r#"public final class ConformanceTest {
  private ConformanceTest() {}
  public static void main(String[] arguments) {
    Runtime.PolyResult<Integer> astral = Runtime.scalarLength("😀");
    List<Integer> original = List.of(1);
    List<Integer> appended = Runtime.listAppend(original, 2);
    boolean[] vectors = {
      Runtime.scalarLength("a").ok(),
      astral.ok() && astral.value() == 1,
      !Runtime.scalarLength("\ud800").ok(),
      appended.size() == 2,
      appended != original,
      original.size() == 1,
      Double.doubleToRawLongBits(-0.0d) == Long.MIN_VALUE,
    };
    if (vectors.length != 7) throw new AssertionError("vector count");
    for (boolean vector : vectors) if (!vector) throw new AssertionError("conformance vector");
  }
}
"#;

#[cfg(test)]
mod tests {
    use super::*;
    use portable_ir::v0::{
        Block, DeclarationHeader, Document as IrDocument, Expression, FunctionDeclaration, Module,
        NodeMeta, SourceRef, Value,
    };

    #[test]
    fn keywords_and_types_are_safe() {
        assert_eq!(identifier("class"), "class_");
        assert_eq!(Generator::new(&fixture()).ty(&TypeRef::I64).text, "Long");
        assert!(JavaImport::type_name("java.util.List").is_ok());
        assert!(JavaImport::static_member("java.util.Objects.requireNonNull").is_ok());
        for invalid in [
            "List",
            "java.util.*",
            "import java.util.List;",
            "java..List",
        ] {
            assert!(
                JavaImport::type_name(invalid).is_err(),
                "accepted {invalid:?}"
            );
        }
    }

    #[test]
    fn generated_manifest_is_deterministic_and_dependency_free() {
        let checked = fixture();
        let first = JavaBackend
            .generate(&checked, &BackendOptions::default())
            .unwrap();
        let second = JavaBackend
            .generate(&checked, &BackendOptions::default())
            .unwrap();
        assert_eq!(first.canonical_json(), second.canonical_json());
        assert!(first.dependencies().is_empty());
        assert!(
            first
                .files()
                .iter()
                .any(|file| file.path().ends_with("GeneratedTest.java"))
        );
    }

    #[test]
    fn large_embedded_documents_are_split_below_java_constant_limits() {
        let expression = java_string_expression(&"x".repeat(100_000));
        assert!(expression.starts_with("String.join(\"\", \""));
        assert!(expression.ends_with("\")"));
        assert!(expression.matches("\", \"").count() >= 12);
        assert!(!expression.contains(&"x".repeat(65_536)));
    }

    #[test]
    fn java_imports_follow_file_and_declaration_requirements() {
        let rich = JavaBackend
            .generate(&fixture(), &BackendOptions::default())
            .unwrap();
        let generated =
            generated_text(&rich, "src/main/java/org/polyrust/generated/Generated.java");
        assert_eq!(generated.matches("import java.util.List;").count(), 1);
        assert_eq!(generated.matches("import java.util.Map;").count(), 1);
        let runtime = generated_text(&rich, "src/main/java/org/polyrust/generated/Runtime.java");
        assert!(!runtime.contains("import java.math.BigInteger;"));
        assert!(!runtime.contains("import java.nio.ByteBuffer;"));
        assert!(!runtime.contains("POLYRUST-BEGIN"));
        assert!(!runtime.contains("POLYRUST-END"));
        assert!(!generated_text(&rich, "negative/InvalidTypes.java").contains("import "));

        let empty = JavaBackend
            .generate(&empty_fixture(), &BackendOptions::default())
            .unwrap();
        let empty_generated = generated_text(
            &empty,
            "src/main/java/org/polyrust/generated/Generated.java",
        );
        assert!(empty_generated.contains("import java.util.List;"));
        assert!(!empty_generated.contains("import java.util.Map;"));
        assert!(!empty_generated.contains("import java.util.LinkedHashMap;"));
        assert!(!empty_generated.contains("import java.util.Collections;"));
        let empty_runtime =
            generated_text(&empty, "src/main/java/org/polyrust/generated/Runtime.java");
        assert!(!empty_runtime.contains("import java.math.BigInteger;"));
        assert!(!empty_runtime.contains("import java.nio.ByteBuffer;"));
        assert!(!empty_runtime.contains("checkedInteger("));
        assert!(!empty_runtime.contains("stringFromUtf8("));
    }

    #[test]
    fn runtime_helper_roots_have_exact_transitive_import_closures() {
        let (graph, common) = java_runtime_helper_graph().unwrap();

        let base = graph.resolve(&common).unwrap();
        let base_imports = fragment_imports(&base);
        for required in [
            "java.util.ArrayList",
            "java.util.LinkedHashMap",
            "java.util.List",
            "java.util.Map",
            "java.util.Objects",
        ] {
            assert!(base_imports.contains(&required), "missing {required}");
        }
        for optional in [
            "java.math.BigInteger",
            "java.nio.ByteBuffer",
            "java.nio.charset.CharacterCodingException",
            "java.nio.charset.CodingErrorAction",
            "java.nio.charset.StandardCharsets",
        ] {
            assert!(!base_imports.contains(&optional), "unexpected {optional}");
        }

        let mut numeric_roots = common.clone();
        numeric_roots.push("feature.numeric".to_owned());
        let numeric_imports = fragment_imports(&graph.resolve(&numeric_roots).unwrap());
        assert!(numeric_imports.contains(&"java.math.BigInteger"));
        assert!(!numeric_imports.contains(&"java.nio.ByteBuffer"));

        let mut utf8_roots = common;
        utf8_roots.push("feature.utf8".to_owned());
        let utf8_imports = fragment_imports(&graph.resolve(&utf8_roots).unwrap());
        for required in [
            "java.nio.ByteBuffer",
            "java.nio.charset.CharacterCodingException",
            "java.nio.charset.CodingErrorAction",
            "java.nio.charset.StandardCharsets",
        ] {
            assert!(utf8_imports.contains(&required), "missing {required}");
        }
        assert!(!utf8_imports.contains(&"java.math.BigInteger"));
    }

    #[test]
    fn checked_program_features_select_only_their_java_runtime_closure() {
        let numeric = JavaBackend
            .generate(
                &intrinsic_fixture(Intrinsic::IntAddChecked),
                &BackendOptions::default(),
            )
            .unwrap();
        let numeric_runtime = generated_text(
            &numeric,
            "src/main/java/org/polyrust/generated/Runtime.java",
        );
        assert!(numeric_runtime.contains("import java.math.BigInteger;"));
        assert!(numeric_runtime.contains("checkedInteger("));
        assert!(!numeric_runtime.contains("import java.nio.ByteBuffer;"));
        assert!(!numeric_runtime.contains("stringFromUtf8("));

        let utf8 = JavaBackend
            .generate(
                &intrinsic_fixture(Intrinsic::StringToUtf8),
                &BackendOptions::default(),
            )
            .unwrap();
        let utf8_runtime =
            generated_text(&utf8, "src/main/java/org/polyrust/generated/Runtime.java");
        assert!(utf8_runtime.contains("import java.nio.ByteBuffer;"));
        assert!(utf8_runtime.contains("stringFromUtf8("));
        assert!(!utf8_runtime.contains("import java.math.BigInteger;"));
        assert!(!utf8_runtime.contains("checkedInteger("));
    }

    #[test]
    fn declaration_fragments_own_their_exact_direct_imports() {
        let checked = fixture();
        let generator = Generator::new(&checked);
        for declaration in &checked.module().declarations {
            let imports = fragment_imports(&generator.declaration(declaration));
            match declaration {
                Declaration::Record(_) => assert_eq!(
                    imports,
                    [
                        "java.util.Collections",
                        "java.util.LinkedHashMap",
                        "java.util.List",
                        "java.util.Map",
                    ]
                ),
                Declaration::Function(_) => assert_eq!(imports, ["java.util.List"]),
                Declaration::Contract(_)
                | Declaration::Implementation(_)
                | Declaration::Test(_) => assert!(imports.is_empty()),
                Declaration::Alias(_) | Declaration::Enum(_) | Declaration::Constant(_) => {}
            }
        }

        let checked = enum_fixture();
        let generator = Generator::new(&checked);
        let enumeration = checked
            .module()
            .declarations
            .iter()
            .find(|declaration| matches!(declaration, Declaration::Enum(_)))
            .expect("enum fixture contains an enum");
        let imports = fragment_imports(&generator.declaration(enumeration));
        for required in [
            "java.util.Collections",
            "java.util.LinkedHashMap",
            "java.util.Map",
        ] {
            assert!(imports.contains(&required), "missing {required}");
        }
        for unrelated in [
            "java.math.BigInteger",
            "java.nio.ByteBuffer",
            "java.util.Objects",
        ] {
            assert!(!imports.contains(&unrelated), "unexpected {unrelated}");
        }
    }

    fn fragment_imports(fragment: &LanguageFragment<JavaImport>) -> Vec<&'static str> {
        fragment
            .imports()
            .groups()
            .flat_map(|(_, imports)| imports.iter())
            .map(|import| import.qualified_name)
            .collect()
    }

    fn generated_text<'a>(manifest: &'a OutputManifest, path: &str) -> &'a str {
        match manifest.file(path).unwrap().contents() {
            portable_codegen::OutputContents::Text(text) => text,
            portable_codegen::OutputContents::Bytes(_) => panic!("Java source must be text"),
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

    fn enum_fixture() -> CheckedProgram {
        portable_check::v0::check_program(
            portable_ir::v0::from_json(
                br#"{"ir_version":"0.1.0","module":{"name":"enum_matrix","declarations":[{"kind":"enum","data":{"header":{"node":{"id":1,"source":{"kind":"logical","data":{"segments":["enum_matrix","Choice"]}}},"name":"Choice","visibility":"public","documentation":[]},"variants":[{"header":{"node":{"id":2,"source":{"kind":"logical","data":{"segments":["enum_matrix","Choice","Empty"]}}},"name":"Empty","documentation":[]},"fields":[]},{"header":{"node":{"id":3,"source":{"kind":"logical","data":{"segments":["enum_matrix","Choice","Named"]}}},"name":"Named","documentation":[]},"fields":[{"header":{"node":{"id":4,"source":{"kind":"logical","data":{"segments":["enum_matrix","Choice","Named","text"]}}},"name":"text","documentation":[]},"ty":{"kind":"string"}}]}]}}]},"metadata":{}}"#,
            )
            .unwrap(),
        )
        .unwrap()
    }

    fn intrinsic_fixture(operation: Intrinsic) -> CheckedProgram {
        let source = |id| SourceRef::logical([format!("runtime-feature-{id}")]);
        let node = |id| NodeMeta::new(NodeId::new(id), source(id));
        let (arguments, return_type) = match operation {
            Intrinsic::IntAddChecked => (
                vec![
                    Expression::Literal {
                        node: node(2),
                        value: Value::I32(20),
                    },
                    Expression::Literal {
                        node: node(3),
                        value: Value::I32(22),
                    },
                ],
                TypeRef::I32,
            ),
            Intrinsic::StringToUtf8 => (
                vec![Expression::Literal {
                    node: node(2),
                    value: Value::String("hello".to_owned()),
                }],
                TypeRef::Bytes,
            ),
            _ => panic!("test fixture supports numeric and UTF-8 roots only"),
        };
        portable_check::v0::check_program(IrDocument::new(
            IrVersion::CURRENT,
            Module {
                name: "runtime_feature".to_owned(),
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
                        node: node(5),
                        statements: vec![],
                        result: Some(Box::new(Expression::Intrinsic {
                            node: node(4),
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
