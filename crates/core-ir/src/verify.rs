use std::collections::BTreeSet;

use portable_diagnostics::{Diagnostic, DiagnosticCode, sort_diagnostics};
use portable_ir::v0::SourceRef;

use crate::*;

pub fn verify_core(program: &CoreProgram) -> Result<(), Vec<Diagnostic>> {
    let mut verifier = Verifier {
        program,
        diagnostics: vec![],
    };
    verifier.check_program();
    sort_diagnostics(&mut verifier.diagnostics);
    if verifier.diagnostics.is_empty() {
        Ok(())
    } else {
        Err(verifier.diagnostics)
    }
}

struct Verifier<'a> {
    program: &'a CoreProgram,
    diagnostics: Vec<Diagnostic>,
}

impl Verifier<'_> {
    fn check_program(&mut self) {
        self.check_types();
        self.check_module();
        self.check_declarations();
        self.check_expressions();
        self.check_blocks();
        self.check_callable_scopes();
        self.check_tests();
    }

    fn check_types(&mut self) {
        let mut seen = BTreeSet::new();
        for (_, ty) in self.program.types().iter() {
            if !seen.insert(ty.clone()) {
                self.error(
                    DiagnosticCode::InvalidStructure,
                    "CoreIR type arena contains a duplicate canonical type",
                    self.root_source(),
                );
            }
            match ty {
                CoreType::List(inner) | CoreType::Option(inner) => {
                    self.require_type(*inner, self.root_source())
                }
                CoreType::Result { ok, error } => {
                    self.require_type(*ok, self.root_source());
                    self.require_type(*error, self.root_source());
                }
                CoreType::Record(id) if self.program.record(*id).is_none() => self.missing(
                    "record referenced by Core type",
                    id.index(),
                    self.root_source(),
                ),
                CoreType::Enum(id) if self.program.enumeration(*id).is_none() => self.missing(
                    "enum referenced by Core type",
                    id.index(),
                    self.root_source(),
                ),
                CoreType::Interface(id) if self.program.interface(*id).is_none() => self.missing(
                    "interface referenced by Core type",
                    id.index(),
                    self.root_source(),
                ),
                CoreType::Unit
                | CoreType::Bool
                | CoreType::I32
                | CoreType::I64
                | CoreType::F64
                | CoreType::Char
                | CoreType::String
                | CoreType::Bytes
                | CoreType::Record(_)
                | CoreType::Enum(_)
                | CoreType::Interface(_) => {}
            }
        }
    }

    fn check_module(&mut self) {
        let mut declarations = BTreeSet::new();
        for declaration in &self.program.module().declarations {
            if !declarations.insert(*declaration) {
                self.error(
                    DiagnosticCode::DuplicateDeclaration,
                    "Core module contains a duplicate declaration reference",
                    self.root_source(),
                );
            }
            let exists = match declaration {
                CoreDeclaration::Constant(id) => self.program.constant(*id).is_some(),
                CoreDeclaration::Alias(id) => self.program.alias(*id).is_some(),
                CoreDeclaration::Record(id) => self.program.record(*id).is_some(),
                CoreDeclaration::Enum(id) => self.program.enumeration(*id).is_some(),
                CoreDeclaration::Interface(id) => self.program.interface(*id).is_some(),
                CoreDeclaration::Implementation(id) => self.program.implementation(*id).is_some(),
                CoreDeclaration::Function(id) => self.program.function(*id).is_some(),
                CoreDeclaration::Test(id) => self.program.test(*id).is_some(),
            };
            if !exists {
                self.error(
                    DiagnosticCode::UnresolvedReference,
                    "Core module declaration reference is out of bounds",
                    self.root_source(),
                );
            }
        }
        let expected = self.program.constants().len()
            + self.program.aliases().len()
            + self.program.records().len()
            + self.program.enums().len()
            + self.program.interfaces().len()
            + self.program.implementations().len()
            + self.program.functions().len()
            + self.program.tests().len();
        if declarations.len() != expected {
            self.error(
                DiagnosticCode::InvalidStructure,
                "Core module declaration index is incomplete",
                self.root_source(),
            );
        }
    }

    fn check_declarations(&mut self) {
        for constant in self.program.constants() {
            self.check_source(&constant.header.source, "constant");
            self.require_type(constant.ty, constant.header.source.clone());
            self.check_constant(&constant.value, constant.ty);
        }
        for alias in self.program.aliases() {
            self.check_source(&alias.header.source, "alias");
            self.require_type(alias.target, alias.header.source.clone());
        }
        for (index, record) in self.program.records().iter().enumerate() {
            self.check_source(&record.header.source, "record");
            let owner = CoreRecordId::from_index(index);
            self.check_unique_ids(&record.fields, &record.header.source, "record field");
            for field in &record.fields {
                match self.program.field(*field) {
                    Some(field) if field.owner == CoreFieldOwner::Record(owner) => {}
                    Some(_) => self.error(
                        DiagnosticCode::InvalidStructure,
                        "record field has a different owner",
                        record.header.source.clone(),
                    ),
                    None => {
                        self.missing("record field", field.index(), record.header.source.clone())
                    }
                }
            }
        }
        for (index, enumeration) in self.program.enums().iter().enumerate() {
            self.check_source(&enumeration.header.source, "enum");
            let owner = CoreEnumId::from_index(index);
            self.check_unique_ids(
                &enumeration.variants,
                &enumeration.header.source,
                "enum variant",
            );
            for variant in &enumeration.variants {
                match self.program.variant(*variant) {
                    Some(variant) if variant.enumeration == owner => {}
                    Some(_) => self.error(
                        DiagnosticCode::InvalidStructure,
                        "enum variant has a different owner",
                        enumeration.header.source.clone(),
                    ),
                    None => self.missing(
                        "enum variant",
                        variant.index(),
                        enumeration.header.source.clone(),
                    ),
                }
            }
        }
        self.check_members_and_interfaces();
    }

    fn check_members_and_interfaces(&mut self) {
        for (index, variant) in self.program.variants().iter().enumerate() {
            self.check_source(&variant.header.source, "variant");
            let owner = CoreVariantId::from_index(index);
            if self.program.enumeration(variant.enumeration).is_none() {
                self.missing(
                    "variant enum",
                    variant.enumeration.index(),
                    variant.header.source.clone(),
                );
            }
            self.check_unique_ids(&variant.fields, &variant.header.source, "variant field");
            for field in &variant.fields {
                match self.program.field(*field) {
                    Some(field) if field.owner == CoreFieldOwner::Variant(owner) => {}
                    Some(_) => self.error(
                        DiagnosticCode::InvalidStructure,
                        "variant field has a different owner",
                        variant.header.source.clone(),
                    ),
                    None => self.missing(
                        "variant field",
                        field.index(),
                        variant.header.source.clone(),
                    ),
                }
            }
        }
        for field in self.program.fields() {
            self.check_source(&field.header.source, "field");
            self.require_type(field.ty, field.header.source.clone());
            match field.owner {
                CoreFieldOwner::Record(id) if self.program.record(id).is_none() => self.missing(
                    "field record owner",
                    id.index(),
                    field.header.source.clone(),
                ),
                CoreFieldOwner::Variant(id) if self.program.variant(id).is_none() => self.missing(
                    "field variant owner",
                    id.index(),
                    field.header.source.clone(),
                ),
                CoreFieldOwner::Record(_) | CoreFieldOwner::Variant(_) => {}
            }
        }
        for (index, interface) in self.program.interfaces().iter().enumerate() {
            self.check_source(&interface.header.source, "interface");
            let owner = CoreInterfaceId::from_index(index);
            self.check_unique_ids(
                &interface.methods,
                &interface.header.source,
                "interface method",
            );
            for method in &interface.methods {
                match self.program.interface_method(*method) {
                    Some(method) if method.interface == owner => self.check_signature(
                        &method.parameters,
                        method.return_type,
                        &method.header.source,
                        false,
                    ),
                    Some(_) => self.error(
                        DiagnosticCode::InterfaceNonconformance,
                        "interface method has a different owner",
                        interface.header.source.clone(),
                    ),
                    None => self.missing(
                        "interface method",
                        method.index(),
                        interface.header.source.clone(),
                    ),
                }
            }
        }
        self.check_implementations();
    }

    fn check_implementations(&mut self) {
        for method in self.program.interface_methods() {
            self.check_source(&method.header.source, "interface method");
            self.require_type(method.return_type, method.header.source.clone());
            if self
                .program
                .interface(method.interface)
                .is_none_or(|owner| {
                    !owner
                        .methods
                        .iter()
                        .any(|id| self.program.interface_method(*id) == Some(method))
                })
            {
                self.error(
                    DiagnosticCode::InterfaceNonconformance,
                    "interface method is not indexed by its owner",
                    method.header.source.clone(),
                );
            }
        }
        for (index, implementation) in self.program.implementations().iter().enumerate() {
            self.check_source(&implementation.header.source, "implementation");
            let implementation_id = CoreImplementationId::from_index(index);
            let Some(interface) = self.program.interface(implementation.interface) else {
                self.missing(
                    "implementation interface",
                    implementation.interface.index(),
                    implementation.header.source.clone(),
                );
                continue;
            };
            if self.program.record(implementation.record).is_none() {
                self.missing(
                    "implementation record",
                    implementation.record.index(),
                    implementation.header.source.clone(),
                );
            }
            self.check_unique_ids(
                &implementation.methods,
                &implementation.header.source,
                "implementation method",
            );
            let mut provided = BTreeSet::new();
            for method_id in &implementation.methods {
                let Some(method) = self.program.implementation_method(*method_id) else {
                    self.missing(
                        "implementation method",
                        method_id.index(),
                        implementation.header.source.clone(),
                    );
                    continue;
                };
                self.check_source(&method.header.source, "implementation method");
                if method.implementation != implementation_id {
                    self.error(
                        DiagnosticCode::InterfaceNonconformance,
                        "implementation method has a different implementation owner",
                        method.header.source.clone(),
                    );
                }
                let Some(required) = self.program.interface_method(method.interface_method) else {
                    self.missing(
                        "implemented interface method",
                        method.interface_method.index(),
                        method.header.source.clone(),
                    );
                    continue;
                };
                provided.insert(method.interface_method);
                if required.interface != implementation.interface
                    || signature_types(&required.parameters) != signature_types(&method.parameters)
                    || required.return_type != method.return_type
                {
                    self.error(
                        DiagnosticCode::InterfaceNonconformance,
                        "implementation method signature does not exactly match its interface method",
                        method.header.source.clone(),
                    );
                }
                self.check_signature(
                    &method.parameters,
                    method.return_type,
                    &method.header.source,
                    true,
                );
                if self.program.blocks().get(method.body).is_none() {
                    self.missing(
                        "method body",
                        method.body.index(),
                        method.header.source.clone(),
                    );
                }
            }
            let required = interface.methods.iter().copied().collect::<BTreeSet<_>>();
            if provided != required {
                self.error(
                    DiagnosticCode::InterfaceNonconformance,
                    "implementation does not provide every interface method exactly once",
                    implementation.header.source.clone(),
                );
            }
        }
        for method in self.program.implementation_methods() {
            if self
                .program
                .implementation(method.implementation)
                .is_none_or(|owner| {
                    !owner
                        .methods
                        .iter()
                        .any(|id| self.program.implementation_method(*id) == Some(method))
                })
            {
                self.error(
                    DiagnosticCode::InterfaceNonconformance,
                    "implementation method is not indexed by its owner",
                    method.header.source.clone(),
                );
            }
        }
        for function in self.program.functions() {
            self.check_source(&function.header.source, "function");
            self.check_signature(
                &function.parameters,
                function.return_type,
                &function.header.source,
                true,
            );
            if self.program.blocks().get(function.body).is_none() {
                self.missing(
                    "function body",
                    function.body.index(),
                    function.header.source.clone(),
                );
            }
        }
    }

    fn check_signature(
        &mut self,
        parameters: &[CoreParameter],
        return_type: CoreTypeId,
        source: &SourceRef,
        requires_locals: bool,
    ) {
        self.require_type(return_type, source.clone());
        for parameter in parameters {
            self.check_source(&parameter.header.source, "parameter");
            self.require_type(parameter.ty, parameter.header.source.clone());
            match (requires_locals, parameter.local) {
                (true, Some(local)) => match self.program.local(local) {
                    Some(local)
                        if local.ty == parameter.ty && local.kind == CoreLocalKind::Parameter => {}
                    Some(_) => self.error(
                        DiagnosticCode::InvalidStructure,
                        "parameter local has the wrong type or kind",
                        parameter.header.source.clone(),
                    ),
                    None => self.missing(
                        "parameter local",
                        local.index(),
                        parameter.header.source.clone(),
                    ),
                },
                (true, None) => self.error(
                    DiagnosticCode::InvalidStructure,
                    "callable parameter has no Core local",
                    parameter.header.source.clone(),
                ),
                (false, Some(_)) => self.error(
                    DiagnosticCode::InvalidStructure,
                    "interface signature unexpectedly owns a runtime local",
                    parameter.header.source.clone(),
                ),
                (false, None) => {}
            }
        }
    }

    fn check_constant(&mut self, expression: &CoreConstantExpr, expected: CoreTypeId) {
        self.check_source(&expression.source, "constant expression");
        match &expression.kind {
            CoreConstantExprKind::Literal(value) => {
                self.check_value(value, expected, &expression.source)
            }
            CoreConstantExprKind::Constant(id) => match self.program.constant(*id) {
                Some(value) if value.ty == expected => {}
                Some(_) => self.type_error("constant reference type", &expression.source),
                None => self.missing("constant reference", id.index(), expression.source.clone()),
            },
            CoreConstantExprKind::Record { record, fields } => {
                self.expect_type(expected, &CoreType::Record(*record), &expression.source);
                self.check_constant_fields(
                    fields,
                    record_fields(self.program, *record),
                    &expression.source,
                );
            }
            CoreConstantExprKind::Enum {
                enumeration,
                variant,
                fields,
            } => {
                self.expect_type(expected, &CoreType::Enum(*enumeration), &expression.source);
                match self.program.variant(*variant) {
                    Some(value) if value.enumeration == *enumeration => {
                        self.check_constant_fields(fields, Some(&value.fields), &expression.source)
                    }
                    Some(_) => self.type_error("constant enum variant owner", &expression.source),
                    None => self.missing(
                        "constant enum variant",
                        variant.index(),
                        expression.source.clone(),
                    ),
                }
            }
            CoreConstantExprKind::Some(value) => match self.program.types().get(expected) {
                Some(CoreType::Option(inner)) => self.check_constant(value, *inner),
                _ => self.type_error("Some constant", &expression.source),
            },
            CoreConstantExprKind::None { inner } => {
                self.expect_type(expected, &CoreType::Option(*inner), &expression.source)
            }
            CoreConstantExprKind::Ok { value, error } => match self.program.types().get(expected) {
                Some(CoreType::Result {
                    ok,
                    error: expected_error,
                }) if error == expected_error => self.check_constant(value, *ok),
                _ => self.type_error("Ok constant", &expression.source),
            },
            CoreConstantExprKind::Err { value, ok } => match self.program.types().get(expected) {
                Some(CoreType::Result {
                    ok: expected_ok,
                    error,
                }) if ok == expected_ok => self.check_constant(value, *error),
                _ => self.type_error("Err constant", &expression.source),
            },
            CoreConstantExprKind::List { element, elements } => {
                self.expect_type(expected, &CoreType::List(*element), &expression.source);
                for value in elements {
                    self.check_constant(value, *element);
                }
            }
            CoreConstantExprKind::Intrinsic(intrinsic) => {
                if let Some(actual) = self.check_constant_intrinsic(intrinsic, &expression.source)
                    && actual != expected
                {
                    self.type_error("constant intrinsic result", &expression.source);
                }
            }
        }
    }

    fn check_constant_fields(
        &mut self,
        fields: &[CoreConstantField],
        expected: Option<&[CoreFieldId]>,
        source: &SourceRef,
    ) {
        let actual = fields.iter().map(|field| field.field).collect::<Vec<_>>();
        self.check_field_set(&actual, expected, source);
        for value in fields {
            match self.program.field(value.field) {
                Some(field) => self.check_constant(&value.value, field.ty),
                None => self.missing("constant field", value.field.index(), source.clone()),
            }
        }
    }

    fn check_constant_intrinsic(
        &mut self,
        intrinsic: &CoreIntrinsicExpr<CoreConstantExpr>,
        source: &SourceRef,
    ) -> Option<CoreTypeId> {
        let (operation, values) = constant_intrinsic_parts(intrinsic);
        let arguments = values
            .into_iter()
            .map(|value| self.constant_expression_type(value))
            .collect::<Option<Vec<_>>>()?;
        self.check_intrinsic(operation, &arguments, source)
    }

    fn constant_expression_type(&mut self, expression: &CoreConstantExpr) -> Option<CoreTypeId> {
        match &expression.kind {
            CoreConstantExprKind::Literal(value) => self.value_type(value),
            CoreConstantExprKind::Constant(id) => self.program.constant(*id).map(|value| value.ty),
            CoreConstantExprKind::Record { record, .. } => {
                self.find_type(&CoreType::Record(*record))
            }
            CoreConstantExprKind::Enum { enumeration, .. } => {
                self.find_type(&CoreType::Enum(*enumeration))
            }
            CoreConstantExprKind::Some(value) => {
                let inner = self.constant_expression_type(value)?;
                self.find_type(&CoreType::Option(inner))
            }
            CoreConstantExprKind::None { inner } => self.find_type(&CoreType::Option(*inner)),
            CoreConstantExprKind::Ok { value, error } => {
                let ok = self.constant_expression_type(value)?;
                self.find_type(&CoreType::Result { ok, error: *error })
            }
            CoreConstantExprKind::Err { value, ok } => {
                let error = self.constant_expression_type(value)?;
                self.find_type(&CoreType::Result { ok: *ok, error })
            }
            CoreConstantExprKind::List { element, .. } => self.find_type(&CoreType::List(*element)),
            CoreConstantExprKind::Intrinsic(value) => {
                self.check_constant_intrinsic(value, &expression.source)
            }
        }
    }

    fn check_value(&mut self, value: &CoreValue, expected: CoreTypeId, source: &SourceRef) {
        match value {
            CoreValue::Unit => self.expect_type(expected, &CoreType::Unit, source),
            CoreValue::Bool(_) => self.expect_type(expected, &CoreType::Bool, source),
            CoreValue::I32(_) => self.expect_type(expected, &CoreType::I32, source),
            CoreValue::I64(_) => self.expect_type(expected, &CoreType::I64, source),
            CoreValue::F64(_) => self.expect_type(expected, &CoreType::F64, source),
            CoreValue::Char(_) => self.expect_type(expected, &CoreType::Char, source),
            CoreValue::String(_) => self.expect_type(expected, &CoreType::String, source),
            CoreValue::Bytes(_) => self.expect_type(expected, &CoreType::Bytes, source),
            CoreValue::List(values) => match self.program.types().get(expected) {
                Some(CoreType::List(element)) => {
                    for value in values {
                        self.check_value(value, *element, source);
                    }
                }
                _ => self.type_error("list value", source),
            },
            CoreValue::None => {
                if !matches!(
                    self.program.types().get(expected),
                    Some(CoreType::Option(_))
                ) {
                    self.type_error("None value", source);
                }
            }
            CoreValue::Some(value) => match self.program.types().get(expected) {
                Some(CoreType::Option(inner)) => self.check_value(value, *inner, source),
                _ => self.type_error("Some value", source),
            },
            CoreValue::Ok(value) => match self.program.types().get(expected) {
                Some(CoreType::Result { ok, .. }) => self.check_value(value, *ok, source),
                _ => self.type_error("Ok value", source),
            },
            CoreValue::Err(value) => match self.program.types().get(expected) {
                Some(CoreType::Result { error, .. }) => self.check_value(value, *error, source),
                _ => self.type_error("Err value", source),
            },
            CoreValue::Record { record, fields } => {
                self.expect_type(expected, &CoreType::Record(*record), source);
                self.check_value_fields(fields, record_fields(self.program, *record), source);
            }
            CoreValue::Enum {
                enumeration,
                variant,
                fields,
            } => {
                self.expect_type(expected, &CoreType::Enum(*enumeration), source);
                match self.program.variant(*variant) {
                    Some(value) if value.enumeration == *enumeration => {
                        self.check_value_fields(fields, Some(&value.fields), source)
                    }
                    Some(_) => self.type_error("enum value variant owner", source),
                    None => self.missing("enum value variant", variant.index(), source.clone()),
                }
            }
        }
    }

    fn check_value_fields(
        &mut self,
        fields: &[CoreValueField],
        expected: Option<&[CoreFieldId]>,
        source: &SourceRef,
    ) {
        let actual = fields.iter().map(|field| field.field).collect::<Vec<_>>();
        self.check_field_set(&actual, expected, source);
        for value in fields {
            match self.program.field(value.field) {
                Some(field) => self.check_value(&value.value, field.ty, source),
                None => self.missing("value field", value.field.index(), source.clone()),
            }
        }
    }

    fn value_type(&self, value: &CoreValue) -> Option<CoreTypeId> {
        let ty = match value {
            CoreValue::Unit => CoreType::Unit,
            CoreValue::Bool(_) => CoreType::Bool,
            CoreValue::I32(_) => CoreType::I32,
            CoreValue::I64(_) => CoreType::I64,
            CoreValue::F64(_) => CoreType::F64,
            CoreValue::Char(_) => CoreType::Char,
            CoreValue::String(_) => CoreType::String,
            CoreValue::Bytes(_) => CoreType::Bytes,
            CoreValue::Record { record, .. } => CoreType::Record(*record),
            CoreValue::Enum { enumeration, .. } => CoreType::Enum(*enumeration),
            CoreValue::List(values) => CoreType::List(self.value_type(values.first()?)?),
            CoreValue::Some(value) => CoreType::Option(self.value_type(value)?),
            CoreValue::None | CoreValue::Ok(_) | CoreValue::Err(_) => return None,
        };
        self.find_type(&ty)
    }

    fn check_field_set(
        &mut self,
        actual: &[CoreFieldId],
        expected: Option<&[CoreFieldId]>,
        source: &SourceRef,
    ) {
        let actual = actual.iter().copied().collect::<BTreeSet<_>>();
        let Some(expected) = expected else {
            self.error(
                DiagnosticCode::UnresolvedReference,
                "aggregate owner is missing",
                source.clone(),
            );
            return;
        };
        if actual.len() != expected.len()
            || actual != expected.iter().copied().collect::<BTreeSet<_>>()
        {
            self.error(
                DiagnosticCode::InvalidInvocation,
                "aggregate fields are missing, duplicated, or owned by another declaration",
                source.clone(),
            );
        }
    }

    fn check_expressions(&mut self) {
        for (id, expression) in self.program.expressions().iter() {
            self.check_source(&expression.source, "expression");
            self.require_type(expression.ty, expression.source.clone());
            if expression.evaluation != CoreEvaluationOrder::OnceLeftToRight
                || expression.ownership != CoreResultOwnership::OwnedImmutableValue
            {
                self.error(
                    DiagnosticCode::InvalidStructure,
                    "Core expression lacks the canonical evaluation or ownership marker",
                    expression.source.clone(),
                );
            }
            if let Some(derived) = self.derive_expression_type(id, expression)
                && derived != expression.ty
            {
                self.type_error("stored expression type", &expression.source);
            }
        }
    }

    fn derive_expression_type(
        &mut self,
        id: CoreExprId,
        expression: &CoreExpr,
    ) -> Option<CoreTypeId> {
        let source = &expression.source;
        match &expression.kind {
            CoreExprKind::Literal(value) => {
                self.check_value(value, expression.ty, source);
                Some(expression.ty)
            }
            CoreExprKind::Local(local) => match self.program.local(*local) {
                Some(local) => Some(local.ty),
                None => {
                    self.missing("expression local", local.index(), source.clone());
                    None
                }
            },
            CoreExprKind::Constant(constant) => match self.program.constant(*constant) {
                Some(constant) => Some(constant.ty),
                None => {
                    self.missing("expression constant", constant.index(), source.clone());
                    None
                }
            },
            CoreExprKind::SelfValue(record) => self.required_named_type(
                &CoreType::Record(*record),
                "self record",
                record.index(),
                source,
            ),
            CoreExprKind::ConstructRecord { record, fields } => {
                self.check_expression_fields(
                    id,
                    fields,
                    record_fields(self.program, *record),
                    source,
                );
                self.required_named_type(
                    &CoreType::Record(*record),
                    "record",
                    record.index(),
                    source,
                )
            }
            CoreExprKind::ConstructEnum {
                enumeration,
                variant,
                fields,
            } => {
                match self.program.variant(*variant) {
                    Some(value) if value.enumeration == *enumeration => {
                        self.check_expression_fields(id, fields, Some(&value.fields), source)
                    }
                    Some(_) => self.type_error("constructed enum variant owner", source),
                    None => {
                        self.missing("constructed enum variant", variant.index(), source.clone())
                    }
                }
                self.required_named_type(
                    &CoreType::Enum(*enumeration),
                    "enum",
                    enumeration.index(),
                    source,
                )
            }
            CoreExprKind::ConstructSome(value) => {
                let inner = self.child_type(id, *value, source)?;
                self.required_type(&CoreType::Option(inner), source)
            }
            CoreExprKind::ConstructNone { inner } => {
                self.require_type(*inner, source.clone());
                self.required_type(&CoreType::Option(*inner), source)
            }
            CoreExprKind::ConstructOk { value, error } => {
                let ok = self.child_type(id, *value, source)?;
                self.require_type(*error, source.clone());
                self.required_type(&CoreType::Result { ok, error: *error }, source)
            }
            CoreExprKind::ConstructErr { value, ok } => {
                let error = self.child_type(id, *value, source)?;
                self.require_type(*ok, source.clone());
                self.required_type(&CoreType::Result { ok: *ok, error }, source)
            }
            CoreExprKind::ConstructList { element, elements } => {
                self.require_type(*element, source.clone());
                for value in elements {
                    if self
                        .child_type(id, *value, source)
                        .is_some_and(|ty| ty != *element)
                    {
                        self.type_error("list element", source);
                    }
                }
                self.required_type(&CoreType::List(*element), source)
            }
            CoreExprKind::CoerceInterface {
                implementation,
                value,
            } => {
                let implementation = match self.program.implementation(*implementation) {
                    Some(value) => value,
                    None => {
                        self.missing(
                            "interface coercion implementation",
                            implementation.index(),
                            source.clone(),
                        );
                        return None;
                    }
                };
                let value_type = self.child_type(id, *value, source)?;
                self.expect_type(value_type, &CoreType::Record(implementation.record), source);
                self.required_type(&CoreType::Interface(implementation.interface), source)
            }
            CoreExprKind::Field { value, field } => {
                let base = self.child_type(id, *value, source)?;
                let field = self.program.field(*field)?;
                let owner = match field.owner {
                    CoreFieldOwner::Record(record) => CoreType::Record(record),
                    CoreFieldOwner::Variant(variant) => {
                        CoreType::Enum(self.program.variant(variant)?.enumeration)
                    }
                };
                self.expect_type(base, &owner, source);
                Some(field.ty)
            }
            CoreExprKind::Call {
                function,
                arguments,
            } => {
                let function = match self.program.function(*function) {
                    Some(value) => value,
                    None => {
                        self.missing("called function", function.index(), source.clone());
                        return None;
                    }
                };
                self.check_call_arguments(id, arguments, &function.parameters, source);
                Some(function.return_type)
            }
            CoreExprKind::StaticMethodCall {
                implementation,
                method,
                receiver,
                arguments,
            } => {
                let implementation_id = *implementation;
                let implementation = match self.program.implementation(implementation_id) {
                    Some(value) => value,
                    None => {
                        self.missing(
                            "static implementation",
                            implementation.index(),
                            source.clone(),
                        );
                        return None;
                    }
                };
                let method = match self.program.implementation_method(*method) {
                    Some(value) if value.implementation == implementation_id => value,
                    Some(_) => {
                        self.error(
                            DiagnosticCode::InvalidInvocation,
                            "static method belongs to another implementation",
                            source.clone(),
                        );
                        return None;
                    }
                    None => {
                        self.missing("static method", method.index(), source.clone());
                        return None;
                    }
                };
                let receiver_type = self.child_type(id, *receiver, source)?;
                self.expect_type(
                    receiver_type,
                    &CoreType::Record(implementation.record),
                    source,
                );
                self.check_call_arguments(id, arguments, &method.parameters, source);
                Some(method.return_type)
            }
            CoreExprKind::InterfaceCall {
                interface,
                method,
                receiver,
                arguments,
            } => {
                let method = match self.program.interface_method(*method) {
                    Some(value) if value.interface == *interface => value,
                    Some(_) => {
                        self.error(
                            DiagnosticCode::InvalidInvocation,
                            "dynamic method belongs to another interface",
                            source.clone(),
                        );
                        return None;
                    }
                    None => {
                        self.missing("dynamic interface method", method.index(), source.clone());
                        return None;
                    }
                };
                let receiver_type = self.child_type(id, *receiver, source)?;
                self.expect_type(receiver_type, &CoreType::Interface(*interface), source);
                self.check_call_arguments(id, arguments, &method.parameters, source);
                Some(method.return_type)
            }
            CoreExprKind::Intrinsic(intrinsic) => {
                let (operation, operands) = expression_intrinsic_parts(intrinsic);
                let arguments = operands
                    .into_iter()
                    .map(|operand| self.child_type(id, operand, source))
                    .collect::<Option<Vec<_>>>()?;
                self.check_intrinsic(operation, &arguments, source)
            }
            CoreExprKind::If {
                condition,
                then_block,
                else_block,
            } => {
                let condition = self.child_type(id, *condition, source)?;
                self.expect_type(condition, &CoreType::Bool, source);
                let then_type = self.block_type(*then_block, source)?;
                let else_type = self.block_type(*else_block, source)?;
                if then_type != else_type {
                    self.type_error("if branch result", source);
                }
                Some(then_type)
            }
            CoreExprKind::Match { value, arms } => {
                let matched_type = self.child_type(id, *value, source)?;
                self.check_match(matched_type, arms, source)
            }
            CoreExprKind::Block(block) => self.block_type(*block, source),
        }
    }

    fn check_expression_fields(
        &mut self,
        parent: CoreExprId,
        fields: &[CoreExprField],
        expected: Option<&[CoreFieldId]>,
        source: &SourceRef,
    ) {
        let actual = fields.iter().map(|field| field.field).collect::<Vec<_>>();
        self.check_field_set(&actual, expected, source);
        for value in fields {
            let actual = self.child_type(parent, value.value, source);
            match (self.program.field(value.field), actual) {
                (Some(field), Some(actual)) if field.ty != actual => {
                    self.type_error("aggregate field expression", source)
                }
                (None, _) => self.missing("aggregate field", value.field.index(), source.clone()),
                _ => {}
            }
        }
    }

    fn check_call_arguments(
        &mut self,
        parent: CoreExprId,
        arguments: &[CoreExprId],
        parameters: &[CoreParameter],
        source: &SourceRef,
    ) {
        if arguments.len() != parameters.len() {
            self.error(
                DiagnosticCode::InvalidInvocation,
                "call argument count does not match its signature",
                source.clone(),
            );
        }
        for (argument, parameter) in arguments.iter().zip(parameters) {
            if self
                .child_type(parent, *argument, source)
                .is_some_and(|actual| actual != parameter.ty)
            {
                self.type_error("call argument", source);
            }
        }
    }

    fn check_match(
        &mut self,
        matched_type: CoreTypeId,
        arms: &[CoreMatchArm],
        source: &SourceRef,
    ) -> Option<CoreTypeId> {
        let expected = self.expected_coverage(matched_type, source)?;
        let mut covered = BTreeSet::new();
        let mut wildcard = false;
        let mut result = None;
        for arm in arms {
            self.check_source(&arm.source, "match arm");
            let coverage = self.check_pattern(&arm.pattern, matched_type);
            if wildcard || coverage.is_some_and(|value| !covered.insert(value)) {
                self.error(
                    DiagnosticCode::UnreachablePattern,
                    "Core match contains a duplicate or unreachable arm",
                    pattern_source(&arm.pattern).clone(),
                );
            }
            wildcard |= coverage == Some(Coverage::Wildcard);
            if let Some(arm_type) = self.block_type(arm.body, &arm.source) {
                if result.is_some_and(|previous| previous != arm_type) {
                    self.type_error("match arm result", &arm.source);
                }
                result.get_or_insert(arm_type);
            }
        }
        if !wildcard && !expected.is_subset(&covered) {
            self.error(
                DiagnosticCode::NonExhaustiveMatch,
                "Core match is not exhaustive",
                source.clone(),
            );
        }
        result
    }

    fn expected_coverage(
        &mut self,
        ty: CoreTypeId,
        source: &SourceRef,
    ) -> Option<BTreeSet<Coverage>> {
        match self.program.types().get(ty) {
            Some(CoreType::Bool) => Some(BTreeSet::from([
                Coverage::Bool(false),
                Coverage::Bool(true),
            ])),
            Some(CoreType::Option(_)) => Some(BTreeSet::from([Coverage::None, Coverage::Some])),
            Some(CoreType::Result { .. }) => Some(BTreeSet::from([Coverage::Ok, Coverage::Err])),
            Some(CoreType::Enum(enumeration)) => self
                .program
                .enumeration(*enumeration)
                .map(|value| {
                    value
                        .variants
                        .iter()
                        .copied()
                        .map(Coverage::Variant)
                        .collect()
                })
                .or_else(|| {
                    self.missing("matched enum", enumeration.index(), source.clone());
                    None
                }),
            _ => {
                self.type_error("match input", source);
                None
            }
        }
    }

    fn check_pattern(
        &mut self,
        pattern: &CorePattern,
        matched_type: CoreTypeId,
    ) -> Option<Coverage> {
        let source = pattern_source(pattern);
        self.check_source(source, "pattern");
        match pattern {
            CorePattern::Wildcard { .. } => Some(Coverage::Wildcard),
            CorePattern::Bool { value, .. } => {
                self.expect_type(matched_type, &CoreType::Bool, source);
                Some(Coverage::Bool(*value))
            }
            CorePattern::None { .. } => {
                if !matches!(
                    self.program.types().get(matched_type),
                    Some(CoreType::Option(_))
                ) {
                    self.type_error("None pattern", source);
                }
                Some(Coverage::None)
            }
            CorePattern::Some { binding, .. } => {
                let inner = match self.program.types().get(matched_type) {
                    Some(CoreType::Option(inner)) => Some(*inner),
                    _ => {
                        self.type_error("Some pattern", source);
                        None
                    }
                };
                self.check_pattern_local(*binding, inner, source);
                Some(Coverage::Some)
            }
            CorePattern::Ok { binding, .. } => {
                let inner = match self.program.types().get(matched_type) {
                    Some(CoreType::Result { ok, .. }) => Some(*ok),
                    _ => {
                        self.type_error("Ok pattern", source);
                        None
                    }
                };
                self.check_pattern_local(*binding, inner, source);
                Some(Coverage::Ok)
            }
            CorePattern::Err { binding, .. } => {
                let inner = match self.program.types().get(matched_type) {
                    Some(CoreType::Result { error, .. }) => Some(*error),
                    _ => {
                        self.type_error("Err pattern", source);
                        None
                    }
                };
                self.check_pattern_local(*binding, inner, source);
                Some(Coverage::Err)
            }
            CorePattern::EnumVariant {
                enumeration,
                variant,
                bindings,
                ..
            } => {
                self.expect_type(matched_type, &CoreType::Enum(*enumeration), source);
                match self.program.variant(*variant) {
                    Some(value) if value.enumeration == *enumeration => {
                        let actual = bindings.iter().map(|value| value.field).collect::<Vec<_>>();
                        self.check_field_set(&actual, Some(&value.fields), source);
                        for binding in bindings {
                            let expected = self.program.field(binding.field).map(|field| field.ty);
                            self.check_pattern_local(binding.binding, expected, source);
                        }
                    }
                    Some(_) => self.type_error("enum pattern variant owner", source),
                    None => self.missing("enum pattern variant", variant.index(), source.clone()),
                }
                Some(Coverage::Variant(*variant))
            }
        }
    }

    fn check_pattern_local(
        &mut self,
        binding: CoreLocalId,
        expected: Option<CoreTypeId>,
        source: &SourceRef,
    ) {
        match self.program.local(binding) {
            Some(local) if local.kind == CoreLocalKind::Pattern && Some(local.ty) == expected => {}
            Some(_) => self.error(
                DiagnosticCode::TypeMismatch,
                "pattern binding has the wrong type or local kind",
                source.clone(),
            ),
            None => self.missing("pattern binding", binding.index(), source.clone()),
        }
    }

    fn check_intrinsic(
        &mut self,
        operation: CoreOperation,
        arguments: &[CoreTypeId],
        source: &SourceRef,
    ) -> Option<CoreTypeId> {
        use CoreBinaryIntrinsic as B;
        use CoreUnaryIntrinsic as U;
        let invalid = |this: &mut Self| {
            this.error(
                DiagnosticCode::InvalidInvocation,
                "Core intrinsic operand types do not match its typed operation",
                source.clone(),
            );
            None
        };
        let bool_id = || self.find_type(&CoreType::Bool);
        let i32_id = || self.find_type(&CoreType::I32);
        let i64_id = || self.find_type(&CoreType::I64);
        let f64_id = || self.find_type(&CoreType::F64);
        let string_id = || self.find_type(&CoreType::String);
        let bytes_id = || self.find_type(&CoreType::Bytes);
        match operation {
            CoreOperation::Unary(U::BoolNot) if self.types_are(arguments, &[CoreType::Bool]) => {
                bool_id()
            }
            CoreOperation::Unary(U::IntNegChecked | U::IntNegWrapping | U::IntBitNot)
                if arguments.len() == 1 && self.is_integer(arguments[0]) =>
            {
                Some(arguments[0])
            }
            CoreOperation::Unary(U::FloatNeg | U::FloatTrunc | U::FloatAbs)
                if self.types_are(arguments, &[CoreType::F64]) =>
            {
                f64_id()
            }
            CoreOperation::Unary(U::FloatIsNaN | U::FloatIsNegativeZero)
                if self.types_are(arguments, &[CoreType::F64]) =>
            {
                bool_id()
            }
            CoreOperation::Unary(U::StringScalarLength | U::StringUtf16Length)
                if self.types_are(arguments, &[CoreType::String]) =>
            {
                i64_id()
            }
            CoreOperation::Unary(U::StringIsEmpty)
                if self.types_are(arguments, &[CoreType::String]) =>
            {
                bool_id()
            }
            CoreOperation::Unary(U::BytesLength)
                if self.types_are(arguments, &[CoreType::Bytes]) =>
            {
                i64_id()
            }
            CoreOperation::Unary(U::BytesIsEmpty)
                if self.types_are(arguments, &[CoreType::Bytes]) =>
            {
                bool_id()
            }
            CoreOperation::Unary(U::ListLength) if self.is_list_unary(arguments) => i64_id(),
            CoreOperation::Unary(U::ListIsEmpty) if self.is_list_unary(arguments) => bool_id(),
            CoreOperation::Unary(U::OptionIsSome | U::OptionIsNone)
                if self.is_option_unary(arguments) =>
            {
                bool_id()
            }
            CoreOperation::Unary(U::ResultIsOk | U::ResultIsErr)
                if self.is_result_unary(arguments) =>
            {
                bool_id()
            }
            CoreOperation::Unary(U::WidenI32ToI64)
                if self.types_are(arguments, &[CoreType::I32]) =>
            {
                i64_id()
            }
            CoreOperation::Unary(U::NarrowI64ToI32Checked)
                if self.types_are(arguments, &[CoreType::I64]) =>
            {
                i32_id()
            }
            CoreOperation::Unary(U::StringToUtf8)
                if self.types_are(arguments, &[CoreType::String]) =>
            {
                bytes_id()
            }
            CoreOperation::Unary(U::StringFromUtf8Checked)
                if self.types_are(arguments, &[CoreType::Bytes]) =>
            {
                string_id()
            }
            CoreOperation::Binary(B::BoolAnd | B::BoolOr)
                if self.types_are(arguments, &[CoreType::Bool, CoreType::Bool]) =>
            {
                bool_id()
            }
            CoreOperation::Binary(B::Equal | B::NotEqual)
                if arguments.len() == 2
                    && arguments[0] == arguments[1]
                    && !matches!(
                        self.program.types().get(arguments[0]),
                        Some(CoreType::Interface(_))
                    ) =>
            {
                bool_id()
            }
            CoreOperation::Binary(B::Less | B::LessEqual | B::Greater | B::GreaterEqual)
                if arguments.len() == 2
                    && arguments[0] == arguments[1]
                    && self.is_ordered(arguments[0]) =>
            {
                bool_id()
            }
            CoreOperation::Binary(
                B::IntAddChecked
                | B::IntSubChecked
                | B::IntMulChecked
                | B::IntDivChecked
                | B::IntRemChecked
                | B::IntAddWrapping
                | B::IntSubWrapping
                | B::IntMulWrapping
                | B::IntBitAnd
                | B::IntBitOr
                | B::IntBitXor
                | B::IntShiftLeftChecked
                | B::IntShiftRightChecked,
            ) if arguments.len() == 2
                && arguments[0] == arguments[1]
                && self.is_integer(arguments[0]) =>
            {
                Some(arguments[0])
            }
            CoreOperation::Binary(
                B::FloatAdd | B::FloatSub | B::FloatMul | B::FloatDiv | B::FloatRemTrunc,
            ) if self.types_are(arguments, &[CoreType::F64, CoreType::F64]) => f64_id(),
            CoreOperation::Binary(B::StringConcat)
                if self.types_are(arguments, &[CoreType::String, CoreType::String]) =>
            {
                string_id()
            }
            CoreOperation::Binary(B::StringIndexOfLiteral)
                if self.types_are(arguments, &[CoreType::String, CoreType::String]) =>
            {
                self.required_type(&CoreType::Option(i64_id()?), source)
            }
            CoreOperation::Binary(B::StringContains | B::StringStartsWith | B::StringEndsWith)
                if self.types_are(arguments, &[CoreType::String, CoreType::String]) =>
            {
                bool_id()
            }
            CoreOperation::Binary(B::StringStripPrefix | B::StringTrimStart | B::StringTrimEnd)
                if self.types_are(arguments, &[CoreType::String, CoreType::String]) =>
            {
                string_id()
            }
            CoreOperation::Binary(B::StringTruncateUtf8Bytes)
                if self.types_are(arguments, &[CoreType::String, CoreType::F64]) =>
            {
                string_id()
            }
            CoreOperation::Binary(B::BytesConcat)
                if self.types_are(arguments, &[CoreType::Bytes, CoreType::Bytes]) =>
            {
                bytes_id()
            }
            CoreOperation::Binary(B::ListGetChecked) if self.is_list_index(arguments) => {
                match self.program.types().get(arguments[0]) {
                    Some(CoreType::List(inner)) => Some(*inner),
                    _ => None,
                }
            }
            CoreOperation::Binary(B::ListAppend) if self.is_list_element(arguments) => {
                Some(arguments[0])
            }
            CoreOperation::Binary(B::ListConcat)
                if arguments.len() == 2
                    && arguments[0] == arguments[1]
                    && matches!(
                        self.program.types().get(arguments[0]),
                        Some(CoreType::List(_))
                    ) =>
            {
                Some(arguments[0])
            }
            CoreOperation::Binary(B::ListContains) if self.is_list_element(arguments) => bool_id(),
            CoreOperation::Binary(B::ListIndexOf) if self.is_list_element(arguments) => {
                self.required_type(&CoreType::Option(i64_id()?), source)
            }
            CoreOperation::Binary(B::OptionUnwrapOr) if self.is_option_fallback(arguments) => {
                Some(arguments[1])
            }
            CoreOperation::Ternary(CoreTernaryIntrinsic::StringSliceScalars)
                if self.types_are(arguments, &[CoreType::String, CoreType::I64, CoreType::I64]) =>
            {
                string_id()
            }
            CoreOperation::Ternary(CoreTernaryIntrinsic::StringReplaceAll)
                if self.types_are(
                    arguments,
                    &[CoreType::String, CoreType::String, CoreType::String],
                ) =>
            {
                string_id()
            }
            CoreOperation::Ternary(CoreTernaryIntrinsic::BytesReplaceAll)
                if self.types_are(
                    arguments,
                    &[CoreType::Bytes, CoreType::Bytes, CoreType::Bytes],
                ) =>
            {
                bytes_id()
            }
            CoreOperation::Variadic(CoreVariadicIntrinsic::StringReplaceMany)
                if arguments.len() >= 3
                    && !arguments.len().is_multiple_of(2)
                    && arguments
                        .iter()
                        .all(|id| self.program.types().get(*id) == Some(&CoreType::String)) =>
            {
                string_id()
            }
            _ => invalid(self),
        }
    }

    fn check_blocks(&mut self) {
        for (id, block) in self.program.blocks().iter() {
            self.check_source(&block.source, "block");
            self.require_type(block.result_type, block.source.clone());
            if let Some(result) = block.result {
                match self.program.expressions().get(result) {
                    Some(value) if value.ty == block.result_type => {}
                    Some(_) => self.type_error("block result", &block.source),
                    None => self.missing(
                        "block result expression",
                        result.index(),
                        block.source.clone(),
                    ),
                }
            } else {
                self.expect_type(block.result_type, &CoreType::Unit, &block.source);
            }
            for statement in &block.statements {
                let source = statement_source(statement);
                self.check_source(source, "statement");
                match statement {
                    CoreStatement::Let { local, value, .. } => {
                        let value_type =
                            self.program.expressions().get(*value).map(|value| value.ty);
                        match self.program.local(*local) {
                            Some(local)
                                if local.kind == CoreLocalKind::Let
                                    && Some(local.ty) == value_type => {}
                            Some(_) => self.type_error("let binding", source),
                            None => self.missing("let binding", local.index(), source.clone()),
                        }
                        self.require_expression(*value, source.clone());
                    }
                    CoreStatement::ForEach {
                        binding,
                        iterable,
                        body,
                        ..
                    } => {
                        let element =
                            self.program
                                .expressions()
                                .get(*iterable)
                                .and_then(|value| match self.program.types().get(value.ty) {
                                    Some(CoreType::List(inner)) => Some(*inner),
                                    _ => None,
                                });
                        match self.program.local(*binding) {
                            Some(local)
                                if local.kind == CoreLocalKind::ForEach
                                    && Some(local.ty) == element => {}
                            Some(_) => self.type_error("for-each binding", source),
                            None => {
                                self.missing("for-each binding", binding.index(), source.clone())
                            }
                        }
                        self.require_expression(*iterable, source.clone());
                        if body.index() >= id.index() {
                            self.error(
                                DiagnosticCode::InvalidControlFlow,
                                "nested block does not precede its containing block",
                                source.clone(),
                            );
                        }
                        self.require_block(*body, source.clone());
                    }
                    CoreStatement::Return { value, .. } => {
                        if let Some(value) = value {
                            self.require_expression(*value, source.clone());
                        }
                    }
                    CoreStatement::Evaluate { value, .. } => {
                        self.require_expression(*value, source.clone());
                    }
                }
            }
        }
    }

    fn check_callable_scopes(&mut self) {
        for function in self.program.functions() {
            let environment = function
                .parameters
                .iter()
                .filter_map(|parameter| parameter.local)
                .collect();
            self.check_block_scope(
                function.body,
                environment,
                function.return_type,
                &mut BTreeSet::new(),
            );
        }
        for method in self.program.implementation_methods() {
            let environment = method
                .parameters
                .iter()
                .filter_map(|parameter| parameter.local)
                .collect();
            self.check_block_scope(
                method.body,
                environment,
                method.return_type,
                &mut BTreeSet::new(),
            );
        }
    }

    fn check_block_scope(
        &mut self,
        block: CoreBlockId,
        mut environment: BTreeSet<CoreLocalId>,
        return_type: CoreTypeId,
        visiting: &mut BTreeSet<CoreBlockId>,
    ) {
        if !visiting.insert(block) {
            self.error(
                DiagnosticCode::InvalidControlFlow,
                "Core block graph contains a cycle",
                self.root_source(),
            );
            return;
        }
        let Some(value) = self.program.blocks().get(block).cloned() else {
            visiting.remove(&block);
            return;
        };
        for statement in &value.statements {
            match statement {
                CoreStatement::Let { local, value, .. } => {
                    self.check_expression_scope(*value, &environment, return_type, visiting);
                    if !environment.insert(*local) {
                        self.error(
                            DiagnosticCode::DuplicateDeclaration,
                            "Core local is defined more than once in a lexical scope",
                            statement_source(statement).clone(),
                        );
                    }
                }
                CoreStatement::ForEach {
                    binding,
                    iterable,
                    body,
                    ..
                } => {
                    self.check_expression_scope(*iterable, &environment, return_type, visiting);
                    let mut body_environment = environment.clone();
                    body_environment.insert(*binding);
                    self.check_block_scope(*body, body_environment, return_type, visiting);
                }
                CoreStatement::Return { value, .. } => {
                    let actual = value.and_then(|value| {
                        self.check_expression_scope(value, &environment, return_type, visiting);
                        self.program.expressions().get(value).map(|value| value.ty)
                    });
                    let unit = self.find_type(&CoreType::Unit);
                    if actual.or(unit) != Some(return_type) {
                        self.type_error("return statement", statement_source(statement));
                    }
                }
                CoreStatement::Evaluate { value, .. } => {
                    self.check_expression_scope(*value, &environment, return_type, visiting);
                }
            }
        }
        if let Some(result) = value.result {
            self.check_expression_scope(result, &environment, return_type, visiting);
        }
        visiting.remove(&block);
    }

    fn check_expression_scope(
        &mut self,
        expression: CoreExprId,
        environment: &BTreeSet<CoreLocalId>,
        return_type: CoreTypeId,
        visiting: &mut BTreeSet<CoreBlockId>,
    ) {
        let Some(value) = self.program.expressions().get(expression).cloned() else {
            return;
        };
        if let CoreExprKind::Local(local) = value.kind {
            if !environment.contains(&local) {
                self.error(
                    DiagnosticCode::InvalidControlFlow,
                    "Core local use is not dominated by its definition",
                    value.source,
                );
            }
            return;
        }
        match &value.kind {
            CoreExprKind::If {
                condition,
                then_block,
                else_block,
            } => {
                if condition.index() < expression.index() {
                    self.check_expression_scope(*condition, environment, return_type, visiting);
                }
                self.check_block_scope(*then_block, environment.clone(), return_type, visiting);
                self.check_block_scope(*else_block, environment.clone(), return_type, visiting);
            }
            CoreExprKind::Match { value, arms } => {
                if value.index() < expression.index() {
                    self.check_expression_scope(*value, environment, return_type, visiting);
                }
                for arm in arms {
                    let mut arm_environment = environment.clone();
                    add_pattern_locals(&arm.pattern, &mut arm_environment);
                    self.check_block_scope(arm.body, arm_environment, return_type, visiting);
                }
            }
            CoreExprKind::Block(block) => {
                self.check_block_scope(*block, environment.clone(), return_type, visiting)
            }
            _ => {
                for child in expression_children(&value.kind) {
                    if child.index() < expression.index() {
                        self.check_expression_scope(child, environment, return_type, visiting);
                    }
                }
            }
        }
    }

    fn check_tests(&mut self) {
        for test in self.program.tests() {
            self.check_source(&test.header.source, "portable test");
            let return_type = match &test.invocation {
                CoreTestInvocation::Function {
                    function,
                    arguments,
                } => match self.program.function(*function) {
                    Some(function) => {
                        self.check_test_arguments(
                            arguments,
                            &function.parameters,
                            &test.header.source,
                        );
                        Some(function.return_type)
                    }
                    None => {
                        self.missing(
                            "test function",
                            function.index(),
                            test.header.source.clone(),
                        );
                        None
                    }
                },
                CoreTestInvocation::Method {
                    implementation: implementation_id,
                    method,
                    receiver,
                    arguments,
                } => match (
                    self.program.implementation(*implementation_id),
                    self.program.implementation_method(*method),
                ) {
                    (Some(implementation), Some(method))
                        if method.implementation == *implementation_id =>
                    {
                        let receiver_type = self.required_type(
                            &CoreType::Record(implementation.record),
                            &test.header.source,
                        );
                        self.check_typed_value(receiver, receiver_type, &test.header.source);
                        self.check_test_arguments(
                            arguments,
                            &method.parameters,
                            &test.header.source,
                        );
                        Some(method.return_type)
                    }
                    (Some(_), Some(_)) => {
                        self.error(
                            DiagnosticCode::InvalidPortableTest,
                            "test method belongs to another implementation",
                            test.header.source.clone(),
                        );
                        None
                    }
                    _ => {
                        self.error(
                            DiagnosticCode::InvalidPortableTest,
                            "test method invocation has an unresolved target",
                            test.header.source.clone(),
                        );
                        None
                    }
                },
            };
            match &test.expected {
                CoreExpectedOutcome::Value(value) => {
                    self.check_typed_value(value, return_type, &test.header.source)
                }
                CoreExpectedOutcome::Error(value) => {
                    self.check_typed_value(value, None, &test.header.source)
                }
            }
        }
    }

    fn check_test_arguments(
        &mut self,
        arguments: &[CoreTypedValue],
        parameters: &[CoreParameter],
        source: &SourceRef,
    ) {
        if arguments.len() != parameters.len() {
            self.error(
                DiagnosticCode::InvalidPortableTest,
                "portable test argument count does not match the callable",
                source.clone(),
            );
        }
        for (argument, parameter) in arguments.iter().zip(parameters) {
            self.check_typed_value(argument, Some(parameter.ty), source);
        }
    }

    fn check_typed_value(
        &mut self,
        value: &CoreTypedValue,
        expected: Option<CoreTypeId>,
        source: &SourceRef,
    ) {
        self.require_type(value.ty, source.clone());
        self.check_value(&value.value, value.ty, source);
        if expected.is_some_and(|expected| !self.is_assignable(value.ty, expected)) {
            self.error(
                DiagnosticCode::InvalidPortableTest,
                "portable test value type does not match its invocation signature",
                source.clone(),
            );
        }
    }

    fn child_type(
        &mut self,
        parent: CoreExprId,
        child: CoreExprId,
        source: &SourceRef,
    ) -> Option<CoreTypeId> {
        if child.index() >= parent.index() {
            self.error(
                DiagnosticCode::InvalidControlFlow,
                "Core expression operand is not defined before its use",
                source.clone(),
            );
        }
        match self.program.expressions().get(child) {
            Some(value) => Some(value.ty),
            None => {
                self.missing("expression operand", child.index(), source.clone());
                None
            }
        }
    }

    fn block_type(&mut self, block: CoreBlockId, source: &SourceRef) -> Option<CoreTypeId> {
        match self.program.blocks().get(block) {
            Some(value) => Some(value.result_type),
            None => {
                self.missing("expression block", block.index(), source.clone());
                None
            }
        }
    }

    fn require_expression(&mut self, id: CoreExprId, source: SourceRef) {
        if self.program.expressions().get(id).is_none() {
            self.missing("statement expression", id.index(), source);
        }
    }

    fn require_block(&mut self, id: CoreBlockId, source: SourceRef) {
        if self.program.blocks().get(id).is_none() {
            self.missing("nested block", id.index(), source);
        }
    }

    fn require_type(&mut self, id: CoreTypeId, source: SourceRef) {
        if self.program.types().get(id).is_none() {
            self.missing("Core type", id.index(), source);
        }
    }

    fn required_type(&mut self, ty: &CoreType, source: &SourceRef) -> Option<CoreTypeId> {
        let result = self.find_type(ty);
        if result.is_none() {
            self.error(
                DiagnosticCode::InvalidStructure,
                "derived Core type is absent from the canonical type arena",
                source.clone(),
            );
        }
        result
    }

    fn required_named_type(
        &mut self,
        ty: &CoreType,
        category: &str,
        index: usize,
        source: &SourceRef,
    ) -> Option<CoreTypeId> {
        let declaration_exists = match ty {
            CoreType::Record(id) => self.program.record(*id).is_some(),
            CoreType::Enum(id) => self.program.enumeration(*id).is_some(),
            CoreType::Interface(id) => self.program.interface(*id).is_some(),
            _ => true,
        };
        if !declaration_exists {
            self.missing(category, index, source.clone());
        }
        self.required_type(ty, source)
    }

    fn find_type(&self, expected: &CoreType) -> Option<CoreTypeId> {
        self.program
            .types()
            .iter()
            .find_map(|(id, ty)| (ty == expected).then_some(id))
    }

    fn expect_type(&mut self, actual: CoreTypeId, expected: &CoreType, source: &SourceRef) {
        match self.program.types().get(actual) {
            Some(actual) if actual == expected => {}
            _ => self.type_error("Core value", source),
        }
    }

    fn types_are(&self, actual: &[CoreTypeId], expected: &[CoreType]) -> bool {
        actual.len() == expected.len()
            && actual
                .iter()
                .zip(expected)
                .all(|(actual, expected)| self.program.types().get(*actual) == Some(expected))
    }

    fn is_integer(&self, ty: CoreTypeId) -> bool {
        matches!(
            self.program.types().get(ty),
            Some(CoreType::I32 | CoreType::I64)
        )
    }

    fn is_ordered(&self, ty: CoreTypeId) -> bool {
        matches!(
            self.program.types().get(ty),
            Some(CoreType::I32 | CoreType::I64 | CoreType::F64 | CoreType::Char | CoreType::String)
        )
    }

    fn is_list_unary(&self, arguments: &[CoreTypeId]) -> bool {
        matches!(arguments, [value] if matches!(self.program.types().get(*value), Some(CoreType::List(_))))
    }

    fn is_option_unary(&self, arguments: &[CoreTypeId]) -> bool {
        matches!(arguments, [value] if matches!(self.program.types().get(*value), Some(CoreType::Option(_))))
    }

    fn is_result_unary(&self, arguments: &[CoreTypeId]) -> bool {
        matches!(arguments, [value] if matches!(self.program.types().get(*value), Some(CoreType::Result { .. })))
    }

    fn is_list_index(&self, arguments: &[CoreTypeId]) -> bool {
        matches!(arguments, [list, index]
            if matches!(self.program.types().get(*list), Some(CoreType::List(_)))
                && self.program.types().get(*index) == Some(&CoreType::I64))
    }

    fn is_list_element(&self, arguments: &[CoreTypeId]) -> bool {
        matches!(arguments, [list, element]
            if matches!(self.program.types().get(*list), Some(CoreType::List(inner)) if inner == element))
    }

    fn is_option_fallback(&self, arguments: &[CoreTypeId]) -> bool {
        matches!(arguments, [option, fallback]
            if matches!(self.program.types().get(*option), Some(CoreType::Option(inner)) if inner == fallback))
    }

    fn is_assignable(&self, actual: CoreTypeId, expected: CoreTypeId) -> bool {
        if actual == expected {
            return true;
        }
        match (
            self.program.types().get(actual),
            self.program.types().get(expected),
        ) {
            (Some(CoreType::Record(record)), Some(CoreType::Interface(interface))) => {
                self.program.implementations().iter().any(|implementation| {
                    implementation.record == *record && implementation.interface == *interface
                })
            }
            _ => false,
        }
    }

    fn check_unique_ids<T: Copy + Ord>(&mut self, ids: &[T], source: &SourceRef, category: &str) {
        if ids.iter().copied().collect::<BTreeSet<_>>().len() != ids.len() {
            self.error(
                DiagnosticCode::DuplicateDeclaration,
                format!("Core {category} index contains a duplicate"),
                source.clone(),
            );
        }
    }

    fn check_source(&mut self, source: &SourceRef, category: &str) {
        let valid = match source {
            SourceRef::File(span) => {
                !span.file.is_empty()
                    && !std::path::Path::new(&span.file).is_absolute()
                    && span.start <= span.end
            }
            SourceRef::Logical(path) => {
                !path.segments.is_empty() && path.segments.iter().all(|part| !part.is_empty())
            }
        };
        if !valid {
            self.error(
                DiagnosticCode::InvalidStructure,
                format!("Core {category} has invalid or missing source provenance"),
                source.clone(),
            );
        }
    }

    fn root_source(&self) -> SourceRef {
        self.program
            .constants()
            .first()
            .map(|value| value.header.source.clone())
            .or_else(|| {
                self.program
                    .aliases()
                    .first()
                    .map(|value| value.header.source.clone())
            })
            .or_else(|| {
                self.program
                    .records()
                    .first()
                    .map(|value| value.header.source.clone())
            })
            .or_else(|| {
                self.program
                    .enums()
                    .first()
                    .map(|value| value.header.source.clone())
            })
            .or_else(|| {
                self.program
                    .interfaces()
                    .first()
                    .map(|value| value.header.source.clone())
            })
            .or_else(|| {
                self.program
                    .functions()
                    .first()
                    .map(|value| value.header.source.clone())
            })
            .unwrap_or_else(|| SourceRef::logical(["core", "module"]))
    }

    fn type_error(&mut self, category: &str, source: &SourceRef) {
        self.error(
            DiagnosticCode::TypeMismatch,
            format!("{category} has a mismatched Core type"),
            source.clone(),
        );
    }

    fn missing(&mut self, category: &str, index: usize, source: SourceRef) {
        self.error(
            DiagnosticCode::UnresolvedReference,
            format!("{category} ID {index} is out of bounds"),
            source,
        );
    }

    fn error(&mut self, code: DiagnosticCode, message: impl Into<String>, source: SourceRef) {
        self.diagnostics
            .push(Diagnostic::error(code, message, source));
    }
}

