use portable_check::v0::{CheckedProgram, check_program};
use portable_diagnostics::{Diagnostic, DiagnosticCode, sort_diagnostics};
use portable_ir::v0::{
    AliasDeclaration, ConstantDeclaration, ContractDeclaration, Declaration, DeclarationHeader,
    Document, EnumDeclaration, EnumVariant, FieldDeclaration, FunctionDeclaration,
    ImplementationDeclaration, IrVersion, MemberHeader, MethodImplementation, MethodSignature,
    Module, Parameter as IrParameter, RecordDeclaration, TestDeclaration,
};

use crate::{
    AliasId, Block, BodyBuilder, BuildContext, ConstantExpr, ConstantId, ContractId,
    ContractMethodId, EnumFieldId, EnumId, EnumVariantId, Expected, FunctionId, ImplementationId,
    ImplementationMethodId, Invocation, RecordFieldId, RecordId, TestId, Type, Visibility,
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Parameter {
    name: String,
    ty: Type,
    documentation: Vec<String>,
}

impl Parameter {
    pub fn new(name: impl Into<String>, ty: Type) -> Self {
        Self {
            name: name.into(),
            ty,
            documentation: vec![],
        }
    }

    pub fn documented(
        name: impl Into<String>,
        ty: Type,
        documentation: impl IntoIterator<Item = impl Into<String>>,
    ) -> Self {
        Self {
            name: name.into(),
            ty,
            documentation: documentation.into_iter().map(Into::into).collect(),
        }
    }
}

pub struct ModuleBuilder {
    context: BuildContext,
    name: String,
    declarations: Vec<Declaration>,
    diagnostics: Vec<Diagnostic>,
}

impl ModuleBuilder {
    pub fn new(name: impl Into<String>) -> Self {
        let name = name.into();
        Self {
            context: BuildContext::new(name.clone()),
            name,
            declarations: vec![],
            diagnostics: vec![],
        }
    }

    pub fn alias(
        &mut self,
        name: impl Into<String>,
        visibility: Visibility,
        documentation: Vec<String>,
        target: Type,
    ) -> AliasId {
        let name = name.into();
        let header = self.declaration_header(&name, visibility, documentation);
        let id = AliasId::new(header.node.id);
        self.declarations.push(Declaration::Alias(AliasDeclaration {
            header,
            target: target.into_ir(),
        }));
        id
    }

    pub fn constant(
        &mut self,
        name: impl Into<String>,
        visibility: Visibility,
        documentation: Vec<String>,
        ty: Type,
        build: impl FnOnce(&mut BodyBuilder<'_>) -> ConstantExpr,
    ) -> ConstantId {
        let name = name.into();
        let header = self.declaration_header(&name, visibility, documentation);
        let id = ConstantId::new(header.node.id);
        let mut body = BodyBuilder {
            context: &mut self.context,
            scope: vec![format!("constant({name})")],
        };
        let value = build(&mut body);
        self.declarations
            .push(Declaration::Constant(ConstantDeclaration {
                header,
                ty: ty.into_ir(),
                value: value.into_ir(),
            }));
        id
    }

    pub fn record<R>(
        &mut self,
        name: impl Into<String>,
        visibility: Visibility,
        documentation: Vec<String>,
        configure: impl FnOnce(&mut RecordBuilder<'_>) -> R,
    ) -> (RecordId, R) {
        let name = name.into();
        let header = self.declaration_header(&name, visibility, documentation);
        let id = RecordId::new(header.node.id);
        let mut builder = RecordBuilder {
            context: &mut self.context,
            scope: vec![format!("record({name})")],
            fields: vec![],
        };
        let result = configure(&mut builder);
        self.declarations
            .push(Declaration::Record(RecordDeclaration {
                header,
                fields: builder.fields,
            }));
        (id, result)
    }

    pub fn enumeration<R>(
        &mut self,
        name: impl Into<String>,
        visibility: Visibility,
        documentation: Vec<String>,
        configure: impl FnOnce(&mut EnumBuilder<'_>) -> R,
    ) -> (EnumId, R) {
        let name = name.into();
        let header = self.declaration_header(&name, visibility, documentation);
        let id = EnumId::new(header.node.id);
        let mut builder = EnumBuilder {
            context: &mut self.context,
            scope: vec![format!("enum({name})")],
            variants: vec![],
        };
        let result = configure(&mut builder);
        self.declarations.push(Declaration::Enum(EnumDeclaration {
            header,
            variants: builder.variants,
        }));
        (id, result)
    }

    pub fn finish_unchecked(mut self) -> Result<Document, Vec<Diagnostic>> {
        sort_diagnostics(&mut self.diagnostics);
        if !self.diagnostics.is_empty() {
            return Err(self.diagnostics);
        }
        Ok(Document::new(
            IrVersion::CURRENT,
            Module {
                name: self.name,
                declarations: self.declarations,
            },
        ))
    }

    pub fn finish(self) -> Result<CheckedProgram, Vec<Diagnostic>> {
        self.finish_unchecked().and_then(check_program)
    }

    fn declaration_header(
        &mut self,
        name: &str,
        visibility: Visibility,
        documentation: Vec<String>,
    ) -> DeclarationHeader {
        DeclarationHeader {
            node: self.context.node(&[], format!("declaration({name})")),
            name: name.to_owned(),
            visibility,
            documentation,
        }
    }
}

pub struct RecordBuilder<'a> {
    context: &'a mut BuildContext,
    scope: Vec<String>,
    fields: Vec<FieldDeclaration>,
}

impl RecordBuilder<'_> {
    pub fn field(
        &mut self,
        name: impl Into<String>,
        ty: Type,
        documentation: Vec<String>,
    ) -> RecordFieldId {
        let name = name.into();
        let header = member_header(
            self.context,
            &self.scope,
            &format!("field({name})"),
            name,
            documentation,
        );
        let id = RecordFieldId::new(header.node.id);
        self.fields.push(FieldDeclaration {
            header,
            ty: ty.into_ir(),
        });
        id
    }
}

pub struct EnumBuilder<'a> {
    context: &'a mut BuildContext,
    scope: Vec<String>,
    variants: Vec<EnumVariant>,
}

impl EnumBuilder<'_> {
    pub fn variant<R>(
        &mut self,
        name: impl Into<String>,
        documentation: Vec<String>,
        configure: impl FnOnce(&mut EnumVariantBuilder<'_>) -> R,
    ) -> (EnumVariantId, R) {
        let name = name.into();
        let header = member_header(
            self.context,
            &self.scope,
            &format!("variant({name})"),
            name.clone(),
            documentation,
        );
        let id = EnumVariantId::new(header.node.id);
        let mut scope = self.scope.clone();
        scope.push(format!("variant({name})"));
        let mut builder = EnumVariantBuilder {
            context: self.context,
            scope,
            fields: vec![],
        };
        let result = configure(&mut builder);
        self.variants.push(EnumVariant {
            header,
            fields: builder.fields,
        });
        (id, result)
    }
}

pub struct EnumVariantBuilder<'a> {
    context: &'a mut BuildContext,
    scope: Vec<String>,
    fields: Vec<FieldDeclaration>,
}

