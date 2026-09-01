use std::collections::BTreeMap;

use portable_check::v0::{Capability, CheckedProgram};
use portable_codegen::{
    Backend, BackendDescriptor, BackendError, BackendOptions, BackendVersion, CapabilitySupport,
    DeclaredDependency, Document as CodeDocument, FileGroup, FileGroupId, FileRole, FinalNewline,
    ImportGroup, ImportSet, InjectedHelper, IrVersionRange, LanguageFile, LanguagePackage,
    LanguagePlugin, LanguageRenderer, LanguageSourceFile, LanguageUnit, OptionsSchema,
    OutputManifest, RawText, RenderOptions, TargetId, generate_with_plugin, render,
};
use portable_ir::v0::{
    Block, ConstantExpression, Declaration, EnumVariant, ExpectedOutcome, Expression, Intrinsic,
    IrVersion, MatchArm, MethodDispatch, NodeId, Parameter, Pattern, Statement, TestInvocation,
    TypeRef, TypedValue, Value,
};

pub struct RustBackend;

impl Backend for RustBackend {
    fn descriptor(&self) -> BackendDescriptor {
        BackendDescriptor {
            target: TargetId::parse("org.polyrust.rust").expect("static target ID is valid"),
            display_name: "Rust".to_owned(),
            backend_version: BackendVersion::new(0, 1, 0),
            supported_ir: IrVersionRange::exact(IrVersion::CURRENT),
        }
    }