#[derive(Clone, Copy)]
enum CoreOperation {
    Unary(CoreUnaryIntrinsic),
    Binary(CoreBinaryIntrinsic),
    Ternary(CoreTernaryIntrinsic),
    Variadic(CoreVariadicIntrinsic),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
enum Coverage {
    Wildcard,
    Bool(bool),
    Variant(CoreVariantId),
    None,
    Some,
    Ok,
    Err,
}

fn signature_types(parameters: &[CoreParameter]) -> Vec<CoreTypeId> {
    parameters.iter().map(|parameter| parameter.ty).collect()
}

fn record_fields(program: &CoreProgram, record: CoreRecordId) -> Option<&[CoreFieldId]> {
    program
        .record(record)
        .map(|record| record.fields.as_slice())
}

fn constant_intrinsic_parts<T>(intrinsic: &CoreIntrinsicExpr<T>) -> (CoreOperation, Vec<&T>) {
    match intrinsic {
        CoreIntrinsicExpr::Unary { operation, operand } => {
            (CoreOperation::Unary(*operation), vec![operand])
        }
        CoreIntrinsicExpr::Binary {
            operation,
            left,
            right,
        } => (CoreOperation::Binary(*operation), vec![left, right]),
        CoreIntrinsicExpr::Ternary {
            operation,
            first,
            second,
            third,
        } => (
            CoreOperation::Ternary(*operation),
            vec![first, second, third],
        ),
        CoreIntrinsicExpr::Variadic {
            operation,
            arguments,
        } => (
            CoreOperation::Variadic(*operation),
            arguments.iter().collect(),
        ),
    }
}

fn expression_intrinsic_parts(
    intrinsic: &CoreIntrinsicExpr<CoreExprId>,
) -> (CoreOperation, Vec<CoreExprId>) {
    let (operation, values) = constant_intrinsic_parts(intrinsic);
    (operation, values.into_iter().copied().collect())
}

fn expression_children(expression: &CoreExprKind) -> Vec<CoreExprId> {
    match expression {
        CoreExprKind::Literal(_)
        | CoreExprKind::Local(_)
        | CoreExprKind::Constant(_)
        | CoreExprKind::SelfValue(_)
        | CoreExprKind::ConstructNone { .. }
        | CoreExprKind::If { .. }
        | CoreExprKind::Match { .. }
        | CoreExprKind::Block(_) => vec![],
        CoreExprKind::ConstructRecord { fields, .. }
        | CoreExprKind::ConstructEnum { fields, .. } => {
            fields.iter().map(|field| field.value).collect()
        }
        CoreExprKind::ConstructSome(value) => vec![*value],
        CoreExprKind::ConstructOk { value, .. }
        | CoreExprKind::ConstructErr { value, .. }
        | CoreExprKind::CoerceInterface { value, .. }
        | CoreExprKind::Field { value, .. } => vec![*value],
        CoreExprKind::ConstructList { elements, .. } => elements.clone(),
        CoreExprKind::Call { arguments, .. } => arguments.clone(),
        CoreExprKind::StaticMethodCall {
            receiver,
            arguments,
            ..
        }
        | CoreExprKind::InterfaceCall {
            receiver,
            arguments,
            ..
        } => std::iter::once(*receiver)
            .chain(arguments.iter().copied())
            .collect(),
        CoreExprKind::Intrinsic(intrinsic) => expression_intrinsic_parts(intrinsic).1,
    }
}

fn add_pattern_locals(pattern: &CorePattern, environment: &mut BTreeSet<CoreLocalId>) {
    match pattern {
        CorePattern::EnumVariant { bindings, .. } => {
            environment.extend(bindings.iter().map(|binding| binding.binding));
        }
        CorePattern::Some { binding, .. }
        | CorePattern::Ok { binding, .. }
        | CorePattern::Err { binding, .. } => {
            environment.insert(*binding);
        }
        CorePattern::Wildcard { .. } | CorePattern::Bool { .. } | CorePattern::None { .. } => {}
    }
}

fn pattern_source(pattern: &CorePattern) -> &SourceRef {
    match pattern {
        CorePattern::Wildcard { source }
        | CorePattern::Bool { source, .. }
        | CorePattern::EnumVariant { source, .. }
        | CorePattern::None { source }
        | CorePattern::Some { source, .. }
        | CorePattern::Ok { source, .. }
        | CorePattern::Err { source, .. } => source,
    }
}

fn statement_source(statement: &CoreStatement) -> &SourceRef {
    match statement {
        CoreStatement::Let { source, .. }
        | CoreStatement::ForEach { source, .. }
        | CoreStatement::Return { source, .. }
        | CoreStatement::Evaluate { source, .. } => source,
    }
}
