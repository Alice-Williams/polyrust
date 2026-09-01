use std::collections::BTreeMap;

use portable_check::v0::{Capability, CheckedProgram};
use portable_codegen::{
    Backend, BackendDescriptor, BackendError, BackendOptions, BackendVersion, CapabilitySupport,
    DeclaredDependency, Document as CodeDocument, FileGroup, FileGroupId, FileRole, ImportSet,
    InjectedHelper, IrVersionRange, LanguageFile, LanguagePackage, LanguagePlugin,
    LanguageRenderer, LanguageSourceFile, LanguageUnit, OptionsSchema, OutputManifest, RawText,
    TargetId, generate_with_plugin,
};
use portable_ir::v0::{
    Declaration, ExpectedOutcome, IrVersion, NodeId, TestInvocation, TypeRef, TypedValue, Value,
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
pub struct GoImport(&'static str);

#[doc(hidden)]
pub struct GoRenderer;

impl LanguageRenderer<GoImport> for GoRenderer {
    fn render_imports(&self, imports: &ImportSet<GoImport>) -> Result<CodeDocument, String> {
        let names = imports
            .groups()
            .flat_map(|(_, imports)| imports.iter())
            .map(|import| format!("\t{:?}", import.0))
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
                        FileRole::Metadata,
                        "module generated.polyrust/package\n\ngo 1.25.0\n",
                    )],
                )
                .map_err(go_generation_error)?,
                FileGroup::new(
                    go_group("runtime")?,
                    vec![LanguageFile::source(go_runtime_file())],
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

fn go_preamble(generated: bool) -> CodeDocument {
    let text = if generated {
        "// Code generated by PolyRust. DO NOT EDIT.\npackage generated"
    } else {
        "package generated"
    };
    CodeDocument::raw_text(RawText::new(text))
}

fn go_runtime_file() -> LanguageSourceFile<GoImport> {
    let mut file = LanguageSourceFile::new("runtime.go", FileRole::Runtime);
    file.set_preamble(LanguageUnit::new(go_preamble(true)));
    let mut body = LanguageUnit::new(CodeDocument::raw_text(RawText::new(RUNTIME)));
    for import in [
        "bytes",
        "encoding/binary",
        "encoding/json",
        "math",
        "strconv",
        "strings",
        "unicode/utf8",
    ] {
        body.require_import(go_import_group(), GoImport(import));
    }
    file.set_body(body);
    file
}

fn go_conformance_file() -> LanguageSourceFile<GoImport> {
    let mut file = LanguageSourceFile::new("conformance_test.go", FileRole::Conformance);
    file.set_preamble(LanguageUnit::new(go_preamble(false)));
    let mut body = LanguageUnit::new(CodeDocument::raw_text(RawText::new(CONFORMANCE_BODY)));
    body.require_import(go_import_group(), GoImport("math"));
    body.require_import(go_import_group(), GoImport("testing"));
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
        let mut file = LanguageSourceFile::new("generated.go", FileRole::Source);
        file.set_preamble(LanguageUnit::new(CodeDocument::raw_text(RawText::new(
            "// Code generated by PolyRust from checked IR v0. DO NOT EDIT.\npackage generated",
        ))));
        let mut output = format!(
            "var generatedRuntime = newRuntime({})\n\n",
            go_string(&document)
        );
        let mut declarations: Vec<_> = self.program.module().declarations.iter().collect();
        declarations.sort_by_key(|item| item.header().node.id);
        for declaration in declarations {
            self.declaration(&mut output, declaration);
        }
        file.set_body(LanguageUnit::new(CodeDocument::raw_text(RawText::new(
            output,
        ))));
        Ok(file)
    }
    fn declaration(&self, output: &mut String, declaration: &Declaration) {
        match declaration {
            Declaration::Alias(item) => output.push_str(&format!("type {} = {}\n\n", exported(&item.header.name), self.ty(&item.target))),
            Declaration::Record(item) => {
                output.push_str(&format!("type {} struct {{\n", exported(&item.header.name))); for member in &item.fields { output.push_str(&format!("\t{} {}\n", exported(&member.header.name), self.ty(&member.ty))); } output.push_str("}\n\n");
                output.push_str(&format!("func (value {}) polyValue() map[string]any {{ return map[string]any{{\"__polyDecl\": int64({})", exported(&item.header.name), item.header.node.id.0)); for member in &item.fields { output.push_str(&format!(", {:?}: value.{}", member.header.name, exported(&member.header.name))); } output.push_str("} }\n\n");
                for implementation in self.implementations(item.header.node.id) { output.push_str(&format!("var _ {} = {}{{}}\n\n", exported(self.name(implementation.contract)), exported(&item.header.name))); for method in &implementation.methods { output.push_str(&format!("func (value {}) {}({}) PolyResult[{}] {{ return castResult[{}](generatedRuntime.invokeMethod({}, {}, value, []any{{{}}})) }}\n\n", exported(&item.header.name), exported(&method.header.name), self.parameters(&method.parameters), self.ty(&method.return_type), self.ty(&method.return_type), implementation.header.node.id.0, method.header.node.id.0, args(&method.parameters))); } }
            }
            Declaration::Enum(item) => { let mut variants = Vec::new(); for variant in &item.variants { let name = format!("{}{}", exported(&item.header.name), exported(&variant.header.name)); variants.push(name.clone()); output.push_str(&format!("type {name} struct {{\n\tTag string\n")); for member in &variant.fields { output.push_str(&format!("\t{} {}\n", exported(&member.header.name), self.ty(&member.ty))); } output.push_str("}\n\n"); } output.push_str(&format!("type {} interface {{ is{}() }}\n", exported(&item.header.name), exported(&item.header.name))); for variant in variants { output.push_str(&format!("func ({variant}) is{}() {{}}\n", exported(&item.header.name))); } output.push('\n'); }
            Declaration::Contract(item) => { output.push_str(&format!("type {} interface {{\n\tpolyValue() map[string]any\n", exported(&item.header.name))); for method in &item.methods { output.push_str(&format!("\t{}({}) PolyResult[{}]\n", exported(&method.header.name), self.parameters(&method.parameters), self.ty(&method.return_type))); } output.push_str("}\n\n"); }
            Declaration::Constant(item) => output.push_str(&format!("func {}() PolyResult[{}] {{ return castResult[{}](generatedRuntime.constant({})) }}\n\n", exported(&item.header.name), self.ty(&item.ty), self.ty(&item.ty), item.header.node.id.0)),
            Declaration::Function(item) => output.push_str(&format!("func {}({}) PolyResult[{}] {{ return castResult[{}](generatedRuntime.invoke({}, []any{{{}}})) }}\n\n", exported(&item.header.name), self.parameters(&item.parameters), self.ty(&item.return_type), self.ty(&item.return_type), item.header.node.id.0, args(&item.parameters))),
            Declaration::Implementation(_) | Declaration::Test(_) => {}
        }
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
    fn parameters(&self, parameters: &[portable_ir::v0::Parameter]) -> String {
        parameters
            .iter()
            .map(|item| format!("{} {}", local(&item.header.name), self.ty(&item.ty)))
            .collect::<Vec<_>>()
            .join(", ")
    }
    fn ty(&self, ty: &TypeRef) -> String {
        match ty {
            TypeRef::Unit => "struct{}".into(),
            TypeRef::Bool => "bool".into(),
            TypeRef::I32 => "int32".into(),
            TypeRef::I64 => "int64".into(),
            TypeRef::F64 => "float64".into(),
            TypeRef::Char => "rune".into(),
            TypeRef::String => "string".into(),
            TypeRef::Bytes => "PolyBytes".into(),
            TypeRef::List(inner) => format!("PolyList[{}]", self.ty(inner)),
            TypeRef::Option(inner) => format!("PolyOption[{}]", self.ty(inner)),
            TypeRef::Result { ok, error } => {
                format!("PolyValueResult[{}, {}]", self.ty(ok), self.ty(error))
            }
            TypeRef::Named(id) | TypeRef::Contract(id) => exported(self.name(*id)),
        }
    }
    fn name(&self, id: NodeId) -> &str {
        self.names.get(&id).map(String::as_str).unwrap_or("Unknown")
    }
    fn tests_file(&self) -> LanguageSourceFile<GoImport> {
        let mut file = LanguageSourceFile::new("generated_test.go", FileRole::Test);
        file.set_preamble(LanguageUnit::new(CodeDocument::raw_text(RawText::new(
            "// Code generated from portable tests. DO NOT EDIT.\npackage generated",
        ))));
        let mut body = LanguageUnit::new(CodeDocument::empty());
        let mut output = String::new();
        let mut has_tests = false;
        let mut requires_math = false;
        for declaration in &self.program.module().declarations {
            if let Declaration::Test(test) = declaration {
                has_tests = true;
                requires_math |= match &test.invocation {
                    TestInvocation::Function { arguments, .. } => {
                        arguments.iter().any(typed_value_uses_f64)
                    }
                    TestInvocation::Method {
                        receiver,
                        arguments,
                        ..
                    } => {
                        typed_value_uses_f64(receiver) || arguments.iter().any(typed_value_uses_f64)
                    }
                };
                if let ExpectedOutcome::Value(expected) = &test.expected {
                    requires_math |= typed_value_uses_f64(expected);
                }
                output.push_str(&format!(
                    "func Test{}(t *testing.T) {{\n",
                    exported(&test.header.name)
                ));
                let call = match &test.invocation {
                    TestInvocation::Function {
                        function,
                        arguments,
                    } => format!(
                        "{}({})",
                        exported(self.name(*function)),
                        arguments
                            .iter()
                            .map(|value| self.value(value))
                            .collect::<Vec<_>>()
                            .join(", ")
                    ),
                    TestInvocation::Method {
                        implementation,
                        method,
                        receiver,
                        arguments,
                    } => format!(
                        "{}.{}({})",
                        self.value(receiver),
                        exported(self.method_name(*implementation, *method)),
                        arguments
                            .iter()
                            .map(|value| self.value(value))
                            .collect::<Vec<_>>()
                            .join(", ")
                    ),
                };
                match &test.expected { ExpectedOutcome::Value(expected) => output.push_str(&format!("\tgot := {call}\n\tif !got.Ok || !equal(got.Value, {}) {{ t.Fatalf(\"unexpected result: %#v\", got) }}\n", self.value(expected))), ExpectedOutcome::Error(_) => output.push_str(&format!("\tgot := {call}\n\tif got.Ok {{ t.Fatalf(\"expected error: %#v\", got) }}\n")) }
                output.push_str("}\n\n");
            }
        }
        if has_tests {
            body.require_import(go_import_group(), GoImport("testing"));
            if requires_math {
                body.require_import(go_import_group(), GoImport("math"));
            }
            body.set_document(CodeDocument::raw_text(RawText::new(output)));
            file.set_body(body);
        }
        file
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
    fn value(&self, typed: &TypedValue) -> String {
        self.raw_value(&typed.value, &typed.ty)
    }
    fn raw_value(&self, value: &Value, ty: &TypeRef) -> String {
        match (value, ty) {
            (Value::Unit, _) => "struct{}{}".into(),
            (Value::Bool(value), _) => value.to_string(),
            (Value::I32(value), _) => format!("int32({value})"),
            (Value::I64(value), _) => format!("int64({value})"),
            (Value::F64(value), _) => {
                format!("math.Float64frombits(0x{:016x})", value.0)
            }
            (Value::String(value), _) => go_string(value),
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
                format!(
                    "{}{{{}}}",
                    exported(&record.header.name),
                    fields
                        .iter()
                        .map(|field| {
                            let member = record
                                .fields
                                .iter()
                                .find(|item| item.header.node.id == field.field)
                                .expect("checked field");
                            format!(
                                "{}: {}",
                                exported(&member.header.name),
                                self.raw_value(&field.value, &member.ty)
                            )
                        })
                        .collect::<Vec<_>>()
                        .join(", ")
                )
            }
            _ => "nil".into(),
        }
    }
}

fn typed_value_uses_f64(value: &TypedValue) -> bool {
    value_uses_f64(&value.value)
}

fn value_uses_f64(value: &Value) -> bool {
    match value {
        Value::F64(_) => true,
        Value::List(values) => values.iter().any(value_uses_f64),
        Value::Some(value) | Value::Ok(value) | Value::Err(value) => value_uses_f64(value),
        Value::Record { fields, .. } | Value::Enum { fields, .. } => {
            fields.iter().any(|field| value_uses_f64(&field.value))
        }
        Value::Unit
        | Value::Bool(_)
        | Value::I32(_)
        | Value::I64(_)
        | Value::Char(_)
        | Value::String(_)
        | Value::Bytes(_)
        | Value::None => false,
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

const CONFORMANCE_BODY: &str = "func TestTwentySemanticVectors(t *testing.T) {\n source := NewPolyList(int32(1)); appended := source.append(2)\n wide32 := uint32(2147483648); wide64 := uint64(9223372036854775808)\n vectors := []bool{\n  checked32(0).Ok, checked32(2147483647).Ok, checked32(-2147483648).Ok, !checked32(2147483648).Ok, !checked32(-2147483649).Ok,\n  int64(0) == 0, int64(9223372036854775807) > 0, int64(-9223372036854775807-1) < 0,\n  int32(wide32) == -2147483648, int64(wide64) == -9223372036854775807-1,\n  len([]rune(\"a\")) == 1, len([]rune(\"😀\")) == 1, NewPolyBytes(1,2).Values()[1] == 2,\n  source.Len() == 1, appended.Len() == 2, len(source.Values()) == 1, len(appended.Values()) == 2,\n  PolyOption[int]{Tag:\"none\"}.Tag == \"none\", PolyOption[int]{Tag:\"some\",Value:0}.Tag == \"some\", math.Signbit(math.Copysign(0,-1)),\n }\n if len(vectors)!=20 { t.Fatal(len(vectors)) }; for index,value:=range vectors { if !value { t.Fatal(index) } }\n}\n";

#[cfg(test)]
mod tests {
    use super::*;
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
        assert_eq!(Generator::new(&checked).ty(&TypeRef::I64), "int64");
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
        assert_eq!(tests.matches("import \"testing\"").count(), 1);
        let runtime = generated_text(&manifest, "runtime.go");
        assert_eq!(runtime.matches("\t\"encoding/json\"").count(), 1);
        assert_eq!(runtime.matches("\t\"unicode/utf8\"").count(), 1);

        let empty = GoV0Backend
            .generate(&empty_fixture(), &BackendOptions::default())
            .unwrap();
        let empty_tests = generated_text(&empty, "generated_test.go");
        assert!(!empty_tests.contains("import"));
        assert!(empty_tests.contains("package generated"));
    }

    fn generated_text<'a>(manifest: &'a OutputManifest, path: &str) -> &'a str {
        match manifest.file(path).unwrap().contents() {
            portable_codegen::OutputContents::Text(text) => text,
            portable_codegen::OutputContents::Bytes(_) => panic!("Go source must be text"),
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
