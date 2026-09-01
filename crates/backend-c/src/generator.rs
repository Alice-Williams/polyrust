use std::collections::BTreeMap;

use portable_check::v0::CheckedProgram;
use portable_codegen::BackendError;
use portable_ir::v0::{
    Block, ConstantExpression, Declaration, ExpectedOutcome, Expression, Intrinsic, MethodDispatch,
    NodeId, Parameter, TestInvocation, TypeRef, TypedValue, Value,
};

pub(crate) struct Generator<'a> {
    program: &'a CheckedProgram,
    prefix: String,
    declarations: BTreeMap<NodeId, &'a Declaration>,
}

impl<'a> Generator<'a> {
    pub(crate) fn new(program: &'a CheckedProgram) -> Self {
        Self {
            program,
            prefix: identifier(&program.module().name),
            declarations: program
                .module()
                .declarations
                .iter()
                .map(|declaration| (declaration.header().node.id, declaration))
                .collect(),
        }
    }

    pub(crate) fn validate(&self) -> Result<(), BackendError> {
        for declaration in &self.program.module().declarations {
            match declaration {
                Declaration::Constant(item) => {
                    self.validate_type(&item.ty)?;
                    self.validate_constant(&item.value)?;
                }
                Declaration::Implementation(item) => {
                    for method in &item.methods {
                        for parameter in &method.parameters {
                            self.validate_type(&parameter.ty)?;
                        }
                        self.validate_type(&method.return_type)?;
                        self.validate_block(&method.body)?;
                    }
                }
                Declaration::Function(item) => {
                    for parameter in &item.parameters {
                        self.validate_type(&parameter.ty)?;
                    }
                    self.validate_type(&item.return_type)?;
                    self.validate_block(&item.body)?;
                }
                Declaration::Alias(item) => self.validate_type(&item.target)?,
                Declaration::Record(item) => {
                    for field in &item.fields {
                        self.validate_type(&field.ty)?;
                    }
                }
                Declaration::Contract(item) => {
                    for method in &item.methods {
                        for parameter in &method.parameters {
                            self.validate_type(&parameter.ty)?;
                        }
                        self.validate_type(&method.return_type)?;
                    }
                }
                Declaration::Test(_) => {}
                Declaration::Enum(_) => {
                    return self
                        .unsupported("enum lowering follows the initial owned-record C slice");
                }
            }
        }
        Ok(())
    }

    fn validate_type(&self, ty: &TypeRef) -> Result<(), BackendError> {
        match self.resolve_alias(ty) {
            TypeRef::List(_) | TypeRef::Option(_) | TypeRef::Result { .. } => self.unsupported(
                "list, option, and value-result types require the next monomorphization slice",
            ),
            TypeRef::Named(id) if !self.is_record(id) => {
                self.unsupported("only named records and scalar aliases are lowered in this slice")
            }
            _ => Ok(()),
        }
    }

    fn validate_constant(&self, value: &ConstantExpression) -> Result<(), BackendError> {
        match value {
            ConstantExpression::Literal { value, .. } if is_scalar_value(value) => Ok(()),
            ConstantExpression::Reference { .. } => Ok(()),
            _ => self.unsupported(
                "aggregate, list, option, and value-result constants are not lowered yet",
            ),
        }
    }

    fn validate_block(&self, block: &Block) -> Result<(), BackendError> {
        if !block.statements.is_empty() {
            return self
                .unsupported("statement and bounded-iteration lowering is not available yet");
        }
        let result = block
            .result
            .as_deref()
            .ok_or_else(|| BackendError::Generation {
                message: "C17 callable blocks currently require a result expression".into(),
            })?;
        self.validate_expression(result)
    }

    fn validate_expression(&self, expression: &Expression) -> Result<(), BackendError> {
        match expression {
            Expression::Literal { value, .. } if is_scalar_value(value) => Ok(()),
            Expression::Local { .. }
            | Expression::Constant { .. }
            | Expression::SelfValue { .. } => Ok(()),
            Expression::Field { base, .. } => self.validate_expression(base),
            Expression::MethodCall {
                receiver,
                arguments,
                ..
            } => {
                self.validate_expression(receiver)?;
                for argument in arguments {
                    self.validate_expression(argument)?;
                }
                Ok(())
            }
            Expression::Intrinsic {
                operation,
                arguments,
                ..
            } => {
                if !matches!(
                    operation,
                    Intrinsic::BoolNot
                        | Intrinsic::BoolAnd
                        | Intrinsic::BoolOr
                        | Intrinsic::Equal
                        | Intrinsic::NotEqual
                        | Intrinsic::Less
                        | Intrinsic::LessEqual
                        | Intrinsic::Greater
                        | Intrinsic::GreaterEqual
                        | Intrinsic::StringConcat
                        | Intrinsic::StringIsEmpty
                        | Intrinsic::StringContains
                        | Intrinsic::StringStartsWith
                        | Intrinsic::StringStripPrefix
                        | Intrinsic::StringEndsWith
                        | Intrinsic::StringReplaceAll
                        | Intrinsic::StringTrimStart
                        | Intrinsic::StringTrimEnd
                ) {
                    return self
                        .unsupported(&format!("intrinsic {operation:?} is not lowered yet"));
                }
                for argument in arguments {
                    self.validate_expression(argument)?;
                }
                Ok(())
            }
            Expression::If {
                condition,
                then_block,
                else_block,
                ..
            } => {
                self.validate_expression(condition)?;
                self.validate_block(then_block)?;
                self.validate_block(else_block)
            }
            Expression::Block(block) => self.validate_block(block),
            _ => self.unsupported("this expression form is not lowered yet"),
        }
    }

    fn unsupported<T>(&self, message: &str) -> Result<T, BackendError> {
        Err(BackendError::Generation {
            message: format!("C17 backend: {message}"),
        })
    }

