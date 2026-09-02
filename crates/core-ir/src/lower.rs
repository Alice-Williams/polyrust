use std::collections::BTreeMap;

use portable_check::v0::CheckedProgram;
use portable_diagnostics::{Diagnostic, DiagnosticCode};
use portable_ir::v0::{
    Block, ConstantExpression, Declaration, DeclarationHeader, ExpectedOutcome, Expression,
    Intrinsic, MemberHeader, MethodDispatch, NodeId, NodeMeta, Pattern, Statement, TestInvocation,
    TypeRef, TypedValue, Value,
};

use crate::*;

#[derive(Clone, Copy, Debug, Default)]
pub struct CanonicalCoreLowerer;

impl portable_codegen::CoreLowerer for CanonicalCoreLowerer {
    type Core = CoreProgram;

    fn lower_core(&self, program: &CheckedProgram) -> Result<Self::Core, Vec<Diagnostic>> {
        lower_checked(program)
    }

    fn verify_core(&self, core: &Self::Core) -> Result<(), Vec<Diagnostic>> {
        crate::verify_core(core)
    }
}

pub fn lower_checked(checked: &CheckedProgram) -> Result<CoreProgram, Vec<Diagnostic>> {
    let mut lowering = Lowering::new(checked);
    let program = lowering.lower_program()?;
    crate::verify_core(&program)?;
    Ok(program)
}

#[derive(Default)]
struct SourceIndex<'a> {
    declarations: BTreeMap<NodeId, &'a Declaration>,
    constants: BTreeMap<NodeId, CoreConstantId>,
    aliases: BTreeMap<NodeId, CoreAliasId>,
    records: BTreeMap<NodeId, CoreRecordId>,
    enums: BTreeMap<NodeId, CoreEnumId>,
    variants: BTreeMap<NodeId, CoreVariantId>,
    fields: BTreeMap<NodeId, CoreFieldId>,
    interfaces: BTreeMap<NodeId, CoreInterfaceId>,
    interface_methods: BTreeMap<NodeId, CoreInterfaceMethodId>,
    implementations: BTreeMap<NodeId, CoreImplementationId>,
    implementation_methods: BTreeMap<NodeId, CoreImplementationMethodId>,
    functions: BTreeMap<NodeId, CoreFunctionId>,
    tests: BTreeMap<NodeId, CoreTestId>,
}

impl<'a> SourceIndex<'a> {
    fn new(declarations: &[&'a Declaration]) -> Self {
        let mut index = Self::default();
        for declaration in declarations {
            index
                .declarations
                .insert(declaration.header().node.id, declaration);
            match declaration {
                Declaration::Constant(value) => {
                    let id = CoreConstantId::from_index(index.constants.len());
                    index.constants.insert(value.header.node.id, id);
                }
                Declaration::Alias(value) => {
                    let id = CoreAliasId::from_index(index.aliases.len());
                    index.aliases.insert(value.header.node.id, id);
                }
                Declaration::Record(value) => {
                    let id = CoreRecordId::from_index(index.records.len());
                    index.records.insert(value.header.node.id, id);
                }
                Declaration::Enum(value) => {
                    let id = CoreEnumId::from_index(index.enums.len());
                    index.enums.insert(value.header.node.id, id);
                }
                Declaration::Contract(value) => {
                    let id = CoreInterfaceId::from_index(index.interfaces.len());
                    index.interfaces.insert(value.header.node.id, id);
                }
                Declaration::Implementation(value) => {
                    let id = CoreImplementationId::from_index(index.implementations.len());
                    index.implementations.insert(value.header.node.id, id);
                }
                Declaration::Function(value) => {
                    let id = CoreFunctionId::from_index(index.functions.len());
                    index.functions.insert(value.header.node.id, id);
                }
                Declaration::Test(value) => {
                    let id = CoreTestId::from_index(index.tests.len());
                    index.tests.insert(value.header.node.id, id);
                }
            }
        }
        for declaration in declarations {
            match declaration {
                Declaration::Record(record) => {
                    for field in &record.fields {
                        let id = CoreFieldId::from_index(index.fields.len());
                        index.fields.insert(field.header.node.id, id);
                    }
                }
                Declaration::Enum(enumeration) => {
                    for variant in &enumeration.variants {
                        let variant_id = CoreVariantId::from_index(index.variants.len());
                        index.variants.insert(variant.header.node.id, variant_id);
                        for field in &variant.fields {
                            let id = CoreFieldId::from_index(index.fields.len());
                            index.fields.insert(field.header.node.id, id);
                        }
                    }
                }
                Declaration::Contract(interface) => {
                    for method in &interface.methods {
                        let id = CoreInterfaceMethodId::from_index(index.interface_methods.len());
                        index.interface_methods.insert(method.header.node.id, id);
                    }
                }
                Declaration::Implementation(implementation) => {
                    for method in &implementation.methods {
                        let id = CoreImplementationMethodId::from_index(
                            index.implementation_methods.len(),
                        );
                        index
                            .implementation_methods
                            .insert(method.header.node.id, id);
                    }
                }
                Declaration::Constant(_)
                | Declaration::Alias(_)
                | Declaration::Function(_)
                | Declaration::Test(_) => {}
            }
        }
        index
    }
}

struct Lowering<'a> {
    checked: &'a CheckedProgram,
    ordered: Vec<&'a Declaration>,
    index: SourceIndex<'a>,
    types: CoreTypeArena,
    interned_types: BTreeMap<CoreType, CoreTypeId>,
    locals: Vec<CoreLocal>,
    local_ids: BTreeMap<(NodeId, String), CoreLocalId>,
    expressions: CoreExprArena,
    blocks: CoreBlockArena,
}

impl<'a> Lowering<'a> {
    fn new(checked: &'a CheckedProgram) -> Self {
        let mut ordered = checked.module().declarations.iter().collect::<Vec<_>>();
        ordered.sort_by_key(|declaration| {
            (
                declaration_rank(declaration),
                declaration.header().name.as_str(),
                declaration.header().node.id,
            )
        });
        let index = SourceIndex::new(&ordered);
        Self {
            checked,
            ordered,
            index,
            types: CoreTypeArena::default(),
            interned_types: BTreeMap::new(),
            locals: vec![],
            local_ids: BTreeMap::new(),
            expressions: CoreExprArena::default(),
            blocks: CoreBlockArena::default(),
        }
    }