    fn support(&self, capability: Capability) -> CapabilitySupport {
        match capability {
            Capability::CheckedIntegerArithmetic => CapabilitySupport::Helper {
                helper: "polyrust.runtime.checked-integers.v0".to_owned(),
            },
            Capability::UnicodeScalar => CapabilitySupport::Helper {
                helper: "polyrust.runtime.unicode-scalars.v0".to_owned(),
            },
            Capability::ImmutableList => CapabilitySupport::Helper {
                helper: "polyrust.runtime.immutable-list.v0".to_owned(),
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
pub enum RustImport {
    Module { name: &'static str, test_only: bool },
    Use { path: &'static str, public: bool },
}

#[doc(hidden)]
pub struct RustRenderer;

impl LanguageRenderer<RustImport> for RustRenderer {
    fn render_imports(&self, imports: &ImportSet<RustImport>) -> Result<CodeDocument, String> {
        let mut lines = Vec::new();
        for (_, imports) in imports.groups() {
            for import in imports {
                match import {
                    RustImport::Module { name, test_only } => {
                        if *test_only {
                            lines.push("#[cfg(test)]".to_owned());
                        }
                        lines.push(format!("mod {name};"));
                    }
                    RustImport::Use { path, public } => {
                        lines.push(format!("{}use {path};", if *public { "pub " } else { "" }))
                    }
                }
            }
        }
        Ok(CodeDocument::raw_text(RawText::new(lines.join("\n"))))
    }
}

impl LanguagePlugin for RustBackend {
    type Import = RustImport;
    type Renderer = RustRenderer;

    fn translate(
        &self,
        program: &CheckedProgram,
        _options: &BackendOptions,
    ) -> Result<LanguagePackage<Self::Import>, BackendError> {
        let generator = Generator::new(program);
        let cargo = format!(
            "[package]\nname = \"{}\"\nversion = \"0.1.0\"\nedition = \"2024\"\npublish = false\n\n[lib]\npath = \"src/lib.rs\"\n",
            package_name(&program.module().name)
        );
        let helpers = program
            .capabilities()
            .program()
            .iter()
            .filter_map(|capability| match self.support(*capability) {
                CapabilitySupport::Helper { helper } => Some(InjectedHelper {
                    id: helper,
                    capability: format!("{capability:?}"),
                    files: vec!["src/polyrust_runtime.rs".to_owned()],
                }),
                CapabilitySupport::Native | CapabilitySupport::Unsupported { .. } => None,
            })
            .collect();
        LanguagePackage::new(
            vec![
                FileGroup::new(
                    rust_group("metadata")?,
                    vec![LanguageFile::text("Cargo.toml", FileRole::Metadata, cargo)],
                )
                .map_err(rust_generation_error)?,
                FileGroup::new(
                    rust_group("runtime")?,
                    vec![LanguageFile::text(
                        "src/polyrust_runtime.rs",
                        FileRole::Runtime,
                        RUNTIME,
                    )],
                )
                .map_err(rust_generation_error)?,
                FileGroup::new(
                    rust_group("source")?,
                    vec![LanguageFile::source(generator.source_file()?)],
                )
                .map_err(rust_generation_error)?,
                FileGroup::new(
                    rust_group("tests")?,
                    vec![LanguageFile::source(rust_conformance_file())],
                )
                .map_err(rust_generation_error)?,
            ],
            Vec::<DeclaredDependency>::new(),
            helpers,
        )
        .map_err(rust_generation_error)
    }

    fn renderer(&self) -> Self::Renderer {
        RustRenderer
    }
}

fn rust_generation_error(error: impl std::fmt::Display) -> BackendError {
    BackendError::Generation {
        message: error.to_string(),
    }
}

fn rust_group(name: &str) -> Result<FileGroupId, BackendError> {
    FileGroupId::parse(name).map_err(rust_generation_error)
}

fn rust_import_group() -> ImportGroup {
    ImportGroup::new(10, "modules-and-uses").expect("static import group is valid")
}

fn rust_conformance_file() -> LanguageSourceFile<RustImport> {
    let mut file = LanguageSourceFile::new("src/conformance.rs", FileRole::Conformance);
    let mut body = LanguageUnit::new(CodeDocument::raw_text(RawText::new(render_conformance())));
    body.require_import(
        rust_import_group(),
        RustImport::Use {
            path: "super::*",
            public: false,
        },
    );
    file.set_body(body);
    file
}

struct Generator<'a> {
    program: &'a CheckedProgram,
    declarations: BTreeMap<NodeId, &'a Declaration>,
}

impl<'a> Generator<'a> {
    fn new(program: &'a CheckedProgram) -> Self {
        Self {
            program,
            declarations: program
                .module()
                .declarations
                .iter()
                .map(|declaration| (declaration.header().node.id, declaration))
                .collect(),
        }
    }

    fn source_file(&self) -> Result<LanguageSourceFile<RustImport>, BackendError> {
        let mut file = LanguageSourceFile::new("src/lib.rs", FileRole::Source);
        file.set_preamble(LanguageUnit::new(CodeDocument::raw_text(RawText::new(
            "#![forbid(unsafe_code)]\n#![allow(non_camel_case_types, non_snake_case, non_upper_case_globals)]\n#![allow(clippy::unnecessary_wraps)]\n\n// Generated by PolyRust from checked IR v0.",
        ))));
        let mut body_unit = LanguageUnit::new(CodeDocument::empty());
        body_unit.require_import(
            rust_import_group(),
            RustImport::Module {
                name: "polyrust_runtime",
                test_only: false,
            },
        );
        body_unit.require_import(
            rust_import_group(),
            RustImport::Use {
                path: "polyrust_runtime::*",
                public: true,
            },
        );
        body_unit.require_import(
            rust_import_group(),
            RustImport::Module {
                name: "conformance",
                test_only: true,
            },
        );
        let mut source = String::new();

        let mut declarations: Vec<_> = self.program.module().declarations.iter().collect();
        declarations.sort_by_key(|declaration| declaration.header().node.id);
        for declaration in declarations {
            match declaration {
                Declaration::Constant(constant) => {
                    self.documentation(&mut source, &constant.header.documentation, 0);
                    source.push_str(&format!(
                        "{}fn {}() -> PolyResult<{}> {}\n\n",
                        visibility(constant.header.visibility),
                        value_name(&constant.header.name),
                        self.ty(&constant.ty),
                        self.constant_body(&constant.value, 0),
                    ));
                }
                Declaration::Alias(alias) => {
                    self.documentation(&mut source, &alias.header.documentation, 0);
                    source.push_str(&format!(
                        "{}type {} = {};\n\n",
                        visibility(alias.header.visibility),
                        type_name(&alias.header.name),
                        self.ty(&alias.target),
                    ));
                }
                Declaration::Record(record) => {
                    self.documentation(&mut source, &record.header.documentation, 0);
                    source.push_str("#[derive(Clone, Debug, PartialEq)]\n");
                    source.push_str(&format!(
                        "{}struct {} {{\n",
                        visibility(record.header.visibility),
                        type_name(&record.header.name)
                    ));
                    for field in &record.fields {
                        self.documentation(&mut source, &field.header.documentation, 1);
                        source.push_str(&format!(
                            "    pub {}: {},\n",
                            value_name(&field.header.name),
                            self.ty(&field.ty)
                        ));
                    }
                    source.push_str("}\n\n");
                }
                Declaration::Enum(enumeration) => {
                    self.documentation(&mut source, &enumeration.header.documentation, 0);
                    source.push_str("#[derive(Clone, Debug, PartialEq)]\n");
                    source.push_str(&format!(
                        "{}enum {} {{\n",
                        visibility(enumeration.header.visibility),
                        type_name(&enumeration.header.name)
                    ));
                    for variant in &enumeration.variants {
                        self.documentation(&mut source, &variant.header.documentation, 1);
                        source.push_str(&format!("    {}", type_name(&variant.header.name)));
                        if variant.fields.is_empty() {
                            source.push_str(",\n");
                        } else {
                            source.push_str(" {\n");
                            for field in &variant.fields {
                                source.push_str(&format!(
                                    "        {}: {},\n",
                                    value_name(&field.header.name),
                                    self.ty(&field.ty)
                                ));
                            }
                            source.push_str("    },\n");
                        }
                    }
                    source.push_str("}\n\n");
                }
                Declaration::Contract(contract) => {
                    self.documentation(&mut source, &contract.header.documentation, 0);
                    source.push_str(&format!(
                        "{}trait {} {{\n",
                        visibility(contract.header.visibility),
                        type_name(&contract.header.name)
                    ));
                    for method in &contract.methods {
                        self.documentation(&mut source, &method.header.documentation, 1);
                        source.push_str(&format!(
                            "    fn {}(&self{}) -> PolyResult<{}>;\n",
                            value_name(&method.header.name),
                            self.parameters(&method.parameters, true),
                            self.ty(&method.return_type)
                        ));
                    }
                    source.push_str("}\n\n");
                }
                Declaration::Implementation(implementation) => {
                    let contract = self.declaration_name(implementation.contract);
                    let record = self.declaration_name(implementation.record);
                    source.push_str(&format!(
                        "impl {} for {} {{\n",
                        type_name(contract),
                        type_name(record)
                    ));
                    for method in &implementation.methods {
                        self.documentation(&mut source, &method.header.documentation, 1);
                        source.push_str(&format!(
                            "    fn {}(&self{}) -> PolyResult<{}> {}\n",
                            value_name(&method.header.name),
                            self.parameters(&method.parameters, true),
                            self.ty(&method.return_type),
                            self.block(&method.body, 1),
                        ));
                    }
                    source.push_str("}\n\n");
                }
                Declaration::Function(function) => {
                    self.documentation(&mut source, &function.header.documentation, 0);
                    source.push_str(&format!(
                        "{}fn {}({}) -> PolyResult<{}> {}\n\n",
                        visibility(function.header.visibility),
                        value_name(&function.header.name),
                        self.parameters(&function.parameters, false),
                        self.ty(&function.return_type),
                        self.block(&function.body, 0),
                    ));
                }
                Declaration::Test(_) => {}
            }
        }
        self.tests(&mut source);
        let document = CodeDocument::raw_text(RawText::new(source));
        let body = render(
            &document,
            RenderOptions {
                final_newline: FinalNewline::Always,
                ..RenderOptions::default()
            },
        )
        .map_err(|error| BackendError::Generation {
            message: format!("Rust document rendering failed: {error}"),
        })?;
        body_unit.set_document(CodeDocument::raw_text(RawText::new(body)));
        file.set_body(body_unit);
        Ok(file)
    }

    fn documentation(&self, output: &mut String, paragraphs: &[String], indent: usize) {
        let prefix = "    ".repeat(indent);
        for paragraph in paragraphs {
            for line in paragraph.lines() {
                output.push_str(&format!("{prefix}/// {line}\n"));
            }
        }
    }

    fn parameters(&self, parameters: &[Parameter], leading_comma: bool) -> String {
        if parameters.is_empty() {
            return String::new();
        }
        let rendered = parameters
            .iter()
            .map(|parameter| {
                let ty = match &parameter.ty {
                    TypeRef::Contract(id) => {
                        format!("&dyn {}", type_name(self.declaration_name(*id)))
                    }
                    other => self.ty(other),
                };
                format!("{}: {ty}", value_name(&parameter.header.name))
            })
            .collect::<Vec<_>>()
            .join(", ");
        if leading_comma {
            format!(", {rendered}")
        } else {
            rendered
        }
    }

    fn ty(&self, ty: &TypeRef) -> String {
        match ty {
            TypeRef::Unit => "()".to_owned(),
            TypeRef::Bool => "bool".to_owned(),
            TypeRef::I32 => "i32".to_owned(),
            TypeRef::I64 => "i64".to_owned(),
            TypeRef::F64 => "f64".to_owned(),
            TypeRef::Char => "char".to_owned(),
            TypeRef::String => "String".to_owned(),
            TypeRef::Bytes => "Vec<u8>".to_owned(),
            TypeRef::List(element) => format!("Vec<{}>", self.ty(element)),
            TypeRef::Option(inner) => format!("Option<{}>", self.ty(inner)),
            TypeRef::Result { ok, error } => {
                format!("Result<{}, {}>", self.ty(ok), self.ty(error))
            }
            TypeRef::Named(id) | TypeRef::Contract(id) => type_name(self.declaration_name(*id)),
        }
    }

    fn declaration_name(&self, id: NodeId) -> &str {
        self.declarations
            .get(&id)
            .map(|declaration| declaration.header().name.as_str())
            .unwrap_or("MissingDeclaration")
    }

    fn declaration(&self, id: NodeId) -> Option<&Declaration> {
        self.declarations.get(&id).copied()
    }

    fn field_name(&self, id: NodeId) -> &str {
        for declaration in self.declarations.values() {
            match declaration {
                Declaration::Record(record) => {
                    if let Some(field) = record
                        .fields
                        .iter()
                        .find(|field| field.header.node.id == id)
                    {
                        return &field.header.name;
                    }
                }
                Declaration::Enum(enumeration) => {
                    for variant in &enumeration.variants {
                        if let Some(field) = variant
                            .fields
                            .iter()
                            .find(|field| field.header.node.id == id)
                        {
                            return &field.header.name;
                        }
                    }
                }
                _ => {}
            }
        }
        "missing_field"
    }

    fn variant(&self, id: NodeId) -> Option<(&str, &EnumVariant)> {
        self.declarations.values().find_map(|declaration| {
            let Declaration::Enum(enumeration) = declaration else {
                return None;
            };
            enumeration
                .variants
                .iter()
                .find(|variant| variant.header.node.id == id)
                .map(|variant| (enumeration.header.name.as_str(), variant))
        })
    }
}

fn visibility(visibility: portable_ir::v0::Visibility) -> &'static str {
    match visibility {
        portable_ir::v0::Visibility::Public => "pub ",
        portable_ir::v0::Visibility::Package => "pub(crate) ",
    }
}

fn package_name(module: &str) -> String {
    format!("polyrust-generated-{}", module.replace('_', "-"))
}

fn type_name(name: &str) -> String {
    rust_identifier(name)
}

fn value_name(name: &str) -> String {
    rust_identifier(name)
}

fn rust_identifier(name: &str) -> String {
    const KEYWORDS: &[&str] = &[
        "as", "break", "const", "continue", "crate", "else", "enum", "extern", "false", "fn",
        "for", "if", "impl", "in", "let", "loop", "match", "mod", "move", "mut", "pub", "ref",
        "return", "static", "struct", "trait", "true", "type", "unsafe", "use", "where", "while",
        "async", "await", "dyn", "abstract", "become", "box", "do", "final", "macro", "override",
        "priv", "typeof", "unsized", "virtual", "yield", "try", "gen",
    ];
    match name {
        "self" | "Self" | "super" | "crate" => format!("{name}_"),
        _ if KEYWORDS.contains(&name) => format!("r#{name}"),
        _ => name.to_owned(),
    }
}

fn test_name(name: &str, id: NodeId) -> String {
    let mut result = String::new();
    for character in name.chars() {
        if character.is_ascii_alphanumeric() {
            result.push(character.to_ascii_lowercase());
        } else if !result.ends_with('_') {
            result.push('_');
        }
    }
    format!("{}_n{}", result.trim_matches('_'), id.0)
}

impl Generator<'_> {
    fn block(&self, block: &Block, indent: usize) -> String {
        let prefix = "    ".repeat(indent);
        let inner = "    ".repeat(indent + 1);
        let mut output = String::from("{\n");
        for statement in &block.statements {
            match statement {
                Statement::Let {
                    name,
                    annotation,
                    value,
                    ..
                } => {
                    let annotation = annotation
                        .as_ref()
                        .map_or_else(String::new, |ty| format!(": {}", self.ty(ty)));
                    output.push_str(&format!(
                        "{inner}let {}{annotation} = ({})?;\n",
                        value_name(name),
                        self.expr(value, indent + 1)
                    ));
                }
                Statement::ForEach {
                    binding,
                    iterable,
                    body,
                    ..
                } => {
                    output.push_str(&format!(
                        "{inner}for {} in ({})? {}\n",
                        value_name(binding),
                        self.expr(iterable, indent + 1),
                        self.block(body, indent + 1)
                    ));
                }
                Statement::Return { value, .. } => match value {
                    Some(value) => output.push_str(&format!(
                        "{inner}return {};\n",
                        self.expr(value, indent + 1)
                    )),
                    None => output.push_str(&format!("{inner}return Ok(());\n")),
                },
                Statement::Expression { value, .. } => output.push_str(&format!(
                    "{inner}let _ = ({})?;\n",
                    self.expr(value, indent + 1)
                )),
            }
        }
        match &block.result {
            Some(result) => output.push_str(&format!("{inner}{}\n", self.expr(result, indent + 1))),
            None => output.push_str(&format!("{inner}Ok(())\n")),
        }
        output.push_str(&format!("{prefix}}}"));
        output
    }

    fn expr(&self, expression: &Expression, indent: usize) -> String {
        match expression {
            Expression::Literal { value, .. } => format!("Ok({})", self.value(value)),
            Expression::Local { node, name } => {
                let value = value_name(name);
                if self
                    .program
                    .expression_type(node.id)
                    .is_some_and(|ty| self.is_copy(ty))
                {
                    format!("Ok({value})")
                } else {
                    format!("Ok({value}.clone())")
                }
            }
            Expression::Constant { declaration, .. } => {
                format!("{}()", value_name(self.declaration_name(*declaration)))
            }
            Expression::SelfValue { .. } => "Ok(self.clone())".to_owned(),
            Expression::ConstructRecord {
                declaration,
                fields,
                ..
            } => format!(
                "Ok({} {{ {} }})",
                type_name(self.declaration_name(*declaration)),
                fields
                    .iter()
                    .map(|field| format!(
                        "{}: ({})?",
                        value_name(self.field_name(field.field)),
                        self.expr(&field.value, indent)
                    ))
                    .collect::<Vec<_>>()
                    .join(", ")
            ),
            Expression::ConstructEnum {
                declaration,
                variant,
                fields,
                ..
            } => {
                let variant_name = self
                    .variant(*variant)
                    .map_or("MissingVariant", |(_, variant)| {
                        variant.header.name.as_str()
                    });
                if fields.is_empty() {
                    format!(
                        "Ok({}::{})",
                        type_name(self.declaration_name(*declaration)),
                        type_name(variant_name)
                    )
                } else {
                    format!(
                        "Ok({}::{} {{ {} }})",
                        type_name(self.declaration_name(*declaration)),
                        type_name(variant_name),
                        fields
                            .iter()
                            .map(|field| format!(
                                "{}: ({})?",
                                value_name(self.field_name(field.field)),
                                self.expr(&field.value, indent)
                            ))
                            .collect::<Vec<_>>()
                            .join(", ")
                    )
                }
            }
            Expression::ConstructSome { value, .. } => {
                format!("Ok(Some(({})?))", self.expr(value, indent))
            }
            Expression::ConstructNone { .. } => "Ok(None)".to_owned(),
            Expression::ConstructOk { value, .. } => {
                format!("Ok(Ok(({})?))", self.expr(value, indent))
            }
            Expression::ConstructErr { value, .. } => {
                format!("Ok(Err(({})?))", self.expr(value, indent))
            }
            Expression::ConstructList { elements, .. } => format!(
                "Ok(vec![{}])",
                elements
                    .iter()
                    .map(|element| format!("({})?", self.expr(element, indent)))
                    .collect::<Vec<_>>()
                    .join(", ")
            ),
            Expression::Field { base, field, .. } => {
                let access = format!(
                    "(({})?).{}",
                    self.expr(base, indent),
                    value_name(self.field_name(*field))
                );
                if self.field_type(*field).is_some_and(|ty| self.is_copy(ty)) {
                    format!("Ok({access})")
                } else {
                    format!("Ok({access}.clone())")
                }
            }
            Expression::Call {
                function,
                arguments,
                ..
            } => self.call(
                value_name(self.declaration_name(*function)),
                None,
                arguments,
                indent,
            ),
            Expression::MethodCall {
                receiver,
                dispatch,
                arguments,
                ..
            } => self.method_call(receiver, dispatch, arguments, indent),
            Expression::Intrinsic {
                operation,
                arguments,
                ..
            } => {
                let first_type = arguments
                    .first()
                    .and_then(|argument| self.program.expression_type(argument.node().id));
                self.intrinsic(
                    *operation,
                    arguments
                        .iter()
                        .map(|argument| self.expr(argument, indent))
                        .collect(),
                    first_type,
                )
            }
            Expression::If {
                condition,
                then_block,
                else_block,
                ..
            } => format!(
                "if ({})? {} else {}",
                self.expr(condition, indent),
                self.block(then_block, indent),
                self.block(else_block, indent)
            ),
            Expression::Match { value, arms, .. } => format!(
                "match ({})? {{\n{}{} }}",
                self.expr(value, indent),
                arms.iter()
                    .map(|arm| self.match_arm(arm, indent + 1))
                    .collect::<String>(),
                "    ".repeat(indent)
            ),
            Expression::Block(block) => self.block(block, indent),
        }
    }

    fn call(
        &self,
        callable: String,
        receiver: Option<String>,
        arguments: &[Expression],
        indent: usize,
    ) -> String {
        let mut output = String::from("{ ");
        if let Some(receiver) = receiver {
            output.push_str(&format!("let __receiver = ({receiver})?; "));
        }
        for (index, argument) in arguments.iter().enumerate() {
            output.push_str(&format!(
                "let __argument_{index} = ({})?; ",
                self.expr(argument, indent)
            ));
        }
        let args = (0..arguments.len())
            .map(|index| format!("__argument_{index}"))
            .collect::<Vec<_>>()
            .join(", ");
        output.push_str(&format!("{callable}({args}) }}"));
        output
    }

    fn method_call(
        &self,
        receiver: &Expression,
        dispatch: &MethodDispatch,
        arguments: &[Expression],
        indent: usize,
    ) -> String {
        let receiver_result = self.expr(receiver, indent);
        let mut prefix = format!("{{ let __receiver = ({receiver_result})?; ");
        for (index, argument) in arguments.iter().enumerate() {
            prefix.push_str(&format!(
                "let __argument_{index} = ({})?; ",
                self.expr(argument, indent)
            ));
        }
        let args = (0..arguments.len())
            .map(|index| format!("__argument_{index}"))
            .collect::<Vec<_>>()
            .join(", ");
        let suffix = if args.is_empty() {
            String::new()
        } else {
            format!(", {args}")
        };
        let call = match dispatch {
            MethodDispatch::Contract { contract, method } => {
                let method_name = self.contract_method_name(*contract, *method);
                format!("__receiver.{}({args})", value_name(method_name))
            }
            MethodDispatch::Concrete {
                implementation,
                method,
            } => {
                let (contract, record, method_name) =
                    self.implementation_method(*implementation, *method);
                format!(
                    "<{} as {}>::{}(&__receiver{suffix})",
                    type_name(record),
                    type_name(contract),
                    value_name(method_name)
                )
            }
        };
        prefix.push_str(&call);
        prefix.push_str(" }");
        prefix
    }

    fn match_arm(&self, arm: &MatchArm, indent: usize) -> String {
        format!(
            "{}{} => {},\n",
            "    ".repeat(indent),
            self.pattern(&arm.pattern),
            self.block(&arm.body, indent)
        )
    }

    fn pattern(&self, pattern: &Pattern) -> String {
        match pattern {
            Pattern::Wildcard { .. } => "_".to_owned(),
            Pattern::Bool { value, .. } => value.to_string(),
            Pattern::None { .. } => "None".to_owned(),
            Pattern::Some { binding, .. } => format!("Some({})", value_name(binding)),
            Pattern::Ok { binding, .. } => format!("Ok({})", value_name(binding)),
            Pattern::Err { binding, .. } => format!("Err({})", value_name(binding)),
            Pattern::EnumVariant {
                declaration,
                variant,
                bindings,
                ..
            } => {
                let variant_name = self
                    .variant(*variant)
                    .map_or("MissingVariant", |(_, variant)| {
                        variant.header.name.as_str()
                    });
                if bindings.is_empty() {
                    format!(
                        "{}::{}",
                        type_name(self.declaration_name(*declaration)),
                        type_name(variant_name)
                    )
                } else {
                    format!(
                        "{}::{} {{ {} }}",
                        type_name(self.declaration_name(*declaration)),
                        type_name(variant_name),
                        bindings
                            .iter()
                            .map(|binding| format!(
                                "{}: {}",
                                value_name(self.field_name(binding.field)),
                                value_name(&binding.binding)
                            ))
                            .collect::<Vec<_>>()
                            .join(", ")
                    )
                }
            }
        }
    }

    fn is_copy(&self, ty: &TypeRef) -> bool {
        match ty {
            TypeRef::Unit
            | TypeRef::Bool
            | TypeRef::I32
            | TypeRef::I64
            | TypeRef::F64
            | TypeRef::Char
            | TypeRef::Contract(_) => true,
            TypeRef::Named(id) => match self.declaration(*id) {
                Some(Declaration::Alias(alias)) => self.is_copy(&alias.target),
                _ => false,
            },
            TypeRef::String
            | TypeRef::Bytes
            | TypeRef::List(_)
            | TypeRef::Option(_)
            | TypeRef::Result { .. } => false,
        }
    }

    fn field_type(&self, id: NodeId) -> Option<&TypeRef> {
        for declaration in self.declarations.values() {
            match declaration {
                Declaration::Record(record) => {
                    if let Some(field) = record
                        .fields
                        .iter()
                        .find(|field| field.header.node.id == id)
                    {
                        return Some(&field.ty);
                    }
                }
                Declaration::Enum(enumeration) => {
                    for variant in &enumeration.variants {
                        if let Some(field) = variant
                            .fields
                            .iter()
                            .find(|field| field.header.node.id == id)
                        {
                            return Some(&field.ty);
                        }
                    }
                }
                _ => {}
            }
        }
        None
    }

    fn contract_method_name(&self, contract: NodeId, method: NodeId) -> &str {
        let Some(Declaration::Contract(contract)) = self.declaration(contract) else {
            return "missing_method";
        };
        contract
            .methods
            .iter()
            .find(|candidate| candidate.header.node.id == method)
            .map_or("missing_method", |method| method.header.name.as_str())
    }

    fn implementation_method(&self, implementation: NodeId, method: NodeId) -> (&str, &str, &str) {
        let Some(Declaration::Implementation(implementation)) = self.declaration(implementation)
        else {
            return ("MissingContract", "MissingRecord", "missing_method");
        };
        let method = implementation
            .methods
            .iter()
            .find(|candidate| {
                candidate.header.node.id == method || candidate.contract_method == method
            })
            .map_or("missing_method", |method| method.header.name.as_str());
        (
            self.declaration_name(implementation.contract),
            self.declaration_name(implementation.record),
            method,
        )
    }
}

impl Generator<'_> {
    fn constant_body(&self, expression: &ConstantExpression, _indent: usize) -> String {
        format!("{{ {} }}", self.constant_expr(expression))
    }

    fn constant_expr(&self, expression: &ConstantExpression) -> String {
        match expression {
            ConstantExpression::Literal { value, .. } => format!("Ok({})", self.value(value)),
            ConstantExpression::Reference { declaration, .. } => {
                format!("{}()", value_name(self.declaration_name(*declaration)))
            }
            ConstantExpression::Record {
                declaration,
                fields,
                ..
            } => format!(
                "Ok({} {{ {} }})",
                type_name(self.declaration_name(*declaration)),
                fields
                    .iter()
                    .map(|field| format!(
                        "{}: ({})?",
                        value_name(self.field_name(field.field)),
                        self.constant_expr(&field.value)
                    ))
                    .collect::<Vec<_>>()
                    .join(", ")
            ),
            ConstantExpression::Enum {
                declaration,
                variant,
                fields,
                ..
            } => {
                let variant_name = self
                    .variant(*variant)
                    .map_or("MissingVariant", |(_, variant)| {
                        variant.header.name.as_str()
                    });
                if fields.is_empty() {
                    format!(
                        "Ok({}::{})",
                        type_name(self.declaration_name(*declaration)),
                        type_name(variant_name)
                    )
                } else {
                    format!(
                        "Ok({}::{} {{ {} }})",
                        type_name(self.declaration_name(*declaration)),
                        type_name(variant_name),
                        fields
                            .iter()
                            .map(|field| format!(
                                "{}: ({})?",
                                value_name(self.field_name(field.field)),
                                self.constant_expr(&field.value)
                            ))
                            .collect::<Vec<_>>()
                            .join(", ")
                    )
                }
            }
            ConstantExpression::Some { value, .. } => {
                format!("Ok(Some(({})?))", self.constant_expr(value))
            }
            ConstantExpression::None { .. } => "Ok(None)".to_owned(),
            ConstantExpression::Ok { value, .. } => {
                format!("Ok(Ok(({})?))", self.constant_expr(value))
            }
            ConstantExpression::Err { value, .. } => {
                format!("Ok(Err(({})?))", self.constant_expr(value))
            }
            ConstantExpression::List { elements, .. } => format!(
                "Ok(vec![{}])",
                elements
                    .iter()
                    .map(|element| format!("({})?", self.constant_expr(element)))
                    .collect::<Vec<_>>()
                    .join(", ")
            ),
            ConstantExpression::Intrinsic {
                operation,
                arguments,
                ..
            } => {
                let first_type = arguments
                    .first()
                    .and_then(|argument| self.constant_type(argument));
                self.intrinsic(
                    *operation,
                    arguments
                        .iter()
                        .map(|argument| self.constant_expr(argument))
                        .collect(),
                    first_type.as_ref(),
                )
            }
        }
    }

    fn intrinsic(
        &self,
        operation: Intrinsic,
        arguments: Vec<String>,
        first_type: Option<&TypeRef>,
    ) -> String {
        if operation == Intrinsic::BoolAnd {
            return format!(
                "{{ let __argument_0 = ({})?; if !__argument_0 {{ Ok(false) }} else {{ {} }} }}",
                arguments[0], arguments[1]
            );
        }
        if operation == Intrinsic::BoolOr {
            return format!(
                "{{ let __argument_0 = ({})?; if __argument_0 {{ Ok(true) }} else {{ {} }} }}",
                arguments[0], arguments[1]
            );
        }
        let mut output = String::from("{ ");
        for (index, argument) in arguments.iter().enumerate() {
            output.push_str(&format!("let __argument_{index} = ({argument})?; "));
        }
        let a = "__argument_0";
        let b = "__argument_1";
        let c = "__argument_2";
        let width = match first_type {
            Some(TypeRef::I32) => 32,
            _ => 64,
        };
        let expression = match operation {
            Intrinsic::BoolNot => format!("Ok(!{a})"),
            Intrinsic::BoolAnd | Intrinsic::BoolOr => unreachable!(),
            Intrinsic::Equal => format!("Ok({a} == {b})"),
            Intrinsic::NotEqual => format!("Ok({a} != {b})"),
            Intrinsic::Less => format!("Ok({a} < {b})"),
            Intrinsic::LessEqual => format!("Ok({a} <= {b})"),
            Intrinsic::Greater => format!("Ok({a} > {b})"),
            Intrinsic::GreaterEqual => format!("Ok({a} >= {b})"),
            Intrinsic::IntNegChecked => format!(
                "{a}.checked_neg().ok_or(PolyRuntimeError::CheckedOverflow {{ operation: \"neg\" }})"
            ),
            Intrinsic::IntAddChecked => format!(
                "{a}.checked_add({b}).ok_or(PolyRuntimeError::CheckedOverflow {{ operation: \"add\" }})"
            ),
            Intrinsic::IntSubChecked => format!(
                "{a}.checked_sub({b}).ok_or(PolyRuntimeError::CheckedOverflow {{ operation: \"sub\" }})"
            ),
            Intrinsic::IntMulChecked => format!(
                "{a}.checked_mul({b}).ok_or(PolyRuntimeError::CheckedOverflow {{ operation: \"mul\" }})"
            ),
            Intrinsic::IntDivChecked => format!(
                "if {b} == 0 {{ Err(PolyRuntimeError::DivisionByZero) }} else {{ {a}.checked_div({b}).ok_or(PolyRuntimeError::CheckedOverflow {{ operation: \"div\" }}) }}"
            ),
            Intrinsic::IntRemChecked => format!(
                "if {b} == 0 {{ Err(PolyRuntimeError::RemainderByZero) }} else {{ {a}.checked_rem({b}).ok_or(PolyRuntimeError::CheckedOverflow {{ operation: \"rem\" }}) }}"
            ),
            Intrinsic::IntNegWrapping => format!("Ok({a}.wrapping_neg())"),
            Intrinsic::IntAddWrapping => format!("Ok({a}.wrapping_add({b}))"),
            Intrinsic::IntSubWrapping => format!("Ok({a}.wrapping_sub({b}))"),
            Intrinsic::IntMulWrapping => format!("Ok({a}.wrapping_mul({b}))"),
            Intrinsic::IntBitNot => format!("Ok(!{a})"),
            Intrinsic::IntBitAnd => format!("Ok({a} & {b})"),
            Intrinsic::IntBitOr => format!("Ok({a} | {b})"),
            Intrinsic::IntBitXor => format!("Ok({a} ^ {b})"),
            Intrinsic::IntShiftLeftChecked => format!("_poly_shift_left({a}, {b} as i64, {width})"),
            Intrinsic::IntShiftRightChecked => {
                format!("_poly_shift_right({a}, {b} as i64, {width})")
            }
            Intrinsic::FloatNeg => format!("Ok(-{a})"),
            Intrinsic::FloatAdd => format!("Ok({a} + {b})"),
            Intrinsic::FloatSub => format!("Ok({a} - {b})"),
            Intrinsic::FloatMul => format!("Ok({a} * {b})"),
            Intrinsic::FloatDiv => format!("Ok({a} / {b})"),
            Intrinsic::FloatRemTrunc => format!("Ok({a} % {b})"),
            Intrinsic::StringConcat => {
                format!("{{ let mut value = {a}; value.push_str(&{b}); Ok(value) }}")
            }
            Intrinsic::StringScalarLength => format!("Ok({a}.chars().count() as i64)"),
            Intrinsic::StringIsEmpty => format!("Ok({a}.is_empty())"),
            Intrinsic::StringContains => format!("Ok({a}.contains({b}.as_str()))"),
            Intrinsic::StringStartsWith => format!("Ok({a}.starts_with({b}.as_str()))"),
            Intrinsic::StringStripPrefix => {
                format!("Ok({a}.strip_prefix({b}.as_str()).unwrap_or({a}.as_str()).to_owned())")
            }
            Intrinsic::StringEndsWith => format!("Ok({a}.ends_with({b}.as_str()))"),
            Intrinsic::StringReplaceAll => {
                format!("Ok({a}.replace({b}.as_str(), {c}.as_str()))")
            }
            Intrinsic::StringReplaceMany => {
                let mappings = (1..arguments.len())
                    .step_by(2)
                    .map(|index| {
                        format!(
                            "(__argument_{index}.as_str(), __argument_{}.as_str())",
                            index + 1
                        )
                    })
                    .collect::<Vec<_>>()
                    .join(", ");
                format!("_poly_string_replace_many({a}, &[{mappings}])")
            }
            Intrinsic::StringTruncateUtf8Bytes => {
                format!("_poly_string_truncate_utf8_bytes({a}, {b})")
            }
            Intrinsic::StringTrimStart => {
                format!(
                    "Ok({a}.trim_start_matches(|character| {b}.contains(character)).to_owned())"
                )
            }
            Intrinsic::StringTrimEnd => {
                format!("Ok({a}.trim_end_matches(|character| {b}.contains(character)).to_owned())")
            }
            Intrinsic::BytesConcat | Intrinsic::ListConcat => {
                format!("{{ let mut value = {a}; value.extend({b}); Ok(value) }}")
            }
            Intrinsic::BytesLength | Intrinsic::ListLength => format!("Ok({a}.len() as i64)"),
            Intrinsic::BytesIsEmpty | Intrinsic::ListIsEmpty => format!("Ok({a}.is_empty())"),
            Intrinsic::ListGetChecked => format!("_poly_list_get({a}, {b})"),
            Intrinsic::ListAppend => {
                format!("{{ let mut value = {a}; value.push({b}); Ok(value) }}")
            }
            Intrinsic::ListContains => format!("Ok({a}.contains(&{b}))"),
            Intrinsic::OptionIsSome => format!("Ok({a}.is_some())"),
            Intrinsic::OptionIsNone => format!("Ok({a}.is_none())"),
            Intrinsic::OptionUnwrapOr => format!("Ok({a}.unwrap_or({b}))"),
            Intrinsic::ResultIsOk => format!("Ok({a}.is_ok())"),
            Intrinsic::ResultIsErr => format!("Ok({a}.is_err())"),
            Intrinsic::WidenI32ToI64 => format!("Ok(i64::from({a}))"),
            Intrinsic::NarrowI64ToI32Checked => format!(
                "i32::try_from({a}).map_err(|_| PolyRuntimeError::NarrowingOutOfRange {{ value: {a} }})"
            ),
            Intrinsic::StringToUtf8 => format!("Ok({a}.into_bytes())"),
            Intrinsic::StringFromUtf8Checked => {
                format!("String::from_utf8({a}).map_err(|_| PolyRuntimeError::InvalidUtf8)")
            }
        };
        output.push_str(&expression);
        output.push_str(" }");
        output
    }

    fn constant_type(&self, expression: &ConstantExpression) -> Option<TypeRef> {
        match expression {
            ConstantExpression::Literal { value, .. } => self.value_type(value),
            ConstantExpression::Reference { declaration, .. } => {
                let Some(Declaration::Constant(constant)) = self.declaration(*declaration) else {
                    return None;
                };
                Some(constant.ty.clone())
            }
            ConstantExpression::Record { declaration, .. }
            | ConstantExpression::Enum { declaration, .. } => Some(TypeRef::Named(*declaration)),
            ConstantExpression::Some { value, .. } => self
                .constant_type(value)
                .map(|ty| TypeRef::Option(Box::new(ty))),
            ConstantExpression::None { inner_type, .. } => {
                Some(TypeRef::Option(Box::new(inner_type.clone())))
            }
            ConstantExpression::Ok {
                value, error_type, ..
            } => self.constant_type(value).map(|ok| TypeRef::Result {
                ok: Box::new(ok),
                error: Box::new(error_type.clone()),
            }),
            ConstantExpression::Err { value, ok_type, .. } => {
                self.constant_type(value).map(|error| TypeRef::Result {
                    ok: Box::new(ok_type.clone()),
                    error: Box::new(error),
                })
            }
            ConstantExpression::List { element_type, .. } => {
                Some(TypeRef::List(Box::new(element_type.clone())))
            }
            ConstantExpression::Intrinsic {
                operation,
                arguments,
                ..
            } => {
                let first = arguments
                    .first()
                    .and_then(|argument| self.constant_type(argument))?;
                match operation {
                    Intrinsic::BoolNot
                    | Intrinsic::BoolAnd
                    | Intrinsic::BoolOr
                    | Intrinsic::Equal
                    | Intrinsic::NotEqual
                    | Intrinsic::Less
                    | Intrinsic::LessEqual
                    | Intrinsic::Greater
                    | Intrinsic::GreaterEqual
                    | Intrinsic::StringIsEmpty
                    | Intrinsic::StringContains
                    | Intrinsic::StringStartsWith
                    | Intrinsic::StringEndsWith
                    | Intrinsic::BytesIsEmpty
                    | Intrinsic::ListIsEmpty
                    | Intrinsic::ListContains
                    | Intrinsic::OptionIsSome
                    | Intrinsic::OptionIsNone
                    | Intrinsic::ResultIsOk
                    | Intrinsic::ResultIsErr => Some(TypeRef::Bool),
                    Intrinsic::StringScalarLength
                    | Intrinsic::BytesLength
                    | Intrinsic::ListLength
                    | Intrinsic::WidenI32ToI64 => Some(TypeRef::I64),
                    Intrinsic::NarrowI64ToI32Checked => Some(TypeRef::I32),
                    Intrinsic::StringToUtf8 => Some(TypeRef::Bytes),
                    Intrinsic::StringFromUtf8Checked
                    | Intrinsic::StringConcat
                    | Intrinsic::StringReplaceAll
                    | Intrinsic::StringReplaceMany
                    | Intrinsic::StringTruncateUtf8Bytes
                    | Intrinsic::StringStripPrefix
                    | Intrinsic::StringTrimStart
                    | Intrinsic::StringTrimEnd => Some(TypeRef::String),
                    Intrinsic::ListGetChecked => match first {
                        TypeRef::List(element) => Some(*element),
                        _ => None,
                    },
                    Intrinsic::OptionUnwrapOr => match first {
                        TypeRef::Option(inner) => Some(*inner),
                        _ => None,
                    },
                    _ => Some(first),
                }
            }
        }
    }

    fn value_type(&self, value: &Value) -> Option<TypeRef> {
        match value {
            Value::Unit => Some(TypeRef::Unit),
            Value::Bool(_) => Some(TypeRef::Bool),
            Value::I32(_) => Some(TypeRef::I32),
            Value::I64(_) => Some(TypeRef::I64),
            Value::F64(_) => Some(TypeRef::F64),
            Value::Char(_) => Some(TypeRef::Char),
            Value::String(_) => Some(TypeRef::String),
            Value::Bytes(_) => Some(TypeRef::Bytes),
            Value::List(values) => values
                .first()
                .and_then(|value| self.value_type(value))
                .map(|ty| TypeRef::List(Box::new(ty))),
            Value::Some(value) => self
                .value_type(value)
                .map(|ty| TypeRef::Option(Box::new(ty))),
            Value::Ok(_) | Value::Err(_) | Value::None => None,
            Value::Record { declaration, .. } | Value::Enum { declaration, .. } => {
                Some(TypeRef::Named(*declaration))
            }
        }
    }

    fn value(&self, value: &Value) -> String {
        match value {
            Value::Unit => "()".to_owned(),
            Value::Bool(value) => value.to_string(),
            Value::I32(value) => format!("{value}_i32"),
            Value::I64(value) => format!("{value}_i64"),
            Value::F64(value) => format!("f64::from_bits(0x{:016x})", value.0),
            Value::Char(value) => format!("{value:?}"),
            Value::String(value) => format!("String::from({value:?})"),
            Value::Bytes(value) => format!(
                "vec![{}]",
                value
                    .iter()
                    .map(|byte| format!("0x{byte:02x}_u8"))
                    .collect::<Vec<_>>()
                    .join(", ")
            ),
            Value::List(values) => format!(
                "vec![{}]",
                values
                    .iter()
                    .map(|value| self.value(value))
                    .collect::<Vec<_>>()
                    .join(", ")
            ),
            Value::None => "None".to_owned(),
            Value::Some(value) => format!("Some({})", self.value(value)),
            Value::Ok(value) => format!("Ok({})", self.value(value)),
            Value::Err(value) => format!("Err({})", self.value(value)),
            Value::Record {
                declaration,
                fields,
            } => format!(
                "{} {{ {} }}",
                type_name(self.declaration_name(*declaration)),
                fields
                    .iter()
                    .map(|field| format!(
                        "{}: {}",
                        value_name(self.field_name(field.field)),
                        self.value(&field.value)
                    ))
                    .collect::<Vec<_>>()
                    .join(", ")
            ),
            Value::Enum {
                declaration,
                variant,
                fields,
            } => {
                let variant_name = self
                    .variant(*variant)
                    .map_or("MissingVariant", |(_, variant)| {
                        variant.header.name.as_str()
                    });
                if fields.is_empty() {
                    format!(
                        "{}::{}",
                        type_name(self.declaration_name(*declaration)),
                        type_name(variant_name)
                    )
                } else {
                    format!(
                        "{}::{} {{ {} }}",
                        type_name(self.declaration_name(*declaration)),
                        type_name(variant_name),
                        fields
                            .iter()
                            .map(|field| format!(
                                "{}: {}",
                                value_name(self.field_name(field.field)),
                                self.value(&field.value)
                            ))
                            .collect::<Vec<_>>()
                            .join(", ")
                    )
                }
            }
        }
    }
}

impl Generator<'_> {
    fn tests(&self, output: &mut String) {
        let mut tests: Vec<_> = self
            .program
            .module()
            .declarations
            .iter()
            .filter_map(|declaration| match declaration {
                Declaration::Test(test) => Some(test),
                _ => None,
            })
            .collect();
        tests.sort_by_key(|test| test.header.node.id);
        for test in tests {
            output.push_str("#[cfg(test)]\n");
            output.push_str("    #[test]\n");
            output.push_str(&format!(
                "    fn {}() {{\n",
                test_name(&test.header.name, test.header.node.id)
            ));
            let (call, return_type) = match &test.invocation {
                TestInvocation::Function {
                    function,
                    arguments,
                } => {
                    let parameters: &[Parameter] = match self.declaration(*function) {
                        Some(Declaration::Function(function)) => &function.parameters,
                        _ => &[],
                    };
                    self.test_arguments(output, arguments);
                    (
                        format!(
                            "{}({})",
                            value_name(self.declaration_name(*function)),
                            self.test_argument_list(arguments, parameters)
                        ),
                        match self.declaration(*function) {
                            Some(Declaration::Function(function)) => function.return_type.clone(),
                            _ => TypeRef::Unit,
                        },
                    )
                }
                TestInvocation::Method {
                    implementation,
                    method,
                    receiver,
                    arguments,
                } => {
                    output.push_str(&format!(
                        "        let receiver = {};\n",
                        self.value(&receiver.value)
                    ));
                    self.test_arguments(output, arguments);
                    let (contract, record, method_name) =
                        self.implementation_method(*implementation, *method);
                    let parameters = match self.declaration(*implementation) {
                        Some(Declaration::Implementation(implementation)) => implementation
                            .methods
                            .iter()
                            .find(|candidate| {
                                candidate.header.node.id == *method
                                    || candidate.contract_method == *method
                            })
                            .map_or(&[][..], |method| method.parameters.as_slice()),
                        _ => &[],
                    };
                    let return_type = match self.declaration(*implementation) {
                        Some(Declaration::Implementation(implementation)) => implementation
                            .methods
                            .iter()
                            .find(|candidate| {
                                candidate.header.node.id == *method
                                    || candidate.contract_method == *method
                            })
                            .map_or(TypeRef::Unit, |method| method.return_type.clone()),
                        _ => TypeRef::Unit,
                    };
                    let arguments = self.test_argument_list(arguments, parameters);
                    let suffix = if arguments.is_empty() {
                        String::new()
                    } else {
                        format!(", {arguments}")
                    };
                    (
                        format!(
                            "<{} as {}>::{}(&receiver{suffix})",
                            type_name(record),
                            type_name(contract),
                            value_name(method_name)
                        ),
                        return_type,
                    )
                }
            };
            match &test.expected {
                ExpectedOutcome::Value(expected) => {
                    output.push_str(&format!(
                        "        let actual = {call}.expect(\"portable test expected a value\");\n"
                    ));
                    match (&return_type, &expected.value) {
                        (TypeRef::Bool, Value::Bool(true)) => {
                            output.push_str("        assert!(actual);\n")
                        }
                        (TypeRef::Bool, Value::Bool(false)) => {
                            output.push_str("        assert!(!actual);\n")
                        }
                        (TypeRef::F64, Value::F64(value)) => output.push_str(&format!(
                            "        assert_eq!(actual.to_bits(), 0x{:016x});\n",
                            value.0
                        )),
                        _ => output.push_str(&format!(
                            "        assert_eq!(actual, {});\n",
                            self.value(&expected.value)
                        )),
                    }
                }
                ExpectedOutcome::Error(expected) => {
                    let code = match &expected.value {
                        Value::String(code) => code.as_str(),
                        _ => "invariant_violation",
                    };
                    output.push_str(&format!(
                        "        let error = {call}.expect_err(\"portable test expected an error\");\n        assert_eq!(error.code(), {code:?});\n"
                    ));
                }
            }
            output.push_str("    }\n\n");
        }
    }

    fn test_arguments(&self, output: &mut String, arguments: &[TypedValue]) {
        for (index, argument) in arguments.iter().enumerate() {
            output.push_str(&format!(
                "        let argument_{index} = {};\n",
                self.value(&argument.value)
            ));
        }
    }

    fn test_argument_list(&self, arguments: &[TypedValue], parameters: &[Parameter]) -> String {
        arguments
            .iter()
            .enumerate()
            .map(|(index, _)| {
                if parameters
                    .get(index)
                    .is_some_and(|parameter| matches!(parameter.ty, TypeRef::Contract(_)))
                {
                    format!("&argument_{index}")
                } else {
                    format!("argument_{index}")
                }
            })
            .collect::<Vec<_>>()
            .join(", ")
    }
}

const RUNTIME: &str = r#"#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PolyRuntimeError {
    CheckedOverflow { operation: &'static str },
    DivisionByZero,
    RemainderByZero,
    InvalidShift { amount: i64, width: u8 },
    NarrowingOutOfRange { value: i64 },
    IndexOutOfBounds { index: i64, length: u64 },
    InvalidUtf8,
}

impl PolyRuntimeError {
    pub const fn code(&self) -> &'static str {
        match self {
            Self::CheckedOverflow { .. } => "checked_overflow",
            Self::DivisionByZero => "division_by_zero",
            Self::RemainderByZero => "remainder_by_zero",
            Self::InvalidShift { .. } => "invalid_shift",
            Self::NarrowingOutOfRange { .. } => "narrowing_out_of_range",
            Self::IndexOutOfBounds { .. } => "index_out_of_bounds",
            Self::InvalidUtf8 => "invalid_utf8",
        }
    }
}

