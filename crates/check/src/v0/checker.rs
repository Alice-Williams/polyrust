use std::collections::{BTreeMap, BTreeSet};

use portable_diagnostics::{Diagnostic, DiagnosticCode, SourceRef};
use portable_ir::v0::{
    AliasDeclaration, Block, ConstantDeclaration, ConstantExpression, ContractDeclaration,
    Declaration, Document, EnumDeclaration, EnumVariant, ExpectedOutcome, Expression,
    ExpressionField, FieldBinding, FieldDeclaration, FunctionDeclaration,
    ImplementationDeclaration, Intrinsic, IrVersion, MatchArm, MethodDispatch,
    MethodImplementation, MethodSignature, Module, NodeId, NodeMeta, Parameter, Pattern,
    RecordDeclaration, Statement, TestDeclaration, TestInvocation, TypeRef, TypedValue, Value,
    ValueField, validate_structure,
};

use super::{Capability, CapabilityReport, CheckedProgram, SymbolId};

pub(super) const MAX_DEPTH: usize = 64;

pub fn check_program(document: Document) -> Result<CheckedProgram, Vec<Diagnostic>> {
    let fallback = SourceRef::logical([format!("module({})", document.module.name)]);
    let mut diagnostics = Vec::new();
    if !document.ir_version.is_compatible_with(IrVersion::CURRENT) {
        diagnostics.push(Diagnostic::error(
            DiagnosticCode::UnsupportedIrMajor,
            format!(
                "IR version {} is not compatible with checker {}",
                document.ir_version,
                IrVersion::CURRENT
            ),
            fallback,
        ));
        return Err(diagnostics);
    }
    if let Err(error) = validate_structure(&document) {
        diagnostics.push(Diagnostic::error(
            DiagnosticCode::InvalidStructure,
            error.to_string(),
            fallback,
        ));
        return Err(diagnostics);
    }

    let index = Index::new(&document.module);
    let mut checker = Checker {
        document: &document,
        index,
        diagnostics,
        expression_types: BTreeMap::new(),
        local_references: BTreeMap::new(),
        capabilities: CapabilityReport::empty(),
        current_declaration: NodeId(0),
    };
    checker.check_module();
    checker.check_recursion();
    portable_diagnostics::sort_diagnostics(&mut checker.diagnostics);

    if !checker.diagnostics.is_empty() {
        return Err(std::mem::take(&mut checker.diagnostics));
    }
    let expression_types = std::mem::take(&mut checker.expression_types);
    let local_references = std::mem::take(&mut checker.local_references);
    let capabilities = std::mem::replace(&mut checker.capabilities, CapabilityReport::empty());
    drop(checker);
    Ok(CheckedProgram::new(
        document,
        expression_types,
        local_references,
        capabilities,
    ))
}

struct Index<'a> {
    declarations: BTreeMap<NodeId, &'a Declaration>,
    fields: BTreeMap<NodeId, (&'a RecordDeclaration, &'a FieldDeclaration)>,
    enum_fields: BTreeMap<NodeId, (&'a EnumDeclaration, &'a EnumVariant, &'a FieldDeclaration)>,
    variants: BTreeMap<NodeId, (&'a EnumDeclaration, &'a EnumVariant)>,
    contract_methods: BTreeMap<NodeId, (&'a ContractDeclaration, &'a MethodSignature)>,
    implementation_methods:
        BTreeMap<NodeId, (&'a ImplementationDeclaration, &'a MethodImplementation)>,
}

impl<'a> Index<'a> {
    fn new(module: &'a Module) -> Self {
        let mut index = Self {
            declarations: BTreeMap::new(),
            fields: BTreeMap::new(),
            enum_fields: BTreeMap::new(),
            variants: BTreeMap::new(),
            contract_methods: BTreeMap::new(),
            implementation_methods: BTreeMap::new(),
        };
        for declaration in &module.declarations {
            index
                .declarations
                .insert(declaration.header().node.id, declaration);
            match declaration {
                Declaration::Record(record) => {
                    for field in &record.fields {
                        index.fields.insert(field.header.node.id, (record, field));
                    }
                }
                Declaration::Enum(enumeration) => {
                    for variant in &enumeration.variants {
                        index
                            .variants
                            .insert(variant.header.node.id, (enumeration, variant));
                        for field in &variant.fields {
                            index
                                .enum_fields
                                .insert(field.header.node.id, (enumeration, variant, field));
                        }
                    }
                }
                Declaration::Contract(contract) => {
                    for method in &contract.methods {
                        index
                            .contract_methods
                            .insert(method.header.node.id, (contract, method));
                    }
                }
                Declaration::Implementation(implementation) => {
                    for method in &implementation.methods {
                        index
                            .implementation_methods
                            .insert(method.header.node.id, (implementation, method));
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

    fn declaration(&self, id: NodeId) -> Option<&'a Declaration> {
        self.declarations.get(&id).copied()
    }

    fn alias(&self, id: NodeId) -> Option<&'a AliasDeclaration> {
        match self.declaration(id) {
            Some(Declaration::Alias(alias)) => Some(alias),
            _ => None,
        }
    }

    fn record(&self, id: NodeId) -> Option<&'a RecordDeclaration> {
        match self.declaration(id) {
            Some(Declaration::Record(record)) => Some(record),
            _ => None,
        }
    }

    fn enumeration(&self, id: NodeId) -> Option<&'a EnumDeclaration> {
        match self.declaration(id) {
            Some(Declaration::Enum(enumeration)) => Some(enumeration),
            _ => None,
        }
    }

    fn contract(&self, id: NodeId) -> Option<&'a ContractDeclaration> {
        match self.declaration(id) {
            Some(Declaration::Contract(contract)) => Some(contract),
            _ => None,
        }
    }

    fn implementation(&self, id: NodeId) -> Option<&'a ImplementationDeclaration> {
        match self.declaration(id) {
            Some(Declaration::Implementation(implementation)) => Some(implementation),
            _ => None,
        }
    }

    fn function(&self, id: NodeId) -> Option<&'a FunctionDeclaration> {
        match self.declaration(id) {
            Some(Declaration::Function(function)) => Some(function),
            _ => None,
        }
    }
}

#[derive(Clone, Copy)]
enum TypePosition {
    General,
    DirectParameter,
}

#[derive(Clone)]
struct Binding {
    symbol: SymbolId,
    ty: TypeRef,
}

type Environment = BTreeMap<String, Binding>;

#[derive(Clone)]
struct Flow {
    ty: TypeRef,
    always_returns: bool,
}

struct Checker<'a> {
    document: &'a Document,
    index: Index<'a>,
    diagnostics: Vec<Diagnostic>,
    expression_types: BTreeMap<NodeId, TypeRef>,
    local_references: BTreeMap<NodeId, SymbolId>,
    capabilities: CapabilityReport,
    current_declaration: NodeId,
}