    fn lower_program(&mut self) -> Result<CoreProgram, Vec<Diagnostic>> {
        let declarations = self
            .ordered
            .iter()
            .map(|declaration| match declaration {
                Declaration::Constant(value) => {
                    CoreDeclaration::Constant(self.index.constants[&value.header.node.id])
                }
                Declaration::Alias(value) => {
                    CoreDeclaration::Alias(self.index.aliases[&value.header.node.id])
                }
                Declaration::Record(value) => {
                    CoreDeclaration::Record(self.index.records[&value.header.node.id])
                }
                Declaration::Enum(value) => {
                    CoreDeclaration::Enum(self.index.enums[&value.header.node.id])
                }
                Declaration::Contract(value) => {
                    CoreDeclaration::Interface(self.index.interfaces[&value.header.node.id])
                }
                Declaration::Implementation(value) => CoreDeclaration::Implementation(
                    self.index.implementations[&value.header.node.id],
                ),
                Declaration::Function(value) => {
                    CoreDeclaration::Function(self.index.functions[&value.header.node.id])
                }
                Declaration::Test(value) => {
                    CoreDeclaration::Test(self.index.tests[&value.header.node.id])
                }
            })
            .collect();

        let ordered = self.ordered.clone();
        let mut constants = Vec::new();
        let mut aliases = Vec::new();
        let mut records = Vec::new();
        let mut enums = Vec::new();
        let mut variants = Vec::new();
        let mut fields = Vec::new();
        let mut interfaces = Vec::new();
        let mut interface_methods = Vec::new();
        let mut implementations = Vec::new();
        let mut implementation_methods = Vec::new();
        let mut functions = Vec::new();
        let mut tests = Vec::new();

        for declaration in ordered {
            match declaration {
                Declaration::Constant(value) => {
                    let ty = self.lower_type(&value.ty, &value.header.node)?;
                    let lowered = self.lower_constant(&value.value, Some(ty))?;
                    constants.push(CoreConstant {
                        header: declaration_header(&value.header),
                        ty,
                        value: lowered,
                    });
                }
                Declaration::Alias(value) => {
                    let target = self.lower_type(&value.target, &value.header.node)?;
                    aliases.push(CoreAlias {
                        header: declaration_header(&value.header),
                        target,
                    });
                }
                Declaration::Record(value) => {
                    let record = self.index.records[&value.header.node.id];
                    let mut record_fields = Vec::new();
                    for source_field in &value.fields {
                        let id = self.index.fields[&source_field.header.node.id];
                        record_fields.push(id);
                        let ty = self.lower_type(&source_field.ty, &source_field.header.node)?;
                        fields.push(CoreField {
                            header: member_header(&source_field.header),
                            owner: CoreFieldOwner::Record(record),
                            ty,
                        });
                    }
                    records.push(CoreRecord {
                        header: declaration_header(&value.header),
                        fields: record_fields,
                    });
                }
                Declaration::Enum(value) => {
                    let enumeration = self.index.enums[&value.header.node.id];
                    let mut enum_variants = Vec::new();
                    for source_variant in &value.variants {
                        let variant = self.index.variants[&source_variant.header.node.id];
                        enum_variants.push(variant);
                        let mut variant_fields = Vec::new();
                        for source_field in &source_variant.fields {
                            let id = self.index.fields[&source_field.header.node.id];
                            variant_fields.push(id);
                            let ty =
                                self.lower_type(&source_field.ty, &source_field.header.node)?;
                            fields.push(CoreField {
                                header: member_header(&source_field.header),
                                owner: CoreFieldOwner::Variant(variant),
                                ty,
                            });
                        }
                        variants.push(CoreVariant {
                            header: member_header(&source_variant.header),
                            enumeration,
                            fields: variant_fields,
                        });
                    }
                    enums.push(CoreEnum {
                        header: declaration_header(&value.header),
                        variants: enum_variants,
                    });
                }
                Declaration::Contract(value) => {
                    let interface = self.index.interfaces[&value.header.node.id];
                    let mut methods = Vec::new();
                    for source_method in &value.methods {
                        let method = self.index.interface_methods[&source_method.header.node.id];
                        methods.push(method);
                        let parameters = self.lower_parameters(&source_method.parameters, false)?;
                        let return_type = self
                            .lower_type(&source_method.return_type, &source_method.header.node)?;
                        interface_methods.push(CoreInterfaceMethod {
                            header: member_header(&source_method.header),
                            interface,
                            parameters,
                            return_type,
                        });
                    }
                    interfaces.push(CoreInterface {
                        header: declaration_header(&value.header),
                        methods,
                    });
                }
                Declaration::Implementation(value) => {
                    let implementation = self.index.implementations[&value.header.node.id];
                    let interface = self.interface(value.contract, &value.header.node)?;
                    let record = self.record(value.record, &value.header.node)?;
                    let mut methods = Vec::new();
                    for source_method in &value.methods {
                        let method =
                            self.index.implementation_methods[&source_method.header.node.id];
                        methods.push(method);
                        let parameters = self.lower_parameters(&source_method.parameters, true)?;
                        let body = self.lower_block(&source_method.body, Some(record))?;
                        let interface_method = self.interface_method(
                            source_method.contract_method,
                            &source_method.header.node,
                        )?;
                        let return_type = self
                            .lower_type(&source_method.return_type, &source_method.header.node)?;
                        implementation_methods.push(CoreImplementationMethod {
                            header: member_header(&source_method.header),
                            implementation,
                            interface_method,
                            parameters,
                            return_type,
                            body,
                        });
                    }
                    implementations.push(CoreImplementation {
                        header: declaration_header(&value.header),
                        interface,
                        record,
                        methods,
                    });
                }
                Declaration::Function(value) => {
                    let parameters = self.lower_parameters(&value.parameters, true)?;
                    let body = self.lower_block(&value.body, None)?;
                    let return_type = self.lower_type(&value.return_type, &value.header.node)?;
                    functions.push(CoreFunction {
                        header: declaration_header(&value.header),
                        parameters,
                        return_type,
                        body,
                    });
                }
                Declaration::Test(value) => {
                    let invocation =
                        self.lower_test_invocation(&value.invocation, &value.header.node)?;
                    let expected = self.lower_expected(&value.expected, &value.header.node)?;
                    tests.push(CoreTest {
                        header: declaration_header(&value.header),
                        invocation,
                        expected,
                    });
                }
            }
        }

        Ok(CoreProgram::new(
            CoreModule {
                name: self.checked.module().name.clone(),
                declarations,
            },
            std::mem::take(&mut self.types),
            constants,
            aliases,
            records,
            enums,
            variants,
            fields,
            interfaces,
            interface_methods,
            implementations,
            implementation_methods,
            functions,
            tests,
            std::mem::take(&mut self.locals),
            std::mem::take(&mut self.expressions),
            std::mem::take(&mut self.blocks),
        ))
    }

    fn lower_parameters(
        &mut self,
        parameters: &[portable_ir::v0::Parameter],
        bind: bool,
    ) -> Result<Vec<CoreParameter>, Vec<Diagnostic>> {
        parameters
            .iter()
            .map(|parameter| {
                let ty = self.lower_type(&parameter.ty, &parameter.header.node)?;
                let local = bind.then(|| {
                    self.local(
                        &parameter.header.node,
                        &parameter.header.name,
                        ty,
                        CoreLocalKind::Parameter,
                    )
                });
                Ok(CoreParameter {
                    header: member_header(&parameter.header),
                    ty,
                    local,
                })
            })
            .collect()
    }

    fn local(
        &mut self,
        node: &NodeMeta,
        name: &str,
        ty: CoreTypeId,
        kind: CoreLocalKind,
    ) -> CoreLocalId {
        let key = (node.id, name.to_owned());
        if let Some(id) = self.local_ids.get(&key) {
            return *id;
        }
        let id = CoreLocalId::from_index(self.locals.len());
        self.locals.push(CoreLocal {
            name: name.to_owned(),
            ty,
            kind,
            source: node.source.clone(),
        });
        self.local_ids.insert(key, id);
        id
    }