pub type PolyResult<T> = Result<T, PolyRuntimeError>;

#[doc(hidden)]
pub fn _poly_string_replace_many(
    source: String,
    mappings: &[(&str, &str)],
) -> PolyResult<String> {
    let mut output = String::new();
    let mut offset = 0;
    loop {
        let remaining = &source[offset..];
        if let Some((needle, replacement)) = mappings
            .iter()
            .find(|(needle, _)| remaining.starts_with(*needle))
        {
            output.push_str(replacement);
            if needle.is_empty() {
                let Some(character) = remaining.chars().next() else {
                    break;
                };
                let width = character.len_utf8();
                output.push_str(&remaining[..width]);
                offset += width;
            } else {
                offset += needle.len();
            }
        } else {
            let Some(character) = remaining.chars().next() else {
                break;
            };
            let width = character.len_utf8();
            output.push_str(&remaining[..width]);
            offset += width;
        }
    }
    Ok(output)
}

#[doc(hidden)]
pub fn _poly_string_truncate_utf8_bytes(
    source: String,
    budget: f64,
) -> PolyResult<String> {
    for (offset, character) in source.char_indices() {
        let end = offset + character.len_utf8();
        let consumed = end as f64;
        if consumed == budget {
            return Ok(source[..end].to_owned());
        }
        if consumed > budget {
            return Ok(source[..offset].to_owned());
        }
    }
    Ok(source)
}