    pub(crate) fn header(&self) -> String {
        let guard = format!("{}_GENERATED_H", self.prefix.to_ascii_uppercase());
        let mut output = format!(
            "#ifndef {guard}\n#define {guard}\n\n/* Generated by PolyRust from checked IR v0. */\n#include \"runtime.h\"\n\n"
        );
        output.push_str(&format!(
            "typedef struct {0}_unit {{ uint8_t unused; }} {0}_unit;\n\
             typedef struct {0}_unit_result {{ bool ok; {0}_unit value; poly_error error; }} {0}_unit_result;\n\
             typedef struct {0}_bool_result {{ bool ok; bool value; poly_error error; }} {0}_bool_result;\n\
             typedef struct {0}_i32_result {{ bool ok; int32_t value; poly_error error; }} {0}_i32_result;\n\
             typedef struct {0}_i64_result {{ bool ok; int64_t value; poly_error error; }} {0}_i64_result;\n\
             typedef struct {0}_f64_result {{ bool ok; double value; poly_error error; }} {0}_f64_result;\n\
             typedef struct {0}_char_result {{ bool ok; uint32_t value; poly_error error; }} {0}_char_result;\n\
             typedef struct {0}_string_result {{ bool ok; poly_string value; poly_error error; }} {0}_string_result;\n\
             typedef struct {0}_bytes_result {{ bool ok; poly_bytes value; poly_error error; }} {0}_bytes_result;\n\
             void {0}_string_result_drop({0}_string_result *value);\n\
             void {0}_bytes_result_drop({0}_bytes_result *value);\n\n",
            self.prefix
        ));
        for declaration in &self.program.module().declarations {
            if let Declaration::Record(item) = declaration {
                let name = self.record_name(item.header.node.id);
                output.push_str(&format!("typedef struct {name} {name};\n"));
            }
        }
        output.push('\n');
        for declaration in &self.program.module().declarations {
            match declaration {
                Declaration::Alias(item) => output.push_str(&format!(
                    "typedef {} {}_{};\n",
                    self.ty(&item.target),
                    self.prefix,
                    type_name(&item.header.name)
                )),
                Declaration::Record(item) => output.push_str(&self.record_header(item)),
                _ => {}
            }
        }
        for declaration in &self.program.module().declarations {
            if let Declaration::Contract(item) = declaration {
                let contract = self.contract_name(item.header.node.id);
                output.push_str(&format!(
                    "typedef struct {contract} {contract};\ntypedef struct {contract}_vtable {contract}_vtable;\nstruct {contract} {{ const void *context; const {contract}_vtable *vtable; }};\nstruct {contract}_vtable {{\n"
                ));
                for method in &item.methods {
                    output.push_str(&format!(
                        "  {} (*{})(poly_allocator allocator, const void *context{});\n",
                        self.result_ty(&method.return_type),
                        value_name(&method.header.name),
                        self.parameters(&method.parameters)
                    ));
                }
                output.push_str("};\n\n");
            }
        }
        for declaration in &self.program.module().declarations {
            if let Declaration::Implementation(item) = declaration {
                output.push_str(&format!(
                    "{} {}_{}_as_{}(const {} *value);\n",
                    self.contract_name(item.contract),
                    self.prefix,
                    type_name(self.declaration_name(item.record)),
                    type_name(self.declaration_name(item.contract)),
                    self.record_name(item.record)
                ));
            }
        }
        output.push('\n');
        for declaration in &self.program.module().declarations {
            match declaration {
                Declaration::Constant(item) => output.push_str(&format!(
                    "{} {}_{}(poly_allocator allocator);\n",
                    self.result_ty(&item.ty),
                    self.prefix,
                    value_name(&item.header.name)
                )),
                Declaration::Function(item) => output.push_str(&format!(
                    "{} {}_{}(poly_allocator allocator{});\n",
                    self.result_ty(&item.return_type),
                    self.prefix,
                    value_name(&item.header.name),
                    self.parameters(&item.parameters)
                )),
                _ => {}
            }
        }
        output.push_str(&format!("\n#endif /* {guard} */\n"));
        output
    }

    fn record_header(&self, item: &portable_ir::v0::RecordDeclaration) -> String {
        let name = self.record_name(item.header.node.id);
        let mut output = format!("struct {name} {{\n");
        for field in &item.fields {
            output.push_str(&format!(
                "  {} {};\n",
                self.ty(&field.ty),
                value_name(&field.header.name)
            ));
        }
        output.push_str(&format!(
            "}};\nbool {name}_clone(poly_allocator allocator, const {name} *source, {name} *output);\nvoid {name}_drop({name} *value);\n\n"
        ));
        output
    }

    fn parameters(&self, parameters: &[Parameter]) -> String {
        parameters
            .iter()
            .map(|parameter| {
                format!(
                    "{} {}",
                    self.parameter_ty(&parameter.ty),
                    value_name(&parameter.header.name)
                )
            })
            .collect::<Vec<_>>()
            .join(", ")
            .pipe(|text| {
                if text.is_empty() {
                    text
                } else {
                    format!(", {text}")
                }
            })
    }

    fn parameter_ty(&self, ty: &TypeRef) -> String {
        match self.resolve_alias(ty) {
            TypeRef::String => "poly_string_view".into(),
            TypeRef::Bytes => "poly_bytes_view".into(),
            TypeRef::Named(id) if self.is_record(id) => {
                format!("const {} *", self.record_name(id))
            }
            TypeRef::Contract(id) => self.contract_name(id),
            other => self.ty(&other),
        }
    }

    fn ty(&self, ty: &TypeRef) -> String {
        match self.resolve_alias(ty) {
            TypeRef::Unit => format!("{}_unit", self.prefix),
            TypeRef::Bool => "bool".into(),
            TypeRef::I32 => "int32_t".into(),
            TypeRef::I64 => "int64_t".into(),
            TypeRef::F64 => "double".into(),
            TypeRef::Char => "uint32_t".into(),
            TypeRef::String => "poly_string".into(),
            TypeRef::Bytes => "poly_bytes".into(),
            TypeRef::Named(id) => self.record_name(id),
            TypeRef::Contract(id) => self.contract_name(id),
            TypeRef::List(_) | TypeRef::Option(_) | TypeRef::Result { .. } => "void *".into(),
        }
    }

    fn result_ty(&self, ty: &TypeRef) -> String {
        let suffix = match self.resolve_alias(ty) {
            TypeRef::Unit => "unit",
            TypeRef::Bool => "bool",
            TypeRef::I32 => "i32",
            TypeRef::I64 => "i64",
            TypeRef::F64 => "f64",
            TypeRef::Char => "char",
            TypeRef::String => "string",
            TypeRef::Bytes => "bytes",
            _ => "unit",
        };
        format!("{}_{suffix}_result", self.prefix)
    }

