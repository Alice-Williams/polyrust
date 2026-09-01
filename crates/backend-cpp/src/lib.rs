//! Dependency-free C++20 generation from checked portable IR v0.

#![forbid(unsafe_code)]

use std::collections::BTreeMap;

use portable_check::v0::{Capability, CheckedProgram};
use portable_codegen::{
    Backend, BackendDescriptor, BackendError, BackendOptions, BackendVersion, CapabilitySupport,
    DeclaredDependency, Document as CodeDocument, FileGroup, FileGroupId, FileRole, ImportGroup,
    ImportSet, InjectedHelper, IrVersionRange, LanguageFile, LanguageFragment, LanguagePackage,
    LanguagePlugin, LanguageRenderer, LanguageSourceFile, OptionsSchema, OutputManifest, RawText,
    TargetId, generate_with_plugin,
};
use portable_ir::v0::{Declaration, IrVersion, NodeId, TypeRef, Visibility};

const RUNTIME: &str = include_str!("runtime.hpp");

pub struct CppBackend;

impl Backend for CppBackend {
    fn descriptor(&self) -> BackendDescriptor {
        BackendDescriptor {
            target: TargetId::parse("org.polyrust.cpp").expect("static target ID is valid"),
            display_name: "C++".to_owned(),
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
pub struct CppImport {
    path: &'static str,
    system: bool,
}

#[doc(hidden)]
pub struct CppRenderer;

impl LanguageRenderer<CppImport> for CppRenderer {
    fn render_imports(&self, imports: &ImportSet<CppImport>) -> Result<CodeDocument, String> {
        let lines = imports
            .groups()
            .flat_map(|(_, imports)| imports.iter())
            .map(|import| {
                if import.system {
                    format!("#include <{}>", import.path)
                } else {
                    format!("#include {:?}", import.path)
                }
            })
            .collect::<Vec<_>>();
        Ok(CodeDocument::raw_text(RawText::new(lines.join("\n"))))
    }
}

impl LanguagePlugin for CppBackend {
    type Import = CppImport;
    type Renderer = CppRenderer;

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
                    files: vec!["src/runtime.hpp".into()],
                }),
                CapabilitySupport::Native | CapabilitySupport::Unsupported { .. } => None,
            })
            .collect();
        LanguagePackage::new(
            vec![
                FileGroup::new(
                    cpp_group("documentation")?,
                    vec![LanguageFile::text(
                        "README.md",
                        FileRole::Documentation,
                        README,
                    )],
                )
                .map_err(cpp_generation_error)?,
                FileGroup::new(
                    cpp_group("runtime")?,
                    vec![LanguageFile::source(cpp_runtime_file())],
                )
                .map_err(cpp_generation_error)?,
                FileGroup::new(
                    cpp_group("source")?,
                    vec![
                        LanguageFile::source(generator.header_file()),
                        LanguageFile::source(generator.source_file()?),
                    ],
                )
                .map_err(cpp_generation_error)?,
                FileGroup::new(
                    cpp_group("tests")?,
                    vec![
                        LanguageFile::source(cpp_generated_test_file()),
                        LanguageFile::source(cpp_conformance_file()),
                    ],
                )
                .map_err(cpp_generation_error)?,
            ],
            Vec::<DeclaredDependency>::new(),
            helpers,
        )
        .map_err(cpp_generation_error)
    }

    fn renderer(&self) -> Self::Renderer {
        CppRenderer
    }
}

fn cpp_generation_error(error: impl std::fmt::Display) -> BackendError {
    BackendError::Generation {
        message: error.to_string(),
    }
}

fn cpp_group(name: &str) -> Result<FileGroupId, BackendError> {
    FileGroupId::parse(name).map_err(cpp_generation_error)
}

fn cpp_system_group() -> ImportGroup {
    ImportGroup::new(10, "system-headers").expect("static import group is valid")
}

fn cpp_local_group() -> ImportGroup {
    ImportGroup::new(20, "local-headers").expect("static import group is valid")
}

fn require_cpp_system(unit: &mut LanguageFragment<CppImport>, path: &'static str) {
    unit.require_import(cpp_system_group(), CppImport { path, system: true });
}