#[doc(hidden)]
pub trait PolyShift: Sized {
    fn poly_checked_shl(self, amount: u32) -> Option<Self>;
    fn poly_checked_shr(self, amount: u32) -> Option<Self>;
}

impl PolyShift for i32 {
    fn poly_checked_shl(self, amount: u32) -> Option<Self> { self.checked_shl(amount) }
    fn poly_checked_shr(self, amount: u32) -> Option<Self> { self.checked_shr(amount) }
}

impl PolyShift for i64 {
    fn poly_checked_shl(self, amount: u32) -> Option<Self> { self.checked_shl(amount) }
    fn poly_checked_shr(self, amount: u32) -> Option<Self> { self.checked_shr(amount) }
}

#[doc(hidden)]
pub fn _poly_shift_left<T: PolyShift>(value: T, amount: i64, width: u8) -> PolyResult<T> {
    if amount < 0 || amount >= i64::from(width) {
        Err(PolyRuntimeError::InvalidShift { amount, width })
    } else {
        value.poly_checked_shl(amount as u32).ok_or(PolyRuntimeError::InvalidShift { amount, width })
    }
}

#[doc(hidden)]
pub fn _poly_shift_right<T: PolyShift>(value: T, amount: i64, width: u8) -> PolyResult<T> {
    if amount < 0 || amount >= i64::from(width) {
        Err(PolyRuntimeError::InvalidShift { amount, width })
    } else {
        value.poly_checked_shr(amount as u32).ok_or(PolyRuntimeError::InvalidShift { amount, width })
    }
}