    fn resolve_alias(&self, ty: &TypeRef) -> TypeRef {
        match ty {
            TypeRef::Named(id) => match self.declarations.get(id) {
                Some(Declaration::Alias(alias)) => self.resolve_alias(&alias.target),
                _ => ty.clone(),
            },
            _ => ty.clone(),
        }
    }

    fn declaration_name(&self, id: NodeId) -> &str {
        self.declarations
            .get(&id)
            .map(|declaration| declaration.header().name.as_str())
            .unwrap_or("unknown")
    }

    fn record_name(&self, id: NodeId) -> String {
        format!("{}_{}", self.prefix, type_name(self.declaration_name(id)))
    }

    fn contract_name(&self, id: NodeId) -> String {
        format!("{}_{}", self.prefix, type_name(self.declaration_name(id)))
    }

    fn is_record(&self, id: NodeId) -> bool {
        matches!(self.declarations.get(&id), Some(Declaration::Record(_)))
    }

    pub(crate) fn source(&self) -> Result<String, BackendError> {
        let mut output = String::from(
            "/* Generated by PolyRust from checked IR v0. */\n#include \"generated.h\"\n\n#include <string.h>\n\n",
        );
        output.push_str(&format!(
            "void {0}_string_result_drop({0}_string_result *value) {{\n  if (value != NULL) {{ poly_string_drop(&value->value); *value = ({0}_string_result){{0}}; }}\n}}\n\
             void {0}_bytes_result_drop({0}_bytes_result *value) {{\n  if (value != NULL) {{ poly_bytes_drop(&value->value); *value = ({0}_bytes_result){{0}}; }}\n}}\n\n",
            self.prefix
        ));
        for declaration in &self.program.module().declarations {
            if let Declaration::Record(item) = declaration {
                output.push_str(&self.record_functions(item));
            }
        }
        for declaration in &self.program.module().declarations {
            if let Declaration::Implementation(item) = declaration {
                let record = self.declaration_name(item.record);
                for method in &item.methods {
                    output.push_str(&format!(
                        "static {} {}_{}_{}_impl(poly_allocator allocator, const {} *self_value{});\n",
                        self.result_ty(&method.return_type),
                        self.prefix,
                        type_name(record),
                        value_name(&method.header.name),
                        self.record_name(item.record),
                        self.parameters(&method.parameters)
                    ));
                }
            }
        }
        output.push('\n');
        for declaration in &self.program.module().declarations {
            if let Declaration::Implementation(item) = declaration {
                output.push_str(&self.implementation(item)?);
            }
        }
        for declaration in &self.program.module().declarations {
            match declaration {
                Declaration::Constant(item) => {
                    output.push_str(&self.constant_function(item)?);
                }
                Declaration::Function(item) => output.push_str(&self.callable(
                    &format!("{}_{}", self.prefix, value_name(&item.header.name)),
                    &item.parameters,
                    &item.return_type,
                    &item.body,
                    None,
                )?),
                _ => {}
            }
        }
        Ok(output)
    }

    fn record_functions(&self, item: &portable_ir::v0::RecordDeclaration) -> String {
        let name = self.record_name(item.header.node.id);
        let mut output = format!(
            "bool {name}_clone(poly_allocator allocator, const {name} *source, {name} *output) {{\n  {name} result = {{0}};\n  (void)allocator;\n  if (source == NULL || output == NULL) {{ return false; }}\n"
        );
        for field in &item.fields {
            let field_name = value_name(&field.header.name);
            match self.resolve_alias(&field.ty) {
                TypeRef::String => output.push_str(&format!(
                    "  if (poly_string_clone(allocator, poly_string_borrow(&source->{field_name}), &result.{field_name}) != POLY_OK) {{ {name}_drop(&result); return false; }}\n"
                )),
                TypeRef::Bytes => output.push_str(&format!(
                    "  if (!poly_bytes_clone(allocator, poly_bytes_borrow(&source->{field_name}), &result.{field_name})) {{ {name}_drop(&result); return false; }}\n"
                )),
                TypeRef::Named(id) if self.is_record(id) => output.push_str(&format!(
                    "  if (!{}_clone(allocator, &source->{field_name}, &result.{field_name})) {{ {name}_drop(&result); return false; }}\n",
                    self.record_name(id)
                )),
                _ => output.push_str(&format!(
                    "  result.{field_name} = source->{field_name};\n"
                )),
            }
        }
        output.push_str("  *output = result;\n  return true;\n}\n");
        output.push_str(&format!(
            "void {name}_drop({name} *value) {{\n  if (value == NULL) {{ return; }}\n"
        ));
        for field in &item.fields {
            let field_name = value_name(&field.header.name);
            match self.resolve_alias(&field.ty) {
                TypeRef::String => {
                    output.push_str(&format!("  poly_string_drop(&value->{field_name});\n"))
                }
                TypeRef::Bytes => {
                    output.push_str(&format!("  poly_bytes_drop(&value->{field_name});\n"))
                }
                TypeRef::Named(id) if self.is_record(id) => output.push_str(&format!(
                    "  {}_drop(&value->{field_name});\n",
                    self.record_name(id)
                )),
                _ => {}
            }
        }
        output.push_str(&format!("  *value = ({name}){{0}};\n}}\n\n"));
        output
    }