    fn lower_type(&mut self, ty: &TypeRef, node: &NodeMeta) -> Result<CoreTypeId, Vec<Diagnostic>> {
        let ty = match ty {
            TypeRef::Unit => CoreType::Unit,
            TypeRef::Bool => CoreType::Bool,
            TypeRef::I32 => CoreType::I32,
            TypeRef::I64 => CoreType::I64,
            TypeRef::F64 => CoreType::F64,
            TypeRef::Char => CoreType::Char,
            TypeRef::String => CoreType::String,
            TypeRef::Bytes => CoreType::Bytes,
            TypeRef::List(inner) => CoreType::List(self.lower_type(inner, node)?),
            TypeRef::Option(inner) => CoreType::Option(self.lower_type(inner, node)?),
            TypeRef::Result { ok, error } => CoreType::Result {
                ok: self.lower_type(ok, node)?,
                error: self.lower_type(error, node)?,
            },
            TypeRef::Contract(id) => CoreType::Interface(self.interface(*id, node)?),
            TypeRef::Named(id) => match self.index.declarations.get(id).copied() {
                Some(Declaration::Alias(alias)) => return self.lower_type(&alias.target, node),
                Some(Declaration::Record(_)) => CoreType::Record(self.record(*id, node)?),
                Some(Declaration::Enum(_)) => CoreType::Enum(self.enumeration(*id, node)?),
                _ => {
                    return Err(self.error(node, format!("named type node {} is invalid", id.0)));
                }
            },
        };
        self.intern(ty)
    }

    fn intern(&mut self, ty: CoreType) -> Result<CoreTypeId, Vec<Diagnostic>> {
        if let Some(id) = self.interned_types.get(&ty) {
            return Ok(*id);
        }
        let id = self.types.push(ty.clone());
        self.interned_types.insert(ty, id);
        Ok(id)
    }

    fn lower_constant(
        &mut self,
        expression: &ConstantExpression,
        expected: Option<CoreTypeId>,
    ) -> Result<CoreConstantExpr, Vec<Diagnostic>> {
        let source_node = constant_node(expression);
        let source = source_node.source.clone();
        let kind = match expression {
            ConstantExpression::Literal { value, node } => {
                let ty = expected.map_or_else(|| self.infer_value_type(value, node), Ok)?;
                CoreConstantExprKind::Literal(self.lower_value(value, ty, node)?)
            }
            ConstantExpression::Reference { declaration, node } => {
                CoreConstantExprKind::Constant(self.constant(*declaration, node)?)
            }
            ConstantExpression::Record {
                declaration,
                fields,
                node,
            } => CoreConstantExprKind::Record {
                record: self.record(*declaration, node)?,
                fields: fields
                    .iter()
                    .map(|value| {
                        let field = self.field(value.field, node)?;
                        let ty = self.source_field_type(field, node)?;
                        Ok(CoreConstantField {
                            field,
                            value: self.lower_constant(&value.value, Some(ty))?,
                        })
                    })
                    .collect::<Result<_, Vec<Diagnostic>>>()?,
            },
            ConstantExpression::Enum {
                declaration,
                variant,
                fields,
                node,
            } => CoreConstantExprKind::Enum {
                enumeration: self.enumeration(*declaration, node)?,
                variant: self.variant(*variant, node)?,
                fields: fields
                    .iter()
                    .map(|value| {
                        let field = self.field(value.field, node)?;
                        let ty = self.source_field_type(field, node)?;
                        Ok(CoreConstantField {
                            field,
                            value: self.lower_constant(&value.value, Some(ty))?,
                        })
                    })
                    .collect::<Result<_, Vec<Diagnostic>>>()?,
            },
            ConstantExpression::Some { value, node } => {
                let inner = self.type_inner(
                    expected
                        .ok_or_else(|| self.error(node, "Some constant has no expected type"))?,
                    node,
                    "Some",
                )?;
                CoreConstantExprKind::Some(Box::new(self.lower_constant(value, Some(inner))?))
            }
            ConstantExpression::None { inner_type, node } => CoreConstantExprKind::None {
                inner: self.lower_type(inner_type, node)?,
            },
            ConstantExpression::Ok {
                value,
                error_type,
                node,
            } => {
                let expected =
                    expected.ok_or_else(|| self.error(node, "Ok constant has no expected type"))?;
                let (ok, _) = self.result_types(expected, node)?;
                CoreConstantExprKind::Ok {
                    value: Box::new(self.lower_constant(value, Some(ok))?),
                    error: self.lower_type(error_type, node)?,
                }
            }
            ConstantExpression::Err {
                value,
                ok_type,
                node,
            } => {
                let expected = expected
                    .ok_or_else(|| self.error(node, "Err constant has no expected type"))?;
                let (_, error) = self.result_types(expected, node)?;
                CoreConstantExprKind::Err {
                    value: Box::new(self.lower_constant(value, Some(error))?),
                    ok: self.lower_type(ok_type, node)?,
                }
            }
            ConstantExpression::List {
                element_type,
                elements,
                node,
            } => {
                let element = self.lower_type(element_type, node)?;
                CoreConstantExprKind::List {
                    element,
                    elements: elements
                        .iter()
                        .map(|value| self.lower_constant(value, Some(element)))
                        .collect::<Result<_, _>>()?,
                }
            }
            ConstantExpression::Intrinsic {
                operation,
                arguments,
                node,
            } => {
                let mut lowered = Vec::with_capacity(arguments.len());
                for argument in arguments {
                    let ty = self.infer_constant_type(argument)?;
                    lowered.push(self.lower_constant(argument, Some(ty))?);
                }
                CoreConstantExprKind::Intrinsic(Box::new(lower_intrinsic(
                    *operation, lowered, node,
                )?))
            }
        };
        Ok(CoreConstantExpr { source, kind })
    }

    fn infer_constant_type(
        &mut self,
        expression: &ConstantExpression,
    ) -> Result<CoreTypeId, Vec<Diagnostic>> {
        let node = constant_node(expression);
        match expression {
            ConstantExpression::Literal { value, .. } => self.infer_value_type(value, node),
            ConstantExpression::Reference { declaration, .. } => {
                let source = self
                    .index
                    .declarations
                    .get(declaration)
                    .copied()
                    .ok_or_else(|| self.error(node, "constant reference is missing"))?;
                match source {
                    Declaration::Constant(value) => self.lower_type(&value.ty, node),
                    _ => Err(self.error(node, "constant reference has the wrong declaration kind")),
                }
            }
            ConstantExpression::Record { declaration, .. } => {
                let record = self.record(*declaration, node)?;
                self.intern(CoreType::Record(record))
            }
            ConstantExpression::Enum { declaration, .. } => {
                let enumeration = self.enumeration(*declaration, node)?;
                self.intern(CoreType::Enum(enumeration))
            }
            ConstantExpression::Some { value, .. } => {
                let inner = self.infer_constant_type(value)?;
                self.intern(CoreType::Option(inner))
            }
            ConstantExpression::None { inner_type, .. } => {
                let inner = self.lower_type(inner_type, node)?;
                self.intern(CoreType::Option(inner))
            }
            ConstantExpression::Ok {
                value, error_type, ..
            } => {
                let ok = self.infer_constant_type(value)?;
                let error = self.lower_type(error_type, node)?;
                self.intern(CoreType::Result { ok, error })
            }
            ConstantExpression::Err { value, ok_type, .. } => {
                let ok = self.lower_type(ok_type, node)?;
                let error = self.infer_constant_type(value)?;
                self.intern(CoreType::Result { ok, error })
            }
            ConstantExpression::List { element_type, .. } => {
                let element = self.lower_type(element_type, node)?;
                self.intern(CoreType::List(element))
            }
            ConstantExpression::Intrinsic {
                operation,
                arguments,
                ..
            } => {
                let arguments = arguments
                    .iter()
                    .map(|argument| self.infer_constant_type(argument))
                    .collect::<Result<Vec<_>, _>>()?;
                self.intrinsic_result_type(*operation, &arguments, node)
            }
        }
    }