impl EnumVariantBuilder<'_> {
    pub fn field(
        &mut self,
        name: impl Into<String>,
        ty: Type,
        documentation: Vec<String>,
    ) -> EnumFieldId {
        let name = name.into();
        let header = member_header(
            self.context,
            &self.scope,
            &format!("field({name})"),
            name,
            documentation,
        );
        let id = EnumFieldId::new(header.node.id);
        self.fields.push(FieldDeclaration {
            header,
            ty: ty.into_ir(),
        });
        id
    }
}

impl ModuleBuilder {
    pub fn contract<R>(
        &mut self,
        name: impl Into<String>,
        visibility: Visibility,
        documentation: Vec<String>,
        configure: impl FnOnce(&mut ContractBuilder<'_>) -> R,
    ) -> (ContractId, R) {
        let name = name.into();
        let header = self.declaration_header(&name, visibility, documentation);
        let id = ContractId::new(header.node.id);
        let mut builder = ContractBuilder {
            context: &mut self.context,
            diagnostics: &mut self.diagnostics,
            scope: vec![format!("contract({name})")],
            methods: vec![],
        };
        let result = configure(&mut builder);
        self.declarations
            .push(Declaration::Contract(ContractDeclaration {
                header,
                methods: builder.methods,
            }));
        (id, result)
    }

    pub fn implementation<R>(
        &mut self,
        name: impl Into<String>,
        visibility: Visibility,
        documentation: Vec<String>,
        contract: ContractId,
        record: RecordId,
        configure: impl FnOnce(&mut ImplementationBuilder<'_>) -> R,
    ) -> (ImplementationId, R) {
        let name = name.into();
        let header = self.declaration_header(&name, visibility, documentation);
        let id = ImplementationId::new(header.node.id);
        let mut builder = ImplementationBuilder {
            context: &mut self.context,
            diagnostics: &mut self.diagnostics,
            scope: vec![format!("implementation({name})")],
            methods: vec![],
        };
        let result = configure(&mut builder);
        self.declarations
            .push(Declaration::Implementation(ImplementationDeclaration {
                header,
                contract: contract.node_id(),
                record: record.node_id(),
                methods: builder.methods,
            }));
        (id, result)
    }

    pub fn function(
        &mut self,
        name: impl Into<String>,
        visibility: Visibility,
        documentation: Vec<String>,
        configure: impl FnOnce(&mut CallableBuilder<'_>),
    ) -> FunctionId {
        let name = name.into();
        let header = self.declaration_header(&name, visibility, documentation);
        let id = FunctionId::new(header.node.id);
        let mut builder = CallableBuilder::new(
            &mut self.context,
            &mut self.diagnostics,
            vec![format!("function({name})")],
        );
        configure(&mut builder);
        if let Some((parameters, return_type, body)) = builder.finish("function") {
            self.declarations
                .push(Declaration::Function(FunctionDeclaration {
                    header,
                    parameters,
                    return_type,
                    body,
                }));
        }
        id
    }

    pub fn portable_test(
        &mut self,
        name: impl Into<String>,
        visibility: Visibility,
        documentation: Vec<String>,
        invocation: Invocation,
        expected: Expected,
    ) -> TestId {
        let name = name.into();
        let header = self.declaration_header(&name, visibility, documentation);
        let id = TestId::new(header.node.id);
        self.declarations.push(Declaration::Test(TestDeclaration {
            header,
            invocation: invocation.into_ir(),
            expected: expected.into_ir(),
        }));
        id
    }
}

pub struct ContractBuilder<'a> {
    context: &'a mut BuildContext,
    diagnostics: &'a mut Vec<Diagnostic>,
    scope: Vec<String>,
    methods: Vec<MethodSignature>,
}

impl ContractBuilder<'_> {
    pub fn method(
        &mut self,
        name: impl Into<String>,
        documentation: Vec<String>,
        parameters: Vec<Parameter>,
        return_type: Option<Type>,
    ) -> ContractMethodId {
        let name = name.into();
        let header = member_header(
            self.context,
            &self.scope,
            &format!("method({name})"),
            name.clone(),
            documentation,
        );
        let id = ContractMethodId::new(header.node.id);
        let mut scope = self.scope.clone();
        scope.push(format!("method({name})"));
        let parameters = build_parameters(self.context, &scope, parameters);
        match return_type {
            Some(return_type) => self.methods.push(MethodSignature {
                header,
                parameters,
                return_type: return_type.into_ir(),
            }),
            None => self.diagnostics.push(Diagnostic::error(
                DiagnosticCode::InvalidStructure,
                format!("contract method {name:?} is missing a return type"),
                header.node.source,
            )),
        }
        id
    }
}

pub struct ImplementationBuilder<'a> {
    context: &'a mut BuildContext,
    diagnostics: &'a mut Vec<Diagnostic>,
    scope: Vec<String>,
    methods: Vec<MethodImplementation>,
}