    fn implementation(
        &self,
        item: &portable_ir::v0::ImplementationDeclaration,
    ) -> Result<String, BackendError> {
        let record = self.declaration_name(item.record);
        let contract = self.declaration_name(item.contract);
        let record_ty = self.record_name(item.record);
        let contract_ty = self.contract_name(item.contract);
        let mut output = String::new();
        for method in &item.methods {
            output.push_str(&self.callable(
                &format!(
                    "{}_{}_{}_impl",
                    self.prefix,
                    type_name(record),
                    value_name(&method.header.name)
                ),
                &method.parameters,
                &method.return_type,
                &method.body,
                Some(format!("const {record_ty} *self_value")),
            )?);
        }
        let contract_declaration = match self.declarations.get(&item.contract) {
            Some(Declaration::Contract(value)) => value,
            _ => return self.unsupported("implementation contract is missing"),
        };
        for signature in &contract_declaration.methods {
            let method = item
                .methods
                .iter()
                .find(|method| method.contract_method == signature.header.node.id)
                .ok_or_else(|| BackendError::Generation {
                    message: "C17 implementation is missing a checked contract method".into(),
                })?;
            output.push_str(&format!(
                "static {} {}_{}_{}_adapter(poly_allocator allocator, const void *context{}) {{\n  return {}_{}_{}_impl(allocator, (const {record_ty} *)context{});\n}}\n",
                self.result_ty(&method.return_type),
                self.prefix,
                type_name(record),
                value_name(&method.header.name),
                self.parameters(&method.parameters),
                self.prefix,
                type_name(record),
                value_name(&method.header.name),
                argument_names(&method.parameters)
            ));
        }
        output.push_str(&format!(
            "static const {contract_ty}_vtable {}_{}_{}_vtable = {{\n",
            self.prefix,
            type_name(record),
            type_name(contract)
        ));
        for signature in &contract_declaration.methods {
            let method = item
                .methods
                .iter()
                .find(|method| method.contract_method == signature.header.node.id)
                .expect("checked implementation method");
            output.push_str(&format!(
                "  .{} = {}_{}_{}_adapter,\n",
                value_name(&signature.header.name),
                self.prefix,
                type_name(record),
                value_name(&method.header.name)
            ));
        }
        output.push_str("};\n");
        output.push_str(&format!(
            "{contract_ty} {}_{}_as_{}(const {record_ty} *value) {{\n  {contract_ty} result = {{value, &{}_{}_{}_vtable}};\n  return result;\n}}\n\n",
            self.prefix,
            type_name(record),
            type_name(contract),
            self.prefix,
            type_name(record),
            type_name(contract)
        ));
        Ok(output)
    }

    fn constant_function(
        &self,
        item: &portable_ir::v0::ConstantDeclaration,
    ) -> Result<String, BackendError> {
        let expression = match &item.value {
            ConstantExpression::Literal { value, .. } => self.literal(value)?,
            ConstantExpression::Reference { declaration, .. } => CExpression {
                prelude: String::new(),
                value: format!(
                    "{}_{}(allocator).value",
                    self.prefix,
                    value_name(self.declaration_name(*declaration))
                ),
                ty: item.ty.clone(),
            },
            _ => return self.unsupported("constant escaped validation"),
        };
        self.render_callable(
            &format!("{}_{}", self.prefix, value_name(&item.header.name)),
            &[],
            &item.ty,
            expression,
            None,
            FunctionEmitter::new(self, &[], false),
        )
    }

    fn callable(
        &self,
        name: &str,
        parameters: &[Parameter],
        return_type: &TypeRef,
        block: &Block,
        self_parameter: Option<String>,
    ) -> Result<String, BackendError> {
        let mut emitter = FunctionEmitter::new(self, parameters, self_parameter.is_some());
        let expression = emitter.expression(block.result.as_deref().expect("validated result"))?;
        self.render_callable(
            name,
            parameters,
            return_type,
            expression,
            self_parameter,
            emitter,
        )
    }

    fn render_callable(
        &self,
        name: &str,
        parameters: &[Parameter],
        return_type: &TypeRef,
        expression: CExpression,
        self_parameter: Option<String>,
        emitter: FunctionEmitter<'_, '_>,
    ) -> Result<String, BackendError> {
        let mut signature = String::from("poly_allocator allocator");
        if let Some(self_parameter) = self_parameter {
            signature.push_str(", ");
            signature.push_str(&self_parameter);
        }
        signature.push_str(&self.parameters(parameters));
        let result_ty = self.result_ty(return_type);
        let mut output = format!(
            "{result_ty} {name}({signature}) {{\n  poly_error error = {{POLY_OK, NULL}};\n  (void)allocator;\n"
        );
        for declaration in &emitter.declarations {
            output.push_str(&format!("  {declaration}\n"));
        }
        output.push_str(&indent(&expression.prelude, 2));
        match self.resolve_alias(return_type) {
            TypeRef::String => output.push_str(&format!(
                "  poly_string final_value = {{0}};\n  poly_error_code final_status = poly_string_clone(allocator, {}, &final_value);\n  if (final_status != POLY_OK) {{ error = (poly_error){{final_status, final_status == POLY_INVALID_UTF8 ? \"invalid UTF-8\" : \"allocation failed\"}}; goto fail; }}\n",
                expression.value
            )),
            TypeRef::Bytes => output.push_str(&format!(
                "  poly_bytes final_value = {{0}};\n  if (!poly_bytes_clone(allocator, {}, &final_value)) {{ error = (poly_error){{POLY_ALLOCATION_FAILED, \"allocation failed\"}}; goto fail; }}\n",
                expression.value
            )),
            _ => output.push_str(&format!(
                "  {} final_value = {};\n",
                self.ty(return_type),
                expression.value
            )),
        }
        for cleanup in emitter.cleanups.iter().rev() {
            output.push_str(&format!("  {cleanup}\n"));
        }
        output.push_str(&format!(
            "  if (error.code != POLY_OK) {{ goto fail; }}\n  return ({result_ty}){{true, final_value, {{POLY_OK, NULL}}}};\nfail:\n"
        ));
        for cleanup in emitter.cleanups.iter().rev() {
            output.push_str(&format!("  {cleanup}\n"));
        }
        output.push_str(&format!(
            "  return ({result_ty}){{.ok = false, .error = error}};\n}}\n\n"
        ));
        Ok(output)
    }
}

#[derive(Clone)]
struct CExpression {
    prelude: String,
    value: String,
    ty: TypeRef,
}

struct FunctionEmitter<'generator, 'program> {
    generator: &'generator Generator<'program>,
    locals: BTreeMap<String, CExpression>,
    self_value: Option<CExpression>,
    declarations: Vec<String>,
    cleanups: Vec<String>,
    next: usize,
}

impl<'generator, 'program> FunctionEmitter<'generator, 'program> {
    fn new(
        generator: &'generator Generator<'program>,
        parameters: &[Parameter],
        has_self: bool,
    ) -> Self {
        let locals = parameters
            .iter()
            .map(|parameter| {
                let name = value_name(&parameter.header.name);
                (
                    name.clone(),
                    CExpression {
                        prelude: String::new(),
                        value: name,
                        ty: parameter.ty.clone(),
                    },
                )
            })
            .collect();
        Self {
            generator,
            locals,
            self_value: has_self.then(|| CExpression {
                prelude: String::new(),
                value: "self_value".into(),
                ty: TypeRef::Unit,
            }),
            declarations: Vec::new(),
            cleanups: Vec::new(),
            next: 0,
        }
    }