    fn lower_block(
        &mut self,
        block: &Block,
        self_record: Option<CoreRecordId>,
    ) -> Result<CoreBlockId, Vec<Diagnostic>> {
        let mut statements = Vec::new();
        for statement in &block.statements {
            statements.push(match statement {
                Statement::Let {
                    node,
                    name,
                    annotation,
                    value,
                } => {
                    let value = self.lower_expression(value, self_record)?;
                    let ty = match annotation {
                        Some(ty) => self.lower_type(ty, node)?,
                        None => self.expressions.get(value).expect("new expression").ty,
                    };
                    CoreStatement::Let {
                        source: node.source.clone(),
                        local: self.local(node, name, ty, CoreLocalKind::Let),
                        value,
                    }
                }
                Statement::ForEach {
                    node,
                    binding,
                    iterable,
                    body,
                } => {
                    let iterable = self.lower_expression(iterable, self_record)?;
                    let iterable_type = self.expressions.get(iterable).expect("new expression").ty;
                    let element = match self.types.get(iterable_type) {
                        Some(CoreType::List(element)) => *element,
                        _ => return Err(self.error(node, "for-each input is not a Core list")),
                    };
                    let binding = self.local(node, binding, element, CoreLocalKind::ForEach);
                    let body = self.lower_block(body, self_record)?;
                    CoreStatement::ForEach {
                        source: node.source.clone(),
                        binding,
                        iterable,
                        body,
                    }
                }
                Statement::Return { node, value } => CoreStatement::Return {
                    source: node.source.clone(),
                    value: value
                        .as_ref()
                        .map(|value| self.lower_expression(value, self_record))
                        .transpose()?,
                },
                Statement::Expression { node, value } => CoreStatement::Evaluate {
                    source: node.source.clone(),
                    value: self.lower_expression(value, self_record)?,
                },
            });
        }
        let result = block
            .result
            .as_deref()
            .map(|value| self.lower_expression(value, self_record))
            .transpose()?;
        let result_type = match result {
            Some(result) => self.expressions.get(result).expect("new expression").ty,
            None => self.lower_type(&TypeRef::Unit, &block.node)?,
        };
        Ok(self.blocks.push(CoreBlock {
            source: block.node.source.clone(),
            statements,
            result,
            result_type,
        }))
    }

    fn lower_expression(
        &mut self,
        expression: &Expression,
        self_record: Option<CoreRecordId>,
    ) -> Result<CoreExprId, Vec<Diagnostic>> {
        let node = expression.node();
        let source_type = self
            .checked
            .expression_type(node.id)
            .cloned()
            .ok_or_else(|| self.error(node, "checked expression has no type"))?;
        let ty = self.lower_type(&source_type, node)?;
        let kind = match expression {
            Expression::Literal { value, .. } => {
                CoreExprKind::Literal(self.lower_value(value, ty, node)?)
            }
            Expression::Local { name, .. } => {
                let resolved = self
                    .checked
                    .resolved_local(node.id)
                    .ok_or_else(|| self.error(node, "checked local has no resolved symbol"))?;
                let key = (resolved.node_id(), name.clone());
                CoreExprKind::Local(
                    self.local_ids
                        .get(&key)
                        .copied()
                        .ok_or_else(|| self.error(node, "resolved local is not in Core scope"))?,
                )
            }
            Expression::Constant { declaration, .. } => {
                CoreExprKind::Constant(self.constant(*declaration, node)?)
            }
            Expression::SelfValue { .. } => CoreExprKind::SelfValue(
                self_record.ok_or_else(|| self.error(node, "self used outside implementation"))?,
            ),
            Expression::ConstructRecord {
                declaration,
                fields,
                ..
            } => {
                let mut lowered = Vec::new();
                for source_field in fields {
                    lowered.push(CoreExprField {
                        field: self.field(source_field.field, node)?,
                        value: self.lower_expression(&source_field.value, self_record)?,
                    });
                }
                CoreExprKind::ConstructRecord {
                    record: self.record(*declaration, node)?,
                    fields: lowered,
                }
            }
            Expression::ConstructEnum {
                declaration,
                variant,
                fields,
                ..
            } => {
                let mut lowered = Vec::new();
                for source_field in fields {
                    lowered.push(CoreExprField {
                        field: self.field(source_field.field, node)?,
                        value: self.lower_expression(&source_field.value, self_record)?,
                    });
                }
                CoreExprKind::ConstructEnum {
                    enumeration: self.enumeration(*declaration, node)?,
                    variant: self.variant(*variant, node)?,
                    fields: lowered,
                }
            }
            Expression::ConstructSome { value, .. } => {
                CoreExprKind::ConstructSome(self.lower_expression(value, self_record)?)
            }
            Expression::ConstructNone {
                inner_type, node, ..
            } => CoreExprKind::ConstructNone {
                inner: self.lower_type(inner_type, node)?,
            },
            Expression::ConstructOk {
                value,
                error_type,
                node,
                ..
            } => CoreExprKind::ConstructOk {
                value: self.lower_expression(value, self_record)?,
                error: self.lower_type(error_type, node)?,
            },
            Expression::ConstructErr {
                value,
                ok_type,
                node,
                ..
            } => CoreExprKind::ConstructErr {
                value: self.lower_expression(value, self_record)?,
                ok: self.lower_type(ok_type, node)?,
            },
            Expression::ConstructList {
                element_type,
                elements,
                node,
                ..
            } => {
                let element = self.lower_type(element_type, node)?;
                let elements = elements
                    .iter()
                    .map(|value| self.lower_expression(value, self_record))
                    .collect::<Result<_, _>>()?;
                CoreExprKind::ConstructList { element, elements }
            }
            Expression::Field { base, field, .. } => CoreExprKind::Field {
                value: self.lower_expression(base, self_record)?,
                field: self.field(*field, node)?,
            },
            Expression::Call {
                function,
                arguments,
                ..
            } => CoreExprKind::Call {
                function: self.function(*function, node)?,
                arguments: arguments
                    .iter()
                    .map(|value| self.lower_expression(value, self_record))
                    .collect::<Result<_, _>>()?,
            },
            Expression::MethodCall {
                receiver,
                dispatch,
                arguments,
                ..
            } => {
                let receiver = self.lower_expression(receiver, self_record)?;
                let arguments = arguments
                    .iter()
                    .map(|value| self.lower_expression(value, self_record))
                    .collect::<Result<_, _>>()?;
                match dispatch {
                    MethodDispatch::Concrete {
                        implementation,
                        method,
                    } => CoreExprKind::StaticMethodCall {
                        implementation: self.implementation(*implementation, node)?,
                        method: self.implementation_method(*method, node)?,
                        receiver,
                        arguments,
                    },
                    MethodDispatch::Contract { contract, method } => CoreExprKind::InterfaceCall {
                        interface: self.interface(*contract, node)?,
                        method: self.interface_method(*method, node)?,
                        receiver,
                        arguments,
                    },
                }
            }
            Expression::Intrinsic {
                operation,
                arguments,
                ..
            } => CoreExprKind::Intrinsic(lower_intrinsic(
                *operation,
                arguments
                    .iter()
                    .map(|value| self.lower_expression(value, self_record))
                    .collect::<Result<_, _>>()?,
                node,
            )?),
            Expression::If {
                condition,
                then_block,
                else_block,
                ..
            } => CoreExprKind::If {
                condition: self.lower_expression(condition, self_record)?,
                then_block: self.lower_block(then_block, self_record)?,
                else_block: self.lower_block(else_block, self_record)?,
            },
            Expression::Match { value, arms, .. } => {
                let value = self.lower_expression(value, self_record)?;
                let matched_type = self.expressions.get(value).expect("new expression").ty;
                let mut lowered_arms = Vec::new();
                for arm in arms {
                    let pattern = self.lower_pattern(&arm.pattern, matched_type)?;
                    lowered_arms.push(CoreMatchArm {
                        source: arm.node.source.clone(),
                        pattern,
                        body: self.lower_block(&arm.body, self_record)?,
                    });
                }
                CoreExprKind::Match {
                    value,
                    arms: lowered_arms,
                }
            }
            Expression::Block(block) => CoreExprKind::Block(self.lower_block(block, self_record)?),
        };
        Ok(self.expressions.push(CoreExpr {
            source: node.source.clone(),
            ty,
            evaluation: CoreEvaluationOrder::OnceLeftToRight,
            ownership: CoreResultOwnership::OwnedImmutableValue,
            kind,
        }))
    }

