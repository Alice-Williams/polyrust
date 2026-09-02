//! Dependency-free C++20 generation from checked portable IR v0.

#![forbid(unsafe_code)]

use std::collections::{BTreeMap, BTreeSet};

use portable_check::v0::{Capability, CheckedProgram};
use portable_codegen::{
    Backend, BackendDescriptor, BackendError, BackendOptions, BackendVersion, CapabilitySupport,
    DeclaredDependency, Document as CodeDocument, FileGroup, FileGroupId, ImportGroup, ImportSet,
    InjectedHelper, IrVersionRange, LanguageFile, LanguageFragment, LanguagePackage,
    LanguagePlugin, LanguageRenderer, LanguageSourceFile, OptionsSchema, OutputManifest, RawText,
    RuntimeHelper, RuntimeHelperGraph, SourceFileRole, TargetId, TextFileRole,
    generate_with_plugin, validate_backend_capability,
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
            | Capability::InterfaceDispatch
            | Capability::F64
            | Capability::Option
            | Capability::Result
            | Capability::WrappingIntegerArithmetic
            | Capability::BoundedIteration => CapabilitySupport::Native,
            Capability::FirstClassInterfaceValues => CapabilitySupport::Unsupported {
                reason: "first-class interface values require the M34A-17 typed C++ backend".into(),
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
#[doc(hidden)]
pub struct CppImport {
    kind: CppImportKind,
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
enum CppImportKind {
    System { path: String },
    Local { path: String },
}

impl CppImport {
    pub fn system(path: &str) -> Result<Self, String> {
        validate_cpp_include(path, false)?;
        Ok(Self {
            kind: CppImportKind::System {
                path: path.to_owned(),
            },
        })
    }

    pub fn local(path: &str) -> Result<Self, String> {
        validate_cpp_include(path, true)?;
        Ok(Self {
            kind: CppImportKind::Local {
                path: path.to_owned(),
            },
        })
    }
}

fn validate_cpp_include(path: &str, local: bool) -> Result<(), String> {
    let valid_character = |character: char| {
        character.is_ascii_alphanumeric() || matches!(character, '_' | '-' | '.' | '/' | '+')
    };
    let valid = !path.is_empty()
        && !path.starts_with('/')
        && !path.ends_with('/')
        && !path.contains("//")
        && !path.split('/').any(|segment| matches!(segment, "." | ".."))
        && path.chars().all(valid_character)
        && (!local || path.ends_with(".h") || path.ends_with(".hpp"));
    if valid {
        Ok(())
    } else {
        Err(format!("invalid C++ include path {path:?}"))
    }
}

#[doc(hidden)]
pub struct CppRenderer;

impl LanguageRenderer<CppImport> for CppRenderer {
    fn render_imports(&self, imports: &ImportSet<CppImport>) -> Result<CodeDocument, String> {
        let lines = imports
            .groups()
            .flat_map(|(_, imports)| imports.iter())
            .map(|import| match &import.kind {
                CppImportKind::System { path } => format!("#include <{path}>"),
                CppImportKind::Local { path } => format!("#include {path:?}"),
            })
            .collect::<Vec<_>>();
        Ok(CodeDocument::raw_text(RawText::new(lines.join("\n"))))
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
struct CppCode {
    text: String,
    imports: BTreeSet<(ImportGroup, CppImport)>,
    helper_roots: BTreeSet<String>,
}

impl CppCode {
    fn new(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            ..Self::default()
        }
    }

    fn with_system(mut self, path: &str) -> Self {
        self.imports.insert((
            cpp_system_group(),
            CppImport::system(path).expect("static C++ system include is valid"),
        ));
        self
    }

    fn with_local(mut self, path: &str) -> Self {
        self.imports.insert((
            cpp_local_group(),
            CppImport::local(path).expect("static C++ local include is valid"),
        ));
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

    fn into_fragment(self) -> LanguageFragment<CppImport> {
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

impl std::fmt::Display for CppCode {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(&self.text)
    }
}

impl LanguagePlugin for CppBackend {
    type Import = CppImport;
    type Renderer = CppRenderer;

    fn translate(
        &self,
        program: &CheckedProgram,
        options: &BackendOptions,
    ) -> Result<LanguagePackage<Self::Import>, BackendError> {
        let _ = options;
        validate_backend_capability(self, program, Capability::FirstClassInterfaceValues)?;
        let generator = Generator::new(program);
        let (source, runtime_roots) = generator.source_file()?;
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
                        TextFileRole::Documentation,
                        README,
                    )],
                )
                .map_err(cpp_generation_error)?,
                FileGroup::new(
                    cpp_group("runtime")?,
                    vec![LanguageFile::source(cpp_runtime_file(&runtime_roots)?)],
                )
                .map_err(cpp_generation_error)?,
                FileGroup::new(
                    cpp_group("source")?,
                    vec![
                        LanguageFile::source(generator.header_file()),
                        LanguageFile::source(source),
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

fn cpp_runtime_file(
    roots: &BTreeSet<String>,
) -> Result<LanguageSourceFile<CppImport>, BackendError> {
    let graph = cpp_runtime_helper_graph()?;
    let mut file = LanguageSourceFile::new("src/runtime.hpp", SourceFileRole::Runtime);
    file.set_preamble(
        CppCode::new(
            "#pragma once\n// Dependency-free runtime copied into generated C++20 packages.",
        )
        .into_fragment(),
    );
    file.set_body(
        graph
            .resolve(roots.iter().cloned())
            .map_err(cpp_generation_error)?,
    );
    Ok(file)
}

fn cpp_runtime_helper_graph() -> Result<RuntimeHelperGraph<CppImport>, BackendError> {
    const BEGIN: &str = "// POLYRUST-BEGIN ";
    const END: &str = "// POLYRUST-END ";

    let mut helpers = Vec::new();
    let mut active: Option<String> = None;
    let mut source = String::new();
    let mut order = 0_u16;
    for line in RUNTIME.split_inclusive('\n') {
        let marker = line.trim().trim_end_matches('\r');
        if let Some(id) = marker.strip_prefix(BEGIN) {
            if active.is_some() || !source.trim().is_empty() {
                return Err(cpp_generation_error(format!(
                    "invalid nested or unowned C++ runtime helper marker {id:?}"
                )));
            }
            active = Some(id.to_owned());
        } else if let Some(id) = marker.strip_prefix(END) {
            let Some(open) = active.take() else {
                return Err(cpp_generation_error(format!(
                    "unmatched C++ runtime helper end marker {id:?}"
                )));
            };
            if open != id || source.trim().is_empty() {
                return Err(cpp_generation_error(format!(
                    "invalid C++ runtime helper marker {open:?} closed by {id:?}"
                )));
            }
            helpers.push(RuntimeHelper::new(
                open.clone(),
                order,
                cpp_runtime_section(&open, std::mem::take(&mut source))?.into_fragment(),
            ));
            order = order
                .checked_add(1)
                .expect("C++ runtime helper order fits u16");
        } else if active.is_some() {
            source.push_str(line);
        } else if !marker.is_empty() {
            return Err(cpp_generation_error(
                "C++ runtime text lacks a helper owner",
            ));
        }
    }
    if let Some(open) = active {
        return Err(cpp_generation_error(format!(
            "unclosed C++ runtime helper marker {open:?}"
        )));
    }
    helpers.push(RuntimeHelper::new(
        "runtime.full",
        u16::MAX,
        CppCode::default()
            .with_helper_root("runtime.model")
            .with_helper_root("runtime.json")
            .with_helper_root("runtime.engine")
            .into_fragment(),
    ));
    RuntimeHelperGraph::new(helpers).map_err(cpp_generation_error)
}

fn cpp_runtime_section(id: &str, source: String) -> Result<CppCode, BackendError> {
    let code = CppCode::new(source);
    let code = match id {
        "runtime.model" => code
            .with_system("any")
            .with_system("cstdint")
            .with_system("map")
            .with_system("optional")
            .with_system("stdexcept")
            .with_system("string")
            .with_system("type_traits")
            .with_system("utility")
            .with_system("variant")
            .with_system("vector"),
        "runtime.json" => code
            .with_system("cstddef")
            .with_system("cstdint")
            .with_system("stdexcept")
            .with_system("string")
            .with_system("string_view")
            .with_system("utility"),
        "runtime.engine" => code
            .with_system("algorithm")
            .with_system("any")
            .with_system("bit")
            .with_system("cmath")
            .with_system("cstddef")
            .with_system("cstdint")
            .with_system("functional")
            .with_system("limits")
            .with_system("map")
            .with_system("optional")
            .with_system("stdexcept")
            .with_system("string")
            .with_system("string_view")
            .with_system("utility")
            .with_system("vector"),
        _ => {
            return Err(cpp_generation_error(format!(
                "unknown C++ runtime helper {id:?}"
            )));
        }
    };
    Ok(code)
}

fn cpp_generated_test_file() -> LanguageSourceFile<CppImport> {
    let mut file = LanguageSourceFile::new("tests/generated_test.cc", SourceFileRole::Test);
    file.set_body(
        CppCode::new("int main() { return polyrust_generated::run_portable_tests() ? 0 : 1; }")
            .with_local("generated.hpp")
            .into_fragment(),
    );
    file
}

fn cpp_conformance_file() -> LanguageSourceFile<CppImport> {
    let mut file =
        LanguageSourceFile::new("tests/conformance_test.cc", SourceFileRole::Conformance);
    file.set_body(
        CppCode::new(CONFORMANCE_BODY)
            .with_system("cstdint")
            .with_system("limits")
            .with_local("generated.hpp")
            .with_local("runtime.hpp")
            .into_fragment(),
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

    fn header_file(&self) -> LanguageSourceFile<CppImport> {
        let mut file = LanguageSourceFile::new("src/generated.hpp", SourceFileRole::Source);
        file.set_preamble(
            CppCode::new("#pragma once\n// Generated by PolyRust from checked IR v0.")
                .into_fragment(),
        );
        let declarations = &self.program.module().declarations;
        let body = CppCode::sequence([
            CppCode::new("namespace poly_runtime { struct aggregate; }\n\nnamespace polyrust_generated {\n"),
            CppCode::new(
                "struct poly_error { std::string code; std::string message; };\n\
                 template <typename T> struct poly_result { bool ok; std::optional<T> value; std::optional<poly_error> error; };\n\
                 template <typename T, typename E> struct value_result { bool is_ok; std::optional<T> value; std::optional<E> error; };\n\n",
            )
            .with_system("optional")
            .with_system("string"),
            CppCode::sequence(declarations.iter().map(|item| self.forward_declaration(item))),
            CppCode::new("\n"),
            CppCode::sequence(
                declarations
                    .iter()
                    .filter(|item| matches!(item, Declaration::Interface(_)))
                    .map(|item| self.type_declaration(item)),
            ),
            CppCode::sequence(
                declarations
                    .iter()
                    .filter(|item| matches!(item, Declaration::Record(_)))
                    .map(|item| self.type_declaration(item)),
            ),
            CppCode::sequence(
                declarations
                    .iter()
                    .filter(|item| matches!(item, Declaration::Enum(_)))
                    .map(|item| self.type_declaration(item)),
            ),
            CppCode::sequence(declarations.iter().map(|item| self.callable_declaration(item))),
            CppCode::new("bool run_portable_tests();\n}\n"),
        ]);
        file.set_body(body.into_fragment());
        file
    }

    fn forward_declaration(&self, declaration: &Declaration) -> CppCode {
        match declaration {
            Declaration::Record(item) => CppCode::new(format!(
                "{}struct {};\n",
                visibility(item.header.visibility),
                type_name(&item.header.name)
            )),
            Declaration::Interface(item) => CppCode::new(format!(
                "{}struct {};\n",
                visibility(item.header.visibility),
                type_name(&item.header.name)
            )),
            Declaration::Enum(item) => CppCode::sequence(item.variants.iter().map(|variant| {
                CppCode::new(format!(
                    "{}struct {}{};\n",
                    visibility(item.header.visibility),
                    type_name(&item.header.name),
                    type_name(&variant.header.name)
                ))
            })),
            _ => CppCode::default(),
        }
    }

    fn type_declaration(&self, declaration: &Declaration) -> CppCode {
        match declaration {
            Declaration::Interface(item) => {
                let mut output = format!(
                    "{}struct {} {{\n  virtual ~{}() = default;\n  virtual std::int64_t polyrust_declaration() const noexcept = 0;\n  virtual poly_runtime::aggregate polyrust_value() const = 0;\n",
                    visibility(item.header.visibility),
                    type_name(&item.header.name),
                    type_name(&item.header.name)
                );
                let mut dependencies = Vec::new();
                for method in &item.methods {
                    let return_type = self.ty(&method.return_type);
                    let parameters = self.parameters(&method.parameters);
                    output.push_str(&format!(
                        "  virtual poly_result<{return_type}> {}({parameters}) const = 0;\n",
                        value_name(&method.header.name)
                    ));
                    dependencies.extend([return_type, parameters]);
                }
                output.push_str("};\n\n");
                CppCode::new(output)
                    .with_system("cstdint")
                    .with_text_from(dependencies)
            }
            Declaration::Record(item) => {
                let implementations = self.implementations(item.header.node.id);
                let bases = implementations
                    .iter()
                    .map(|implementation| {
                        format!("public {}", type_name(self.name(implementation.interface)))
                    })
                    .collect::<Vec<_>>();
                let mut output = format!(
                    "{}struct {}{} {{\n",
                    visibility(item.header.visibility),
                    type_name(&item.header.name),
                    if bases.is_empty() {
                        String::new()
                    } else {
                        format!(" : {}", bases.join(", "))
                    }
                );
                let mut dependencies = Vec::new();
                for field in &item.fields {
                    let field_type = self.ty(&field.ty);
                    output.push_str(&format!(
                        "  {field_type} {};\n",
                        value_name(&field.header.name)
                    ));
                    dependencies.push(field_type);
                }
                if item.fields.is_empty() {
                    output.push_str(&format!(
                        "  {}() = default;\n",
                        type_name(&item.header.name)
                    ));
                } else {
                    let constructor_parameters = self.parameters(
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
                            .collect::<Vec<_>>(),
                    );
                    output.push_str(&format!(
                        "  {}({constructor_parameters}) : {} {{}}\n",
                        type_name(&item.header.name),
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
                    dependencies.push(constructor_parameters);
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
                for implementation in &implementations {
                    for method in &implementation.methods {
                        let return_type = self.ty(&method.return_type);
                        let parameters = self.parameters(&method.parameters);
                        output.push_str(&format!(
                            "  poly_result<{return_type}> {}({parameters}) const override;\n",
                            value_name(&method.header.name)
                        ));
                        dependencies.extend([return_type, parameters]);
                    }
                }
                output.push_str(&format!(
                    "  bool operator==(const {}&{} const {{ {} }}\n",
                    type_name(&item.header.name),
                    if item.fields.is_empty() {
                        ")"
                    } else {
                        " other)"
                    },
                    if item.fields.is_empty() {
                        "return true;".to_owned()
                    } else {
                        format!(
                            "return {};",
                            item.fields
                                .iter()
                                .map(|field| {
                                    let name = value_name(&field.header.name);
                                    format!("{name} == other.{name}")
                                })
                                .collect::<Vec<_>>()
                                .join(" && ")
                        )
                    }
                ));
                output.push_str("};\n\n");
                let code = CppCode::new(output).with_text_from(dependencies);
                if implementations.is_empty() {
                    code
                } else {
                    code.with_system("cstdint")
                }
            }
            Declaration::Enum(item) => {
                let mut output = String::new();
                let mut variants = Vec::new();
                let mut dependencies = Vec::new();
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
                        let field_type = self.ty(&field.ty);
                        output.push_str(&format!(
                            "  {field_type} {};\n",
                            value_name(&field.header.name)
                        ));
                        dependencies.push(field_type);
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
                CppCode::new(output)
                    .with_system("variant")
                    .with_text_from(dependencies)
            }
            _ => CppCode::default(),
        }
    }

    fn callable_declaration(&self, declaration: &Declaration) -> CppCode {
        match declaration {
            Declaration::Constant(item) => {
                let ty = self.ty(&item.ty);
                CppCode::new(format!(
                    "{}poly_result<{ty}> {}();\n",
                    visibility(item.header.visibility),
                    value_name(&item.header.name)
                ))
                .with_text_from([ty])
            }
            Declaration::Function(item) => {
                let return_type = self.ty(&item.return_type);
                let parameters = self.parameters(&item.parameters);
                CppCode::new(format!(
                    "{}poly_result<{return_type}> {}({parameters});\n",
                    visibility(item.header.visibility),
                    value_name(&item.header.name)
                ))
                .with_text_from([return_type, parameters])
            }
            _ => CppCode::default(),
        }
    }

    fn source_file(
        &self,
    ) -> Result<(LanguageSourceFile<CppImport>, BTreeSet<String>), BackendError> {
        let mut document = serde_json::to_value(self.program.document()).map_err(|error| {
            BackendError::Generation {
                message: format!("cannot serialize checked IR: {error}"),
            }
        })?;
        stringify_wide_numbers(&mut document);
        let document = serde_json::to_string(&document).expect("checked document serializes");
        let mut file = LanguageSourceFile::new("src/generated.cc", SourceFileRole::Source);
        file.set_preamble(
            CppCode::new("// Generated by PolyRust from checked IR v0.").into_fragment(),
        );
        let embedded_document = cpp_string_expression(&document);
        let runtime = CppCode::new(format!(
            "\nnamespace polyrust_generated {{\nnamespace {{ poly_runtime::runtime runtime_instance({embedded_document}); }}\n\n"
        ))
        .with_local("generated.hpp")
        .with_local("runtime.hpp")
        .with_helper_root("runtime.full")
        .with_text_from([embedded_document]);
        let body = CppCode::sequence([
            self.conversions(),
            runtime,
            CppCode::sequence(
                self.program
                    .module()
                    .declarations
                    .iter()
                    .map(|declaration| self.definition(declaration)),
            ),
            CppCode::new(
                "\nbool run_portable_tests() { return runtime_instance.run_tests(); }\n}\n",
            ),
        ]);
        let roots = body.helper_roots.clone();
        file.set_body(body.into_fragment());
        Ok((file, roots))
    }

    fn definition(&self, declaration: &Declaration) -> CppCode {
        match declaration {
            Declaration::Constant(item) => {
                let ty = self.ty(&item.ty);
                CppCode::new(format!(
                    "poly_result<{ty}> {}() {{ return poly_runtime::convert_result<{ty}>(runtime_instance.read_constant({})); }}\n",
                    value_name(&item.header.name),
                    item.header.node.id.0
                ))
                .with_text_from([ty])
            }
            Declaration::Function(item) => {
                let return_type = self.ty(&item.return_type);
                let parameters = self.parameters(&item.parameters);
                let arguments = CppCode::joined(
                    item.parameters.iter().map(|parameter| {
                        self.argument(&parameter.ty, &value_name(&parameter.header.name))
                    }),
                    ", ",
                );
                CppCode::new(format!(
                    "poly_result<{return_type}> {}({parameters}) {{ return poly_runtime::convert_result<{return_type}>(runtime_instance.invoke({}, {{{arguments}}})); }}\n",
                    value_name(&item.header.name),
                    item.header.node.id.0
                ))
                .with_text_from([return_type, parameters, arguments])
            }
            Declaration::Implementation(item) => {
                let record = type_name(self.name(item.record));
                CppCode::sequence(item.methods.iter().map(|method| {
                    let return_type = self.ty(&method.return_type);
                    let parameters = self.parameters(&method.parameters);
                    let arguments = CppCode::joined(
                        method.parameters.iter().map(|parameter| {
                            self.argument(&parameter.ty, &value_name(&parameter.header.name))
                        }),
                        ", ",
                    );
                    CppCode::new(format!(
                        "poly_result<{return_type}> {record}::{}({parameters}) const {{ return poly_runtime::convert_result<{return_type}>(runtime_instance.invoke_method({}, {}, poly_runtime::to_any(*this), {{{arguments}}})); }}\n",
                        value_name(&method.header.name),
                        item.header.node.id.0,
                        method.header.node.id.0
                    ))
                    .with_text_from([return_type, parameters, arguments])
                }))
            }
            _ => CppCode::default(),
        }
    }

    fn argument(&self, ty: &TypeRef, name: &str) -> CppCode {
        if matches!(ty, TypeRef::Interface(_)) {
            CppCode::new(format!("{name}.polyrust_value()"))
        } else {
            CppCode::new(format!("poly_runtime::to_any({name})"))
        }
    }

    fn conversions(&self) -> CppCode {
        let declarations = &self.program.module().declarations;
        CppCode::sequence([
            CppCode::new("namespace poly_runtime {\n"),
            CppCode::sequence(
                declarations
                    .iter()
                    .map(|declaration| self.conversion_forward(declaration)),
            ),
            CppCode::new("}\n\nnamespace polyrust_generated {\n"),
            CppCode::sequence(
                declarations
                    .iter()
                    .map(|declaration| self.value_bridge_definition(declaration)),
            ),
            CppCode::new("}\n\nnamespace poly_runtime {\n"),
            CppCode::sequence(
                declarations
                    .iter()
                    .map(|declaration| self.conversion_definition(declaration)),
            ),
            CppCode::new("}\n"),
        ])
    }

    fn conversion_forward(&self, declaration: &Declaration) -> CppCode {
        match declaration {
            Declaration::Record(item) => {
                let name = type_name(&item.header.name);
                CppCode::new(format!(
                    "template <> any to_any<polyrust_generated::{name}>(const polyrust_generated::{name}& value);\n\
                     template <> polyrust_generated::{name} from_any<polyrust_generated::{name}>(const any& value);\n"
                ))
            }
            Declaration::Enum(item) => {
                let enum_name = type_name(&item.header.name);
                let mut output = String::new();
                for variant in &item.variants {
                    let name = format!("{enum_name}{}", type_name(&variant.header.name));
                    output.push_str(&format!(
                        "template <> any to_any<polyrust_generated::{name}>(const polyrust_generated::{name}& value);\n"
                    ));
                }
                output.push_str(&format!(
                    "template <> any to_any<polyrust_generated::{enum_name}>(const polyrust_generated::{enum_name}& value);\n\
                     template <> polyrust_generated::{enum_name} from_any<polyrust_generated::{enum_name}>(const any& value);\n"
                ));
                CppCode::new(output)
            }
            _ => CppCode::default(),
        }
    }

    fn value_bridge_definition(&self, declaration: &Declaration) -> CppCode {
        let Declaration::Record(item) = declaration else {
            return CppCode::default();
        };
        let name = type_name(&item.header.name);
        CppCode::new(format!(
            "poly_runtime::aggregate {name}::polyrust_value() const {{\n  return {{{}, \"\", {{{}}}}};\n}}\n",
            item.header.node.id.0,
            item.fields
                .iter()
                .map(|field| format!(
                    "{{{}, poly_runtime::to_any({})}}",
                    cpp_string(&field.header.name),
                    value_name(&field.header.name)
                ))
                .collect::<Vec<_>>()
                .join(", ")
        ))
    }

    fn conversion_definition(&self, declaration: &Declaration) -> CppCode {
        match declaration {
            Declaration::Record(item) => {
                let name = type_name(&item.header.name);
                let fields = CppCode::joined(
                    item.fields.iter().map(|field| {
                        let ty = self.ty(&field.ty);
                        CppCode::new(format!(
                            "from_any<{ty}>(item.fields.at({}))",
                            cpp_string(&field.header.name)
                        ))
                        .with_text_from([ty])
                    }),
                    ", ",
                );
                CppCode::new(format!(
                    "template <> any to_any<polyrust_generated::{name}>(const polyrust_generated::{name}& value) {{ return value.polyrust_value(); }}\n\
                     template <> polyrust_generated::{name} from_any<polyrust_generated::{name}>(const any& value) {{\n\
                       const auto& item = std::any_cast<const aggregate&>(value);\n\
                       return {{{fields}}};\n}}\n"
                ))
                .with_system("any")
                .with_text_from([fields])
            }
            Declaration::Enum(item) => {
                let enum_name = type_name(&item.header.name);
                let mut output = String::new();
                let mut dependencies = Vec::new();
                for variant in &item.variants {
                    let variant_name = type_name(&variant.header.name);
                    let name = format!("{enum_name}{variant_name}");
                    output.push_str(&format!(
                        "template <> any to_any<polyrust_generated::{name}>(const polyrust_generated::{name}& value) {{\n  return aggregate{{{}, {}, {{{}}}}};\n}}\n",
                        item.header.node.id.0,
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
                    "template <> any to_any<polyrust_generated::{enum_name}>(const polyrust_generated::{enum_name}& value) {{\n  return std::visit([](const auto& item) -> any {{ return to_any(item); }}, value);\n}}\n\
                     template <> polyrust_generated::{enum_name} from_any<polyrust_generated::{enum_name}>(const any& value) {{\n  const auto& item = std::any_cast<const aggregate&>(value);\n"
                ));
                for variant in &item.variants {
                    let name = format!("{enum_name}{}", type_name(&variant.header.name));
                    let fields = CppCode::joined(
                        variant.fields.iter().map(|field| {
                            let ty = self.ty(&field.ty);
                            CppCode::new(format!(
                                "from_any<{ty}>(item.fields.at({}))",
                                cpp_string(&field.header.name)
                            ))
                            .with_text_from([ty])
                        }),
                        ", ",
                    );
                    output.push_str(&format!(
                        "  if (item.tag == {}) return polyrust_generated::{name}{{{fields}}};\n",
                        cpp_string(&variant.header.name)
                    ));
                    dependencies.push(fields);
                }
                output.push_str("  throw std::runtime_error(\"unknown generated enum tag\");\n}\n");
                CppCode::new(output)
                    .with_system("any")
                    .with_system("stdexcept")
                    .with_system("variant")
                    .with_text_from(dependencies)
            }
            _ => CppCode::default(),
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

    fn parameters(&self, parameters: &[portable_ir::v0::Parameter]) -> CppCode {
        CppCode::joined(
            parameters.iter().map(|parameter| {
                parameter_type(self.ty(&parameter.ty))
                    .map_text(|ty| format!("{ty} {}", value_name(&parameter.header.name)))
            }),
            ", ",
        )
    }

    fn ty(&self, ty: &TypeRef) -> CppCode {
        match ty {
            TypeRef::Unit => CppCode::new("std::monostate").with_system("variant"),
            TypeRef::Bool => CppCode::new("bool"),
            TypeRef::I32 => CppCode::new("std::int32_t").with_system("cstdint"),
            TypeRef::I64 => CppCode::new("std::int64_t").with_system("cstdint"),
            TypeRef::F64 => CppCode::new("double"),
            TypeRef::Char => CppCode::new("char32_t"),
            TypeRef::String => CppCode::new("std::string").with_system("string"),
            TypeRef::Bytes => CppCode::new("std::vector<std::uint8_t>")
                .with_system("cstdint")
                .with_system("vector"),
            TypeRef::List(inner) => {
                let inner = self.ty(inner);
                CppCode::new(format!("std::vector<{inner}>"))
                    .with_system("vector")
                    .with_text_from([inner])
            }
            TypeRef::Option(inner) => {
                let inner = self.ty(inner);
                CppCode::new(format!("std::optional<{inner}>"))
                    .with_system("optional")
                    .with_text_from([inner])
            }
            TypeRef::Result { ok, error } => {
                let ok = self.ty(ok);
                let error = self.ty(error);
                CppCode::new(format!("value_result<{ok}, {error}>")).with_text_from([ok, error])
            }
            TypeRef::Named(id) => self.named_ty(*id),
            TypeRef::Interface(id) => CppCode::new(type_name(self.name(*id))),
        }
    }

    fn named_ty(&self, id: NodeId) -> CppCode {
        self.program
            .module()
            .declarations
            .iter()
            .find_map(|declaration| match declaration {
                Declaration::Alias(item) if item.header.node.id == id => {
                    Some(self.ty(&item.target))
                }
                declaration if declaration.header().node.id == id => {
                    Some(CppCode::new(type_name(&declaration.header().name)))
                }
                _ => None,
            })
            .unwrap_or_else(|| CppCode::new("std::monostate").with_system("variant"))
    }

    fn name(&self, id: NodeId) -> &str {
        self.names.get(&id).map(String::as_str).unwrap_or("unknown")
    }
}

fn parameter_type(ty: CppCode) -> CppCode {
    if matches!(
        ty.text.as_str(),
        "bool" | "std::int32_t" | "std::int64_t" | "double" | "char32_t"
    ) {
        ty
    } else {
        ty.map_text(|text| format!("const {text}&"))
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

fn cpp_string(value: &str) -> CppCode {
    CppCode::new(serde_json::to_string(value).expect("C++ string serializes"))
}

const CPP_LITERAL_CHUNK_BYTES: usize = 8 * 1024;

fn cpp_string_expression(value: &str) -> CppCode {
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
        expression.push_str(&chunk.text);
    }
    CppCode::new(expression).with_system("string")
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
        let third = CppBackend
            .generate(&checked, &BackendOptions::default())
            .unwrap();
        assert_eq!(first.canonical_json(), second.canonical_json());
        assert_eq!(second.canonical_json(), third.canonical_json());
        assert!(first.dependencies().is_empty());
    }

    #[test]
    fn cpp_includes_and_nested_types_are_validated_fragments() {
        for header in ["any", "cstdint", "string_view", "vendor/library.hpp"] {
            assert!(CppImport::system(header).is_ok(), "{header}");
        }
        for header in ["generated.hpp", "detail/runtime.h"] {
            assert!(CppImport::local(header).is_ok(), "{header}");
        }
        for header in [
            "",
            "../escape.hpp",
            "/absolute.hpp",
            "bad\\path.hpp",
            "x.hpp>\n#include <y",
        ] {
            assert!(CppImport::system(header).is_err(), "{header}");
            assert!(CppImport::local(header).is_err(), "{header}");
        }
        assert!(CppImport::local("not_a_header").is_err());

        let program = fixture();
        let generator = Generator::new(&program);
        for (ty, expected_text, expected_headers) in [
            (TypeRef::Bool, "bool", &[][..]),
            (TypeRef::I64, "std::int64_t", &["cstdint"][..]),
            (TypeRef::String, "std::string", &["string"][..]),
            (TypeRef::Unit, "std::monostate", &["variant"][..]),
            (
                TypeRef::Bytes,
                "std::vector<std::uint8_t>",
                &["cstdint", "vector"][..],
            ),
        ] {
            let code = generator.ty(&ty);
            assert_eq!(code.text, expected_text);
            assert_eq!(system_headers(&code), string_set(expected_headers));
            assert!(code.helper_roots.is_empty());
        }
        let nested = generator.ty(&TypeRef::Result {
            ok: Box::new(TypeRef::Option(Box::new(TypeRef::List(Box::new(
                TypeRef::I64,
            ))))),
            error: Box::new(TypeRef::String),
        });
        assert_eq!(
            nested.text,
            "value_result<std::optional<std::vector<std::int64_t>>, std::string>"
        );
        assert_eq!(
            system_headers(&nested),
            string_set(&["cstdint", "optional", "string", "vector"])
        );
    }

    #[test]
    fn cpp_runtime_sections_own_exact_headers_and_resolve_from_source_roots() {
        let sections = [
            (
                "runtime.model",
                &[
                    "any",
                    "cstdint",
                    "map",
                    "optional",
                    "stdexcept",
                    "string",
                    "type_traits",
                    "utility",
                    "variant",
                    "vector",
                ][..],
            ),
            (
                "runtime.json",
                &[
                    "cstddef",
                    "cstdint",
                    "stdexcept",
                    "string",
                    "string_view",
                    "utility",
                ][..],
            ),
            (
                "runtime.engine",
                &[
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
                    "utility",
                    "vector",
                ][..],
            ),
        ];
        for (id, expected) in sections {
            let code = cpp_runtime_section(id, "int owned;\n".to_owned()).unwrap();
            assert_eq!(system_headers(&code), string_set(expected), "{id}");
        }

        let program = fixture();
        let generator = Generator::new(&program);
        let (_, roots) = generator.source_file().unwrap();
        assert_eq!(roots, string_set(&["runtime.full"]));
        let runtime = render_runtime(&roots);
        assert!(!runtime.contains("POLYRUST-"));
        assert!(runtime.contains("if (name == \"float_is_negative_zero\")"));
        assert!(runtime.contains("value == 0.0 && std::signbit(value)"));
        assert!(runtime.contains("if (name == \"float_abs\")"));
        assert!(runtime.contains("bits & UINT64_C(0x7fffffffffffffff)"));
        assert_eq!(
            include_headers(&runtime),
            string_set(&[
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
            ])
        );
    }

    #[test]
    fn large_embedded_documents_are_runtime_joined_below_cpp_literal_limits() {
        let expression = cpp_string_expression(&"x".repeat(100_000));
        assert!(expression.text.starts_with("std::string(\""));
        assert!(expression.text.ends_with('"'));
        assert!(expression.text.matches(" + \"").count() >= 12);
        assert!(!expression.text.contains(&"x".repeat(65_536)));
        assert_eq!(
            expression.imports,
            BTreeSet::from([(cpp_system_group(), CppImport::system("string").unwrap())])
        );
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

    fn system_headers(code: &CppCode) -> BTreeSet<String> {
        code.imports
            .iter()
            .filter_map(|(_, import)| match &import.kind {
                CppImportKind::System { path } => Some(path.clone()),
                CppImportKind::Local { .. } => None,
            })
            .collect()
    }

    fn string_set(values: &[&str]) -> BTreeSet<String> {
        values.iter().map(|value| (*value).to_owned()).collect()
    }

    fn include_headers(source: &str) -> BTreeSet<String> {
        source
            .lines()
            .filter_map(|line| line.strip_prefix("#include <")?.strip_suffix('>'))
            .map(str::to_owned)
            .collect()
    }

    fn render_runtime(roots: &BTreeSet<String>) -> String {
        let file = cpp_runtime_file(roots).unwrap();
        let group = FileGroup::new(
            FileGroupId::parse("test").unwrap(),
            vec![LanguageFile::source(file)],
        )
        .unwrap();
        let package =
            LanguagePackage::new(vec![group], Vec::<DeclaredDependency>::new(), Vec::new())
                .unwrap();
        let manifest = portable_codegen::render_language_package(&package, &CppRenderer).unwrap();
        generated_text(&manifest, "src/runtime.hpp").to_owned()
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