    fn temporary(&mut self, declaration: impl FnOnce(&str) -> String) -> String {
        let name = format!("temporary_{}", self.next);
        self.next += 1;
        self.declarations.push(declaration(&name));
        name
    }

    fn expression(&mut self, expression: &Expression) -> Result<CExpression, BackendError> {
        match expression {
            Expression::Literal { value, .. } => self.generator.literal(value),
            Expression::Local { name, .. } => {
                self.locals.get(&value_name(name)).cloned().ok_or_else(|| {
                    BackendError::Generation {
                        message: format!("C17 local {name:?} is missing"),
                    }
                })
            }
            Expression::Constant { declaration, .. } => {
                let item = match self.generator.declarations.get(declaration) {
                    Some(Declaration::Constant(item)) => item,
                    _ => return self.generator.unsupported("constant reference is missing"),
                };
                self.call_result(
                    format!(
                        "{}_{}(allocator)",
                        self.generator.prefix,
                        value_name(&item.header.name)
                    ),
                    item.ty.clone(),
                )
            }
            Expression::SelfValue { .. } => {
                self.self_value
                    .clone()
                    .ok_or_else(|| BackendError::Generation {
                        message: "C17 self expression outside a method".into(),
                    })
            }
            Expression::Field { base, field, .. } => {
                let base = self.expression(base)?;
                let (name, ty) =
                    self.generator
                        .field(*field)
                        .ok_or_else(|| BackendError::Generation {
                            message: "C17 field reference is missing".into(),
                        })?;
                let access = format!("({})->{}", base.value, value_name(name));
                Ok(CExpression {
                    prelude: base.prelude,
                    value: match self.generator.resolve_alias(ty) {
                        TypeRef::String => format!("poly_string_borrow(&{access})"),
                        TypeRef::Bytes => format!("poly_bytes_borrow(&{access})"),
                        TypeRef::Named(id) if self.generator.is_record(id) => {
                            format!("&{access}")
                        }
                        _ => access,
                    },
                    ty: ty.clone(),
                })
            }
            Expression::MethodCall {
                receiver,
                dispatch,
                arguments,
                ..
            } => self.method_call(receiver, dispatch, arguments),
            Expression::Intrinsic {
                operation,
                arguments,
                ..
            } => {
                let mut values = Vec::new();
                let mut prelude = String::new();
                for argument in arguments {
                    let argument = self.expression(argument)?;
                    prelude.push_str(&argument.prelude);
                    values.push(argument);
                }
                self.intrinsic(*operation, values, prelude)
            }
            Expression::If {
                condition,
                then_block,
                else_block,
                ..
            } => self.if_expression(condition, then_block, else_block),
            Expression::Block(block) => {
                self.expression(block.result.as_deref().expect("validated result"))
            }
            _ => self
                .generator
                .unsupported("validated expression has no C17 lowering"),
        }
    }

    fn method_call(
        &mut self,
        receiver: &Expression,
        dispatch: &MethodDispatch,
        arguments: &[Expression],
    ) -> Result<CExpression, BackendError> {
        let receiver = self.expression(receiver)?;
        let mut prelude = receiver.prelude;
        let mut values = Vec::new();
        for argument in arguments {
            let argument = self.expression(argument)?;
            prelude.push_str(&argument.prelude);
            values.push(argument.value);
        }
        let suffix = if values.is_empty() {
            String::new()
        } else {
            format!(", {}", values.join(", "))
        };
        let (call, ty) = match dispatch {
            MethodDispatch::Concrete {
                implementation,
                method,
            } => {
                let (implementation, method) = self
                    .generator
                    .implementation_method(*implementation, *method)
                    .ok_or_else(|| BackendError::Generation {
                        message: "C17 concrete method is missing".into(),
                    })?;
                (
                    format!(
                        "{}_{}_{}_impl(allocator, {}{suffix})",
                        self.generator.prefix,
                        type_name(self.generator.declaration_name(implementation.record)),
                        value_name(&method.header.name),
                        receiver.value
                    ),
                    method.return_type.clone(),
                )
            }
            MethodDispatch::Contract { contract, method } => {
                let method = self
                    .generator
                    .contract_method(*contract, *method)
                    .ok_or_else(|| BackendError::Generation {
                        message: "C17 contract method is missing".into(),
                    })?;
                (
                    format!(
                        "({0}.vtable->{1}(allocator, {0}.context{suffix}))",
                        receiver.value,
                        value_name(&method.header.name)
                    ),
                    method.return_type.clone(),
                )
            }
        };
        let mut result = self.call_result(call, ty)?;
        result.prelude = format!("{prelude}{}", result.prelude);
        Ok(result)
    }

    fn if_expression(
        &mut self,
        condition: &Expression,
        then_block: &Block,
        else_block: &Block,
    ) -> Result<CExpression, BackendError> {
        let condition = self.expression(condition)?;
        let then_value =
            self.expression(then_block.result.as_deref().expect("validated result"))?;
        let else_value =
            self.expression(else_block.result.as_deref().expect("validated result"))?;
        let ty = then_value.ty.clone();
        match self.generator.resolve_alias(&ty) {
            TypeRef::String => {
                let temporary = self.temporary(|name| format!("poly_string {name} = {{0}};"));
                self.cleanups
                    .push(format!("poly_string_drop(&{temporary});"));
                let prelude = format!(
                    "{}if ({}) {{\n{}  poly_error_code branch_status = poly_string_clone(allocator, {}, &{});\n  if (branch_status != POLY_OK) {{ error = (poly_error){{branch_status, branch_status == POLY_INVALID_UTF8 ? \"invalid UTF-8\" : \"allocation failed\"}}; goto fail; }}\n}} else {{\n{}  poly_error_code branch_status = poly_string_clone(allocator, {}, &{});\n  if (branch_status != POLY_OK) {{ error = (poly_error){{branch_status, branch_status == POLY_INVALID_UTF8 ? \"invalid UTF-8\" : \"allocation failed\"}}; goto fail; }}\n}}\n",
                    condition.prelude,
                    condition.value,
                    indent(&then_value.prelude, 2),
                    then_value.value,
                    temporary,
                    indent(&else_value.prelude, 2),
                    else_value.value,
                    temporary
                );
                Ok(CExpression {
                    prelude,
                    value: format!("poly_string_borrow(&{temporary})"),
                    ty,
                })
            }
            _ => {
                let c_ty = self.generator.ty(&ty);
                let temporary = self.temporary(|name| format!("{c_ty} {name} = {{0}};"));
                let prelude = format!(
                    "{}if ({}) {{\n{}  {} = {};\n}} else {{\n{}  {} = {};\n}}\n",
                    condition.prelude,
                    condition.value,
                    indent(&then_value.prelude, 2),
                    temporary,
                    then_value.value,
                    indent(&else_value.prelude, 2),
                    temporary,
                    else_value.value
                );
                Ok(CExpression {
                    prelude,
                    value: temporary,
                    ty,
                })
            }
        }
    }