    fn lower_pattern(
        &mut self,
        pattern: &Pattern,
        matched_type: CoreTypeId,
    ) -> Result<CorePattern, Vec<Diagnostic>> {
        Ok(match pattern {
            Pattern::Wildcard { node } => CorePattern::Wildcard {
                source: node.source.clone(),
            },
            Pattern::Bool { node, value } => CorePattern::Bool {
                source: node.source.clone(),
                value: *value,
            },
            Pattern::EnumVariant {
                node,
                declaration,
                variant,
                bindings,
            } => {
                let enumeration = self.enumeration(*declaration, node)?;
                let variant = self.variant(*variant, node)?;
                let mut lowered = Vec::with_capacity(bindings.len());
                for source_binding in bindings {
                    let field = self.field(source_binding.field, node)?;
                    let ty = self.source_field_type(field, node)?;
                    lowered.push(CoreFieldBinding {
                        field,
                        binding: self.local(
                            node,
                            &source_binding.binding,
                            ty,
                            CoreLocalKind::Pattern,
                        ),
                    });
                }
                CorePattern::EnumVariant {
                    source: node.source.clone(),
                    enumeration,
                    variant,
                    bindings: lowered,
                }
            }
            Pattern::None { node } => CorePattern::None {
                source: node.source.clone(),
            },
            Pattern::Some { node, binding } => {
                let inner = self.type_inner(matched_type, node, "Some pattern")?;
                CorePattern::Some {
                    source: node.source.clone(),
                    binding: self.local(node, binding, inner, CoreLocalKind::Pattern),
                }
            }
            Pattern::Ok { node, binding } => {
                let (ok, _) = self.result_types(matched_type, node)?;
                CorePattern::Ok {
                    source: node.source.clone(),
                    binding: self.local(node, binding, ok, CoreLocalKind::Pattern),
                }
            }
            Pattern::Err { node, binding } => {
                let (_, error) = self.result_types(matched_type, node)?;
                CorePattern::Err {
                    source: node.source.clone(),
                    binding: self.local(node, binding, error, CoreLocalKind::Pattern),
                }
            }
        })
    }

    fn lower_test_invocation(
        &mut self,
        invocation: &TestInvocation,
        node: &NodeMeta,
    ) -> Result<CoreTestInvocation, Vec<Diagnostic>> {
        Ok(match invocation {
            TestInvocation::Function {
                function,
                arguments,
            } => CoreTestInvocation::Function {
                function: self.function(*function, node)?,
                arguments: arguments
                    .iter()
                    .map(|value| self.lower_typed_value(value, node))
                    .collect::<Result<_, _>>()?,
            },
            TestInvocation::Method {
                implementation,
                method,
                receiver,
                arguments,
            } => CoreTestInvocation::Method {
                implementation: self.implementation(*implementation, node)?,
                method: self.implementation_method(*method, node)?,
                receiver: self.lower_typed_value(receiver, node)?,
                arguments: arguments
                    .iter()
                    .map(|value| self.lower_typed_value(value, node))
                    .collect::<Result<_, _>>()?,
            },
        })
    }

    fn lower_expected(
        &mut self,
        expected: &ExpectedOutcome,
        node: &NodeMeta,
    ) -> Result<CoreExpectedOutcome, Vec<Diagnostic>> {
        Ok(match expected {
            ExpectedOutcome::Value(value) => {
                CoreExpectedOutcome::Value(self.lower_typed_value(value, node)?)
            }
            ExpectedOutcome::Error(value) => {
                CoreExpectedOutcome::Error(self.lower_typed_value(value, node)?)
            }
        })
    }

    fn lower_typed_value(
        &mut self,
        value: &TypedValue,
        node: &NodeMeta,
    ) -> Result<CoreTypedValue, Vec<Diagnostic>> {
        let ty = self.lower_type(&value.ty, node)?;
        Ok(CoreTypedValue {
            ty,
            value: self.lower_value(&value.value, ty, node)?,
        })
    }

    fn infer_value_type(
        &mut self,
        value: &Value,
        node: &NodeMeta,
    ) -> Result<CoreTypeId, Vec<Diagnostic>> {
        match value {
            Value::Unit => self.intern(CoreType::Unit),
            Value::Bool(_) => self.intern(CoreType::Bool),
            Value::I32(_) => self.intern(CoreType::I32),
            Value::I64(_) => self.intern(CoreType::I64),
            Value::F64(_) => self.intern(CoreType::F64),
            Value::Char(_) => self.intern(CoreType::Char),
            Value::String(_) => self.intern(CoreType::String),
            Value::Bytes(_) => self.intern(CoreType::Bytes),
            Value::Record { declaration, .. } => {
                let record = self.record(*declaration, node)?;
                self.intern(CoreType::Record(record))
            }
            Value::Enum { declaration, .. } => {
                let enumeration = self.enumeration(*declaration, node)?;
                self.intern(CoreType::Enum(enumeration))
            }
            Value::List(values) if !values.is_empty() => {
                let element = self.infer_value_type(&values[0], node)?;
                self.intern(CoreType::List(element))
            }
            Value::Some(value) => {
                let inner = self.infer_value_type(value, node)?;
                self.intern(CoreType::Option(inner))
            }
            Value::None | Value::List(_) | Value::Ok(_) | Value::Err(_) => Err(self.error(
                node,
                "value requires an explicit surrounding type during CoreIR lowering",
            )),
        }
    }