impl ImplementationBuilder<'_> {
    pub fn method<R>(
        &mut self,
        name: impl Into<String>,
        contract_method: ContractMethodId,
        documentation: Vec<String>,
        configure: impl FnOnce(&mut CallableBuilder<'_>) -> R,
    ) -> (ImplementationMethodId, R) {
        let name = name.into();
        let header = member_header(
            self.context,
            &self.scope,
            &format!("method({name})"),
            name.clone(),
            documentation,
        );
        let id = ImplementationMethodId::new(header.node.id);
        let mut scope = self.scope.clone();
        scope.push(format!("method({name})"));
        let mut builder = CallableBuilder::new(self.context, self.diagnostics, scope);
        let result = configure(&mut builder);
        if let Some((parameters, return_type, body)) = builder.finish("implementation method") {
            self.methods.push(MethodImplementation {
                header,
                contract_method: contract_method.node_id(),
                parameters,
                return_type,
                body,
            });
        }
        (id, result)
    }
}

pub struct CallableBuilder<'a> {
    context: &'a mut BuildContext,
    diagnostics: &'a mut Vec<Diagnostic>,
    scope: Vec<String>,
    parameters: Vec<Parameter>,
    return_type: Option<Type>,
    body: Option<Block>,
}

impl<'a> CallableBuilder<'a> {
    fn new(
        context: &'a mut BuildContext,
        diagnostics: &'a mut Vec<Diagnostic>,
        scope: Vec<String>,
    ) -> Self {
        Self {
            context,
            diagnostics,
            scope,
            parameters: vec![],
            return_type: None,
            body: None,
        }
    }