    fn call_result(&mut self, call: String, ty: TypeRef) -> Result<CExpression, BackendError> {
        let result_ty = self.generator.result_ty(&ty);
        let temporary = self.temporary(|name| format!("{result_ty} {name} = {{0}};"));
        let prelude = format!(
            "{temporary} = {call};\nif (!{temporary}.ok) {{ error = {temporary}.error; goto fail; }}\n"
        );
        let value = match self.generator.resolve_alias(&ty) {
            TypeRef::String => {
                self.cleanups.push(format!(
                    "{}_string_result_drop(&{temporary});",
                    self.generator.prefix
                ));
                format!("poly_string_borrow(&{temporary}.value)")
            }
            TypeRef::Bytes => {
                self.cleanups.push(format!(
                    "{}_bytes_result_drop(&{temporary});",
                    self.generator.prefix
                ));
                format!("poly_bytes_borrow(&{temporary}.value)")
            }
            _ => format!("{temporary}.value"),
        };
        Ok(CExpression { prelude, value, ty })
    }

    fn intrinsic(
        &mut self,
        operation: Intrinsic,
        values: Vec<CExpression>,
        mut prelude: String,
    ) -> Result<CExpression, BackendError> {
        let value = |index: usize| {
            values
                .get(index)
                .map(|item| item.value.as_str())
                .unwrap_or("0")
        };
        let scalar = |text: String, ty: TypeRef, prelude: String| CExpression {
            prelude,
            value: text,
            ty,
        };
        let result = match operation {
            Intrinsic::BoolNot => scalar(format!("!({})", value(0)), TypeRef::Bool, prelude),
            Intrinsic::BoolAnd => scalar(
                format!("({}) && ({})", value(0), value(1)),
                TypeRef::Bool,
                prelude,
            ),
            Intrinsic::BoolOr => scalar(
                format!("({}) || ({})", value(0), value(1)),
                TypeRef::Bool,
                prelude,
            ),
            Intrinsic::Equal | Intrinsic::NotEqual => {
                let equal =
                    if matches!(self.generator.resolve_alias(&values[0].ty), TypeRef::String) {
                        format!("poly_string_equal({}, {})", value(0), value(1))
                    } else {
                        format!("({}) == ({})", value(0), value(1))
                    };
                scalar(
                    if operation == Intrinsic::NotEqual {
                        format!("!({equal})")
                    } else {
                        equal
                    },
                    TypeRef::Bool,
                    prelude,
                )
            }
            Intrinsic::Less
            | Intrinsic::LessEqual
            | Intrinsic::Greater
            | Intrinsic::GreaterEqual => {
                let operator = match operation {
                    Intrinsic::Less => "<",
                    Intrinsic::LessEqual => "<=",
                    Intrinsic::Greater => ">",
                    _ => ">=",
                };
                scalar(
                    format!("({}) {operator} ({})", value(0), value(1)),
                    TypeRef::Bool,
                    prelude,
                )
            }
            Intrinsic::StringIsEmpty => scalar(
                format!("({}).length == 0U", value(0)),
                TypeRef::Bool,
                prelude,
            ),
            Intrinsic::StringContains => scalar(
                format!("poly_string_contains({}, {})", value(0), value(1)),
                TypeRef::Bool,
                prelude,
            ),
            Intrinsic::StringStartsWith => scalar(
                format!("poly_string_starts_with({}, {})", value(0), value(1)),
                TypeRef::Bool,
                prelude,
            ),
            Intrinsic::StringEndsWith => scalar(
                format!("poly_string_ends_with({}, {})", value(0), value(1)),
                TypeRef::Bool,
                prelude,
            ),
            Intrinsic::StringConcat
            | Intrinsic::StringStripPrefix
            | Intrinsic::StringReplaceAll
            | Intrinsic::StringTrimStart
            | Intrinsic::StringTrimEnd => {
                let temporary = self.temporary(|name| format!("poly_string {name} = {{0}};"));
                self.cleanups
                    .push(format!("poly_string_drop(&{temporary});"));
                let call = match operation {
                    Intrinsic::StringConcat => format!(
                        "poly_string_concat(allocator, {}, {}, &{temporary})",
                        value(0),
                        value(1)
                    ),
                    Intrinsic::StringStripPrefix => format!(
                        "poly_string_strip_prefix(allocator, {}, {}, &{temporary})",
                        value(0),
                        value(1)
                    ),
                    Intrinsic::StringReplaceAll => format!(
                        "poly_string_replace_all(allocator, {}, {}, {}, &{temporary})",
                        value(0),
                        value(1),
                        value(2)
                    ),
                    Intrinsic::StringTrimStart => format!(
                        "poly_string_trim_start(allocator, {}, {}, &{temporary})",
                        value(0),
                        value(1)
                    ),
                    Intrinsic::StringTrimEnd => format!(
                        "poly_string_trim_end(allocator, {}, {}, &{temporary})",
                        value(0),
                        value(1)
                    ),
                    _ => unreachable!(),
                };
                let status = self.temporary(|name| format!("poly_error_code {name} = POLY_OK;"));
                prelude.push_str(&format!(
                    "{status} = {call};\nif ({status} != POLY_OK) {{ error = (poly_error){{{status}, {status} == POLY_INVALID_UTF8 ? \"invalid UTF-8\" : \"allocation failed\"}}; goto fail; }}\n"
                ));
                scalar(
                    format!("poly_string_borrow(&{temporary})"),
                    TypeRef::String,
                    prelude,
                )
            }
            _ => {
                return self
                    .generator
                    .unsupported("intrinsic escaped C17 validation");
            }
        };
        Ok(result)
    }
}