    fn lower_value(
        &mut self,
        value: &Value,
        expected: CoreTypeId,
        node: &NodeMeta,
    ) -> Result<CoreValue, Vec<Diagnostic>> {
        Ok(match value {
            Value::Unit => CoreValue::Unit,
            Value::Bool(value) => CoreValue::Bool(*value),
            Value::I32(value) => CoreValue::I32(*value),
            Value::I64(value) => CoreValue::I64(*value),
            Value::F64(value) => CoreValue::F64(*value),
            Value::Char(value) => CoreValue::Char(*value),
            Value::String(value) => CoreValue::String(value.clone()),
            Value::Bytes(value) => CoreValue::Bytes(value.clone()),
            Value::List(values) => {
                let element = self.type_inner(expected, node, "list value")?;
                CoreValue::List(
                    values
                        .iter()
                        .map(|value| self.lower_value(value, element, node))
                        .collect::<Result<_, _>>()?,
                )
            }
            Value::None => CoreValue::None,
            Value::Some(value) => {
                let inner = self.type_inner(expected, node, "Some value")?;
                CoreValue::Some(Box::new(self.lower_value(value, inner, node)?))
            }
            Value::Ok(value) => {
                let (ok, _) = self.result_types(expected, node)?;
                CoreValue::Ok(Box::new(self.lower_value(value, ok, node)?))
            }
            Value::Err(value) => {
                let (_, error) = self.result_types(expected, node)?;
                CoreValue::Err(Box::new(self.lower_value(value, error, node)?))
            }
            Value::Record {
                declaration,
                fields,
            } => CoreValue::Record {
                record: self.record(*declaration, node)?,
                fields: fields
                    .iter()
                    .map(|value| {
                        let field = self.field(value.field, node)?;
                        let ty = self.source_field_type(field, node)?;
                        Ok(CoreValueField {
                            field,
                            value: self.lower_value(&value.value, ty, node)?,
                        })
                    })
                    .collect::<Result<_, Vec<Diagnostic>>>()?,
            },
            Value::Enum {
                declaration,
                variant,
                fields,
            } => CoreValue::Enum {
                enumeration: self.enumeration(*declaration, node)?,
                variant: self.variant(*variant, node)?,
                fields: fields
                    .iter()
                    .map(|value| {
                        let field = self.field(value.field, node)?;
                        let ty = self.source_field_type(field, node)?;
                        Ok(CoreValueField {
                            field,
                            value: self.lower_value(&value.value, ty, node)?,
                        })
                    })
                    .collect::<Result<_, Vec<Diagnostic>>>()?,
            },
        })
    }

    fn type_inner(
        &self,
        ty: CoreTypeId,
        node: &NodeMeta,
        context: &str,
    ) -> Result<CoreTypeId, Vec<Diagnostic>> {
        match self.types.get(ty) {
            Some(CoreType::List(inner) | CoreType::Option(inner)) => Ok(*inner),
            _ => Err(self.error(node, format!("{context} requires a list or option type"))),
        }
    }

    fn result_types(
        &self,
        ty: CoreTypeId,
        node: &NodeMeta,
    ) -> Result<(CoreTypeId, CoreTypeId), Vec<Diagnostic>> {
        match self.types.get(ty) {
            Some(CoreType::Result { ok, error }) => Ok((*ok, *error)),
            _ => Err(self.error(node, "value requires a result type")),
        }
    }

    fn source_field_type(
        &mut self,
        field: CoreFieldId,
        node: &NodeMeta,
    ) -> Result<CoreTypeId, Vec<Diagnostic>> {
        let source_id = self
            .index
            .fields
            .iter()
            .find_map(|(source, candidate)| (*candidate == field).then_some(*source))
            .ok_or_else(|| self.error(node, "Core field has no source field"))?;
        let source = self
            .ordered
            .iter()
            .find_map(|declaration| match declaration {
                Declaration::Record(record) => record
                    .fields
                    .iter()
                    .find(|candidate| candidate.header.node.id == source_id)
                    .map(|candidate| (candidate.ty.clone(), candidate.header.node.clone())),
                Declaration::Enum(enumeration) => enumeration.variants.iter().find_map(|variant| {
                    variant
                        .fields
                        .iter()
                        .find(|candidate| candidate.header.node.id == source_id)
                        .map(|candidate| (candidate.ty.clone(), candidate.header.node.clone()))
                }),
                _ => None,
            });
        let (ty, source_node) =
            source.ok_or_else(|| self.error(node, "Core field source is missing"))?;
        self.lower_type(&ty, &source_node)
    }

    fn intrinsic_result_type(
        &mut self,
        operation: Intrinsic,
        arguments: &[CoreTypeId],
        node: &NodeMeta,
    ) -> Result<CoreTypeId, Vec<Diagnostic>> {
        use Intrinsic::*;
        match operation {
            BoolNot | BoolAnd | BoolOr | Equal | NotEqual | Less | LessEqual | Greater
            | GreaterEqual | FloatIsNaN | FloatIsNegativeZero | StringIsEmpty | StringContains
            | StringStartsWith | StringEndsWith | BytesIsEmpty | ListIsEmpty | ListContains
            | OptionIsSome | OptionIsNone | ResultIsOk | ResultIsErr => self.intern(CoreType::Bool),
            StringScalarLength | StringUtf16Length | BytesLength | ListLength | WidenI32ToI64 => {
                self.intern(CoreType::I64)
            }
            NarrowI64ToI32Checked => self.intern(CoreType::I32),
            FloatNeg | FloatTrunc | FloatAbs | FloatAdd | FloatSub | FloatMul | FloatDiv
            | FloatRemTrunc => self.intern(CoreType::F64),
            StringConcat
            | StringSliceScalars
            | StringStripPrefix
            | StringReplaceAll
            | StringReplaceMany
            | StringTruncateUtf8Bytes
            | StringTrimStart
            | StringTrimEnd
            | StringFromUtf8Checked => self.intern(CoreType::String),
            BytesConcat | BytesReplaceAll | StringToUtf8 => self.intern(CoreType::Bytes),
            StringIndexOfLiteral | ListIndexOf => {
                let inner = self.intern(CoreType::I64)?;
                self.intern(CoreType::Option(inner))
            }
            IntNegChecked | IntAddChecked | IntSubChecked | IntMulChecked | IntDivChecked
            | IntRemChecked | IntNegWrapping | IntAddWrapping | IntSubWrapping | IntMulWrapping
            | IntBitNot | IntBitAnd | IntBitOr | IntBitXor | IntShiftLeftChecked
            | IntShiftRightChecked | ListAppend | ListConcat => arguments
                .first()
                .copied()
                .ok_or_else(|| self.error(node, "intrinsic has no first operand")),
            ListGetChecked => arguments
                .first()
                .and_then(|ty| match self.types.get(*ty) {
                    Some(CoreType::List(inner)) => Some(*inner),
                    _ => None,
                })
                .ok_or_else(|| self.error(node, "list-get operand is not a list")),
            OptionUnwrapOr => arguments
                .get(1)
                .copied()
                .ok_or_else(|| self.error(node, "option unwrap has no fallback operand")),
        }
    }

    fn constant(&self, id: NodeId, node: &NodeMeta) -> Result<CoreConstantId, Vec<Diagnostic>> {
        self.index
            .constants
            .get(&id)
            .copied()
            .ok_or_else(|| self.error(node, format!("constant node {} is missing", id.0)))
    }

