//! Typed Python 3.13 generation from checked portable IR v0.

#![forbid(unsafe_code)]

use std::collections::{BTreeMap, BTreeSet};

use portable_check::v0::{Capability, CheckedProgram};
use portable_codegen::{
    Backend, BackendDescriptor, BackendError, BackendOptions, BackendVersion, CapabilitySupport,
    DeclaredDependency, Document as CodeDocument, FileGroup, FileGroupId, FileRole, ImportGroup,
    ImportSet, InjectedHelper, IrVersionRange, LanguageFile, LanguageFragment, LanguagePackage,
    LanguagePlugin, LanguageRenderer, LanguageSourceFile, OptionsSchema, OutputManifest, RawText,
    TargetId, generate_with_plugin,
};
use portable_ir::v0::{Declaration, IrVersion, NodeId, TypeRef};

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
#[doc(hidden)]
pub enum PythonImport {
    Future(&'static str),
    Module(&'static str),
    From {
        module: &'static str,
        name: &'static str,
    },
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
                match import {
                    PythonImport::Future(name) => {
                        futures.insert(*name);
                    }
                    PythonImport::Module(module) => {
                        modules.insert(*module);
                    }
                    PythonImport::From { module, name } => {
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
                        FileRole::Metadata,
                        PYPROJECT,
                    )],
                )
                .map_err(generation_error)?,
                FileGroup::new(
                    file_group("runtime")?,
                    vec![LanguageFile::source(runtime_file())],
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

fn runtime_file() -> LanguageSourceFile<PythonImport> {
    let mut file = LanguageSourceFile::new("src/generated_polyrust/runtime.py", FileRole::Runtime);
    let mut body = LanguageFragment::new(CodeDocument::raw_text(RawText::new(RUNTIME)));
    body.require_import(future_group(), PythonImport::Future("annotations"));
    for module in ["math", "struct"] {
        body.require_import(standard_group(), PythonImport::Module(module));
    }
    body.require_import(
        standard_group(),
        PythonImport::From {
            module: "dataclasses",
            name: "dataclass",
        },
    );
    body.require_import(
        standard_group(),
        PythonImport::From {
            module: "types",
            name: "MappingProxyType",
        },
    );
    for name in ["Any", "Generic", "TypeVar"] {
        body.require_import(
            standard_group(),
            PythonImport::From {
                module: "typing",
                name,
            },
        );
    }
    file.set_body(body);
    file
}

fn conformance_file() -> LanguageSourceFile<PythonImport> {
    let mut file = LanguageSourceFile::new("tests/test_conformance.py", FileRole::Conformance);
    let mut body = LanguageFragment::new(CodeDocument::raw_text(RawText::new(CONFORMANCE_BODY)));
    for name in ["checked_i32", "checked_i64", "scalar_length", "wrapping"] {
        body.require_import(
            local_group(),
            PythonImport::From {
                module: "generated_polyrust.runtime",
                name,
            },
        );
    }
    file.set_body(body);
    file
}

fn type_negative_file() -> LanguageSourceFile<PythonImport> {
    let mut file = LanguageSourceFile::new("negative/invalid_option.py", FileRole::NegativeTest);
    let mut body = LanguageFragment::new(CodeDocument::raw_text(RawText::new(TYPE_NEGATIVE_BODY)));
    body.require_import(
        local_group(),
        PythonImport::From {
            module: "generated_polyrust.runtime",
            name: "PolyOption",
        },
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
            LanguageSourceFile::new("src/generated_polyrust/__init__.py", FileRole::Source);
        file.set_preamble(LanguageFragment::new(CodeDocument::raw_text(RawText::new(
            "# Generated by PolyRust from checked IR v0.",
        ))));
        let mut body = LanguageFragment::new(CodeDocument::empty());
        body.require_import(standard_group(), PythonImport::Module("json"));
        require_from(&mut body, local_group(), ".runtime", "Runtime");
        let mut output = String::new();
        output.push_str(&format!("_runtime = Runtime(json.loads({literal}))\n\n"));
        let mut declarations: Vec<_> = self.program.module().declarations.iter().collect();
        declarations.sort_by_key(|declaration| declaration.header().node.id);
        for declaration in declarations {
            self.require_declaration_imports(&mut body, declaration);
            self.declaration(&mut output, declaration);
        }
        let tests: Vec<_> = self.program.module().declarations.iter().filter_map(|declaration| if let Declaration::Test(test) = declaration { Some(serde_json::json!({"invocation": test.invocation, "expected": test.expected})) } else { None }).collect();
        if !tests.is_empty() {
            body.require_import(future_group(), PythonImport::Future("annotations"));
            require_from(&mut body, standard_group(), "typing", "cast");
            require_from(&mut body, local_group(), ".runtime", "PolyResult");
            output.push_str(&format!("_TESTS: list[dict[str, object]] = json.loads({})\n\ndef _run_test(index: int) -> tuple[PolyResult[object], object, bool]:\n    test = _TESTS[index]\n    invocation = cast(dict[str, object], test[\"invocation\"])\n    data = cast(dict[str, object], invocation[\"data\"])\n    arguments = tuple(_runtime.decode(cast(dict[str, object], item)) for item in cast(list[object], data[\"arguments\"]))\n    if invocation[\"kind\"] == \"function\":\n        actual = _runtime.invoke(cast(int, data[\"function\"]), arguments)\n    else:\n        actual = _runtime.invoke_method(cast(int, data[\"implementation\"]), cast(int, data[\"method\"]), _runtime.decode(cast(dict[str, object], data[\"receiver\"])), arguments)\n    expected = cast(dict[str, object], test[\"expected\"])\n    return actual, _runtime.decode(cast(dict[str, object], expected[\"data\"])), expected[\"kind\"] == \"error\"\n", serde_json::to_string(&serde_json::to_string(&tests).expect("tests serialize")).expect("test JSON literal")));
        }
        body = body.map_document(|_| CodeDocument::raw_text(RawText::new(output)));
        file.set_body(body);
        Ok(file)
    }

    fn require_declaration_imports(
        &self,
        unit: &mut LanguageFragment<PythonImport>,
        declaration: &Declaration,
    ) {
        match declaration {
            Declaration::Alias(item) => {
                require_annotations(unit);
                require_from(unit, standard_group(), "typing", "TypeAlias");
                require_type(unit, &item.target);
            }
            Declaration::Record(item) => {
                require_annotations(unit);
                require_from(unit, standard_group(), "dataclasses", "dataclass");
                require_from(unit, standard_group(), "dataclasses", "field");
                for field in &item.fields {
                    require_type(unit, &field.ty);
                }
            }
            Declaration::Enum(item) => {
                require_annotations(unit);
                require_from(unit, standard_group(), "dataclasses", "dataclass");
                require_from(unit, standard_group(), "dataclasses", "field");
                require_from(unit, standard_group(), "typing", "TypeAlias");
                for variant in &item.variants {
                    for field in &variant.fields {
                        require_type(unit, &field.ty);
                    }
                }
            }
            Declaration::Contract(item) => {
                require_annotations(unit);
                require_from(unit, standard_group(), "typing", "Protocol");
                require_from(unit, local_group(), ".runtime", "PolyResult");
                for method in &item.methods {
                    for parameter in &method.parameters {
                        require_type(unit, &parameter.ty);
                    }
                    require_type(unit, &method.return_type);
                }
            }
            Declaration::Constant(item) => {
                require_callable_imports(unit);
                require_type(unit, &item.ty);
            }
            Declaration::Function(item) => {
                require_callable_imports(unit);
                for parameter in &item.parameters {
                    require_type(unit, &parameter.ty);
                }
                require_type(unit, &item.return_type);
            }
            Declaration::Implementation(item) => {
                if !item.methods.is_empty() {
                    require_callable_imports(unit);
                }
                for method in &item.methods {
                    for parameter in &method.parameters {
                        require_type(unit, &parameter.ty);
                    }
                    require_type(unit, &method.return_type);
                }
            }
            Declaration::Test(_) => {}
        }
    }

    fn declaration(&self, output: &mut String, declaration: &Declaration) {
        match declaration {
            Declaration::Alias(item) => output.push_str(&format!("{}: TypeAlias = {}\n\n", type_name(&item.header.name), self.ty(&item.target))),
            Declaration::Record(item) => {
                output.push_str("@dataclass(frozen=True, slots=True)\n");
                output.push_str(&format!("class {}:\n    __poly_decl__: int = field(default={}, init=False, repr=False)\n", type_name(&item.header.name), item.header.node.id.0));
                if item.fields.is_empty() { output.push_str("    pass\n"); } else { for field in &item.fields { output.push_str(&format!("    {}: {}\n", value_name(&field.header.name), self.ty(&field.ty))); } }
                for implementation in self.implementations(item.header.node.id) { for method in &implementation.methods { output.push_str(&format!("\n    def {}(self, {}) -> PolyResult[{}]:\n        return cast(PolyResult[{}], _runtime.invoke_method({}, {}, self, ({})))\n", value_name(&method.header.name), self.parameters(&method.parameters), self.ty(&method.return_type), self.ty(&method.return_type), implementation.header.node.id.0, method.header.node.id.0, tuple_arguments(&method.parameters))); } }
                output.push('\n');
            }
            Declaration::Enum(item) => {
                let mut names = Vec::new();
                for variant in &item.variants { let name = format!("{}{}", type_name(&item.header.name), type_name(&variant.header.name)); names.push(name.clone()); output.push_str("@dataclass(frozen=True, slots=True)\n"); output.push_str(&format!("class {name}:\n    tag: str = field(default={:?}, init=False)\n", variant.header.name)); if variant.fields.is_empty() { output.push_str("    pass\n"); } else { for field in &variant.fields { output.push_str(&format!("    {}: {}\n", value_name(&field.header.name), self.ty(&field.ty))); } } output.push('\n'); }
                output.push_str(&format!("{}: TypeAlias = {}\n\n", type_name(&item.header.name), names.join(" | ")));
            }
            Declaration::Contract(item) => { output.push_str(&format!("class {}(Protocol):\n", type_name(&item.header.name))); if item.methods.is_empty() { output.push_str("    pass\n"); } else { for method in &item.methods { output.push_str(&format!("    def {}(self, {}) -> PolyResult[{}]: ...\n", value_name(&method.header.name), self.parameters(&method.parameters), self.ty(&method.return_type))); } } output.push('\n'); }
            Declaration::Constant(item) => output.push_str(&format!("def {}() -> PolyResult[{}]:\n    return cast(PolyResult[{}], _runtime.read_constant({}))\n\n", value_name(&item.header.name), self.ty(&item.ty), self.ty(&item.ty), item.header.node.id.0)),
            Declaration::Function(item) => output.push_str(&format!("def {}({}) -> PolyResult[{}]:\n    return cast(PolyResult[{}], _runtime.invoke({}, ({})))\n\n", value_name(&item.header.name), self.parameters(&item.parameters), self.ty(&item.return_type), self.ty(&item.return_type), item.header.node.id.0, tuple_arguments(&item.parameters))),
            Declaration::Implementation(_) | Declaration::Test(_) => {}
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
                    "{}: {}",
                    value_name(&parameter.header.name),
                    self.ty(&parameter.ty)
                )
            })
            .collect::<Vec<_>>()
            .join(", ")
    }
    fn ty(&self, ty: &TypeRef) -> String {
        match ty {
            TypeRef::Unit => "None".into(),
            TypeRef::Bool => "bool".into(),
            TypeRef::I32 | TypeRef::I64 => "int".into(),
            TypeRef::F64 => "float".into(),
            TypeRef::Char | TypeRef::String => "str".into(),
            TypeRef::Bytes => "bytes".into(),
            TypeRef::List(inner) => format!("tuple[{}, ...]", self.ty(inner)),
            TypeRef::Option(inner) => format!("PolyOption[{}]", self.ty(inner)),
            TypeRef::Result { ok, error } => {
                format!("PolyValueResult[{}, {}]", self.ty(ok), self.ty(error))
            }
            TypeRef::Named(id) | TypeRef::Contract(id) => {
                type_name(self.names.get(id).map(String::as_str).unwrap_or("Unknown"))
            }
        }
    }
    fn tests_file(&self) -> LanguageSourceFile<PythonImport> {
        let mut file = LanguageSourceFile::new("tests/test_generated.py", FileRole::Test);
        let mut body = LanguageFragment::new(CodeDocument::empty());
        let mut output = String::new();
        let mut index = 0;
        for declaration in &self.program.module().declarations {
            if let Declaration::Test(test) = declaration {
                if index == 0 {
                    require_from(&mut body, local_group(), "generated_polyrust", "_run_test");
                    require_from(
                        &mut body,
                        local_group(),
                        "generated_polyrust.runtime",
                        "portable_test_equal",
                    );
                }
                output.push_str(&format!("def test_{}() -> None:\n    actual, expected, expects_error = _run_test({index})\n    assert actual.ok is not expects_error\n    if actual.ok:\n        assert portable_test_equal(actual.value, expected)\n\n", value_name(&test.header.name)));
                index += 1;
            }
        }
        if !output.is_empty() {
            body = body.map_document(|_| CodeDocument::raw_text(RawText::new(output)));
            file.set_body(body);
        }
        file
    }
}

fn require_from(
    unit: &mut LanguageFragment<PythonImport>,
    group: ImportGroup,
    module: &'static str,
    name: &'static str,
) {
    unit.require_import(group, PythonImport::From { module, name });
}

fn require_annotations(unit: &mut LanguageFragment<PythonImport>) {
    unit.require_import(future_group(), PythonImport::Future("annotations"));
}

fn require_callable_imports(unit: &mut LanguageFragment<PythonImport>) {
    require_annotations(unit);
    require_from(unit, standard_group(), "typing", "cast");
    require_from(unit, local_group(), ".runtime", "PolyResult");
}

fn require_type(unit: &mut LanguageFragment<PythonImport>, ty: &TypeRef) {
    match ty {
        TypeRef::List(inner) => require_type(unit, inner),
        TypeRef::Option(inner) => {
            require_from(unit, local_group(), ".runtime", "PolyOption");
            require_type(unit, inner);
        }
        TypeRef::Result { ok, error } => {
            require_from(unit, local_group(), ".runtime", "PolyValueResult");
            require_type(unit, ok);
            require_type(unit, error);
        }
        TypeRef::Unit
        | TypeRef::Bool
        | TypeRef::I32
        | TypeRef::I64
        | TypeRef::F64
        | TypeRef::Char
        | TypeRef::String
        | TypeRef::Bytes
        | TypeRef::Named(_)
        | TypeRef::Contract(_) => {}
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
    #[test]
    fn keyword_and_types() {
        assert_eq!(identifier("class"), "class_");
        assert_eq!(Generator::new(&fixture()).ty(&TypeRef::I64), "int");
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
        assert_eq!(runtime.matches("import math").count(), 1);
        assert_eq!(runtime.matches("import struct").count(), 1);
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