impl Generator<'_> {
    fn field(&self, id: NodeId) -> Option<(&str, &TypeRef)> {
        for declaration in self.declarations.values() {
            match declaration {
                Declaration::Record(record) => {
                    if let Some(field) = record
                        .fields
                        .iter()
                        .find(|field| field.header.node.id == id)
                    {
                        return Some((&field.header.name, &field.ty));
                    }
                }
                Declaration::Enum(enumeration) => {
                    for variant in &enumeration.variants {
                        if let Some(field) = variant
                            .fields
                            .iter()
                            .find(|field| field.header.node.id == id)
                        {
                            return Some((&field.header.name, &field.ty));
                        }
                    }
                }
                _ => {}
            }
        }
        None
    }

    fn implementation_method(
        &self,
        implementation: NodeId,
        method: NodeId,
    ) -> Option<(
        &portable_ir::v0::ImplementationDeclaration,
        &portable_ir::v0::MethodImplementation,
    )> {
        match self.declarations.get(&implementation) {
            Some(Declaration::Implementation(item)) => item
                .methods
                .iter()
                .find(|candidate| {
                    candidate.header.node.id == method || candidate.contract_method == method
                })
                .map(|method| (item, method)),
            _ => None,
        }
    }

    fn contract_method(
        &self,
        contract: NodeId,
        method: NodeId,
    ) -> Option<&portable_ir::v0::MethodSignature> {
        match self.declarations.get(&contract) {
            Some(Declaration::Contract(item)) => item
                .methods
                .iter()
                .find(|candidate| candidate.header.node.id == method),
            _ => None,
        }
    }

    fn literal(&self, value: &Value) -> Result<CExpression, BackendError> {
        let (value, ty) = match value {
            Value::Unit => (format!("({}_unit){{0}}", self.prefix), TypeRef::Unit),
            Value::Bool(value) => (value.to_string(), TypeRef::Bool),
            Value::I32(value) => (i32_literal(*value), TypeRef::I32),
            Value::I64(value) => (i64_literal(*value), TypeRef::I64),
            Value::F64(value) => (
                format!("poly_f64_from_bits(UINT64_C(0x{:016x}))", value.0),
                TypeRef::F64,
            ),
            Value::Char(value) => (format!("UINT32_C({})", u32::from(*value)), TypeRef::Char),
            Value::String(value) => (string_view(value.as_bytes()), TypeRef::String),
            Value::Bytes(value) => (bytes_view(value), TypeRef::Bytes),
            _ => return self.unsupported("aggregate literal escaped validation"),
        };
        Ok(CExpression {
            prelude: String::new(),
            value,
            ty,
        })
    }

    pub(crate) fn tests(&self) -> Result<String, BackendError> {
        let mut output = String::from(
            "#include \"generated.h\"\n\n#include <string.h>\n\nint main(void) {\n  poly_allocator allocator = poly_default_allocator();\n",
        );
        let mut index = 0usize;
        for declaration in &self.program.module().declarations {
            let Declaration::Test(test) = declaration else {
                continue;
            };
            output.push_str(&format!("  /* {} */\n  {{\n", test.header.name));
            let (call, return_type, cleanups) = match &test.invocation {
                TestInvocation::Function {
                    function,
                    arguments,
                } => {
                    let function = match self.declarations.get(function) {
                        Some(Declaration::Function(value)) => value,
                        _ => return self.unsupported("portable test function is missing"),
                    };
                    let mut rendered = Vec::new();
                    let mut cleanups = Vec::new();
                    for (argument_index, argument) in arguments.iter().enumerate() {
                        rendered.push(self.test_value(
                            &mut output,
                            argument,
                            &function.parameters[argument_index].ty,
                            index,
                            argument_index,
                            &mut cleanups,
                        )?);
                    }
                    let suffix = if rendered.is_empty() {
                        String::new()
                    } else {
                        format!(", {}", rendered.join(", "))
                    };
                    (
                        format!(
                            "{}_{}(allocator{suffix})",
                            self.prefix,
                            value_name(&function.header.name)
                        ),
                        function.return_type.clone(),
                        cleanups,
                    )
                }
                TestInvocation::Method { .. } => {
                    return self.unsupported("direct method portable tests are not emitted yet");
                }
            };
            let result_ty = self.result_ty(&return_type);
            output.push_str(&format!("    {result_ty} result = {call};\n"));
            match &test.expected {
                ExpectedOutcome::Value(expected) => {
                    output.push_str("    if (!result.ok");
                    match (self.resolve_alias(&return_type), &expected.value) {
                        (TypeRef::Bool, Value::Bool(value)) => {
                            output.push_str(&format!(" || result.value != {value}"));
                        }
                        (TypeRef::I32, Value::I32(value)) => {
                            output
                                .push_str(&format!(" || result.value != {}", i32_literal(*value)));
                        }
                        (TypeRef::I64, Value::I64(value)) => {
                            output
                                .push_str(&format!(" || result.value != {}", i64_literal(*value)));
                        }
                        (TypeRef::String, Value::String(value)) => {
                            output.push_str(&format!(
                                " || !poly_string_equal(poly_string_borrow(&result.value), {})",
                                string_view(value.as_bytes())
                            ));
                        }
                        _ => {
                            return self
                                .unsupported("this portable-test expectation is not emitted yet");
                        }
                    }
                    output.push_str(&format!(") {{ return {}; }}\n", 10 + index));
                }
                ExpectedOutcome::Error(_) => {
                    output.push_str(&format!(
                        "    if (result.ok) {{ return {}; }}\n",
                        10 + index
                    ));
                }
            }
            match self.resolve_alias(&return_type) {
                TypeRef::String => output.push_str(&format!(
                    "    {}_string_result_drop(&result);\n",
                    self.prefix
                )),
                TypeRef::Bytes => output.push_str(&format!(
                    "    {}_bytes_result_drop(&result);\n",
                    self.prefix
                )),
                _ => {}
            }
            for cleanup in cleanups.iter().rev() {
                output.push_str(&format!("    {cleanup}\n"));
            }
            output.push_str("  }\n");
            index += 1;
        }
        output.push_str("  return 0;\n}\n");
        Ok(output)
    }

    fn test_value(
        &self,
        output: &mut String,
        argument: &TypedValue,
        parameter_type: &TypeRef,
        test_index: usize,
        argument_index: usize,
        cleanups: &mut Vec<String>,
    ) -> Result<String, BackendError> {
        match (&argument.value, self.resolve_alias(&argument.ty)) {
            (Value::String(value), TypeRef::String) => Ok(string_view(value.as_bytes())),
            (Value::Bool(value), TypeRef::Bool) => Ok(value.to_string()),
            (Value::I32(value), TypeRef::I32) => Ok(i32_literal(*value)),
            (Value::I64(value), TypeRef::I64) => Ok(i64_literal(*value)),
            (
                Value::Record {
                    declaration,
                    fields,
                },
                TypeRef::Named(_),
            ) => {
                let record = match self.declarations.get(declaration) {
                    Some(Declaration::Record(value)) => value,
                    _ => return self.unsupported("portable test record is missing"),
                };
                let variable = format!("argument_{test_index}_{argument_index}");
                let record_ty = self.record_name(*declaration);
                output.push_str(&format!("    {record_ty} {variable} = {{0}};\n"));
                for field in fields {
                    let declaration = record
                        .fields
                        .iter()
                        .find(|candidate| candidate.header.node.id == field.field)
                        .expect("checked test field exists");
                    let name = value_name(&declaration.header.name);
                    match (&field.value, self.resolve_alias(&declaration.ty)) {
                        (Value::String(value), TypeRef::String) => output.push_str(&format!(
                            "    if (poly_string_clone(allocator, {}, &{variable}.{name}) != POLY_OK) {{ return {}; }}\n",
                            string_view(value.as_bytes()),
                            100 + test_index
                        )),
                        (Value::I64(value), TypeRef::I64) => output.push_str(&format!(
                            "    {variable}.{name} = {};\n",
                            i64_literal(*value)
                        )),
                        (Value::I32(value), TypeRef::I32) => output.push_str(&format!(
                            "    {variable}.{name} = {};\n",
                            i32_literal(*value)
                        )),
                        (Value::Bool(value), TypeRef::Bool) => output.push_str(&format!(
                            "    {variable}.{name} = {value};\n"
                        )),
                        _ => {
                            return self.unsupported(
                                "nested portable-test record fields are not emitted yet",
                            );
                        }
                    }
                }
                cleanups.push(format!("{record_ty}_drop(&{variable});"));
                if let TypeRef::Contract(contract) = self.resolve_alias(parameter_type) {
                    Ok(format!(
                        "{}_{}_as_{}(&{variable})",
                        self.prefix,
                        type_name(&record.header.name),
                        type_name(self.declaration_name(contract))
                    ))
                } else {
                    Ok(format!("&{variable}"))
                }
            }
            _ => self.unsupported("this portable-test argument is not emitted yet"),
        }
    }
}