    fn record(&self, id: NodeId, node: &NodeMeta) -> Result<CoreRecordId, Vec<Diagnostic>> {
        self.index
            .records
            .get(&id)
            .copied()
            .ok_or_else(|| self.error(node, format!("record node {} is missing", id.0)))
    }

    fn enumeration(&self, id: NodeId, node: &NodeMeta) -> Result<CoreEnumId, Vec<Diagnostic>> {
        self.index
            .enums
            .get(&id)
            .copied()
            .ok_or_else(|| self.error(node, format!("enum node {} is missing", id.0)))
    }

    fn variant(&self, id: NodeId, node: &NodeMeta) -> Result<CoreVariantId, Vec<Diagnostic>> {
        self.index
            .variants
            .get(&id)
            .copied()
            .ok_or_else(|| self.error(node, format!("variant node {} is missing", id.0)))
    }

    fn field(&self, id: NodeId, node: &NodeMeta) -> Result<CoreFieldId, Vec<Diagnostic>> {
        self.index
            .fields
            .get(&id)
            .copied()
            .ok_or_else(|| self.error(node, format!("field node {} is missing", id.0)))
    }

    fn interface(&self, id: NodeId, node: &NodeMeta) -> Result<CoreInterfaceId, Vec<Diagnostic>> {
        self.index
            .interfaces
            .get(&id)
            .copied()
            .ok_or_else(|| self.error(node, format!("interface node {} is missing", id.0)))
    }

    fn interface_method(
        &self,
        id: NodeId,
        node: &NodeMeta,
    ) -> Result<CoreInterfaceMethodId, Vec<Diagnostic>> {
        self.index
            .interface_methods
            .get(&id)
            .copied()
            .ok_or_else(|| self.error(node, format!("interface method node {} is missing", id.0)))
    }

    fn implementation(
        &self,
        id: NodeId,
        node: &NodeMeta,
    ) -> Result<CoreImplementationId, Vec<Diagnostic>> {
        self.index
            .implementations
            .get(&id)
            .copied()
            .ok_or_else(|| self.error(node, format!("implementation node {} is missing", id.0)))
    }

    fn implementation_method(
        &self,
        id: NodeId,
        node: &NodeMeta,
    ) -> Result<CoreImplementationMethodId, Vec<Diagnostic>> {
        self.index
            .implementation_methods
            .get(&id)
            .copied()
            .ok_or_else(|| {
                self.error(
                    node,
                    format!("implementation method node {} is missing", id.0),
                )
            })
    }

    fn function(&self, id: NodeId, node: &NodeMeta) -> Result<CoreFunctionId, Vec<Diagnostic>> {
        self.index
            .functions
            .get(&id)
            .copied()
            .ok_or_else(|| self.error(node, format!("function node {} is missing", id.0)))
    }

    fn error(&self, node: &NodeMeta, message: impl Into<String>) -> Vec<Diagnostic> {
        vec![Diagnostic::error(
            DiagnosticCode::InvalidStructure,
            message,
            node.source.clone(),
        )]
    }
}

fn declaration_rank(declaration: &Declaration) -> u8 {
    match declaration {
        Declaration::Constant(_) => 0,
        Declaration::Alias(_) => 1,
        Declaration::Record(_) => 2,
        Declaration::Enum(_) => 3,
        Declaration::Contract(_) => 4,
        Declaration::Implementation(_) => 5,
        Declaration::Function(_) => 6,
        Declaration::Test(_) => 7,
    }
}

fn declaration_header(header: &DeclarationHeader) -> CoreDeclarationHeader {
    CoreDeclarationHeader {
        name: header.name.clone(),
        visibility: header.visibility,
        documentation: header.documentation.clone(),
        source: header.node.source.clone(),
    }
}

fn member_header(header: &MemberHeader) -> CoreMemberHeader {
    CoreMemberHeader {
        name: header.name.clone(),
        documentation: header.documentation.clone(),
        source: header.node.source.clone(),
    }
}

fn constant_node(expression: &ConstantExpression) -> &NodeMeta {
    match expression {
        ConstantExpression::Literal { node, .. }
        | ConstantExpression::Reference { node, .. }
        | ConstantExpression::Record { node, .. }
        | ConstantExpression::Enum { node, .. }
        | ConstantExpression::Some { node, .. }
        | ConstantExpression::None { node, .. }
        | ConstantExpression::Ok { node, .. }
        | ConstantExpression::Err { node, .. }
        | ConstantExpression::List { node, .. }
        | ConstantExpression::Intrinsic { node, .. } => node,
    }
}

enum IntrinsicClass {
    Unary(CoreUnaryIntrinsic),
    Binary(CoreBinaryIntrinsic),
    Ternary(CoreTernaryIntrinsic),
    Variadic(CoreVariadicIntrinsic),
}