impl Checker<'_> {
    fn check_module(&mut self) {
        if !valid_identifier(&self.document.module.name) {
            self.error(
                DiagnosticCode::InvalidIdentifier,
                format!(
                    "module name {:?} is not a portable identifier",
                    self.document.module.name
                ),
                SourceRef::logical([format!("module({})", self.document.module.name)]),
            );
        }
        self.check_declaration_names();
        self.check_aliases();
        for declaration in &self.document.module.declarations {
            self.current_declaration = declaration.header().node.id;
            self.check_declaration(declaration);
        }
    }

    fn check_declaration_names(&mut self) {
        let mut names = BTreeMap::<&str, SourceRef>::new();
        for declaration in &self.document.module.declarations {
            let header = declaration.header();
            self.check_identifier(&header.name, &header.node);
            if let Some(first) = names.insert(&header.name, header.node.source.clone()) {
                let mut diagnostic = Diagnostic::error(
                    DiagnosticCode::DuplicateDeclaration,
                    format!("duplicate top-level name {:?}", header.name),
                    header.node.source.clone(),
                );
                diagnostic
                    .related
                    .push(portable_diagnostics::RelatedLocation {
                        source: first,
                        message: "first declaration is here".to_owned(),
                    });
                self.diagnostics.push(diagnostic);
            }
        }
    }

    fn check_aliases(&mut self) {
        for declaration in &self.document.module.declarations {
            if let Declaration::Alias(alias) = declaration {
                let mut stack = Vec::new();
                self.normalize_type(
                    &TypeRef::Named(alias.header.node.id),
                    &alias.header.node,
                    TypePosition::General,
                    &mut stack,
                );
            }
        }
    }

    fn check_declaration(&mut self, declaration: &Declaration) {
        match declaration {
            Declaration::Constant(value) => self.check_constant(value),
            Declaration::Alias(alias) => {
                self.check_type(&alias.target, &alias.header.node, TypePosition::General);
            }
            Declaration::Record(record) => self.check_record(record),
            Declaration::Enum(enumeration) => self.check_enum(enumeration),
            Declaration::Contract(contract) => self.check_contract(contract),
            Declaration::Implementation(implementation) => {
                self.check_implementation(implementation);
            }
            Declaration::Function(function) => self.check_function(function),
            Declaration::Test(test) => self.check_test(test),
        }
    }

    fn check_type(&mut self, ty: &TypeRef, node: &NodeMeta, position: TypePosition) {
        let mut stack = Vec::new();
        self.normalize_type(ty, node, position, &mut stack);
        self.collect_type_capabilities(node.id, ty);
    }

    fn normalize_type(
        &mut self,
        ty: &TypeRef,
        node: &NodeMeta,
        position: TypePosition,
        stack: &mut Vec<NodeId>,
    ) -> Option<TypeRef> {
        let normalized = match ty {
            TypeRef::List(inner) => TypeRef::List(Box::new(self.normalize_type(
                inner,
                node,
                TypePosition::General,
                stack,
            )?)),
            TypeRef::Option(inner) => TypeRef::Option(Box::new(self.normalize_type(
                inner,
                node,
                TypePosition::General,
                stack,
            )?)),
            TypeRef::Result { ok, error } => TypeRef::Result {
                ok: Box::new(self.normalize_type(ok, node, TypePosition::General, stack)?),
                error: Box::new(self.normalize_type(error, node, TypePosition::General, stack)?),
            },
            TypeRef::Named(id) => {
                if let Some(alias) = self.index.alias(*id) {
                    if stack.contains(id) {
                        self.error(
                            DiagnosticCode::AliasCycle,
                            format!("type alias {:?} is recursive", alias.header.name),
                            node.source.clone(),
                        );
                        return None;
                    }
                    stack.push(*id);
                    let result = self.normalize_type(
                        &alias.target,
                        &alias.header.node,
                        TypePosition::General,
                        stack,
                    );
                    stack.pop();
                    return result;
                }
                if self.index.record(*id).is_none() && self.index.enumeration(*id).is_none() {
                    self.unresolved("named type", *id, &node.source);
                    return None;
                }
                ty.clone()
            }
            TypeRef::Contract(id) => {
                if self.index.contract(*id).is_none() {
                    self.unresolved("contract type", *id, &node.source);
                    return None;
                }
                if !matches!(position, TypePosition::DirectParameter) {
                    self.error(
                        DiagnosticCode::InvalidContractPosition,
                        "contract types are allowed only as direct parameters",
                        node.source.clone(),
                    );
                    return None;
                }
                ty.clone()
            }
            TypeRef::Unit
            | TypeRef::Bool
            | TypeRef::I32
            | TypeRef::I64
            | TypeRef::F64
            | TypeRef::Char
            | TypeRef::String
            | TypeRef::Bytes => ty.clone(),
        };
        Some(normalized)
    }

    fn same_type(&mut self, left: &TypeRef, right: &TypeRef, node: &NodeMeta) -> bool {
        if left == right {
            return true;
        }
        let mut left_stack = Vec::new();
        let mut right_stack = Vec::new();
        let left = self.normalize_type(left, node, TypePosition::General, &mut left_stack);
        let right = self.normalize_type(right, node, TypePosition::General, &mut right_stack);
        left.is_some() && left == right
    }

    fn check_identifier(&mut self, name: &str, node: &NodeMeta) {
        if !valid_identifier(name) {
            self.error(
                DiagnosticCode::InvalidIdentifier,
                format!("{name:?} is not a portable identifier"),
                node.source.clone(),
            );
        }
    }

    fn check_record(&mut self, record: &RecordDeclaration) {
        let mut names = BTreeSet::new();
        for field in &record.fields {
            self.check_identifier(&field.header.name, &field.header.node);
            if !names.insert(field.header.name.as_str()) {
                self.error(
                    DiagnosticCode::DuplicateDeclaration,
                    format!(
                        "duplicate field {:?} in record {:?}",
                        field.header.name, record.header.name
                    ),
                    field.header.node.source.clone(),
                );
            }
            self.check_type(&field.ty, &field.header.node, TypePosition::General);
        }
    }

    fn check_enum(&mut self, enumeration: &EnumDeclaration) {
        let mut variants = BTreeSet::new();
        for variant in &enumeration.variants {
            self.check_identifier(&variant.header.name, &variant.header.node);
            if !variants.insert(variant.header.name.as_str()) {
                self.error(
                    DiagnosticCode::DuplicateDeclaration,
                    format!(
                        "duplicate variant {:?} in enum {:?}",
                        variant.header.name, enumeration.header.name
                    ),
                    variant.header.node.source.clone(),
                );
            }
            let mut fields = BTreeSet::new();
            for field in &variant.fields {
                self.check_identifier(&field.header.name, &field.header.node);
                if !fields.insert(field.header.name.as_str()) {
                    self.error(
                        DiagnosticCode::DuplicateDeclaration,
                        format!(
                            "duplicate field {:?} in enum variant {:?}",
                            field.header.name, variant.header.name
                        ),
                        field.header.node.source.clone(),
                    );
                }
                self.check_type(&field.ty, &field.header.node, TypePosition::General);
            }
        }
    }

    fn check_contract(&mut self, contract: &ContractDeclaration) {
        let mut names = BTreeSet::new();
        for method in &contract.methods {
            self.check_identifier(&method.header.name, &method.header.node);
            if !names.insert(method.header.name.as_str()) {
                self.error(
                    DiagnosticCode::DuplicateDeclaration,
                    format!(
                        "duplicate method {:?} in contract {:?}",
                        method.header.name, contract.header.name
                    ),
                    method.header.node.source.clone(),
                );
            }
            self.check_parameters(&method.parameters);
            self.check_type(
                &method.return_type,
                &method.header.node,
                TypePosition::General,
            );
        }
    }

    fn check_parameters(&mut self, parameters: &[Parameter]) -> Environment {
        let mut environment = Environment::new();
        for parameter in parameters {
            self.check_identifier(&parameter.header.name, &parameter.header.node);
            self.check_type(
                &parameter.ty,
                &parameter.header.node,
                TypePosition::DirectParameter,
            );
            let binding = Binding {
                symbol: SymbolId::new(parameter.header.node.id),
                ty: parameter.ty.clone(),
            };
            if environment
                .insert(parameter.header.name.clone(), binding)
                .is_some()
            {
                self.error(
                    DiagnosticCode::DuplicateDeclaration,
                    format!("duplicate parameter {:?}", parameter.header.name),
                    parameter.header.node.source.clone(),
                );
            }
        }
        environment
    }

    fn check_constant(&mut self, constant: &ConstantDeclaration) {
        self.check_type(&constant.ty, &constant.header.node, TypePosition::General);
        let actual = self.check_constant_expression(&constant.value, 0);
        if let Some(actual) = actual
            && !self.same_type(&actual, &constant.ty, &constant.header.node)
        {
            self.error(
                DiagnosticCode::TypeMismatch,
                format!(
                    "constant {:?} has the wrong value type",
                    constant.header.name
                ),
                constant.header.node.source.clone(),
            );
        }
    }

    fn check_constant_expression(
        &mut self,
        expression: &ConstantExpression,
        depth: usize,
    ) -> Option<TypeRef> {
        if depth > MAX_DEPTH {
            self.error(
                DiagnosticCode::ExcessiveComplexity,
                "constant expression exceeds the checker depth limit",
                constant_node(expression).source.clone(),
            );
            return None;
        }
        let node = constant_node(expression);
        let ty = match expression {
            ConstantExpression::Literal { value, .. } => self.infer_value(value, node, depth + 1),
            ConstantExpression::Reference { declaration, .. } => {
                match self.index.declaration(*declaration) {
                    Some(Declaration::Constant(constant)) => Some(constant.ty.clone()),
                    _ => {
                        self.unresolved("constant", *declaration, &node.source);
                        None
                    }
                }
            }
            ConstantExpression::Record {
                declaration,
                fields,
                ..
            } => self.check_constant_record(*declaration, fields, node, depth + 1),
            ConstantExpression::Enum {
                declaration,
                variant,
                fields,
                ..
            } => self.check_constant_enum(*declaration, *variant, fields, node, depth + 1),
            ConstantExpression::Some { value, .. } => Some(TypeRef::Option(Box::new(
                self.check_constant_expression(value, depth + 1)?,
            ))),
            ConstantExpression::None { inner_type, .. } => {
                self.check_type(inner_type, node, TypePosition::General);
                Some(TypeRef::Option(Box::new(inner_type.clone())))
            }
            ConstantExpression::Ok {
                value, error_type, ..
            } => {
                self.check_type(error_type, node, TypePosition::General);
                Some(TypeRef::Result {
                    ok: Box::new(self.check_constant_expression(value, depth + 1)?),
                    error: Box::new(error_type.clone()),
                })
            }
            ConstantExpression::Err { value, ok_type, .. } => {
                self.check_type(ok_type, node, TypePosition::General);
                Some(TypeRef::Result {
                    ok: Box::new(ok_type.clone()),
                    error: Box::new(self.check_constant_expression(value, depth + 1)?),
                })
            }
            ConstantExpression::List {
                element_type,
                elements,
                ..
            } => {
                self.check_type(element_type, node, TypePosition::General);
                for element in elements {
                    let actual = self.check_constant_expression(element, depth + 1);
                    if actual
                        .as_ref()
                        .is_some_and(|actual| !self.same_type(actual, element_type, node))
                    {
                        self.type_error("constant list element", node);
                    }
                }
                Some(TypeRef::List(Box::new(element_type.clone())))
            }
            ConstantExpression::Intrinsic {
                operation,
                arguments,
                ..
            } => {
                let types = arguments
                    .iter()
                    .map(|argument| self.check_constant_expression(argument, depth + 1))
                    .collect::<Option<Vec<_>>>()?;
                self.check_intrinsic(*operation, &types, node)
            }
        };
        if let Some(ty) = &ty {
            self.collect_type_capabilities(node.id, ty);
        }
        ty
    }

    fn check_constant_record(
        &mut self,
        declaration: NodeId,
        fields: &[portable_ir::v0::ConstantField],
        node: &NodeMeta,
        depth: usize,
    ) -> Option<TypeRef> {
        let Some(record) = self.index.record(declaration) else {
            self.unresolved("record", declaration, &node.source);
            return None;
        };
        self.check_constant_fields(fields, &record.fields, node, depth);
        Some(TypeRef::Named(declaration))
    }

    fn check_constant_enum(
        &mut self,
        declaration: NodeId,
        variant: NodeId,
        fields: &[portable_ir::v0::ConstantField],
        node: &NodeMeta,
        depth: usize,
    ) -> Option<TypeRef> {
        let Some((enumeration, declared_variant)) = self.index.variants.get(&variant).copied()
        else {
            self.unresolved("enum variant", variant, &node.source);
            return None;
        };
        if enumeration.header.node.id != declaration {
            self.type_error("enum variant belongs to a different declaration", node);
            return None;
        }
        self.check_constant_fields(fields, &declared_variant.fields, node, depth);
        Some(TypeRef::Named(declaration))
    }

    fn check_constant_fields(
        &mut self,
        supplied: &[portable_ir::v0::ConstantField],
        expected: &[FieldDeclaration],
        node: &NodeMeta,
        depth: usize,
    ) {
        let mut seen = BTreeSet::new();
        for field in supplied {
            if !seen.insert(field.field) {
                self.error(
                    DiagnosticCode::DuplicateDeclaration,
                    format!("duplicate aggregate field node {}", field.field.0),
                    node.source.clone(),
                );
            }
            let Some(declared) = expected
                .iter()
                .find(|expected| expected.header.node.id == field.field)
            else {
                self.unresolved("aggregate field", field.field, &node.source);
                continue;
            };
            let actual = self.check_constant_expression(&field.value, depth + 1);
            if actual
                .as_ref()
                .is_some_and(|actual| !self.same_type(actual, &declared.ty, node))
            {
                self.type_error("aggregate field value", node);
            }
        }
        if seen.len() != expected.len() {
            self.error(
                DiagnosticCode::InvalidInvocation,
                "aggregate initializer does not supply every field exactly once",
                node.source.clone(),
            );
        }
    }

    fn infer_value(&mut self, value: &Value, node: &NodeMeta, depth: usize) -> Option<TypeRef> {
        if depth > MAX_DEPTH {
            self.error(
                DiagnosticCode::ExcessiveComplexity,
                "literal value exceeds the checker depth limit",
                node.source.clone(),
            );
            return None;
        }
        match value {
            Value::Unit => Some(TypeRef::Unit),
            Value::Bool(_) => Some(TypeRef::Bool),
            Value::I32(_) => Some(TypeRef::I32),
            Value::I64(_) => Some(TypeRef::I64),
            Value::F64(_) => Some(TypeRef::F64),
            Value::Char(_) => Some(TypeRef::Char),
            Value::String(_) => Some(TypeRef::String),
            Value::Bytes(_) => Some(TypeRef::Bytes),
            Value::Record {
                declaration,
                fields,
            } => {
                let Some(record) = self.index.record(*declaration) else {
                    self.unresolved("record literal", *declaration, &node.source);
                    return None;
                };
                self.check_value_fields(fields, &record.fields, node, depth + 1);
                Some(TypeRef::Named(*declaration))
            }
            Value::Enum {
                declaration,
                variant,
                fields,
            } => {
                let Some((enumeration, declared_variant)) =
                    self.index.variants.get(variant).copied()
                else {
                    self.unresolved("enum literal variant", *variant, &node.source);
                    return None;
                };
                if enumeration.header.node.id != *declaration {
                    self.type_error("literal enum variant declaration", node);
                    return None;
                }
                self.check_value_fields(fields, &declared_variant.fields, node, depth + 1);
                Some(TypeRef::Named(*declaration))
            }
            Value::List(_) | Value::None | Value::Some(_) | Value::Ok(_) | Value::Err(_) => {
                self.error(
                    DiagnosticCode::TypeMismatch,
                    "container literals require an explicit expected type",
                    node.source.clone(),
                );
                None
            }
        }
    }

    fn check_value_fields(
        &mut self,
        fields: &[ValueField],
        expected: &[FieldDeclaration],
        node: &NodeMeta,
        depth: usize,
    ) {
        let mut seen = BTreeSet::new();
        for field in fields {
            if !seen.insert(field.field) {
                self.error(
                    DiagnosticCode::DuplicateDeclaration,
                    format!("duplicate value field node {}", field.field.0),
                    node.source.clone(),
                );
            }
            let Some(declared) = expected
                .iter()
                .find(|expected| expected.header.node.id == field.field)
            else {
                self.unresolved("value field", field.field, &node.source);
                continue;
            };
            self.check_value_against(&field.value, &declared.ty, node, depth + 1);
        }
        if seen.len() != expected.len() {
            self.error(
                DiagnosticCode::InvalidInvocation,
                "value does not supply every field exactly once",
                node.source.clone(),
            );
        }
    }

    fn check_value_against(
        &mut self,
        value: &Value,
        expected: &TypeRef,
        node: &NodeMeta,
        depth: usize,
    ) -> bool {
        if depth > MAX_DEPTH {
            self.error(
                DiagnosticCode::ExcessiveComplexity,
                "typed value exceeds the checker depth limit",
                node.source.clone(),
            );
            return false;
        }
        let matches = match (value, expected) {
            (Value::Unit, TypeRef::Unit)
            | (Value::Bool(_), TypeRef::Bool)
            | (Value::I32(_), TypeRef::I32)
            | (Value::I64(_), TypeRef::I64)
            | (Value::F64(_), TypeRef::F64)
            | (Value::Char(_), TypeRef::Char)
            | (Value::String(_), TypeRef::String)
            | (Value::Bytes(_), TypeRef::Bytes)
            | (Value::None, TypeRef::Option(_)) => true,
            (Value::List(values), TypeRef::List(inner)) => values
                .iter()
                .all(|value| self.check_value_against(value, inner, node, depth + 1)),
            (Value::Some(value), TypeRef::Option(inner)) => {
                self.check_value_against(value, inner, node, depth + 1)
            }
            (Value::Ok(value), TypeRef::Result { ok, .. }) => {
                self.check_value_against(value, ok, node, depth + 1)
            }
            (Value::Err(value), TypeRef::Result { error, .. }) => {
                self.check_value_against(value, error, node, depth + 1)
            }
            (
                Value::Record {
                    declaration,
                    fields,
                },
                TypeRef::Named(expected_id),
            ) if declaration == expected_id => {
                if let Some(record) = self.index.record(*declaration) {
                    self.check_value_fields(fields, &record.fields, node, depth + 1);
                    true
                } else {
                    false
                }
            }
            (
                Value::Enum {
                    declaration,
                    variant,
                    fields,
                },
                TypeRef::Named(expected_id),
            ) if declaration == expected_id => {
                if let Some((enumeration, declared_variant)) =
                    self.index.variants.get(variant).copied()
                {
                    if enumeration.header.node.id == *declaration {
                        self.check_value_fields(fields, &declared_variant.fields, node, depth + 1);
                        true
                    } else {
                        false
                    }
                } else {
                    false
                }
            }
            (_, TypeRef::Named(alias)) if self.index.alias(*alias).is_some() => {
                let target = self
                    .index
                    .alias(*alias)
                    .expect("checked above")
                    .target
                    .clone();
                self.check_value_against(value, &target, node, depth + 1)
            }
            _ => false,
        };
        if !matches {
            self.type_error("typed value", node);
        }
        matches
    }

    fn type_error(&mut self, subject: &str, node: &NodeMeta) {
        self.error(
            DiagnosticCode::TypeMismatch,
            format!("{subject} has an incompatible portable type"),
            node.source.clone(),
        );
    }

    fn check_function(&mut self, function: &FunctionDeclaration) {
        let environment = self.check_parameters(&function.parameters);
        self.check_type(
            &function.return_type,
            &function.header.node,
            TypePosition::General,
        );
        let flow = self.check_block(&function.body, &environment, &function.return_type, None, 0);
        if !flow.always_returns
            && !self.same_type(&flow.ty, &function.return_type, &function.body.node)
        {
            self.error(
                DiagnosticCode::InvalidControlFlow,
                format!(
                    "function {:?} can finish without producing its return type",
                    function.header.name
                ),
                function.body.node.source.clone(),
            );
        }
    }

    fn check_implementation(&mut self, implementation: &ImplementationDeclaration) {
        let Some(contract) = self.index.contract(implementation.contract) else {
            self.unresolved(
                "implementation contract",
                implementation.contract,
                &implementation.header.node.source,
            );
            return;
        };
        let Some(record) = self.index.record(implementation.record) else {
            self.unresolved(
                "implementation record",
                implementation.record,
                &implementation.header.node.source,
            );
            return;
        };
        let duplicates = self
            .document
            .module
            .declarations
            .iter()
            .filter_map(|declaration| match declaration {
                Declaration::Implementation(candidate)
                    if candidate.contract == implementation.contract
                        && candidate.record == implementation.record =>
                {
                    Some(candidate.header.node.id)
                }
                _ => None,
            })
            .collect::<Vec<_>>();
        if duplicates.len() > 1 && duplicates[0] != implementation.header.node.id {
            self.error(
                DiagnosticCode::DuplicateDeclaration,
                "duplicate implementation for the same record and contract",
                implementation.header.node.source.clone(),
            );
        }

        let mut supplied = BTreeSet::new();
        for method in &implementation.methods {
            self.check_identifier(&method.header.name, &method.header.node);
            if !supplied.insert(method.contract_method) {
                self.error(
                    DiagnosticCode::DuplicateDeclaration,
                    format!("duplicate implemented method {:?}", method.header.name),
                    method.header.node.source.clone(),
                );
            }
            let Some((owner, required)) = self
                .index
                .contract_methods
                .get(&method.contract_method)
                .copied()
            else {
                self.error(
                    DiagnosticCode::ContractNonconformance,
                    format!(
                        "extra method {:?} does not reference a required contract method",
                        method.header.name
                    ),
                    method.header.node.source.clone(),
                );
                continue;
            };
            if owner.header.node.id != contract.header.node.id {
                self.error(
                    DiagnosticCode::ContractNonconformance,
                    format!(
                        "method {:?} belongs to a different contract",
                        method.header.name
                    ),
                    method.header.node.source.clone(),
                );
                continue;
            }
            if !self.signatures_match(method, required) {
                self.error(
                    DiagnosticCode::ContractNonconformance,
                    format!(
                        "method {:?} does not match its contract signature",
                        method.header.name
                    ),
                    method.header.node.source.clone(),
                );
            }
            let environment = self.check_parameters(&method.parameters);
            self.check_type(
                &method.return_type,
                &method.header.node,
                TypePosition::General,
            );
            let flow = self.check_block(
                &method.body,
                &environment,
                &method.return_type,
                Some(TypeRef::Named(record.header.node.id)),
                0,
            );
            if !flow.always_returns
                && !self.same_type(&flow.ty, &method.return_type, &method.body.node)
            {
                self.error(
                    DiagnosticCode::InvalidControlFlow,
                    format!(
                        "method {:?} can finish without producing its return type",
                        method.header.name
                    ),
                    method.body.node.source.clone(),
                );
            }
        }
        for required in &contract.methods {
            if !supplied.contains(&required.header.node.id) {
                self.error(
                    DiagnosticCode::ContractNonconformance,
                    format!(
                        "implementation is missing method {:?}",
                        required.header.name
                    ),
                    implementation.header.node.source.clone(),
                );
            }
        }
    }

    fn signatures_match(
        &mut self,
        implementation: &MethodImplementation,
        required: &MethodSignature,
    ) -> bool {
        implementation.parameters.len() == required.parameters.len()
            && implementation
                .parameters
                .iter()
                .zip(&required.parameters)
                .all(|(actual, expected)| {
                    self.same_type(&actual.ty, &expected.ty, &actual.header.node)
                })
            && self.same_type(
                &implementation.return_type,
                &required.return_type,
                &implementation.header.node,
            )
    }

    fn check_block(
        &mut self,
        block: &Block,
        outer: &Environment,
        expected_return: &TypeRef,
        self_type: Option<TypeRef>,
        depth: usize,
    ) -> Flow {
        if depth > MAX_DEPTH {
            self.error(
                DiagnosticCode::ExcessiveComplexity,
                "block exceeds the checker depth limit",
                block.node.source.clone(),
            );
            return Flow {
                ty: TypeRef::Unit,
                always_returns: false,
            };
        }
        let mut environment = outer.clone();
        let mut always_returns = false;
        for statement in &block.statements {
            let node = statement_node(statement);
            if always_returns {
                self.error(
                    DiagnosticCode::InvalidControlFlow,
                    "statement is unreachable after return",
                    node.source.clone(),
                );
            }
            match statement {
                Statement::Let {
                    name,
                    annotation,
                    value,
                    ..
                } => {
                    self.check_identifier(name, node);
                    let actual = self.check_expression(
                        value,
                        &environment,
                        expected_return,
                        self_type.clone(),
                        depth + 1,
                    );
                    let ty = annotation
                        .clone()
                        .or(actual.clone())
                        .unwrap_or(TypeRef::Unit);
                    if let Some(annotation) = annotation {
                        self.check_type(annotation, node, TypePosition::General);
                        if actual
                            .as_ref()
                            .is_some_and(|actual| !self.same_type(actual, annotation, node))
                        {
                            self.type_error("let binding", node);
                        }
                    }
                    if environment
                        .insert(
                            name.clone(),
                            Binding {
                                symbol: SymbolId::new(node.id),
                                ty,
                            },
                        )
                        .is_some()
                    {
                        self.error(
                            DiagnosticCode::DuplicateDeclaration,
                            format!("local binding {name:?} shadows an existing name"),
                            node.source.clone(),
                        );
                    }
                    if expression_always_returns(value) {
                        always_returns = true;
                    }
                }
                Statement::ForEach {
                    binding,
                    iterable,
                    body,
                    ..
                } => {
                    self.check_identifier(binding, node);
                    self.require(node.id, Capability::BoundedIteration);
                    let iterable_type = self.check_expression(
                        iterable,
                        &environment,
                        expected_return,
                        self_type.clone(),
                        depth + 1,
                    );
                    let Some(TypeRef::List(element)) = iterable_type else {
                        self.type_error("for-each iterable", node);
                        continue;
                    };
                    let mut body_environment = environment.clone();
                    if body_environment
                        .insert(
                            binding.clone(),
                            Binding {
                                symbol: SymbolId::new(node.id),
                                ty: *element,
                            },
                        )
                        .is_some()
                    {
                        self.error(
                            DiagnosticCode::DuplicateDeclaration,
                            format!("for-each binding {binding:?} shadows an existing name"),
                            node.source.clone(),
                        );
                    }
                    let body_flow = self.check_block(
                        body,
                        &body_environment,
                        expected_return,
                        self_type.clone(),
                        depth + 1,
                    );
                    if body_flow.ty != TypeRef::Unit && !body_flow.always_returns {
                        self.type_error("for-each body", &body.node);
                    }
                }
                Statement::Return { value, .. } => {
                    let actual = match value {
                        Some(value) => self.check_expression(
                            value,
                            &environment,
                            expected_return,
                            self_type.clone(),
                            depth + 1,
                        ),
                        None => Some(TypeRef::Unit),
                    };
                    if actual
                        .as_ref()
                        .is_some_and(|actual| !self.is_assignable(actual, expected_return, node))
                    {
                        self.type_error("return value", node);
                    }
                    always_returns = true;
                }
                Statement::Expression { value, .. } => {
                    self.check_expression(
                        value,
                        &environment,
                        expected_return,
                        self_type.clone(),
                        depth + 1,
                    );
                    if expression_always_returns(value) {
                        always_returns = true;
                    }
                }
            }
        }
        let result_always_returns = block
            .result
            .as_deref()
            .is_some_and(expression_always_returns);
        let ty = match &block.result {
            Some(result) => {
                if always_returns {
                    self.error(
                        DiagnosticCode::InvalidControlFlow,
                        "block result is unreachable after return",
                        result.node().source.clone(),
                    );
                }
                self.check_expression(result, &environment, expected_return, self_type, depth + 1)
                    .unwrap_or(TypeRef::Unit)
            }
            None if always_returns => expected_return.clone(),
            None => TypeRef::Unit,
        };
        Flow {
            ty,
            always_returns: always_returns || result_always_returns,
        }
    }

    fn check_expression(
        &mut self,
        expression: &Expression,
        environment: &Environment,
        expected_return: &TypeRef,
        self_type: Option<TypeRef>,
        depth: usize,
    ) -> Option<TypeRef> {
        let node = expression.node();
        if depth > MAX_DEPTH {
            self.error(
                DiagnosticCode::ExcessiveComplexity,
                "expression exceeds the checker depth limit",
                node.source.clone(),
            );
            return None;
        }
        let ty = match expression {
            Expression::Literal { value, .. } => self.infer_value(value, node, depth + 1),
            Expression::Local { name, .. } => {
                if let Some(binding) = environment.get(name) {
                    self.local_references.insert(node.id, binding.symbol);
                    Some(binding.ty.clone())
                } else {
                    self.error(
                        DiagnosticCode::UnresolvedReference,
                        format!("unknown local {name:?}"),
                        node.source.clone(),
                    );
                    None
                }
            }
            Expression::Constant { declaration, .. } => {
                match self.index.declaration(*declaration) {
                    Some(Declaration::Constant(constant)) => Some(constant.ty.clone()),
                    _ => {
                        self.unresolved("constant", *declaration, &node.source);
                        None
                    }
                }
            }
            Expression::SelfValue { .. } => self_type.or_else(|| {
                self.error(
                    DiagnosticCode::UnresolvedReference,
                    "self is available only inside an implementation method",
                    node.source.clone(),
                );
                None
            }),
            Expression::ConstructRecord {
                declaration,
                fields,
                ..
            } => self.check_expression_record(
                *declaration,
                fields,
                node,
                environment,
                expected_return,
                self_type,
                depth + 1,
            ),
            Expression::ConstructEnum {
                declaration,
                variant,
                fields,
                ..
            } => self.check_expression_enum(
                *declaration,
                *variant,
                fields,
                node,
                environment,
                expected_return,
                self_type,
                depth + 1,
            ),
            Expression::ConstructSome { value, .. } => Some(TypeRef::Option(Box::new(
                self.check_expression(value, environment, expected_return, self_type, depth + 1)?,
            ))),
            Expression::ConstructNone { inner_type, .. } => {
                self.check_type(inner_type, node, TypePosition::General);
                Some(TypeRef::Option(Box::new(inner_type.clone())))
            }
            Expression::ConstructOk {
                value, error_type, ..
            } => {
                self.check_type(error_type, node, TypePosition::General);
                Some(TypeRef::Result {
                    ok: Box::new(self.check_expression(
                        value,
                        environment,
                        expected_return,
                        self_type,
                        depth + 1,
                    )?),
                    error: Box::new(error_type.clone()),
                })
            }
            Expression::ConstructErr { value, ok_type, .. } => {
                self.check_type(ok_type, node, TypePosition::General);
                Some(TypeRef::Result {
                    ok: Box::new(ok_type.clone()),
                    error: Box::new(self.check_expression(
                        value,
                        environment,
                        expected_return,
                        self_type,
                        depth + 1,
                    )?),
                })
            }
            Expression::ConstructList {
                element_type,
                elements,
                ..
            } => {
                self.check_type(element_type, node, TypePosition::General);
                for element in elements {
                    let actual = self.check_expression(
                        element,
                        environment,
                        expected_return,
                        self_type.clone(),
                        depth + 1,
                    );
                    if actual
                        .as_ref()
                        .is_some_and(|actual| !self.same_type(actual, element_type, node))
                    {
                        self.type_error("list element", node);
                    }
                }
                Some(TypeRef::List(Box::new(element_type.clone())))
            }
            Expression::Field { base, field, .. } => {
                let base = self.check_expression(
                    base,
                    environment,
                    expected_return,
                    self_type,
                    depth + 1,
                )?;
                let mut stack = Vec::new();
                let normalized =
                    self.normalize_type(&base, node, TypePosition::General, &mut stack)?;
                let TypeRef::Named(record_id) = normalized else {
                    self.type_error("field receiver", node);
                    return None;
                };
                match self.index.fields.get(field).copied() {
                    Some((record, declared)) if record.header.node.id == record_id => {
                        Some(declared.ty.clone())
                    }
                    _ => {
                        self.unresolved("record field", *field, &node.source);
                        None
                    }
                }
            }
            Expression::Call {
                function,
                arguments,
                ..
            } => self.check_call(
                *function,
                arguments,
                node,
                environment,
                expected_return,
                self_type,
                depth + 1,
            ),
            Expression::MethodCall {
                receiver,
                dispatch,
                arguments,
                ..
            } => self.check_method_call(
                receiver,
                dispatch,
                arguments,
                node,
                environment,
                expected_return,
                self_type,
                depth + 1,
            ),
            Expression::Intrinsic {
                operation,
                arguments,
                ..
            } => {
                let types = arguments
                    .iter()
                    .map(|argument| {
                        self.check_expression(
                            argument,
                            environment,
                            expected_return,
                            self_type.clone(),
                            depth + 1,
                        )
                    })
                    .collect::<Option<Vec<_>>>()?;
                self.check_intrinsic(*operation, &types, node)
            }
            Expression::If {
                condition,
                then_block,
                else_block,
                ..
            } => {
                let condition_type = self.check_expression(
                    condition,
                    environment,
                    expected_return,
                    self_type.clone(),
                    depth + 1,
                );
                if condition_type.as_ref() != Some(&TypeRef::Bool) {
                    self.type_error("if condition", condition.node());
                }
                let then_flow = self.check_block(
                    then_block,
                    environment,
                    expected_return,
                    self_type.clone(),
                    depth + 1,
                );
                let else_flow = self.check_block(
                    else_block,
                    environment,
                    expected_return,
                    self_type,
                    depth + 1,
                );
                self.combine_branch_types(&then_flow, &else_flow, node)
            }
            Expression::Match { value, arms, .. } => self.check_match(
                value,
                arms,
                node,
                environment,
                expected_return,
                self_type,
                depth + 1,
            ),
            Expression::Block(block) => Some(
                self.check_block(block, environment, expected_return, self_type, depth + 1)
                    .ty,
            ),
        };
        if let Some(ty) = &ty {
            self.expression_types.insert(node.id, ty.clone());
            self.collect_type_capabilities(node.id, ty);
        }
        ty
    }

    fn combine_branch_types(
        &mut self,
        left: &Flow,
        right: &Flow,
        node: &NodeMeta,
    ) -> Option<TypeRef> {
        match (left.always_returns, right.always_returns) {
            (true, true) => Some(left.ty.clone()),
            (true, false) => Some(right.ty.clone()),
            (false, true) => Some(left.ty.clone()),
            (false, false) if self.same_type(&left.ty, &right.ty, node) => Some(left.ty.clone()),
            (false, false) => {
                self.error(
                    DiagnosticCode::InvalidControlFlow,
                    "branches produce incompatible types",
                    node.source.clone(),
                );
                None
            }
        }
    }

    fn is_assignable(&mut self, actual: &TypeRef, expected: &TypeRef, node: &NodeMeta) -> bool {
        if actual == expected {
            return true;
        }
        if let (TypeRef::Named(record), TypeRef::Contract(contract)) = (actual, expected) {
            return self.document.module.declarations.iter().any(|declaration| {
                matches!(
                    declaration,
                    Declaration::Implementation(implementation)
                        if implementation.record == *record
                            && implementation.contract == *contract
                )
            });
        }
        if matches!(actual, TypeRef::Contract(_)) || matches!(expected, TypeRef::Contract(_)) {
            return false;
        }
        self.same_type(actual, expected, node)
    }

    #[allow(clippy::too_many_arguments)]
    fn check_expression_record(
        &mut self,
        declaration: NodeId,
        fields: &[ExpressionField],
        node: &NodeMeta,
        environment: &Environment,
        expected_return: &TypeRef,
        self_type: Option<TypeRef>,
        depth: usize,
    ) -> Option<TypeRef> {
        let Some(record) = self.index.record(declaration) else {
            self.unresolved("record", declaration, &node.source);
            return None;
        };
        self.check_expression_fields(
            fields,
            &record.fields,
            node,
            environment,
            expected_return,
            self_type,
            depth,
        );
        Some(TypeRef::Named(declaration))
    }

    #[allow(clippy::too_many_arguments)]
    fn check_expression_enum(
        &mut self,
        declaration: NodeId,
        variant: NodeId,
        fields: &[ExpressionField],
        node: &NodeMeta,
        environment: &Environment,
        expected_return: &TypeRef,
        self_type: Option<TypeRef>,
        depth: usize,
    ) -> Option<TypeRef> {
        let Some((enumeration, declared_variant)) = self.index.variants.get(&variant).copied()
        else {
            self.unresolved("enum variant", variant, &node.source);
            return None;
        };
        if enumeration.header.node.id != declaration {
            self.type_error("enum variant declaration", node);
            return None;
        }
        self.check_expression_fields(
            fields,
            &declared_variant.fields,
            node,
            environment,
            expected_return,
            self_type,
            depth,
        );
        Some(TypeRef::Named(declaration))
    }

    #[allow(clippy::too_many_arguments)]
    fn check_expression_fields(
        &mut self,
        supplied: &[ExpressionField],
        expected: &[FieldDeclaration],
        node: &NodeMeta,
        environment: &Environment,
        expected_return: &TypeRef,
        self_type: Option<TypeRef>,
        depth: usize,
    ) {
        let mut seen = BTreeSet::new();
        for field in supplied {
            if !seen.insert(field.field) {
                self.error(
                    DiagnosticCode::DuplicateDeclaration,
                    format!("duplicate expression field node {}", field.field.0),
                    node.source.clone(),
                );
            }
            let Some(declared) = expected
                .iter()
                .find(|expected| expected.header.node.id == field.field)
            else {
                self.unresolved("aggregate field", field.field, &node.source);
                continue;
            };
            let actual = self.check_expression(
                &field.value,
                environment,
                expected_return,
                self_type.clone(),
                depth + 1,
            );
            if actual
                .as_ref()
                .is_some_and(|actual| !self.is_assignable(actual, &declared.ty, node))
            {
                self.type_error("aggregate field expression", node);
            }
        }
        if seen.len() != expected.len() {
            self.error(
                DiagnosticCode::InvalidInvocation,
                "aggregate constructor does not supply every field exactly once",
                node.source.clone(),
            );
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn check_call(
        &mut self,
        function: NodeId,
        arguments: &[Expression],
        node: &NodeMeta,
        environment: &Environment,
        expected_return: &TypeRef,
        self_type: Option<TypeRef>,
        depth: usize,
    ) -> Option<TypeRef> {
        let Some(function) = self.index.function(function) else {
            self.unresolved("function", function, &node.source);
            return None;
        };
        self.check_arguments(
            arguments,
            &function.parameters,
            node,
            environment,
            expected_return,
            self_type,
            depth,
        );
        Some(function.return_type.clone())
    }

    #[allow(clippy::too_many_arguments)]
    fn check_method_call(
        &mut self,
        receiver: &Expression,
        dispatch: &MethodDispatch,
        arguments: &[Expression],
        node: &NodeMeta,
        environment: &Environment,
        expected_return: &TypeRef,
        self_type: Option<TypeRef>,
        depth: usize,
    ) -> Option<TypeRef> {
        let receiver_type = self.check_expression(
            receiver,
            environment,
            expected_return,
            self_type.clone(),
            depth + 1,
        )?;
        self.require(node.id, Capability::ContractDispatch);
        match dispatch {
            MethodDispatch::Concrete {
                implementation,
                method,
            } => {
                let Some(implementation) = self.index.implementation(*implementation) else {
                    self.unresolved("implementation", *implementation, &node.source);
                    return None;
                };
                let Some((owner, method)) = self.index.implementation_methods.get(method).copied()
                else {
                    self.unresolved("implementation method", *method, &node.source);
                    return None;
                };
                if owner.header.node.id != implementation.header.node.id
                    || !self.is_assignable(
                        &receiver_type,
                        &TypeRef::Named(implementation.record),
                        node,
                    )
                {
                    self.error(
                        DiagnosticCode::InvalidInvocation,
                        "concrete method receiver or dispatch owner does not match",
                        node.source.clone(),
                    );
                }
                self.check_arguments(
                    arguments,
                    &method.parameters,
                    node,
                    environment,
                    expected_return,
                    self_type,
                    depth,
                );
                Some(method.return_type.clone())
            }
            MethodDispatch::Contract { contract, method } => {
                let Some((owner, method)) = self.index.contract_methods.get(method).copied() else {
                    self.unresolved("contract method", *method, &node.source);
                    return None;
                };
                if owner.header.node.id != *contract
                    || !self.is_assignable(&receiver_type, &TypeRef::Contract(*contract), node)
                {
                    self.error(
                        DiagnosticCode::InvalidInvocation,
                        "contract method receiver or dispatch owner does not match",
                        node.source.clone(),
                    );
                }
                self.check_arguments(
                    arguments,
                    &method.parameters,
                    node,
                    environment,
                    expected_return,
                    self_type,
                    depth,
                );
                Some(method.return_type.clone())
            }
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn check_arguments(
        &mut self,
        arguments: &[Expression],
        parameters: &[Parameter],
        node: &NodeMeta,
        environment: &Environment,
        expected_return: &TypeRef,
        self_type: Option<TypeRef>,
        depth: usize,
    ) {
        if arguments.len() != parameters.len() {
            self.error(
                DiagnosticCode::InvalidInvocation,
                format!(
                    "invocation has {} arguments but requires {}",
                    arguments.len(),
                    parameters.len()
                ),
                node.source.clone(),
            );
        }
        for (argument, parameter) in arguments.iter().zip(parameters) {
            let actual = self.check_expression(
                argument,
                environment,
                expected_return,
                self_type.clone(),
                depth + 1,
            );
            if actual
                .as_ref()
                .is_some_and(|actual| !self.is_assignable(actual, &parameter.ty, argument.node()))
            {
                self.error(
                    DiagnosticCode::InvalidInvocation,
                    format!(
                        "argument for parameter {:?} has the wrong type",
                        parameter.header.name
                    ),
                    argument.node().source.clone(),
                );
            }
        }
    }

    fn check_intrinsic(
        &mut self,
        operation: Intrinsic,
        arguments: &[TypeRef],
        node: &NodeMeta,
    ) -> Option<TypeRef> {
        use Intrinsic::{
            BoolAnd, BoolNot, BoolOr, BytesConcat, BytesIsEmpty, BytesLength, Equal, FloatAdd,
            FloatDiv, FloatMul, FloatNeg, FloatRemTrunc, FloatSub, FloatTrunc, Greater,
            GreaterEqual, IntAddChecked, IntAddWrapping, IntBitAnd, IntBitNot, IntBitOr, IntBitXor,
            IntDivChecked, IntMulChecked, IntMulWrapping, IntNegChecked, IntNegWrapping,
            IntRemChecked, IntShiftLeftChecked, IntShiftRightChecked, IntSubChecked,
            IntSubWrapping, Less, LessEqual, ListAppend, ListConcat, ListContains, ListGetChecked,
            ListIsEmpty, ListLength, NarrowI64ToI32Checked, NotEqual, OptionIsNone, OptionIsSome,
            OptionUnwrapOr, ResultIsErr, ResultIsOk, StringConcat, StringContains, StringEndsWith,
            StringFromUtf8Checked, StringIsEmpty, StringReplaceAll, StringReplaceMany,
            StringScalarLength, StringStartsWith, StringStripPrefix, StringToUtf8, StringTrimEnd,
            StringTrimStart, StringTruncateUtf8Bytes, WidenI32ToI64,
        };
        let invalid = |checker: &mut Self| {
            checker.error(
                DiagnosticCode::InvalidInvocation,
                format!("intrinsic {operation:?} does not accept operand types {arguments:?}"),
                node.source.clone(),
            );
            None
        };
        let result = match operation {
            BoolNot if arguments == [TypeRef::Bool] => Some(TypeRef::Bool),
            BoolAnd | BoolOr if arguments == [TypeRef::Bool, TypeRef::Bool] => Some(TypeRef::Bool),
            Equal | NotEqual if arguments.len() == 2 && arguments[0] == arguments[1] => {
                if matches!(arguments[0], TypeRef::Contract(_)) {
                    self.error(
                        DiagnosticCode::InvalidContractPosition,
                        "contract values cannot be compared for equality",
                        node.source.clone(),
                    );
                    None
                } else {
                    Some(TypeRef::Bool)
                }
            }
            Less | LessEqual | Greater | GreaterEqual
                if arguments.len() == 2
                    && arguments[0] == arguments[1]
                    && matches!(
                        arguments[0],
                        TypeRef::I32
                            | TypeRef::I64
                            | TypeRef::F64
                            | TypeRef::Char
                            | TypeRef::String
                    ) =>
            {
                Some(TypeRef::Bool)
            }
            IntNegChecked | IntNegWrapping | IntBitNot
                if arguments.len() == 1 && is_integer(&arguments[0]) =>
            {
                Some(arguments[0].clone())
            }
            IntAddChecked | IntSubChecked | IntMulChecked | IntDivChecked | IntRemChecked
            | IntAddWrapping | IntSubWrapping | IntMulWrapping | IntBitAnd | IntBitOr
            | IntBitXor | IntShiftLeftChecked | IntShiftRightChecked
                if arguments.len() == 2
                    && arguments[0] == arguments[1]
                    && is_integer(&arguments[0]) =>
            {
                Some(arguments[0].clone())
            }
            FloatNeg | FloatTrunc if arguments == [TypeRef::F64] => Some(TypeRef::F64),
            FloatAdd | FloatSub | FloatMul | FloatDiv | FloatRemTrunc
                if arguments == [TypeRef::F64, TypeRef::F64] =>
            {
                Some(TypeRef::F64)
            }
            StringConcat if arguments == [TypeRef::String, TypeRef::String] => {
                Some(TypeRef::String)
            }
            StringScalarLength if arguments == [TypeRef::String] => Some(TypeRef::I64),
            StringIsEmpty if arguments == [TypeRef::String] => Some(TypeRef::Bool),
            StringContains | StringStartsWith | StringEndsWith
                if arguments == [TypeRef::String, TypeRef::String] =>
            {
                Some(TypeRef::Bool)
            }
            StringReplaceAll
                if arguments == [TypeRef::String, TypeRef::String, TypeRef::String] =>
            {
                Some(TypeRef::String)
            }
            StringReplaceMany
                if arguments.len() >= 3
                    && !arguments.len().is_multiple_of(2)
                    && arguments
                        .iter()
                        .all(|argument| *argument == TypeRef::String) =>
            {
                Some(TypeRef::String)
            }
            StringTruncateUtf8Bytes if arguments == [TypeRef::String, TypeRef::F64] => {
                Some(TypeRef::String)
            }
            StringStripPrefix | StringTrimStart | StringTrimEnd
                if arguments == [TypeRef::String, TypeRef::String] =>
            {
                Some(TypeRef::String)
            }
            BytesConcat if arguments == [TypeRef::Bytes, TypeRef::Bytes] => Some(TypeRef::Bytes),
            BytesLength if arguments == [TypeRef::Bytes] => Some(TypeRef::I64),
            BytesIsEmpty if arguments == [TypeRef::Bytes] => Some(TypeRef::Bool),
            ListLength if matches!(arguments, [TypeRef::List(_)]) => Some(TypeRef::I64),
            ListIsEmpty if matches!(arguments, [TypeRef::List(_)]) => Some(TypeRef::Bool),
            ListGetChecked if matches!(arguments, [TypeRef::List(_), TypeRef::I64]) => {
                let TypeRef::List(element) = &arguments[0] else {
                    unreachable!()
                };
                Some((**element).clone())
            }
            ListAppend
                if arguments.len() == 2
                    && matches!(&arguments[0], TypeRef::List(element) if **element == arguments[1]) =>
            {
                Some(arguments[0].clone())
            }
            ListConcat
                if arguments.len() == 2
                    && arguments[0] == arguments[1]
                    && matches!(arguments[0], TypeRef::List(_)) =>
            {
                Some(arguments[0].clone())
            }
            ListContains
                if arguments.len() == 2
                    && matches!(&arguments[0], TypeRef::List(element) if **element == arguments[1]) =>
            {
                Some(TypeRef::Bool)
            }
            OptionIsSome | OptionIsNone if matches!(arguments, [TypeRef::Option(_)]) => {
                Some(TypeRef::Bool)
            }
            OptionUnwrapOr
                if arguments.len() == 2
                    && matches!(&arguments[0], TypeRef::Option(inner) if **inner == arguments[1]) =>
            {
                Some(arguments[1].clone())
            }
            ResultIsOk | ResultIsErr if matches!(arguments, [TypeRef::Result { .. }]) => {
                Some(TypeRef::Bool)
            }
            WidenI32ToI64 if arguments == [TypeRef::I32] => Some(TypeRef::I64),
            NarrowI64ToI32Checked if arguments == [TypeRef::I64] => Some(TypeRef::I32),
            StringToUtf8 if arguments == [TypeRef::String] => Some(TypeRef::Bytes),
            StringFromUtf8Checked if arguments == [TypeRef::Bytes] => Some(TypeRef::String),
            _ => invalid(self),
        };

        if matches!(
            operation,
            IntNegChecked
                | IntAddChecked
                | IntSubChecked
                | IntMulChecked
                | IntDivChecked
                | IntRemChecked
                | IntShiftLeftChecked
                | IntShiftRightChecked
                | NarrowI64ToI32Checked
        ) {
            self.require(node.id, Capability::CheckedIntegerArithmetic);
        }
        if matches!(
            operation,
            IntNegWrapping | IntAddWrapping | IntSubWrapping | IntMulWrapping
        ) {
            self.require(node.id, Capability::WrappingIntegerArithmetic);
        }
        result
    }

    #[allow(clippy::too_many_arguments)]
    fn check_match(
        &mut self,
        value: &Expression,
        arms: &[MatchArm],
        node: &NodeMeta,
        environment: &Environment,
        expected_return: &TypeRef,
        self_type: Option<TypeRef>,
        depth: usize,
    ) -> Option<TypeRef> {
        let matched_type = self.check_expression(
            value,
            environment,
            expected_return,
            self_type.clone(),
            depth + 1,
        )?;
        let mut stack = Vec::new();
        let matched_type =
            self.normalize_type(&matched_type, node, TypePosition::General, &mut stack)?;
        let expected_coverage = self.expected_coverage(&matched_type, node)?;
        let mut covered = BTreeSet::new();
        let mut wildcard = false;
        let mut combined: Option<Flow> = None;

        for arm in arms {
            let pattern_node = pattern_node(&arm.pattern);
            let mut arm_environment = environment.clone();
            let key = self.check_pattern(&arm.pattern, &matched_type, &mut arm_environment, node);
            if wildcard || key.as_ref().is_some_and(|key| !covered.insert(key.clone())) {
                self.error(
                    DiagnosticCode::UnreachablePattern,
                    "match pattern is duplicate or unreachable",
                    pattern_node.source.clone(),
                );
            }
            if key == Some(Coverage::Wildcard) {
                wildcard = true;
            }
            let flow = self.check_block(
                &arm.body,
                &arm_environment,
                expected_return,
                self_type.clone(),
                depth + 1,
            );
            combined = match combined {
                None => Some(flow),
                Some(previous) => {
                    let ty = self.combine_branch_types(&previous, &flow, node)?;
                    Some(Flow {
                        ty,
                        always_returns: previous.always_returns && flow.always_returns,
                    })
                }
            };
        }

        if !wildcard && !expected_coverage.is_subset(&covered) {
            self.error(
                DiagnosticCode::NonExhaustiveMatch,
                format!(
                    "match is not exhaustive; missing {:?}",
                    expected_coverage.difference(&covered).collect::<Vec<_>>()
                ),
                node.source.clone(),
            );
        }
        combined.map(|flow| flow.ty)
    }

    fn expected_coverage(&mut self, ty: &TypeRef, node: &NodeMeta) -> Option<BTreeSet<Coverage>> {
        let mut stack = Vec::new();
        let normalized = self.normalize_type(ty, node, TypePosition::General, &mut stack)?;
        match normalized {
            TypeRef::Bool => Some(BTreeSet::from([
                Coverage::Bool(false),
                Coverage::Bool(true),
            ])),
            TypeRef::Option(_) => Some(BTreeSet::from([Coverage::None, Coverage::Some])),
            TypeRef::Result { .. } => Some(BTreeSet::from([Coverage::Ok, Coverage::Err])),
            TypeRef::Named(id) => {
                let Some(enumeration) = self.index.enumeration(id) else {
                    self.type_error("match input", node);
                    return None;
                };
                Some(
                    enumeration
                        .variants
                        .iter()
                        .map(|variant| Coverage::Variant(variant.header.node.id))
                        .collect(),
                )
            }
            _ => {
                self.type_error("match input", node);
                None
            }
        }
    }

    fn check_pattern(
        &mut self,
        pattern: &Pattern,
        matched_type: &TypeRef,
        environment: &mut Environment,
        match_node: &NodeMeta,
    ) -> Option<Coverage> {
        let node = pattern_node(pattern);
        match pattern {
            Pattern::Wildcard { .. } => Some(Coverage::Wildcard),
            Pattern::Bool { value, .. } => {
                if !self.same_type(matched_type, &TypeRef::Bool, match_node) {
                    self.type_error("boolean pattern", node);
                }
                Some(Coverage::Bool(*value))
            }
            Pattern::None { .. } => {
                if !matches!(matched_type, TypeRef::Option(_)) {
                    self.type_error("None pattern", node);
                }
                Some(Coverage::None)
            }
            Pattern::Some { binding, .. } => {
                let TypeRef::Option(inner) = matched_type else {
                    self.type_error("Some pattern", node);
                    return Some(Coverage::Some);
                };
                self.add_pattern_binding(environment, binding, (**inner).clone(), node);
                Some(Coverage::Some)
            }
            Pattern::Ok { binding, .. } => {
                let TypeRef::Result { ok, .. } = matched_type else {
                    self.type_error("Ok pattern", node);
                    return Some(Coverage::Ok);
                };
                self.add_pattern_binding(environment, binding, (**ok).clone(), node);
                Some(Coverage::Ok)
            }
            Pattern::Err { binding, .. } => {
                let TypeRef::Result { error, .. } = matched_type else {
                    self.type_error("Err pattern", node);
                    return Some(Coverage::Err);
                };
                self.add_pattern_binding(environment, binding, (**error).clone(), node);
                Some(Coverage::Err)
            }
            Pattern::EnumVariant {
                declaration,
                variant,
                bindings,
                ..
            } => {
                if !self.same_type(matched_type, &TypeRef::Named(*declaration), match_node) {
                    self.type_error("enum pattern", node);
                }
                let Some((enumeration, declared_variant)) =
                    self.index.variants.get(variant).copied()
                else {
                    self.unresolved("enum pattern variant", *variant, &node.source);
                    return None;
                };
                if enumeration.header.node.id != *declaration {
                    self.type_error("enum pattern declaration", node);
                }
                self.check_field_bindings(bindings, &declared_variant.fields, environment, node);
                Some(Coverage::Variant(*variant))
            }
        }
    }

    fn check_field_bindings(
        &mut self,
        bindings: &[FieldBinding],
        fields: &[FieldDeclaration],
        environment: &mut Environment,
        node: &NodeMeta,
    ) {
        let mut seen = BTreeSet::new();
        for binding in bindings {
            if !seen.insert(binding.field) {
                self.error(
                    DiagnosticCode::UnreachablePattern,
                    format!("duplicate field binding node {}", binding.field.0),
                    node.source.clone(),
                );
            }
            let Some(field) = fields
                .iter()
                .find(|field| field.header.node.id == binding.field)
            else {
                self.unresolved("pattern field", binding.field, &node.source);
                continue;
            };
            self.add_pattern_binding(environment, &binding.binding, field.ty.clone(), node);
        }
        if seen.len() != fields.len() {
            self.error(
                DiagnosticCode::InvalidInvocation,
                "enum pattern must bind every payload field exactly once",
                node.source.clone(),
            );
        }
    }

    fn add_pattern_binding(
        &mut self,
        environment: &mut Environment,
        name: &str,
        ty: TypeRef,
        node: &NodeMeta,
    ) {
        self.check_identifier(name, node);
        if environment
            .insert(
                name.to_owned(),
                Binding {
                    symbol: SymbolId::new(node.id),
                    ty,
                },
            )
            .is_some()
        {
            self.error(
                DiagnosticCode::DuplicateDeclaration,
                format!("pattern binding {name:?} shadows an existing name"),
                node.source.clone(),
            );
        }
    }

    fn check_test(&mut self, test: &TestDeclaration) {
        match &test.invocation {
            TestInvocation::Function {
                function,
                arguments,
            } => {
                let Some(function) = self.index.function(*function) else {
                    self.test_error(
                        format!("test calls unresolved function node {}", function.0),
                        test,
                    );
                    return;
                };
                self.check_typed_arguments(
                    arguments,
                    &function.parameters,
                    &test.header.node,
                    test,
                );
                self.check_expected(&test.expected, &function.return_type, test);
            }
            TestInvocation::Method {
                implementation,
                method,
                receiver,
                arguments,
            } => {
                let Some(implementation) = self.index.implementation(*implementation) else {
                    self.test_error(
                        format!(
                            "test calls unresolved implementation node {}",
                            implementation.0
                        ),
                        test,
                    );
                    return;
                };
                let Some((owner, method)) = self.index.implementation_methods.get(method).copied()
                else {
                    self.test_error(
                        format!("test calls unresolved method node {}", method.0),
                        test,
                    );
                    return;
                };
                if owner.header.node.id != implementation.header.node.id {
                    self.test_error("test method belongs to another implementation", test);
                }
                self.check_typed_value(
                    receiver,
                    Some(&TypeRef::Named(implementation.record)),
                    &test.header.node,
                    test,
                );
                self.check_typed_arguments(arguments, &method.parameters, &test.header.node, test);
                self.check_expected(&test.expected, &method.return_type, test);
            }
        }
    }

    fn check_typed_arguments(
        &mut self,
        arguments: &[TypedValue],
        parameters: &[Parameter],
        node: &NodeMeta,
        test: &TestDeclaration,
    ) {
        if arguments.len() != parameters.len() {
            self.test_error(
                format!(
                    "test has {} arguments but invocation requires {}",
                    arguments.len(),
                    parameters.len()
                ),
                test,
            );
        }
        for (argument, parameter) in arguments.iter().zip(parameters) {
            self.check_typed_value(argument, Some(&parameter.ty), node, test);
        }
    }

    fn check_expected(
        &mut self,
        expected: &ExpectedOutcome,
        return_type: &TypeRef,
        test: &TestDeclaration,
    ) {
        match expected {
            ExpectedOutcome::Value(value) => {
                self.check_typed_value(value, Some(return_type), &test.header.node, test);
            }
            ExpectedOutcome::Error(value) => {
                self.check_typed_value(value, None, &test.header.node, test);
            }
        }
    }

    fn check_typed_value(
        &mut self,
        value: &TypedValue,
        expected: Option<&TypeRef>,
        node: &NodeMeta,
        test: &TestDeclaration,
    ) {
        self.check_type(&value.ty, node, TypePosition::General);
        if !self.check_value_against(&value.value, &value.ty, node, 0) {
            self.test_error("typed test value does not match its declared type", test);
        }
        if expected.is_some_and(|expected| !self.is_assignable(&value.ty, expected, node)) {
            self.test_error(
                "typed test value does not match the invocation signature",
                test,
            );
        }
    }

    fn test_error(&mut self, message: impl Into<String>, test: &TestDeclaration) {
        self.error(
            DiagnosticCode::InvalidPortableTest,
            message,
            test.header.node.source.clone(),
        );
    }

    fn check_recursion(&mut self) {
        let graph = call_graph(&self.document.module, &self.index);
        for callable in graph.keys().copied() {
            if reaches(&graph, callable, callable, &mut BTreeSet::new()) {
                let source = self.callable_source(callable);
                self.error(
                    DiagnosticCode::RecursiveCall,
                    format!(
                        "callable node {} participates in a recursive cycle",
                        callable.0
                    ),
                    source,
                );
            }
        }
        let constants = constant_graph(&self.document.module);
        for constant in constants.keys().copied() {
            if reaches(&constants, constant, constant, &mut BTreeSet::new()) {
                let source = self
                    .index
                    .declaration(constant)
                    .map(|declaration| declaration.header().node.source.clone())
                    .unwrap_or_else(|| SourceRef::logical(["constant"]));
                self.error(
                    DiagnosticCode::RecursiveCall,
                    format!(
                        "constant node {} participates in a dependency cycle",
                        constant.0
                    ),
                    source,
                );
            }
        }
    }

    fn callable_source(&self, callable: NodeId) -> SourceRef {
        self.index
            .function(callable)
            .map(|function| function.header.node.source.clone())
            .or_else(|| {
                self.index
                    .implementation_methods
                    .get(&callable)
                    .map(|(_, method)| method.header.node.source.clone())
            })
            .unwrap_or_else(|| SourceRef::logical([format!("callable({})", callable.0)]))
    }

    fn unresolved(&mut self, kind: &str, id: NodeId, source: &SourceRef) {
        self.error(
            DiagnosticCode::UnresolvedReference,
            format!("unresolved {kind} node {}", id.0),
            source.clone(),
        );
    }

    fn error(&mut self, code: DiagnosticCode, message: impl Into<String>, source: SourceRef) {
        self.diagnostics
            .push(Diagnostic::error(code, message, source));
    }

    fn collect_type_capabilities(&mut self, node: NodeId, ty: &TypeRef) {
        match ty {
            TypeRef::F64 => self.require(node, Capability::F64),
            TypeRef::Char | TypeRef::String => self.require(node, Capability::UnicodeScalar),
            TypeRef::Bytes => self.require(node, Capability::Bytes),
            TypeRef::List(inner) => {
                self.require(node, Capability::ImmutableList);
                self.collect_type_capabilities(node, inner);
            }
            TypeRef::Option(inner) => {
                self.require(node, Capability::Option);
                self.collect_type_capabilities(node, inner);
            }
            TypeRef::Result { ok, error } => {
                self.require(node, Capability::Result);
                self.collect_type_capabilities(node, ok);
                self.collect_type_capabilities(node, error);
            }
            TypeRef::Contract(_) => self.require(node, Capability::ContractDispatch),
            TypeRef::Unit | TypeRef::Bool | TypeRef::I32 | TypeRef::I64 | TypeRef::Named(_) => {}
        }
    }

    fn require(&mut self, node: NodeId, capability: Capability) {
        self.capabilities
            .require(self.current_declaration, node, capability);
    }
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
enum Coverage {
    Wildcard,
    Bool(bool),
    Variant(NodeId),
    None,
    Some,
    Ok,
    Err,
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

fn statement_node(statement: &Statement) -> &NodeMeta {
    match statement {
        Statement::Let { node, .. }
        | Statement::ForEach { node, .. }
        | Statement::Return { node, .. }
        | Statement::Expression { node, .. } => node,
    }
}

fn pattern_node(pattern: &Pattern) -> &NodeMeta {
    match pattern {
        Pattern::Wildcard { node }
        | Pattern::Bool { node, .. }
        | Pattern::EnumVariant { node, .. }
        | Pattern::None { node }
        | Pattern::Some { node, .. }
        | Pattern::Ok { node, .. }
        | Pattern::Err { node, .. } => node,
    }
}

fn is_integer(ty: &TypeRef) -> bool {
    matches!(ty, TypeRef::I32 | TypeRef::I64)
}

fn expression_always_returns(expression: &Expression) -> bool {
    match expression {
        Expression::If {
            then_block,
            else_block,
            ..
        } => block_always_returns(then_block) && block_always_returns(else_block),
        Expression::Match { arms, .. } => {
            !arms.is_empty() && arms.iter().all(|arm| block_always_returns(&arm.body))
        }
        Expression::Block(block) => block_always_returns(block),
        _ => false,
    }
}

fn block_always_returns(block: &Block) -> bool {
    for statement in &block.statements {
        match statement {
            Statement::Return { .. } => return true,
            Statement::Let { value, .. } | Statement::Expression { value, .. }
                if expression_always_returns(value) =>
            {
                return true;
            }
            Statement::Let { .. } | Statement::ForEach { .. } | Statement::Expression { .. } => {}
        }
    }
    block
        .result
        .as_deref()
        .is_some_and(expression_always_returns)
}

fn call_graph(module: &Module, index: &Index<'_>) -> BTreeMap<NodeId, BTreeSet<NodeId>> {
    let mut graph = BTreeMap::new();
    for declaration in &module.declarations {
        match declaration {
            Declaration::Function(function) => {
                let edges = graph.entry(function.header.node.id).or_default();
                collect_block_calls(&function.body, edges, index);
            }
            Declaration::Implementation(implementation) => {
                for method in &implementation.methods {
                    let edges = graph.entry(method.header.node.id).or_default();
                    collect_block_calls(&method.body, edges, index);
                }
            }
            Declaration::Constant(_)
            | Declaration::Alias(_)
            | Declaration::Record(_)
            | Declaration::Enum(_)
            | Declaration::Contract(_)
            | Declaration::Test(_) => {}
        }
    }
    graph
}

fn collect_block_calls(block: &Block, edges: &mut BTreeSet<NodeId>, index: &Index<'_>) {
    for statement in &block.statements {
        match statement {
            Statement::Let { value, .. } | Statement::Expression { value, .. } => {
                collect_expression_calls(value, edges, index);
            }
            Statement::ForEach { iterable, body, .. } => {
                collect_expression_calls(iterable, edges, index);
                collect_block_calls(body, edges, index);
            }
            Statement::Return { value, .. } => {
                if let Some(value) = value {
                    collect_expression_calls(value, edges, index);
                }
            }
        }
    }
    if let Some(result) = &block.result {
        collect_expression_calls(result, edges, index);
    }
}

fn collect_expression_calls(
    expression: &Expression,
    edges: &mut BTreeSet<NodeId>,
    index: &Index<'_>,
) {
    match expression {
        Expression::Call {
            function,
            arguments,
            ..
        } => {
            edges.insert(*function);
            for argument in arguments {
                collect_expression_calls(argument, edges, index);
            }
        }
        Expression::MethodCall {
            receiver,
            dispatch,
            arguments,
            ..
        } => {
            collect_expression_calls(receiver, edges, index);
            match dispatch {
                MethodDispatch::Concrete { method, .. } => {
                    edges.insert(*method);
                }
                MethodDispatch::Contract { contract, method } => {
                    for (implementation, candidate) in
                        index.implementation_methods.values().copied()
                    {
                        if implementation.contract == *contract
                            && candidate.contract_method == *method
                        {
                            edges.insert(candidate.header.node.id);
                        }
                    }
                }
            }
            for argument in arguments {
                collect_expression_calls(argument, edges, index);
            }
        }
        Expression::ConstructRecord { fields, .. } | Expression::ConstructEnum { fields, .. } => {
            for field in fields {
                collect_expression_calls(&field.value, edges, index);
            }
        }
        Expression::ConstructSome { value, .. }
        | Expression::ConstructOk { value, .. }
        | Expression::ConstructErr { value, .. }
        | Expression::Field { base: value, .. } => {
            collect_expression_calls(value, edges, index);
        }
        Expression::ConstructList { elements, .. }
        | Expression::Intrinsic {
            arguments: elements,
            ..
        } => {
            for element in elements {
                collect_expression_calls(element, edges, index);
            }
        }
        Expression::If {
            condition,
            then_block,
            else_block,
            ..
        } => {
            collect_expression_calls(condition, edges, index);
            collect_block_calls(then_block, edges, index);
            collect_block_calls(else_block, edges, index);
        }
        Expression::Match { value, arms, .. } => {
            collect_expression_calls(value, edges, index);
            for arm in arms {
                collect_block_calls(&arm.body, edges, index);
            }
        }
        Expression::Block(block) => collect_block_calls(block, edges, index),
        Expression::Literal { .. }
        | Expression::Local { .. }
        | Expression::Constant { .. }
        | Expression::SelfValue { .. }
        | Expression::ConstructNone { .. } => {}
    }
}

fn constant_graph(module: &Module) -> BTreeMap<NodeId, BTreeSet<NodeId>> {
    let mut graph = BTreeMap::new();
    for declaration in &module.declarations {
        if let Declaration::Constant(constant) = declaration {
            let edges = graph.entry(constant.header.node.id).or_default();
            collect_constant_references(&constant.value, edges);
        }
    }
    graph
}

fn collect_constant_references(expression: &ConstantExpression, edges: &mut BTreeSet<NodeId>) {
    match expression {
        ConstantExpression::Reference { declaration, .. } => {
            edges.insert(*declaration);
        }
        ConstantExpression::Record { fields, .. } | ConstantExpression::Enum { fields, .. } => {
            for field in fields {
                collect_constant_references(&field.value, edges);
            }
        }
        ConstantExpression::Some { value, .. }
        | ConstantExpression::Ok { value, .. }
        | ConstantExpression::Err { value, .. } => collect_constant_references(value, edges),
        ConstantExpression::List { elements, .. }
        | ConstantExpression::Intrinsic {
            arguments: elements,
            ..
        } => {
            for element in elements {
                collect_constant_references(element, edges);
            }
        }
        ConstantExpression::Literal { .. } | ConstantExpression::None { .. } => {}
    }
}

fn reaches(
    graph: &BTreeMap<NodeId, BTreeSet<NodeId>>,
    current: NodeId,
    goal: NodeId,
    visited: &mut BTreeSet<NodeId>,
) -> bool {
    graph.get(&current).is_some_and(|edges| {
        edges.iter().any(|next| {
            *next == goal || (visited.insert(*next) && reaches(graph, *next, goal, visited))
        })
    })
}

fn valid_identifier(name: &str) -> bool {
    let mut characters = name.chars();
    matches!(characters.next(), Some('_' | 'a'..='z' | 'A'..='Z'))
        && characters.all(|character| character == '_' || character.is_ascii_alphanumeric())
}