fn require_cpp_local(unit: &mut LanguageFragment<CppImport>, path: &'static str) {
    unit.require_import(
        cpp_local_group(),
        CppImport {
            path,
            system: false,
        },
    );
}

fn cpp_runtime_file() -> LanguageSourceFile<CppImport> {
    let mut file = LanguageSourceFile::new("src/runtime.hpp", FileRole::Runtime);
    file.set_preamble(LanguageFragment::new(CodeDocument::raw_text(RawText::new(
        "#pragma once\n// Dependency-free runtime copied into generated C++20 packages.",
    ))));
    let mut body = LanguageFragment::new(CodeDocument::raw_text(RawText::new(RUNTIME)));
    for header in [
        "algorithm",
        "any",
        "bit",
        "cmath",
        "cstddef",
        "cstdint",
        "functional",
        "limits",
        "map",
        "optional",
        "stdexcept",
        "string",
        "string_view",
        "type_traits",
        "utility",
        "variant",
        "vector",
    ] {
        require_cpp_system(&mut body, header);
    }
    file.set_body(body);
    file
}

fn cpp_generated_test_file() -> LanguageSourceFile<CppImport> {
    let mut file = LanguageSourceFile::new("tests/generated_test.cc", FileRole::Test);
    let mut body = LanguageFragment::new(CodeDocument::raw_text(RawText::new(
        "int main() { return polyrust_generated::run_portable_tests() ? 0 : 1; }",
    )));
    require_cpp_local(&mut body, "generated.hpp");
    file.set_body(body);
    file
}