fn is_scalar_value(value: &Value) -> bool {
    matches!(
        value,
        Value::Unit
            | Value::Bool(_)
            | Value::I32(_)
            | Value::I64(_)
            | Value::F64(_)
            | Value::Char(_)
            | Value::String(_)
            | Value::Bytes(_)
    )
}

trait Pipe: Sized {
    fn pipe<T>(self, function: impl FnOnce(Self) -> T) -> T {
        function(self)
    }
}
impl<T> Pipe for T {}

fn argument_names(parameters: &[Parameter]) -> String {
    let text = parameters
        .iter()
        .map(|parameter| value_name(&parameter.header.name))
        .collect::<Vec<_>>()
        .join(", ");
    if text.is_empty() {
        text
    } else {
        format!(", {text}")
    }
}

fn identifier(name: &str) -> String {
    let mut output = String::new();
    for (index, character) in name.chars().enumerate() {
        if character.is_ascii_alphanumeric() || character == '_' {
            if index == 0 && character.is_ascii_digit() {
                output.push('_');
            }
            output.push(character);
        } else {
            output.push('_');
        }
    }
    if output.is_empty() || C_KEYWORDS.contains(&output.as_str()) {
        output.push('_');
    }
    output
}

fn type_name(name: &str) -> String {
    identifier(name)
}

fn value_name(name: &str) -> String {
    identifier(name)
}

fn i32_literal(value: i32) -> String {
    if value == i32::MIN {
        "(-INT32_C(2147483647) - INT32_C(1))".into()
    } else {
        format!("INT32_C({value})")
    }
}

fn i64_literal(value: i64) -> String {
    if value == i64::MIN {
        "(-INT64_C(9223372036854775807) - INT64_C(1))".into()
    } else {
        format!("INT64_C({value})")
    }
}

fn string_view(bytes: &[u8]) -> String {
    view_literal("poly_string_view", bytes)
}

fn bytes_view(bytes: &[u8]) -> String {
    view_literal("poly_bytes_view", bytes)
}

fn view_literal(ty: &str, bytes: &[u8]) -> String {
    if bytes.is_empty() {
        format!("({ty}){{NULL, 0U}}")
    } else {
        format!(
            "({ty}){{(const uint8_t[]){{{}}}, {}U}}",
            bytes
                .iter()
                .map(|byte| format!("UINT8_C(0x{byte:02x})"))
                .collect::<Vec<_>>()
                .join(", "),
            bytes.len()
        )
    }
}

fn indent(value: &str, spaces: usize) -> String {
    let prefix = " ".repeat(spaces);
    value
        .lines()
        .map(|line| format!("{prefix}{line}\n"))
        .collect()
}

const C_KEYWORDS: &[&str] = &[
    "auto",
    "break",
    "case",
    "char",
    "const",
    "continue",
    "default",
    "do",
    "double",
    "else",
    "enum",
    "extern",
    "float",
    "for",
    "goto",
    "if",
    "inline",
    "int",
    "long",
    "register",
    "restrict",
    "return",
    "short",
    "signed",
    "sizeof",
    "static",
    "struct",
    "switch",
    "typedef",
    "union",
    "unsigned",
    "void",
    "volatile",
    "while",
    "_Alignas",
    "_Alignof",
    "_Atomic",
    "_Bool",
    "_Complex",
    "_Generic",
    "_Imaginary",
    "_Noreturn",
    "_Static_assert",
    "_Thread_local",
];
