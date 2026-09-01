//! Strict ESM TypeScript generation from checked portable IR v0.

#![forbid(unsafe_code)]

use std::collections::{BTreeMap, BTreeSet};

use portable_check::v0::{Capability, CheckedProgram};
use portable_codegen::{
    Backend, BackendDescriptor, BackendError, BackendOptions, BackendVersion, CapabilitySupport,
    DeclaredDependency, Document as CodeDocument, FileGroup, FileGroupId, FileRole, ImportGroup,
    ImportSet, InjectedHelper, IrVersionRange, LanguageFile, LanguagePackage, LanguagePlugin,
    LanguageRenderer, LanguageSourceFile, LanguageUnit, OptionsSchema, OutputManifest, RawText,
    TargetId, generate_with_plugin,
};
use portable_ir::v0::{Declaration, IrVersion, NodeId, TypeRef, Visibility};

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
#[doc(hidden)]
pub enum EcmaImport {
    Default {
        module: &'static str,
        name: &'static str,
    },
    Named {
        module: &'static str,
        name: &'static str,
        type_only: bool,
    },
    ExportAll {
        module: &'static str,
    },
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
                match import {
                    EcmaImport::Default { module, name } => {
                        defaults.push(format!("import {name} from {module:?};"));
                    }
                    EcmaImport::Named {
                        module,
                        name,
                        type_only,
                    } => {
                        let names = named.entry(*module).or_default();
                        if *type_only {
                            names.1.insert(*name);
                        } else {
                            names.0.insert(*name);
                        }
                    }
                    EcmaImport::ExportAll { module } => {
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
            FileRole::Metadata,
            JAVASCRIPT_PACKAGE_JSON,
        )]
    } else {
        vec![
            LanguageFile::text("package.json", FileRole::Metadata, PACKAGE_JSON),
            LanguageFile::text("tsconfig.json", FileRole::Metadata, TSCONFIG),
        ]
    };
    let runtime = if javascript {
        JAVASCRIPT_RUNTIME
    } else {
        RUNTIME
    };
    let mut groups = vec![
        FileGroup::new(ecma_group("metadata")?, metadata).map_err(ecma_generation_error)?,
        FileGroup::new(
            ecma_group("runtime")?,
            vec![LanguageFile::text(runtime_path, FileRole::Runtime, runtime)],
        )
        .map_err(ecma_generation_error)?,
        FileGroup::new(
            ecma_group("source")?,
            vec![LanguageFile::source(if javascript {
                generator.javascript_index_file()?
            } else {
                generator.index_file()?
            })],
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
                    LanguageFile::text("src/node-shims.d.ts", FileRole::Source, NODE_SHIMS),
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

fn require_default(
    unit: &mut LanguageUnit<EcmaImport>,
    group: ImportGroup,
    module: &'static str,
    name: &'static str,
) {
    unit.require_import(group, EcmaImport::Default { module, name });
}

fn require_named(
    unit: &mut LanguageUnit<EcmaImport>,
    module: &'static str,
    name: &'static str,
    type_only: bool,
) {
    unit.require_import(
        local_import_group(),
        EcmaImport::Named {
            module,
            name,
            type_only,
        },
    );
}

fn conformance_file(javascript: bool) -> LanguageSourceFile<EcmaImport> {
    let path = if javascript {
        "src/conformance.test.js"
    } else {
        "src/conformance.test.ts"
    };
    let mut file = LanguageSourceFile::new(path, FileRole::Conformance);
    let mut body = LanguageUnit::new(CodeDocument::raw_text(RawText::new(if javascript {
        JAVASCRIPT_CONFORMANCE_BODY
    } else {
        CONFORMANCE_BODY
    })));
    require_default(
        &mut body,
        node_import_group(),
        "node:assert/strict",
        "assert",
    );
    require_default(&mut body, node_import_group(), "node:test", "test");
    for name in [
        "checkedI32",
        "checkedI64",
        "listAppend",
        "scalarLength",
        "wrappingI32",
        "wrappingI64",
    ] {
        require_named(&mut body, "./runtime.js", name, false);
    }
    file.set_body(body);
    file
}

fn invalid_types_file() -> LanguageSourceFile<EcmaImport> {
    let mut file = LanguageSourceFile::new("tests/invalid-types.ts", FileRole::NegativeTest);
    let mut body = LanguageUnit::new(CodeDocument::raw_text(RawText::new(INVALID_TYPES_BODY)));
    require_named(&mut body, "../src/runtime.js", "PolyOption", true);
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

    fn index_file(&self) -> Result<LanguageSourceFile<EcmaImport>, BackendError> {
        let mut document = serde_json::to_value(self.program.document()).map_err(|error| {
            BackendError::Generation {
                message: format!("cannot serialize checked IR: {error}"),
            }
        })?;
        stringify_wide_numbers(&mut document);
        let document = serde_json::to_string(&document).expect("checked document serializes");
        let mut file = LanguageSourceFile::new("src/index.ts", FileRole::Source);
        file.set_preamble(LanguageUnit::new(CodeDocument::raw_text(RawText::new(
            "// Generated by PolyRust from checked IR v0.",
        ))));
        let mut body = LanguageUnit::new(CodeDocument::empty());
        require_named(&mut body, "./runtime.js", "Runtime", false);
        require_named(&mut body, "./runtime.js", "PolyResult", true);
        body.require_import(
            export_group(),
            EcmaImport::ExportAll {
                module: "./runtime.js",
            },
        );
        let mut output = String::new();
        output.push_str("const runtime = new Runtime(");
        output.push_str(&document);
        output.push_str(");\nconst castResult = <T>(value: PolyResult<unknown>): PolyResult<T> => value as PolyResult<T>;\n\n");
        let mut declarations: Vec<_> = self.program.module().declarations.iter().collect();
        declarations.sort_by_key(|declaration| declaration.header().node.id);
        for declaration in declarations {
            self.declaration(&mut output, declaration);
        }
        output.push_str("export const __invokeTest = (index: number): Readonly<{ actual: PolyResult<unknown>; expected: unknown; expectsError: boolean }> => {\n  const test = TESTS[index];\n  if (test === undefined) return { actual: { ok: false, error: { code: \"invalid_test\", message: \"unknown test\" } }, expected: undefined, expectsError: true };\n  const invocation = test.invocation;\n  const arguments_ = invocation.data.arguments.map((value: unknown) => runtime.decode(value));\n  const actual = invocation.kind === \"function\" ? runtime.invoke(invocation.data.function, arguments_) : runtime.invokeMethod(invocation.data.implementation, invocation.data.method, runtime.decode(invocation.data.receiver), arguments_);\n  return { actual, expected: runtime.decode(test.expected.data), expectsError: test.expected.kind === \"error\" };\n};\n");
        let tests: Vec<_> = self.program.module().declarations.iter().filter_map(|declaration| if let Declaration::Test(test) = declaration { Some(serde_json::json!({"invocation": test.invocation, "expected": test.expected})) } else { None }).collect();
        output.push_str("const TESTS: readonly any[] = ");
        output.push_str(&serde_json::to_string(&tests).expect("tests serialize"));
        output.push_str(";\n");
        body.set_document(CodeDocument::raw_text(RawText::new(output)));
        file.set_body(body);
        Ok(file)
    }

    fn javascript_index_file(&self) -> Result<LanguageSourceFile<EcmaImport>, BackendError> {
        let mut document = serde_json::to_value(self.program.document()).map_err(|error| {
            BackendError::Generation {
                message: format!("cannot serialize checked IR: {error}"),
            }
        })?;
        stringify_wide_numbers(&mut document);
        let document = serde_json::to_string(&document).expect("checked document serializes");
        let mut file = LanguageSourceFile::new("src/index.js", FileRole::Source);
        file.set_preamble(LanguageUnit::new(CodeDocument::raw_text(RawText::new(
            "// Generated by PolyRust from the TypeScript target implementation.",
        ))));
        let mut body = LanguageUnit::new(CodeDocument::empty());
        require_named(&mut body, "./runtime.js", "Runtime", false);
        body.require_import(
            export_group(),
            EcmaImport::ExportAll {
                module: "./runtime.js",
            },
        );
        let mut output = String::new();
        output.push_str("const runtime = new Runtime(");
        output.push_str(&document);
        output.push_str(");\n\n");
        let mut declarations: Vec<_> = self.program.module().declarations.iter().collect();
        declarations.sort_by_key(|declaration| declaration.header().node.id);
        for declaration in declarations {
            self.javascript_declaration(&mut output, declaration);
        }
        output.push_str(
            "export const __invokeTest = (index) => {\n\
             \x20 const test = TESTS[index];\n\
             \x20 if (test === undefined) return { actual: { ok: false, error: { code: \"invalid_test\", message: \"unknown test\" } }, expected: undefined, expectsError: true };\n\
             \x20 const invocation = test.invocation;\n\
             \x20 const arguments_ = invocation.data.arguments.map((value) => runtime.decode(value));\n\
             \x20 const actual = invocation.kind === \"function\" ? runtime.invoke(invocation.data.function, arguments_) : runtime.invokeMethod(invocation.data.implementation, invocation.data.method, runtime.decode(invocation.data.receiver), arguments_);\n\
             \x20 return { actual, expected: runtime.decode(test.expected.data), expectsError: test.expected.kind === \"error\" };\n\
             };\n",
        );
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
        output.push_str("const TESTS = ");
        output.push_str(&serde_json::to_string(&tests).expect("tests serialize"));
        output.push_str(";\n");
        body.set_document(CodeDocument::raw_text(RawText::new(output)));
        file.set_body(body);
        Ok(file)
    }

    fn javascript_declaration(&self, output: &mut String, declaration: &Declaration) {
        match declaration {
            Declaration::Alias(_)
            | Declaration::Enum(_)
            | Declaration::Contract(_)
            | Declaration::Implementation(_)
            | Declaration::Test(_) => {}
            Declaration::Record(item) => {
                output.push_str(&format!(
                    "{}class {} {{\n  __polyDecl = {};\n",
                    export(item.header.visibility),
                    type_name(&item.header.name),
                    item.header.node.id.0
                ));
                output.push_str(&format!(
                    "  constructor({}) {{\n",
                    item.fields
                        .iter()
                        .map(|field| value_name(&field.header.name))
                        .collect::<Vec<_>>()
                        .join(", ")
                ));
                for field in &item.fields {
                    output.push_str(&format!(
                        "    this.{} = {};\n",
                        value_name(&field.header.name),
                        value_name(&field.header.name)
                    ));
                }
                output.push_str("    Object.freeze(this);\n  }\n");
                for implementation in self.program.module().declarations.iter().filter_map(
                    |candidate| match candidate {
                        Declaration::Implementation(value)
                            if value.record == item.header.node.id =>
                        {
                            Some(value)
                        }
                        _ => None,
                    },
                ) {
                    for method in &implementation.methods {
                        output.push_str(&format!(
                            "  {}({}) {{ return runtime.invokeMethod({}, {}, this, [{}]); }}\n",
                            value_name(&method.header.name),
                            method
                                .parameters
                                .iter()
                                .map(|parameter| value_name(&parameter.header.name))
                                .collect::<Vec<_>>()
                                .join(", "),
                            implementation.header.node.id.0,
                            method.header.node.id.0,
                            method
                                .parameters
                                .iter()
                                .map(|parameter| value_name(&parameter.header.name))
                                .collect::<Vec<_>>()
                                .join(", ")
                        ));
                    }
                }
                output.push_str("}\n\n");
            }
            Declaration::Constant(item) => output.push_str(&format!(
                "{}const {} = () => runtime.readConstant({});\n\n",
                export(item.header.visibility),
                value_name(&item.header.name),
                item.header.node.id.0
            )),
            Declaration::Function(item) => output.push_str(&format!(
                "{}const {} = ({}) => runtime.invoke({}, [{}]);\n\n",
                export(item.header.visibility),
                value_name(&item.header.name),
                item.parameters
                    .iter()
                    .map(|parameter| value_name(&parameter.header.name))
                    .collect::<Vec<_>>()
                    .join(", "),
                item.header.node.id.0,
                item.parameters
                    .iter()
                    .map(|parameter| value_name(&parameter.header.name))
                    .collect::<Vec<_>>()
                    .join(", ")
            )),
        }
    }

    fn declaration(&self, output: &mut String, declaration: &Declaration) {
        match declaration {
            Declaration::Alias(item) => output.push_str(&format!(
                "{}type {} = {};\n\n",
                export(item.header.visibility),
                type_name(&item.header.name),
                self.ty(&item.target)
            )),
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
                output.push_str(&format!(
                    "{}class {}{} {{\n  public readonly __polyDecl = {};\n",
                    export(item.header.visibility),
                    type_name(&item.header.name),
                    if contracts.is_empty() {
                        String::new()
                    } else {
                        format!(" implements {contracts}")
                    },
                    item.header.node.id.0
                ));
                for field in &item.fields {
                    output.push_str(&format!(
                        "  public readonly {}: {};\n",
                        value_name(&field.header.name),
                        self.ty(&field.ty)
                    ));
                }
                output.push_str(&format!(
                    "  public constructor({}) {{\n",
                    item.fields
                        .iter()
                        .map(|field| format!(
                            "{}: {}",
                            value_name(&field.header.name),
                            self.ty(&field.ty)
                        ))
                        .collect::<Vec<_>>()
                        .join(", ")
                ));
                for field in &item.fields {
                    output.push_str(&format!(
                        "    this.{} = {};\n",
                        value_name(&field.header.name),
                        value_name(&field.header.name)
                    ));
                }
                output.push_str("    Object.freeze(this);\n  }\n");
                for implementation in implementations {
                    for method in &implementation.methods {
                        output.push_str(&format!("  public {}({}): PolyResult<{}> {{ return castResult(runtime.invokeMethod({}, {}, this, [{}])); }}\n", value_name(&method.header.name), self.parameters(&method.parameters), self.ty(&method.return_type), implementation.header.node.id.0, method.header.node.id.0, method.parameters.iter().map(|parameter| value_name(&parameter.header.name)).collect::<Vec<_>>().join(", ")));
                    }
                }
                output.push_str("}\n\n");
            }
            Declaration::Enum(item) => {
                let variants = item
                    .variants
                    .iter()
                    .map(|variant| {
                        let mut fields = vec![format!("readonly tag: {:?}", variant.header.name)];
                        fields.extend(variant.fields.iter().map(|field| {
                            format!(
                                "readonly {}: {}",
                                value_name(&field.header.name),
                                self.ty(&field.ty)
                            )
                        }));
                        format!("Readonly<{{ {} }}>", fields.join("; "))
                    })
                    .collect::<Vec<_>>()
                    .join(" | ");
                output.push_str(&format!(
                    "{}type {} = {};\n\n",
                    export(item.header.visibility),
                    type_name(&item.header.name),
                    variants
                ));
            }
            Declaration::Contract(item) => {
                output.push_str(&format!(
                    "{}interface {} {{\n",
                    export(item.header.visibility),
                    type_name(&item.header.name)
                ));
                for method in &item.methods {
                    output.push_str(&format!(
                        "  {}({}): PolyResult<{}>;\n",
                        value_name(&method.header.name),
                        self.parameters(&method.parameters),
                        self.ty(&method.return_type)
                    ));
                }
                output.push_str("}\n\n");
            }
            Declaration::Constant(item) => output.push_str(&format!(
                "{}const {} = (): PolyResult<{}> => castResult(runtime.readConstant({}));\n\n",
                export(item.header.visibility),
                value_name(&item.header.name),
                self.ty(&item.ty),
                item.header.node.id.0
            )),
            Declaration::Function(item) => output.push_str(&format!(
                "{}const {} = ({}): PolyResult<{}> => castResult(runtime.invoke({}, [{}]));\n\n",
                export(item.header.visibility),
                value_name(&item.header.name),
                self.parameters(&item.parameters),
                self.ty(&item.return_type),
                item.header.node.id.0,
                item.parameters
                    .iter()
                    .map(|parameter| value_name(&parameter.header.name))
                    .collect::<Vec<_>>()
                    .join(", ")
            )),
            Declaration::Implementation(_) | Declaration::Test(_) => {}
        }
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
            TypeRef::Unit => "undefined".into(),
            TypeRef::Bool => "boolean".into(),
            TypeRef::I32 | TypeRef::F64 => "number".into(),
            TypeRef::I64 => "bigint".into(),
            TypeRef::Char | TypeRef::String => "string".into(),
            TypeRef::Bytes => "readonly number[]".into(),
            TypeRef::List(inner) => format!("readonly {}[]", parenthesize(self.ty(inner))),
            TypeRef::Option(inner) => {
                format!("import(\"./runtime.js\").PolyOption<{}>", self.ty(inner))
            }
            TypeRef::Result { ok, error } => format!(
                "import(\"./runtime.js\").PolyValueResult<{}, {}>",
                self.ty(ok),
                self.ty(error)
            ),
            TypeRef::Named(id) | TypeRef::Contract(id) => type_name(self.name(*id)),
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
        let mut file = LanguageSourceFile::new(path, FileRole::Test);
        let mut body = LanguageUnit::new(CodeDocument::empty());
        let mut output = String::new();
        let mut index = 0;
        for declaration in &self.program.module().declarations {
            if let Declaration::Test(test_declaration) = declaration {
                if index == 0 {
                    require_default(
                        &mut body,
                        node_import_group(),
                        "node:assert/strict",
                        "assert",
                    );
                    require_default(&mut body, node_import_group(), "node:test", "test");
                    require_named(&mut body, "./index.js", "__invokeTest", false);
                }
                output.push_str(&format!("test({:?}, () => {{ const result = __invokeTest({index}); assert.equal(result.actual.ok, !result.expectsError); if (result.actual.ok) assert.deepEqual(result.actual.value, result.expected); }});\n", test_declaration.header.name));
                index += 1;
            }
        }
        if !output.is_empty() {
            body.set_document(CodeDocument::raw_text(RawText::new(output)));
            file.set_body(body);
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
        assert_eq!(Generator::new(&fixture()).ty(&TypeRef::I64), "bigint");
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
        assert_eq!(first.canonical_json(), second.canonical_json());
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
        assert_eq!(first.canonical_json(), second.canonical_json());
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

    fn generated_text<'a>(manifest: &'a OutputManifest, path: &str) -> &'a str {
        match manifest.file(path).unwrap().contents() {
            portable_codegen::OutputContents::Text(text) => text,
            portable_codegen::OutputContents::Bytes(_) => panic!("ECMAScript source must be text"),
        }
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