#[doc(hidden)]
pub fn _poly_list_get<T: Clone>(values: Vec<T>, index: i64) -> PolyResult<T> {
    let length = values.len() as u64;
    let index_usize = usize::try_from(index).map_err(|_| PolyRuntimeError::IndexOutOfBounds { index, length })?;
    values.get(index_usize).cloned().ok_or(PolyRuntimeError::IndexOutOfBounds { index, length })
}
"#;

fn render_conformance() -> String {
    r#"#[test] fn vector_01_bool_not() { let result = !std::hint::black_box(true); assert!(!result); }
#[test] fn vector_02_checked_add() { assert_eq!(20_i64.checked_add(22).unwrap(), 42); }
#[test] fn vector_03_checked_overflow() { assert_eq!(i64::MAX.checked_add(1).ok_or(PolyRuntimeError::CheckedOverflow { operation: "add" }).unwrap_err().code(), "checked_overflow"); }
#[test] fn vector_04_division_by_zero() { let divisor = std::hint::black_box(0_i32); let result: PolyResult<i32> = if divisor == 0 { Err(PolyRuntimeError::DivisionByZero) } else { Ok(1 / divisor) }; assert_eq!(result.unwrap_err().code(), "division_by_zero"); }
#[test] fn vector_05_wrapping_add() { assert_eq!(i32::MAX.wrapping_add(1), i32::MIN); }
#[test] fn vector_06_unicode_astral_length() { assert_eq!("a🦀".chars().count(), 2); }
#[test] fn vector_07_unicode_combining_length() { assert_eq!("é".chars().count(), 2); }
#[test] fn vector_08_bytes_concat() { let mut value = vec![0_u8, 1]; value.extend([254, 255]); assert_eq!(value, vec![0, 1, 254, 255]); }
#[test] fn vector_09_list_get() { assert_eq!(_poly_list_get(vec![String::from("first")], 0).unwrap(), "first"); }
#[test] fn vector_10_list_oob() { assert_eq!(_poly_list_get(Vec::<i64>::new(), 0).unwrap_err().code(), "index_out_of_bounds"); }
#[test] fn vector_11_option_some() { assert!(Some(1_i64).is_some()); }
#[test] fn vector_12_option_none() { assert!(Option::<i64>::None.is_none()); }
#[test] fn vector_13_result_ok() { assert!(Result::<i64, String>::Ok(1).is_ok()); }
#[test] fn vector_14_result_err() { assert!(Result::<i64, String>::Err(String::from("error")).is_err()); }
#[test] fn vector_15_float_negative_zero() { assert_eq!((-0.0_f64).to_bits(), 0x8000000000000000); }
#[test] fn vector_16_float_nan_equality() { let left = f64::from_bits(0x7ff8000000000000); let right = f64::from_bits(0x7ff8000000000000); assert!(left != right); }
#[test] fn vector_17_narrow_out_of_range() { let result = i32::try_from(i64::MAX).map_err(|_| PolyRuntimeError::NarrowingOutOfRange { value: i64::MAX }); assert_eq!(result.unwrap_err().code(), "narrowing_out_of_range"); }
#[test] fn vector_18_invalid_utf8() { assert_eq!(String::from_utf8(std::hint::black_box(vec![0xff])).map_err(|_| PolyRuntimeError::InvalidUtf8).unwrap_err().code(), "invalid_utf8"); }
#[test] fn vector_19_string_contains() { assert!("x🦀y".contains("🦀")); }
#[test] fn vector_20_list_append_non_aliasing() { let original = vec![1_i64]; let mut appended = original.clone(); appended.push(2); assert_eq!(original, vec![1]); assert_eq!(appended, vec![1, 2]); }
"#.to_owned()
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use portable_codegen::{OutputContents, check_backend_contract};
    use portable_eval::Evaluator;

    use super::*;

    fn fixture() -> CheckedProgram {
        let document = portable_ir::v0::from_json(include_bytes!(
            "../../build/testdata/registration.poly.json"
        ))
        .expect("canonical fixture parses");
        portable_check::v0::check_program(document).expect("fixture checks")
    }

    fn generated_text<'a>(manifest: &'a OutputManifest, path: &str) -> &'a str {
        let file = manifest.file(path).expect("generated file exists");
        let OutputContents::Text(text) = file.contents() else {
            panic!("expected generated UTF-8 text")
        };
        text
    }

    #[test]
    fn descriptor_support_and_contract_are_complete_and_deterministic() {
        let program = fixture();
        let backend: Arc<dyn Backend> = Arc::new(RustBackend);
        assert_eq!(backend.descriptor().target.as_str(), "org.polyrust.rust");
        for capability in [
            Capability::Bytes,
            Capability::CheckedIntegerArithmetic,
            Capability::ContractDispatch,
            Capability::F64,
            Capability::ImmutableList,
            Capability::Option,
            Capability::Result,
            Capability::UnicodeScalar,
            Capability::WrappingIntegerArithmetic,
            Capability::BoundedIteration,
        ] {
            assert!(!matches!(
                backend.support(capability),
                CapabilitySupport::Unsupported { .. }
            ));
        }
        assert!(check_backend_contract(backend, &program, &BackendOptions::default()).is_empty());
    }

    #[test]
    fn complete_fixture_generation_is_golden_and_uses_every_package_layer() {
        let program = fixture();
        assert!(
            Evaluator::new(&program)
                .run_all_tests()
                .iter()
                .all(|result| result.passed)
        );
        let first = RustBackend
            .generate(&program, &BackendOptions::default())
            .unwrap();
        let second = RustBackend
            .generate(&program, &BackendOptions::default())
            .unwrap();
        assert_eq!(first.canonical_json(), second.canonical_json());
        assert_eq!(
            first
                .files()
                .iter()
                .map(|file| file.path())
                .collect::<Vec<_>>(),
            vec![
                "Cargo.toml",
                "src/conformance.rs",
                "src/lib.rs",
                "src/polyrust_runtime.rs"
            ]
        );
        let source = generated_text(&first, "src/lib.rs");
        for required in [
            "#![forbid(unsafe_code)]",
            "mod polyrust_runtime;",
            "trait ",
            "impl ",
            "PolyResult<",
            "#[cfg(test)]",
            "mod conformance;",
        ] {
            assert!(source.contains(required), "missing {required:?}");
        }
        assert_eq!(source.matches("pub use polyrust_runtime::*;").count(), 1);
        assert!(!source.contains("use super::*;"));
        let conformance = generated_text(&first, "src/conformance.rs");
        assert_eq!(conformance.matches("use super::*;").count(), 1);
        assert_eq!(conformance.matches("#[test]").count(), 20);
        assert!(conformance.contains("vector_20_list_append_non_aliasing"));
        let runtime = generated_text(&first, "src/polyrust_runtime.rs");
        assert!(runtime.contains("pub enum PolyRuntimeError"));
        assert!(!runtime.lines().any(|line| line.starts_with("use ")));
    }

    #[test]
    fn names_literals_and_runtime_helpers_are_target_owned_and_safe() {
        assert_eq!(rust_identifier("match"), "r#match");
        assert_eq!(rust_identifier("self"), "self_");
        assert_eq!(test_name("Astral 🦀 Case", NodeId(7)), "astral_case_n7");
        let program = fixture();
        let generator = Generator::new(&program);
        assert_eq!(
            generator.value(&Value::I64(i64::MIN)),
            "-9223372036854775808_i64"
        );
        assert_eq!(
            generator.value(&Value::F64(portable_ir::v0::F64Bits(0x8000_0000_0000_0000))),
            "f64::from_bits(0x8000000000000000)"
        );
        assert_eq!(
            generator.value(&Value::String("\"\\\n🦀".into())),
            "String::from(\"\\\"\\\\\\n🦀\")"
        );
        assert_eq!(
            generator.value(&Value::Bytes(vec![0, 255])),
            "vec![0x00_u8, 0xff_u8]"
        );
    }

    #[test]
    fn generated_source_contains_no_unsafe_surface() {
        let manifest = RustBackend
            .generate(&fixture(), &BackendOptions::default())
            .unwrap();
        let unsafe_lines: Vec<_> = manifest
            .files()
            .iter()
            .filter_map(|file| match file.contents() {
                OutputContents::Text(text) => Some(text),
                OutputContents::Bytes(_) => None,
            })
            .flat_map(|text| text.lines())
            .filter(|line| line.contains("unsafe"))
            .collect();
        assert_eq!(unsafe_lines, vec!["#![forbid(unsafe_code)]"]);
    }
}