fn classify_intrinsic(operation: Intrinsic) -> IntrinsicClass {
    use Intrinsic::*;
    match operation {
        BoolNot => IntrinsicClass::Unary(CoreUnaryIntrinsic::BoolNot),
        IntNegChecked => IntrinsicClass::Unary(CoreUnaryIntrinsic::IntNegChecked),
        IntNegWrapping => IntrinsicClass::Unary(CoreUnaryIntrinsic::IntNegWrapping),
        IntBitNot => IntrinsicClass::Unary(CoreUnaryIntrinsic::IntBitNot),
        FloatNeg => IntrinsicClass::Unary(CoreUnaryIntrinsic::FloatNeg),
        FloatTrunc => IntrinsicClass::Unary(CoreUnaryIntrinsic::FloatTrunc),
        FloatIsNaN => IntrinsicClass::Unary(CoreUnaryIntrinsic::FloatIsNaN),
        FloatIsNegativeZero => IntrinsicClass::Unary(CoreUnaryIntrinsic::FloatIsNegativeZero),
        FloatAbs => IntrinsicClass::Unary(CoreUnaryIntrinsic::FloatAbs),
        StringScalarLength => IntrinsicClass::Unary(CoreUnaryIntrinsic::StringScalarLength),
        StringUtf16Length => IntrinsicClass::Unary(CoreUnaryIntrinsic::StringUtf16Length),
        StringIsEmpty => IntrinsicClass::Unary(CoreUnaryIntrinsic::StringIsEmpty),
        BytesLength => IntrinsicClass::Unary(CoreUnaryIntrinsic::BytesLength),
        BytesIsEmpty => IntrinsicClass::Unary(CoreUnaryIntrinsic::BytesIsEmpty),
        ListLength => IntrinsicClass::Unary(CoreUnaryIntrinsic::ListLength),
        ListIsEmpty => IntrinsicClass::Unary(CoreUnaryIntrinsic::ListIsEmpty),
        OptionIsSome => IntrinsicClass::Unary(CoreUnaryIntrinsic::OptionIsSome),
        OptionIsNone => IntrinsicClass::Unary(CoreUnaryIntrinsic::OptionIsNone),
        ResultIsOk => IntrinsicClass::Unary(CoreUnaryIntrinsic::ResultIsOk),
        ResultIsErr => IntrinsicClass::Unary(CoreUnaryIntrinsic::ResultIsErr),
        WidenI32ToI64 => IntrinsicClass::Unary(CoreUnaryIntrinsic::WidenI32ToI64),
        NarrowI64ToI32Checked => IntrinsicClass::Unary(CoreUnaryIntrinsic::NarrowI64ToI32Checked),
        StringToUtf8 => IntrinsicClass::Unary(CoreUnaryIntrinsic::StringToUtf8),
        StringFromUtf8Checked => IntrinsicClass::Unary(CoreUnaryIntrinsic::StringFromUtf8Checked),
        BoolAnd => IntrinsicClass::Binary(CoreBinaryIntrinsic::BoolAnd),
        BoolOr => IntrinsicClass::Binary(CoreBinaryIntrinsic::BoolOr),
        Equal => IntrinsicClass::Binary(CoreBinaryIntrinsic::Equal),
        NotEqual => IntrinsicClass::Binary(CoreBinaryIntrinsic::NotEqual),
        Less => IntrinsicClass::Binary(CoreBinaryIntrinsic::Less),
        LessEqual => IntrinsicClass::Binary(CoreBinaryIntrinsic::LessEqual),
        Greater => IntrinsicClass::Binary(CoreBinaryIntrinsic::Greater),
        GreaterEqual => IntrinsicClass::Binary(CoreBinaryIntrinsic::GreaterEqual),
        IntAddChecked => IntrinsicClass::Binary(CoreBinaryIntrinsic::IntAddChecked),
        IntSubChecked => IntrinsicClass::Binary(CoreBinaryIntrinsic::IntSubChecked),
        IntMulChecked => IntrinsicClass::Binary(CoreBinaryIntrinsic::IntMulChecked),
        IntDivChecked => IntrinsicClass::Binary(CoreBinaryIntrinsic::IntDivChecked),
        IntRemChecked => IntrinsicClass::Binary(CoreBinaryIntrinsic::IntRemChecked),
        IntAddWrapping => IntrinsicClass::Binary(CoreBinaryIntrinsic::IntAddWrapping),
        IntSubWrapping => IntrinsicClass::Binary(CoreBinaryIntrinsic::IntSubWrapping),
        IntMulWrapping => IntrinsicClass::Binary(CoreBinaryIntrinsic::IntMulWrapping),
        IntBitAnd => IntrinsicClass::Binary(CoreBinaryIntrinsic::IntBitAnd),
        IntBitOr => IntrinsicClass::Binary(CoreBinaryIntrinsic::IntBitOr),
        IntBitXor => IntrinsicClass::Binary(CoreBinaryIntrinsic::IntBitXor),
        IntShiftLeftChecked => IntrinsicClass::Binary(CoreBinaryIntrinsic::IntShiftLeftChecked),
        IntShiftRightChecked => IntrinsicClass::Binary(CoreBinaryIntrinsic::IntShiftRightChecked),
        FloatAdd => IntrinsicClass::Binary(CoreBinaryIntrinsic::FloatAdd),
        FloatSub => IntrinsicClass::Binary(CoreBinaryIntrinsic::FloatSub),
        FloatMul => IntrinsicClass::Binary(CoreBinaryIntrinsic::FloatMul),
        FloatDiv => IntrinsicClass::Binary(CoreBinaryIntrinsic::FloatDiv),
        FloatRemTrunc => IntrinsicClass::Binary(CoreBinaryIntrinsic::FloatRemTrunc),
        StringConcat => IntrinsicClass::Binary(CoreBinaryIntrinsic::StringConcat),
        StringIndexOfLiteral => IntrinsicClass::Binary(CoreBinaryIntrinsic::StringIndexOfLiteral),
        StringContains => IntrinsicClass::Binary(CoreBinaryIntrinsic::StringContains),
        StringStartsWith => IntrinsicClass::Binary(CoreBinaryIntrinsic::StringStartsWith),
        StringStripPrefix => IntrinsicClass::Binary(CoreBinaryIntrinsic::StringStripPrefix),
        StringEndsWith => IntrinsicClass::Binary(CoreBinaryIntrinsic::StringEndsWith),
        StringTruncateUtf8Bytes => {
            IntrinsicClass::Binary(CoreBinaryIntrinsic::StringTruncateUtf8Bytes)
        }
        StringTrimStart => IntrinsicClass::Binary(CoreBinaryIntrinsic::StringTrimStart),
        StringTrimEnd => IntrinsicClass::Binary(CoreBinaryIntrinsic::StringTrimEnd),
        BytesConcat => IntrinsicClass::Binary(CoreBinaryIntrinsic::BytesConcat),
        ListGetChecked => IntrinsicClass::Binary(CoreBinaryIntrinsic::ListGetChecked),
        ListAppend => IntrinsicClass::Binary(CoreBinaryIntrinsic::ListAppend),
        ListConcat => IntrinsicClass::Binary(CoreBinaryIntrinsic::ListConcat),
        ListContains => IntrinsicClass::Binary(CoreBinaryIntrinsic::ListContains),
        ListIndexOf => IntrinsicClass::Binary(CoreBinaryIntrinsic::ListIndexOf),
        OptionUnwrapOr => IntrinsicClass::Binary(CoreBinaryIntrinsic::OptionUnwrapOr),
        StringSliceScalars => IntrinsicClass::Ternary(CoreTernaryIntrinsic::StringSliceScalars),
        StringReplaceAll => IntrinsicClass::Ternary(CoreTernaryIntrinsic::StringReplaceAll),
        BytesReplaceAll => IntrinsicClass::Ternary(CoreTernaryIntrinsic::BytesReplaceAll),
        StringReplaceMany => IntrinsicClass::Variadic(CoreVariadicIntrinsic::StringReplaceMany),
    }
}

fn lower_intrinsic<T>(
    operation: Intrinsic,
    mut arguments: Vec<T>,
    node: &NodeMeta,
) -> Result<CoreIntrinsicExpr<T>, Vec<Diagnostic>> {
    let invalid = |expected: &str, actual: usize| {
        vec![Diagnostic::error(
            DiagnosticCode::InvalidInvocation,
            format!("intrinsic {operation:?} expects {expected}; received {actual}"),
            node.source.clone(),
        )]
    };
    match classify_intrinsic(operation) {
        IntrinsicClass::Unary(operation) => {
            if arguments.len() != 1 {
                return Err(invalid("one operand", arguments.len()));
            }
            Ok(CoreIntrinsicExpr::Unary {
                operation,
                operand: arguments.remove(0),
            })
        }
        IntrinsicClass::Binary(operation) => {
            if arguments.len() != 2 {
                return Err(invalid("two operands", arguments.len()));
            }
            let right = arguments.pop().expect("length checked");
            let left = arguments.pop().expect("length checked");
            Ok(CoreIntrinsicExpr::Binary {
                operation,
                left,
                right,
            })
        }
        IntrinsicClass::Ternary(operation) => {
            if arguments.len() != 3 {
                return Err(invalid("three operands", arguments.len()));
            }
            let third = arguments.pop().expect("length checked");
            let second = arguments.pop().expect("length checked");
            let first = arguments.pop().expect("length checked");
            Ok(CoreIntrinsicExpr::Ternary {
                operation,
                first,
                second,
                third,
            })
        }
        IntrinsicClass::Variadic(operation) => {
            if arguments.len() < 3 || arguments.len().is_multiple_of(2) {
                return Err(invalid(
                    "an odd operand count of at least three",
                    arguments.len(),
                ));
            }
            Ok(CoreIntrinsicExpr::Variadic {
                operation,
                arguments,
            })
        }
    }
}