    pub fn parameter(&mut self, parameter: Parameter) -> &mut Self {
        self.parameters.push(parameter);
        self
    }

    pub fn returns(&mut self, ty: Type) -> &mut Self {
        self.return_type = Some(ty);
        self
    }

    pub fn body(&mut self, build: impl FnOnce(&mut BodyBuilder<'_>) -> Block) -> &mut Self {
        let mut body = BodyBuilder {
            context: self.context,
            scope: self.scope.clone(),
        };
        self.body = Some(build(&mut body));
        self
    }

    fn finish(
        mut self,
        kind: &str,
    ) -> Option<(
        Vec<IrParameter>,
        portable_ir::v0::TypeRef,
        portable_ir::v0::Block,
    )> {
        let source = self.context.node(&self.scope, "incomplete");
        let mut complete = true;
        if self.return_type.is_none() {
            self.diagnostics.push(Diagnostic::error(
                DiagnosticCode::InvalidStructure,
                format!("{kind} is missing a return type"),
                source.source.clone(),
            ));
            complete = false;
        }
        if self.body.is_none() {
            self.diagnostics.push(Diagnostic::error(
                DiagnosticCode::InvalidStructure,
                format!("{kind} is missing a body"),
                source.source,
            ));
            complete = false;
        }
        if !complete {
            return None;
        }
        let parameters = build_parameters(self.context, &self.scope, self.parameters);
        Some((
            parameters,
            self.return_type.take()?.into_ir(),
            self.body.take()?.into_ir(),
        ))
    }
}

fn build_parameters(
    context: &mut BuildContext,
    scope: &[String],
    parameters: Vec<Parameter>,
) -> Vec<IrParameter> {
    parameters
        .into_iter()
        .map(|parameter| IrParameter {
            header: member_header(
                context,
                scope,
                &format!("parameter({})", parameter.name),
                parameter.name,
                parameter.documentation,
            ),
            ty: parameter.ty.into_ir(),
        })
        .collect()
}

fn member_header(
    context: &mut BuildContext,
    scope: &[String],
    role: &str,
    name: String,
    documentation: Vec<String>,
) -> MemberHeader {
    MemberHeader {
        node: context.node(scope, role),
        name,
        documentation,
    }
}