fn cpp_conformance_file() -> LanguageSourceFile<CppImport> {
    let mut file = LanguageSourceFile::new("tests/conformance_test.cc", FileRole::Conformance);
    let mut body = LanguageFragment::new(CodeDocument::raw_text(RawText::new(CONFORMANCE_BODY)));
    require_cpp_system(&mut body, "cstdint");
    require_cpp_system(&mut body, "limits");
    require_cpp_local(&mut body, "generated.hpp");
    require_cpp_local(&mut body, "runtime.hpp");
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

    fn header_file(&self) -> LanguageSourceFile<CppImport> {
        let mut file = LanguageSourceFile::new("src/generated.hpp", FileRole::Source);
        file.set_preamble(LanguageFragment::new(CodeDocument::raw_text(RawText::new(
            "#pragma once\n// Generated by PolyRust from checked IR v0.",
        ))));
        let mut body = LanguageFragment::new(CodeDocument::empty());
        for header in ["cstdint", "optional", "string"] {
            require_cpp_system(&mut body, header);
        }
        if self
            .program
            .module()
            .declarations
            .iter()
            .any(|declaration| matches!(declaration, Declaration::Enum(_)))
        {
            require_cpp_system(&mut body, "variant");
        }
        if self
            .program
            .capabilities()
            .program()
            .iter()
            .any(|capability| matches!(capability, Capability::Bytes | Capability::ImmutableList))
        {
            require_cpp_system(&mut body, "vector");
        }
        let mut output = String::from(
            "namespace poly_runtime { struct aggregate; }\n\n\
             namespace polyrust_generated {\n\
             struct poly_error { std::string code; std::string message; };\n\
             template <typename T> struct poly_result { bool ok; std::optional<T> value; std::optional<poly_error> error; };\n\
             template <typename T, typename E> struct value_result { bool is_ok; std::optional<T> value; std::optional<E> error; };\n\n",
        );
        for declaration in &self.program.module().declarations {
            match declaration {
                Declaration::Record(item) => output.push_str(&format!(
                    "{}struct {};\n",
                    visibility(item.header.visibility),
                    type_name(&item.header.name)
                )),
                Declaration::Enum(item) => {
                    for variant in &item.variants {
                        output.push_str(&format!(
                            "{}struct {}{};\n",
                            visibility(item.header.visibility),
                            type_name(&item.header.name),
                            type_name(&variant.header.name)
                        ));
                    }
                }
                Declaration::Contract(item) => output.push_str(&format!(
                    "{}struct {};\n",
                    visibility(item.header.visibility),
                    type_name(&item.header.name)
                )),
                _ => {}
            }
        }
        output.push('\n');
        for declaration in &self.program.module().declarations {
            if let Declaration::Contract(item) = declaration {
                output.push_str(&format!(
                    "{}struct {} {{\n  virtual ~{}() = default;\n  virtual std::int64_t polyrust_declaration() const noexcept = 0;\n",
                    visibility(item.header.visibility),
                    type_name(&item.header.name),
                    type_name(&item.header.name)
                ));
                output.push_str("  virtual poly_runtime::aggregate polyrust_value() const = 0;\n");
                for method in &item.methods {
                    output.push_str(&format!(
                        "  virtual poly_result<{}> {}({}) const = 0;\n",
                        self.ty(&method.return_type),
                        value_name(&method.header.name),
                        self.parameters(&method.parameters)
                    ));
                }
                output.push_str("};\n\n");
            }
        }
        for declaration in &self.program.module().declarations {
            if let Declaration::Record(item) = declaration {
                let implementations = self.implementations(item.header.node.id);
                let bases = implementations
                    .iter()
                    .map(|implementation| {
                        format!("public {}", type_name(self.name(implementation.contract)))
                    })
                    .collect::<Vec<_>>();
                output.push_str(&format!(
                    "{}struct {}{} {{\n",
                    visibility(item.header.visibility),
                    type_name(&item.header.name),
                    if bases.is_empty() {
                        String::new()
                    } else {
                        format!(" : {}", bases.join(", "))
                    }
                ));
                for field in &item.fields {
                    output.push_str(&format!(
                        "  {} {};\n",
                        self.ty(&field.ty),
                        value_name(&field.header.name)
                    ));
                }
                if item.fields.is_empty() {
                    output.push_str(&format!(
                        "  {}() = default;\n",
                        type_name(&item.header.name)
                    ));
                } else {
                    output.push_str(&format!(
                        "  {}({}) : {} {{}}\n",
                        type_name(&item.header.name),
                        self.parameters(
                            &item
                                .fields
                                .iter()
                                .map(|field| portable_ir::v0::Parameter {
                                    header: portable_ir::v0::MemberHeader {
                                        node: field.header.node.clone(),
                                        name: format!("{}_value", value_name(&field.header.name)),
                                        documentation: Vec::new(),
                                    },
                                    ty: field.ty.clone(),
                                })
                                .collect::<Vec<_>>()
                        ),
                        item.fields
                            .iter()
                            .map(|field| format!(
                                "{}({}_value)",
                                value_name(&field.header.name),
                                value_name(&field.header.name)
                            ))
                            .collect::<Vec<_>>()
                            .join(", ")
                    ));
                }
                if !implementations.is_empty() {
                    output.push_str(&format!(
                        "  std::int64_t polyrust_declaration() const noexcept override {{ return {}; }}\n",
                        item.header.node.id.0
                    ));
                }
                output.push_str("  poly_runtime::aggregate polyrust_value() const");
                if !implementations.is_empty() {
                    output.push_str(" override");
                }
                output.push_str(";\n");
                for implementation in implementations {
                    for method in &implementation.methods {
                        output.push_str(&format!(
                            "  poly_result<{}> {}({}) const override;\n",
                            self.ty(&method.return_type),
                            value_name(&method.header.name),
                            self.parameters(&method.parameters)
                        ));
                    }
                }
                if item.fields.is_empty() {
                    output.push_str("  bool operator==(const ");
                    output.push_str(&type_name(&item.header.name));
                    output.push_str("&) const { return true; }\n");
                } else {
                    output.push_str("  bool operator==(const ");
                    output.push_str(&type_name(&item.header.name));
                    output.push_str("& other) const { return ");
                    output.push_str(
                        &item
                            .fields
                            .iter()
                            .map(|field| {
                                let name = value_name(&field.header.name);
                                format!("{name} == other.{name}")
                            })
                            .collect::<Vec<_>>()
                            .join(" && "),
                    );
                    output.push_str("; }\n");
                }
                output.push_str("};\n\n");
            }
        }
        for declaration in &self.program.module().declarations {
            if let Declaration::Enum(item) = declaration {
                let mut variants = Vec::new();
                for variant in &item.variants {
                    let name = format!(
                        "{}{}",
                        type_name(&item.header.name),
                        type_name(&variant.header.name)
                    );
                    variants.push(name.clone());
                    output.push_str(&format!(
                        "{}struct {name} {{\n",
                        visibility(item.header.visibility)
                    ));
                    for field in &variant.fields {
                        output.push_str(&format!(
                            "  {} {};\n",
                            self.ty(&field.ty),
                            value_name(&field.header.name)
                        ));
                    }
                    output.push_str(&format!(
                        "  bool operator==(const {name}&) const = default;\n}};\n"
                    ));
                }
                output.push_str(&format!(
                    "{}using {} = std::variant<{}>;\n\n",
                    visibility(item.header.visibility),
                    type_name(&item.header.name),
                    variants.join(", ")
                ));
            }
        }
        for declaration in &self.program.module().declarations {
            match declaration {
                Declaration::Constant(item) => output.push_str(&format!(
                    "{}poly_result<{}> {}();\n",
                    visibility(item.header.visibility),
                    self.ty(&item.ty),
                    value_name(&item.header.name)
                )),
                Declaration::Function(item) => output.push_str(&format!(
                    "{}poly_result<{}> {}({});\n",
                    visibility(item.header.visibility),
                    self.ty(&item.return_type),
                    value_name(&item.header.name),
                    self.parameters(&item.parameters)
                )),
                _ => {}
            }
        }
        output.push_str("bool run_portable_tests();\n}\n");
        body = body.map_document(|_| CodeDocument::raw_text(RawText::new(output)));
        file.set_body(body);
        file
    }

    fn source_file(&self) -> Result<LanguageSourceFile<CppImport>, BackendError> {
        let mut document = serde_json::to_value(self.program.document()).map_err(|error| {
            BackendError::Generation {
                message: format!("cannot serialize checked IR: {error}"),
            }
        })?;
        stringify_wide_numbers(&mut document);
        let document = serde_json::to_string(&document).expect("checked document serializes");
        let mut file = LanguageSourceFile::new("src/generated.cc", FileRole::Source);
        file.set_preamble(LanguageFragment::new(CodeDocument::raw_text(RawText::new(
            "// Generated by PolyRust from checked IR v0.",
        ))));
        let mut body = LanguageFragment::new(CodeDocument::empty());
        require_cpp_local(&mut body, "generated.hpp");
        require_cpp_local(&mut body, "runtime.hpp");
        if document.len() > CPP_LITERAL_CHUNK_BYTES {
            require_cpp_system(&mut body, "string");
        }
        let mut output = String::new();
        output.push_str(&self.conversions());
        output.push_str(
            "\nnamespace polyrust_generated {\nnamespace { poly_runtime::runtime runtime_instance(",
        );
        output.push_str(&cpp_string_expression(&document));
        output.push_str("); }\n\n");
        for declaration in &self.program.module().declarations {
            match declaration {
                Declaration::Constant(item) => output.push_str(&format!(
                    "poly_result<{}> {}() {{ return poly_runtime::convert_result<{}>(runtime_instance.read_constant({})); }}\n",
                    self.ty(&item.ty),
                    value_name(&item.header.name),
                    self.ty(&item.ty),
                    item.header.node.id.0
                )),
                Declaration::Function(item) => output.push_str(&format!(
                    "poly_result<{}> {}({}) {{ return poly_runtime::convert_result<{}>(runtime_instance.invoke({}, {{{}}})); }}\n",
                    self.ty(&item.return_type),
                    value_name(&item.header.name),
                    self.parameters(&item.parameters),
                    self.ty(&item.return_type),
                    item.header.node.id.0,
                    item.parameters
                        .iter()
                        .map(|parameter| {
                            self.argument(&parameter.ty, &value_name(&parameter.header.name))
                        })
                        .collect::<Vec<_>>()
                        .join(", ")
                )),
                Declaration::Implementation(item) => {
                    let record = type_name(self.name(item.record));
                    for method in &item.methods {
                        output.push_str(&format!(
                            "poly_result<{}> {}::{}({}) const {{ return poly_runtime::convert_result<{}>(runtime_instance.invoke_method({}, {}, poly_runtime::to_any(*this), {{{}}})); }}\n",
                            self.ty(&method.return_type),
                            record,
                            value_name(&method.header.name),
                            self.parameters(&method.parameters),
                            self.ty(&method.return_type),
                            item.header.node.id.0,
                            method.header.node.id.0,
                            method.parameters.iter().map(|parameter| self.argument(&parameter.ty, &value_name(&parameter.header.name))).collect::<Vec<_>>().join(", ")
                        ));
                    }
                }
                _ => {}
            }
        }
        output
            .push_str("\nbool run_portable_tests() { return runtime_instance.run_tests(); }\n}\n");
        body = body.map_document(|_| CodeDocument::raw_text(RawText::new(output)));
        file.set_body(body);
        Ok(file)
    }

    fn argument(&self, ty: &TypeRef, name: &str) -> String {
        if matches!(ty, TypeRef::Contract(_)) {
            format!("{name}.polyrust_value()")
        } else {
            format!("poly_runtime::to_any({name})")
        }
    }

    fn conversions(&self) -> String {
        let records = self
            .program
            .module()
            .declarations
            .iter()
            .filter_map(|declaration| match declaration {
                Declaration::Record(item) => Some(item),
                _ => None,
            })
            .collect::<Vec<_>>();
        let enums = self
            .program
            .module()
            .declarations
            .iter()
            .filter_map(|declaration| match declaration {
                Declaration::Enum(item) => Some(item),
                _ => None,
            })
            .collect::<Vec<_>>();
        let mut output = String::from("namespace poly_runtime {\n");
        for record in &records {
            let name = type_name(&record.header.name);
            output.push_str(&format!(
                "template <> any to_any<polyrust_generated::{name}>(const polyrust_generated::{name}& value);\n\
                 template <> polyrust_generated::{name} from_any<polyrust_generated::{name}>(const any& value);\n"
            ));
        }
        for enumeration in &enums {
            let enum_name = type_name(&enumeration.header.name);
            for variant in &enumeration.variants {
                let name = format!("{enum_name}{}", type_name(&variant.header.name));
                output.push_str(&format!(
                    "template <> any to_any<polyrust_generated::{name}>(const polyrust_generated::{name}& value);\n"
                ));
            }
            output.push_str(&format!(
                "template <> any to_any<polyrust_generated::{enum_name}>(const polyrust_generated::{enum_name}& value);\n\
                 template <> polyrust_generated::{enum_name} from_any<polyrust_generated::{enum_name}>(const any& value);\n"
            ));
        }
        output.push_str("}\n\nnamespace polyrust_generated {\n");
        for record in &records {
            let name = type_name(&record.header.name);
            output.push_str(&format!(
                "poly_runtime::aggregate {name}::polyrust_value() const {{\n  return {{{}, \"\", {{{}}}}};\n}}\n",
                record.header.node.id.0,
                record
                    .fields
                    .iter()
                    .map(|field| format!(
                        "{{{}, poly_runtime::to_any({})}}",
                        cpp_string(&field.header.name),
                        value_name(&field.header.name)
                    ))
                    .collect::<Vec<_>>()
                    .join(", ")
            ));
        }
        output.push_str("}\n\nnamespace poly_runtime {\n");
        for record in &records {
            let name = type_name(&record.header.name);
            output.push_str(&format!(
                "template <> any to_any<polyrust_generated::{name}>(const polyrust_generated::{name}& value) {{ return value.polyrust_value(); }}\n\
                 template <> polyrust_generated::{name} from_any<polyrust_generated::{name}>(const any& value) {{\n\
                   const auto& item = std::any_cast<const aggregate&>(value);\n\
                   return {{{}}};\n}}\n",
                record
                    .fields
                    .iter()
                    .map(|field| format!(
                        "from_any<{}>(item.fields.at({}))",
                        self.ty(&field.ty),
                        cpp_string(&field.header.name)
                    ))
                    .collect::<Vec<_>>()
                    .join(", ")
            ));
        }
        for enumeration in &enums {
            let enum_name = type_name(&enumeration.header.name);
            for variant in &enumeration.variants {
                let variant_name = type_name(&variant.header.name);
                let name = format!("{enum_name}{variant_name}");
                output.push_str(&format!(
                    "template <> any to_any<polyrust_generated::{name}>(const polyrust_generated::{name}& value) {{\n\
                       return aggregate{{{}, {}, {{{}}}}};\n}}\n",
                    enumeration.header.node.id.0,
                    cpp_string(&variant.header.name),
                    variant
                        .fields
                        .iter()
                        .map(|field| format!(
                            "{{{}, to_any(value.{})}}",
                            cpp_string(&field.header.name),
                            value_name(&field.header.name)
                        ))
                        .collect::<Vec<_>>()
                        .join(", ")
                ));
            }
            output.push_str(&format!(
                "template <> any to_any<polyrust_generated::{enum_name}>(const polyrust_generated::{enum_name}& value) {{\n\
                   return std::visit([](const auto& item) -> any {{ return to_any(item); }}, value);\n}}\n\
                 template <> polyrust_generated::{enum_name} from_any<polyrust_generated::{enum_name}>(const any& value) {{\n\
                   const auto& item = std::any_cast<const aggregate&>(value);\n"
            ));
            for variant in &enumeration.variants {
                let name = format!("{enum_name}{}", type_name(&variant.header.name));
                output.push_str(&format!(
                    "  if (item.tag == {}) return polyrust_generated::{name}{{{}}};\n",
                    cpp_string(&variant.header.name),
                    variant
                        .fields
                        .iter()
                        .map(|field| format!(
                            "from_any<{}>(item.fields.at({}))",
                            self.ty(&field.ty),
                            cpp_string(&field.header.name)
                        ))
                        .collect::<Vec<_>>()
                        .join(", ")
                ));
            }
            output.push_str("  throw std::runtime_error(\"unknown generated enum tag\");\n}\n");
        }
        output.push_str("}\n");
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

    fn parameters(&self, parameters: &[portable_ir::v0::Parameter]) -> String {
        parameters
            .iter()
            .map(|parameter| {
                format!(
                    "{} {}",
                    parameter_type(&self.ty(&parameter.ty)),
                    value_name(&parameter.header.name)
                )
            })
            .collect::<Vec<_>>()
            .join(", ")
    }

    fn ty(&self, ty: &TypeRef) -> String {
        match ty {
            TypeRef::Unit => "std::monostate".into(),
            TypeRef::Bool => "bool".into(),
            TypeRef::I32 => "std::int32_t".into(),
            TypeRef::I64 => "std::int64_t".into(),
            TypeRef::F64 => "double".into(),
            TypeRef::Char => "char32_t".into(),
            TypeRef::String => "std::string".into(),
            TypeRef::Bytes => "std::vector<std::uint8_t>".into(),
            TypeRef::List(inner) => format!("std::vector<{}>", self.ty(inner)),
            TypeRef::Option(inner) => format!("std::optional<{}>", self.ty(inner)),
            TypeRef::Result { ok, error } => {
                format!("value_result<{}, {}>", self.ty(ok), self.ty(error))
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
            .unwrap_or_else(|| "std::monostate".into())
    }

    fn name(&self, id: NodeId) -> &str {
        self.names.get(&id).map(String::as_str).unwrap_or("unknown")
    }
}

fn parameter_type(ty: &str) -> String {
    if matches!(
        ty,
        "bool" | "std::int32_t" | "std::int64_t" | "double" | "char32_t"
    ) {
        ty.into()
    } else {
        format!("const {ty}&")
    }
}

fn visibility(_visibility: Visibility) -> &'static str {
    ""
}

fn type_name(name: &str) -> String {
    identifier(name)
}

fn value_name(name: &str) -> String {
    identifier(name)
}

fn identifier(name: &str) -> String {
    const KEYWORDS: &[&str] = &[
        "alignas",
        "alignof",
        "and",
        "and_eq",
        "asm",
        "auto",
        "bitand",
        "bitor",
        "bool",
        "break",
        "case",
        "catch",
        "char",
        "class",
        "compl",
        "concept",
        "const",
        "consteval",
        "constexpr",
        "constinit",
        "const_cast",
        "continue",
        "co_await",
        "co_return",
        "co_yield",
        "decltype",
        "default",
        "delete",
        "do",
        "double",
        "dynamic_cast",
        "else",
        "enum",
        "explicit",
        "export",
        "extern",
        "false",
        "float",
        "for",
        "friend",
        "goto",
        "if",
        "inline",
        "int",
        "long",
        "mutable",
        "namespace",
        "new",
        "noexcept",
        "not",
        "not_eq",
        "nullptr",
        "operator",
        "or",
        "or_eq",
        "private",
        "protected",
        "public",
        "register",
        "reinterpret_cast",
        "requires",
        "return",
        "short",
        "signed",
        "sizeof",
        "static",
        "static_assert",
        "static_cast",
        "struct",
        "switch",
        "template",
        "this",
        "thread_local",
        "throw",
        "true",
        "try",
        "typedef",
        "typeid",
        "typename",
        "union",
        "unsigned",
        "using",
        "virtual",
        "void",
        "volatile",
        "wchar_t",
        "while",
        "xor",
        "xor_eq",
    ];
    if KEYWORDS.contains(&name) {
        format!("{name}_")
    } else {
        name.into()
    }
}

fn cpp_string(value: &str) -> String {
    serde_json::to_string(value).expect("C++ string serializes")
}

const CPP_LITERAL_CHUNK_BYTES: usize = 8 * 1024;

fn cpp_string_expression(value: &str) -> String {
    if value.len() <= CPP_LITERAL_CHUNK_BYTES {
        return cpp_string(value);
    }
    let mut chunks = Vec::new();
    let mut start = 0;
    while start < value.len() {
        let mut end = (start + CPP_LITERAL_CHUNK_BYTES).min(value.len());
        while !value.is_char_boundary(end) {
            end -= 1;
        }
        chunks.push(cpp_string(&value[start..end]));
        start = end;
    }
    let mut expression = format!("std::string({})", chunks[0]);
    for chunk in &chunks[1..] {
        expression.push_str(" + ");
        expression.push_str(chunk);
    }
    expression
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

const README: &str = "# Generated PolyRust C++20 package\n\nDependency-free C++20 source with value semantics and explicit portable results.\n";
const CONFORMANCE_BODY: &str = "int main() {\n  using poly_runtime::checked_i32;\n  return checked_i32(0).ok && checked_i32(INT32_MAX).ok && checked_i32(INT32_MIN).ok && !checked_i32(INT64_C(2147483648)).ok ? 0 : 1;\n}\n";

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn descriptor_and_manifest_are_deterministic() {
        let checked = fixture();
        assert_eq!(CppBackend.descriptor().target.as_str(), "org.polyrust.cpp");
        let first = CppBackend
            .generate(&checked, &BackendOptions::default())
            .unwrap();
        let second = CppBackend
            .generate(&checked, &BackendOptions::default())
            .unwrap();
        assert_eq!(first.canonical_json(), second.canonical_json());
        assert!(first.dependencies().is_empty());
    }

    #[test]
    fn large_embedded_documents_are_runtime_joined_below_cpp_literal_limits() {
        let expression = cpp_string_expression(&"x".repeat(100_000));
        assert!(expression.starts_with("std::string(\""));
        assert!(expression.ends_with('"'));
        assert!(expression.matches(" + \"").count() >= 12);
        assert!(!expression.contains(&"x".repeat(65_536)));
    }

    #[test]
    fn includes_are_owned_by_the_cpp_file_that_uses_them() {
        let manifest = CppBackend
            .generate(&fixture(), &BackendOptions::default())
            .unwrap();
        let runtime = generated_text(&manifest, "src/runtime.hpp");
        assert_eq!(runtime.matches("#include <variant>").count(), 1);
        assert_eq!(runtime.matches("#include <vector>").count(), 1);
        let header = generated_text(&manifest, "src/generated.hpp");
        assert_eq!(header.matches("#include <cstdint>").count(), 1);
        assert!(!header.contains("#include <variant>"));
        assert!(!header.contains("#include <vector>"));
        let source = generated_text(&manifest, "src/generated.cc");
        assert_eq!(source.matches("#include \"generated.hpp\"").count(), 1);
        assert_eq!(source.matches("#include \"runtime.hpp\"").count(), 1);
        let test = generated_text(&manifest, "tests/generated_test.cc");
        assert_eq!(test.matches("#include \"generated.hpp\"").count(), 1);
        assert!(!test.contains("runtime.hpp"));
    }

    fn generated_text<'a>(manifest: &'a OutputManifest, path: &str) -> &'a str {
        match manifest.file(path).unwrap().contents() {
            portable_codegen::OutputContents::Text(text) => text,
            portable_codegen::OutputContents::Bytes(_) => panic!("C++ source must be text"),
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
}
