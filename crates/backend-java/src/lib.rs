//! Dependency-free Java 21 generation from checked portable IR v0.

#![forbid(unsafe_code)]

use std::collections::BTreeMap;

use portable_check::v0::{Capability, CheckedProgram};
use portable_codegen::{
    Backend, BackendDescriptor, BackendError, BackendOptions, BackendVersion, CapabilitySupport,
    DeclaredDependency, Document as CodeDocument, FileGroup, FileGroupId, FileRole, ImportGroup,
    ImportSet, InjectedHelper, IrVersionRange, LanguageFile, LanguagePackage, LanguagePlugin,
    LanguageRenderer, LanguageSourceFile, LanguageUnit, OptionsSchema, OutputManifest, RawText,
    TargetId, generate_with_plugin,
};
use portable_ir::v0::{Declaration, IrVersion, NodeId, TypeRef, Visibility};

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

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
#[doc(hidden)]
pub struct JavaImport(&'static str);

#[doc(hidden)]
pub struct JavaRenderer;

impl LanguageRenderer<JavaImport> for JavaRenderer {
    fn render_imports(&self, imports: &ImportSet<JavaImport>) -> Result<CodeDocument, String> {
        let lines = imports
            .groups()
            .flat_map(|(_, imports)| imports.iter())
            .map(|import| format!("import {};", import.0))
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
                        FileRole::Documentation,
                        README,
                    )],
                )
                .map_err(java_generation_error)?,
                FileGroup::new(
                    java_group("runtime")?,
                    vec![LanguageFile::source(java_runtime_file())],
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

fn require_java(unit: &mut LanguageUnit<JavaImport>, import: &'static str) {
    unit.require_import(java_import_group(), JavaImport(import));
}

fn java_runtime_file() -> LanguageSourceFile<JavaImport> {
    let mut file = LanguageSourceFile::new(
        "src/main/java/org/polyrust/generated/Runtime.java",
        FileRole::Runtime,
    );
    file.set_preamble(LanguageUnit::new(java_preamble(Some(
        "// Generated packages copy this dependency-free Java 21 runtime verbatim.",
    ))));
    let mut body = LanguageUnit::new(CodeDocument::raw_text(RawText::new(RUNTIME)));
    for import in [
        "java.math.BigInteger",
        "java.nio.ByteBuffer",
        "java.nio.charset.CharacterCodingException",
        "java.nio.charset.CodingErrorAction",
        "java.nio.charset.StandardCharsets",
        "java.util.ArrayList",
        "java.util.LinkedHashMap",
        "java.util.List",
        "java.util.Map",
        "java.util.Objects",
    ] {
        require_java(&mut body, import);
    }
    file.set_body(body);
    file
}

fn java_conformance_file() -> LanguageSourceFile<JavaImport> {
    let mut file = LanguageSourceFile::new(
        "src/test/java/org/polyrust/generated/ConformanceTest.java",
        FileRole::Conformance,
    );
    file.set_preamble(LanguageUnit::new(java_preamble(None)));
    let mut body = LanguageUnit::new(CodeDocument::raw_text(RawText::new(CONFORMANCE_BODY)));
    require_java(&mut body, "java.util.List");
    file.set_body(body);
    file
}

fn java_invalid_types_file() -> LanguageSourceFile<JavaImport> {
    let mut file = LanguageSourceFile::new("negative/InvalidTypes.java", FileRole::NegativeTest);
    file.set_preamble(LanguageUnit::new(java_preamble(None)));
    file.set_body(LanguageUnit::new(CodeDocument::raw_text(RawText::new(
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
            FileRole::Source,
        );
        file.set_preamble(LanguageUnit::new(java_preamble(Some(
            "// Generated by PolyRust from checked IR v0.",
        ))));
        let mut body = LanguageUnit::new(CodeDocument::empty());
        require_java(&mut body, "java.util.List");
        if self
            .program
            .module()
            .declarations
            .iter()
            .any(|declaration| matches!(declaration, Declaration::Record(_) | Declaration::Enum(_)))
        {
            require_java(&mut body, "java.util.Collections");
            require_java(&mut body, "java.util.LinkedHashMap");
            require_java(&mut body, "java.util.Map");
        }
        let mut output = format!(
            "public final class Generated {{\n\
             \x20 private static final Runtime RUNTIME = new Runtime({document_literal});\n\
             \x20 private static final List<Object> TESTS = Runtime.jsonArray({tests_literal});\n\
             \x20 private Generated() {{}}\n\n"
        );
        let mut declarations: Vec<_> = self.program.module().declarations.iter().collect();
        declarations.sort_by_key(|declaration| declaration.header().node.id);
        for declaration in declarations {
            self.declaration(&mut output, declaration);
        }
        output.push_str(
            "  static Runtime.TestOutcome invokeTest(int index) {\n\
             \x20   return RUNTIME.invokeTest(TESTS, index);\n\
             \x20 }\n\
             }\n",
        );
        body.set_document(CodeDocument::raw_text(RawText::new(output)));
        file.set_body(body);
        Ok(file)
    }

    fn declaration(&self, output: &mut String, declaration: &Declaration) {
        match declaration {
            Declaration::Alias(_) | Declaration::Implementation(_) | Declaration::Test(_) => {}
            Declaration::Contract(item) => {
                output.push_str(&format!(
                    "  {}interface {} {{\n",
                    visibility(item.header.visibility),
                    type_name(&item.header.name)
                ));
                for method in &item.methods {
                    output.push_str(&format!(
                        "    Runtime.PolyResult<{}> {}({});\n",
                        self.ty(&method.return_type),
                        value_name(&method.header.name),
                        self.parameters(&method.parameters)
                    ));
                }
                output.push_str("  }\n\n");
            }
            Declaration::Record(item) => {
                let implementations = self.implementations(item.header.node.id);
                let contracts = implementations
                    .iter()
                    .map(|implementation| type_name(self.name(implementation.contract)))
                    .collect::<Vec<_>>();
                let mut implemented = vec!["Runtime.PolyRecord".to_owned()];
                implemented.extend(contracts);
                output.push_str(&format!(
                    "  {}record {}({}) implements {} {{\n",
                    visibility(item.header.visibility),
                    type_name(&item.header.name),
                    item.fields
                        .iter()
                        .map(|field| format!(
                            "{} {}",
                            self.ty(&field.ty),
                            value_name(&field.header.name)
                        ))
                        .collect::<Vec<_>>()
                        .join(", "),
                    implemented.join(", ")
                ));
                output.push_str("    @Override public Map<String, Object> polyValue() {\n");
                output.push_str("      Map<String, Object> value = new LinkedHashMap<>();\n");
                output.push_str(&format!(
                    "      value.put(\"__polyDecl\", {}L);\n",
                    item.header.node.id.0
                ));
                for field in &item.fields {
                    output.push_str(&format!(
                        "      value.put({}, {});\n",
                        java_string(&field.header.name),
                        value_name(&field.header.name)
                    ));
                }
                output.push_str("      return Collections.unmodifiableMap(value);\n    }\n");
                for implementation in implementations {
                    for method in &implementation.methods {
                        output.push_str(&format!(
                            "    @Override public Runtime.PolyResult<{}> {}({}) {{\n\
                             \x20     return Runtime.cast(RUNTIME.invokeMethod({}L, {}L, this, List.of({})));\n\
                             \x20   }}\n",
                            self.ty(&method.return_type),
                            value_name(&method.header.name),
                            self.parameters(&method.parameters),
                            implementation.header.node.id.0,
                            method.header.node.id.0,
                            self.arguments(&method.parameters)
                        ));
                    }
                }
                output.push_str("  }\n\n");
            }
            Declaration::Enum(item) => {
                let name = type_name(&item.header.name);
                output.push_str(&format!(
                    "  {}sealed interface {name} extends Runtime.PolyRecord permits {} {{}}\n",
                    visibility(item.header.visibility),
                    item.variants
                        .iter()
                        .map(|variant| format!("{name}{}", type_name(&variant.header.name)))
                        .collect::<Vec<_>>()
                        .join(", ")
                ));
                for variant in &item.variants {
                    output.push_str(&format!(
                        "  {}record {name}{}({}) implements {name} {{\n",
                        visibility(item.header.visibility),
                        type_name(&variant.header.name),
                        variant
                            .fields
                            .iter()
                            .map(|field| format!(
                                "{} {}",
                                self.ty(&field.ty),
                                value_name(&field.header.name)
                            ))
                            .collect::<Vec<_>>()
                            .join(", ")
                    ));
                    output.push_str("    @Override public Map<String, Object> polyValue() {\n");
                    output.push_str("      Map<String, Object> value = new LinkedHashMap<>();\n");
                    output.push_str(&format!(
                        "      value.put(\"__polyDecl\", {}L);\n\
                         \x20     value.put(\"tag\", {});\n",
                        item.header.node.id.0,
                        java_string(&variant.header.name)
                    ));
                    for field in &variant.fields {
                        output.push_str(&format!(
                            "      value.put({}, {});\n",
                            java_string(&field.header.name),
                            value_name(&field.header.name)
                        ));
                    }
                    output
                        .push_str("      return Collections.unmodifiableMap(value);\n    }\n  }\n");
                }
                output.push('\n');
            }
            Declaration::Constant(item) => output.push_str(&format!(
                "  {}static Runtime.PolyResult<{}> {}() {{\n\
                 \x20   return Runtime.cast(RUNTIME.readConstant({}L));\n\
                 \x20 }}\n\n",
                visibility(item.header.visibility),
                self.ty(&item.ty),
                value_name(&item.header.name),
                item.header.node.id.0
            )),
            Declaration::Function(item) => output.push_str(&format!(
                "  {}static Runtime.PolyResult<{}> {}({}) {{\n\
                 \x20   return Runtime.cast(RUNTIME.invoke({}L, List.of({})));\n\
                 \x20 }}\n\n",
                visibility(item.header.visibility),
                self.ty(&item.return_type),
                value_name(&item.header.name),
                self.parameters(&item.parameters),
                item.header.node.id.0,
                self.arguments(&item.parameters)
            )),
        }
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

    fn parameters(&self, parameters: &[portable_ir::v0::Parameter]) -> String {
        parameters
            .iter()
            .map(|parameter| {
                format!(
                    "{} {}",
                    self.ty(&parameter.ty),
                    value_name(&parameter.header.name)
                )
            })
            .collect::<Vec<_>>()
            .join(", ")
    }

    fn arguments(&self, parameters: &[portable_ir::v0::Parameter]) -> String {
        parameters
            .iter()
            .map(|parameter| value_name(&parameter.header.name))
            .collect::<Vec<_>>()
            .join(", ")
    }

    fn ty(&self, ty: &TypeRef) -> String {
        match ty {
            TypeRef::Unit => "Void".into(),
            TypeRef::Bool => "Boolean".into(),
            TypeRef::I32 => "Integer".into(),
            TypeRef::I64 => "Long".into(),
            TypeRef::F64 => "Double".into(),
            TypeRef::Char | TypeRef::String => "String".into(),
            TypeRef::Bytes => "List<Integer>".into(),
            TypeRef::List(inner) => format!("List<{}>", self.ty(inner)),
            TypeRef::Option(inner) => format!("Runtime.PolyOption<{}>", self.ty(inner)),
            TypeRef::Result { ok, error } => {
                format!(
                    "Runtime.PolyValueResult<{}, {}>",
                    self.ty(ok),
                    self.ty(error)
                )
            }
            TypeRef::Named(id) => self.named_ty(*id),
            TypeRef::Contract(id) => type_name(self.name(*id)),
        }
    }

    fn named_ty(&self, id: NodeId) -> String {
        self.program
            .module()
            .declarations
            .iter()
            .find_map(|declaration| match declaration {
                Declaration::Alias(item) if item.header.node.id == id => {
                    Some(self.ty(&item.target))
                }
                declaration if declaration.header().node.id == id => {
                    Some(type_name(&declaration.header().name))
                }
                _ => None,
            })
            .unwrap_or_else(|| "Object".into())
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
            FileRole::Test,
        );
        file.set_preamble(LanguageUnit::new(java_preamble(Some(
            "// Generated by PolyRust from checked IR v0.",
        ))));
        file.set_body(LanguageUnit::new(CodeDocument::raw_text(RawText::new(format!(
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
const CONFORMANCE_BODY: &str = "public final class ConformanceTest {\n  private ConformanceTest() {}\n  public static void main(String[] arguments) {\n    Runtime.PolyResult<Integer> astral = Runtime.scalarLength(\"😀\");\n    List<Integer> original = List.of(1);\n    List<Integer> appended = Runtime.listAppend(original, 2);\n    boolean[] vectors = {\n      Runtime.checkedI32(0L).ok(), Runtime.checkedI32(2147483647L).ok(), Runtime.checkedI32(-2147483648L).ok(), !Runtime.checkedI32(2147483648L).ok(), !Runtime.checkedI32(-2147483649L).ok(),\n      Runtime.checkedI64(java.math.BigInteger.ZERO).ok(), Runtime.checkedI64(java.math.BigInteger.valueOf(Long.MAX_VALUE)).ok(), Runtime.checkedI64(java.math.BigInteger.valueOf(Long.MIN_VALUE)).ok(), !Runtime.checkedI64(java.math.BigInteger.ONE.shiftLeft(63)).ok(), !Runtime.checkedI64(java.math.BigInteger.ONE.shiftLeft(63).negate().subtract(java.math.BigInteger.ONE)).ok(),\n      Runtime.wrappingI32(2147483648L) == Integer.MIN_VALUE, Runtime.wrappingI32(-2147483649L) == Integer.MAX_VALUE, Runtime.wrappingI64(java.math.BigInteger.ONE.shiftLeft(63)) == Long.MIN_VALUE, Runtime.wrappingI64(java.math.BigInteger.ONE.shiftLeft(63).negate().subtract(java.math.BigInteger.ONE)) == Long.MAX_VALUE,\n      Runtime.scalarLength(\"a\").ok(), astral.ok() && astral.value() == 1, !Runtime.scalarLength(\"\\ud800\").ok(), appended.size() == 2, appended != original, Double.doubleToRawLongBits(-0.0d) == Long.MIN_VALUE,\n    };\n    if (vectors.length != 20) throw new AssertionError(\"vector count\");\n    for (boolean vector : vectors) if (!vector) throw new AssertionError(\"conformance vector\");\n  }\n}\n";

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keywords_and_types_are_safe() {
        assert_eq!(identifier("class"), "class_");
        assert_eq!(Generator::new(&fixture()).ty(&TypeRef::I64), "Long");
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
        assert_eq!(runtime.matches("import java.math.BigInteger;").count(), 1);
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
}
