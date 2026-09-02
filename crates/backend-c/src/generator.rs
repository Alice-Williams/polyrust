use std::collections::{BTreeMap, BTreeSet};

use portable_check::v0::CheckedProgram;
use portable_codegen::BackendError;
use portable_ir::v0::{
    Block, ConstantExpression, Declaration, ExpectedOutcome, Expression, ExpressionField,
    Intrinsic, MethodDispatch, NodeId, Parameter, TestInvocation, TypeRef, TypedValue, Value,
};

use super::CCode;

pub(crate) struct Generator<'a> {
    program: &'a CheckedProgram,
    prefix: String,
    declarations: BTreeMap<NodeId, &'a Declaration>,
}

#[derive(Clone)]
enum AbiShape {
    Record(NodeId),
    Enum(NodeId),
    Composite(TypeRef),
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
                            self.validate_callable_type(&parameter.ty)?;
                        }
                        self.validate_callable_type(&method.return_type)?;
                        self.validate_block(&method.body)?;
                    }
                }
                Declaration::Function(item) => {
                    for parameter in &item.parameters {
                        self.validate_callable_type(&parameter.ty)?;
                    }
                    self.validate_callable_type(&item.return_type)?;
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
                            self.validate_callable_type(&parameter.ty)?;
                        }
                        self.validate_callable_type(&method.return_type)?;
                    }
                }
                Declaration::Test(_) => {}
                Declaration::Enum(item) => {
                    for variant in &item.variants {
                        for field in &variant.fields {
                            self.validate_type(&field.ty)?;
                        }
                    }
                }
            }
        }
        self.definition_order()?;
        Ok(())
    }

    fn validate_type(&self, ty: &TypeRef) -> Result<(), BackendError> {
        match self.resolve_alias(ty) {
            TypeRef::List(inner) | TypeRef::Option(inner) => self.validate_type(&inner),
            TypeRef::Result { ok, error } => {
                self.validate_type(&ok)?;
                self.validate_type(&error)
            }
            TypeRef::Named(id) if !self.is_record(id) && !self.is_enum(id) => {
                self.unsupported("named type does not resolve to a record, enum, or scalar alias")
            }
            _ => Ok(()),
        }
    }

    fn validate_callable_type(&self, ty: &TypeRef) -> Result<(), BackendError> {
        self.validate_type(ty)?;
        match self.resolve_alias(ty) {
            TypeRef::List(inner) if *inner == TypeRef::String => Ok(()),
            TypeRef::Option(inner) if *inner == TypeRef::I64 => Ok(()),
            TypeRef::List(_) | TypeRef::Option(_) | TypeRef::Result { .. } => self.unsupported(
                "callable container lowering currently admits List<String> and Option<I64>",
            ),
            TypeRef::Named(id) if self.is_enum(id) => self
                .unsupported("enum ABI is defined, but callable enum lowering is not complete yet"),
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
            Expression::ConstructRecord { fields, .. } => {
                for field in fields {
                    self.validate_expression(&field.value)?;
                }
                Ok(())
            }
            Expression::ConstructList {
                element_type,
                elements,
                ..
            } => {
                if self.resolve_alias(element_type) != TypeRef::String {
                    return self.unsupported(
                        "list construction currently supports String elements in C17",
                    );
                }
                for element in elements {
                    self.validate_expression(element)?;
                }
                Ok(())
            }
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
                        | Intrinsic::FloatNeg
                        | Intrinsic::FloatTrunc
                        | Intrinsic::FloatIsNaN
                        | Intrinsic::FloatIsNegativeZero
                        | Intrinsic::FloatAbs
                        | Intrinsic::FloatAdd
                        | Intrinsic::FloatSub
                        | Intrinsic::FloatMul
                        | Intrinsic::FloatDiv
                        | Intrinsic::FloatRemTrunc
                        | Intrinsic::StringConcat
                        | Intrinsic::StringUtf16Length
                        | Intrinsic::StringIndexOfLiteral
                        | Intrinsic::StringSliceScalars
                        | Intrinsic::StringIsEmpty
                        | Intrinsic::StringContains
                        | Intrinsic::StringStartsWith
                        | Intrinsic::StringStripPrefix
                        | Intrinsic::StringEndsWith
                        | Intrinsic::StringReplaceAll
                        | Intrinsic::StringReplaceMany
                        | Intrinsic::StringTruncateUtf8Bytes
                        | Intrinsic::StringTrimStart
                        | Intrinsic::StringTrimEnd
                        | Intrinsic::BytesReplaceAll
                        | Intrinsic::ListIndexOf
                        | Intrinsic::OptionIsSome
                        | Intrinsic::OptionIsNone
                        | Intrinsic::OptionUnwrapOr
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

    pub(crate) fn header(&self) -> CCode {
        let fundamental_results = CCode::new(format!(
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
        ))
        .with_system("stdbool.h")
        .with_system("stdint.h")
        .with_helper_root("runtime.core");
        let declarations = &self.program.module().declarations;
        CCode::sequence([
            fundamental_results,
            CCode::sequence(
                declarations
                    .iter()
                    .map(|declaration| self.forward_declaration(declaration)),
            ),
            CCode::sequence(self.composite_types().values().map(|ty| {
                let name = self.ty(ty);
                CCode::new(format!("typedef struct {name} {name};\n")).with_text_from([name])
            })),
            CCode::new("\n"),
            CCode::sequence(
                self.definition_order()
                    .expect("C ABI graph was validated")
                    .iter()
                    .map(|shape| self.shape_header(shape)),
            ),
            CCode::sequence(
                declarations
                    .iter()
                    .map(|declaration| self.alias_declaration(declaration)),
            ),
            CCode::sequence(
                declarations
                    .iter()
                    .map(|declaration| self.contract_declaration(declaration)),
            ),
            CCode::sequence(
                declarations
                    .iter()
                    .map(|declaration| self.implementation_declaration(declaration)),
            ),
            CCode::new("\n"),
            CCode::sequence(
                declarations
                    .iter()
                    .map(|declaration| self.callable_declaration(declaration)),
            ),
        ])
    }

    fn forward_declaration(&self, declaration: &Declaration) -> CCode {
        match declaration {
            Declaration::Record(item) => {
                let name = self.named_name(item.header.node.id);
                CCode::new(format!("typedef struct {name} {name};\n"))
            }
            Declaration::Enum(item) => {
                let name = self.named_name(item.header.node.id);
                CCode::new(format!("typedef struct {name} {name};\n"))
            }
            _ => CCode::default(),
        }
    }

    fn alias_declaration(&self, declaration: &Declaration) -> CCode {
        let Declaration::Alias(item) = declaration else {
            return CCode::default();
        };
        let target = self.ty(&item.target);
        CCode::new(format!(
            "typedef {target} {}_{};\n",
            self.prefix,
            type_name(&item.header.name)
        ))
        .with_text_from([target])
    }

    fn contract_declaration(&self, declaration: &Declaration) -> CCode {
        let Declaration::Contract(item) = declaration else {
            return CCode::default();
        };
        let contract = self.contract_name(item.header.node.id);
        let mut output = format!(
            "typedef struct {contract} {contract};\ntypedef struct {contract}_vtable {contract}_vtable;\nstruct {contract} {{ const void *context; const {contract}_vtable *vtable; }};\nstruct {contract}_vtable {{\n"
        );
        let mut dependencies = Vec::new();
        for method in &item.methods {
            let result = self.result_ty(&method.return_type);
            let parameters = self.parameters(&method.parameters);
            output.push_str(&format!(
                "  {result} (*{})(poly_allocator allocator, const void *context{parameters});\n",
                value_name(&method.header.name)
            ));
            dependencies.extend([result, parameters]);
        }
        output.push_str("};\n\n");
        CCode::new(output)
            .with_helper_root("runtime.core")
            .with_text_from(dependencies)
    }

    fn implementation_declaration(&self, declaration: &Declaration) -> CCode {
        let Declaration::Implementation(item) = declaration else {
            return CCode::default();
        };
        CCode::new(format!(
            "{} {}_{}_as_{}(const {} *value);\n",
            self.contract_name(item.contract),
            self.prefix,
            type_name(self.declaration_name(item.record)),
            type_name(self.declaration_name(item.contract)),
            self.record_name(item.record)
        ))
    }

    fn callable_declaration(&self, declaration: &Declaration) -> CCode {
        match declaration {
            Declaration::Constant(item) => {
                let result = self.result_ty(&item.ty);
                CCode::new(format!(
                    "{result} {}_{}(poly_allocator allocator);\n",
                    self.prefix,
                    value_name(&item.header.name)
                ))
                .with_helper_root("runtime.core")
                .with_text_from([result])
            }
            Declaration::Function(item) => {
                let result = self.result_ty(&item.return_type);
                let parameters = self.parameters(&item.parameters);
                CCode::new(format!(
                    "{result} {}_{}(poly_allocator allocator{parameters});\n",
                    self.prefix,
                    value_name(&item.header.name)
                ))
                .with_helper_root("runtime.core")
                .with_text_from([result, parameters])
            }
            _ => CCode::default(),
        }
    }

    pub(crate) fn header_guard(&self) -> String {
        format!("{}_GENERATED_H", self.prefix.to_ascii_uppercase())
    }

    fn parameters(&self, parameters: &[Parameter]) -> CCode {
        CCode::joined(
            parameters.iter().map(|parameter| {
                self.parameter_ty(&parameter.ty)
                    .map_text(|ty| format!("{ty} {}", value_name(&parameter.header.name)))
            }),
            ", ",
        )
        .map_text(|text| {
            if text.is_empty() {
                text
            } else {
                format!(", {text}")
            }
        })
    }

    fn parameter_ty(&self, ty: &TypeRef) -> CCode {
        match self.resolve_alias(ty) {
            TypeRef::String => CCode::new("poly_string_view").with_helper_root("runtime.core"),
            TypeRef::Bytes => CCode::new("poly_bytes_view").with_helper_root("runtime.core"),
            TypeRef::Named(id) if self.is_record(id) => {
                CCode::new(format!("const {} *", self.record_name(id)))
            }
            TypeRef::Contract(id) => CCode::new(self.contract_name(id)),
            TypeRef::List(_) => self.ty(ty).map_text(|ty| format!("const {ty} *")),
            other => self.ty(&other),
        }
    }

    pub(crate) fn ty(&self, ty: &TypeRef) -> CCode {
        match self.resolve_alias(ty) {
            TypeRef::Unit => CCode::new(format!("{}_unit", self.prefix)),
            TypeRef::Bool => CCode::new("bool").with_system("stdbool.h"),
            TypeRef::I32 => CCode::new("int32_t").with_system("stdint.h"),
            TypeRef::I64 => CCode::new("int64_t").with_system("stdint.h"),
            TypeRef::F64 => CCode::new("double"),
            TypeRef::Char => CCode::new("uint32_t").with_system("stdint.h"),
            TypeRef::String => CCode::new("poly_string").with_helper_root("runtime.core"),
            TypeRef::Bytes => CCode::new("poly_bytes").with_helper_root("runtime.core"),
            TypeRef::Named(id) => CCode::new(self.named_name(id)),
            TypeRef::Contract(id) => CCode::new(self.contract_name(id)),
            TypeRef::List(_) | TypeRef::Option(_) | TypeRef::Result { .. } => CCode::new(format!(
                "{}_{}",
                self.prefix,
                self.shape_key(&self.resolve_alias(ty))
            )),
        }
    }

    fn result_ty(&self, ty: &TypeRef) -> CCode {
        let resolved = self.resolve_alias(ty);
        if let TypeRef::Named(id) = resolved
            && (self.is_record(id) || self.is_enum(id))
        {
            return CCode::new(format!("{}_call_result", self.named_name(id)));
        }
        if matches!(
            &resolved,
            TypeRef::List(_) | TypeRef::Option(_) | TypeRef::Result { .. }
        ) {
            return self
                .ty(&resolved)
                .map_text(|ty| format!("{ty}_call_result"));
        }
        let suffix = match resolved {
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
        CCode::new(format!("{}_{suffix}_result", self.prefix)).with_helper_root("runtime.core")
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
        self.named_name(id)
    }

    fn named_name(&self, id: NodeId) -> String {
        format!("{}_{}", self.prefix, type_name(self.declaration_name(id)))
    }

    fn contract_name(&self, id: NodeId) -> String {
        format!("{}_{}", self.prefix, type_name(self.declaration_name(id)))
    }

    fn is_record(&self, id: NodeId) -> bool {
        matches!(self.declarations.get(&id), Some(Declaration::Record(_)))
    }

    fn is_enum(&self, id: NodeId) -> bool {
        matches!(self.declarations.get(&id), Some(Declaration::Enum(_)))
    }

    fn shape_key(&self, ty: &TypeRef) -> String {
        match self.resolve_alias(ty) {
            TypeRef::Unit => "unit".into(),
            TypeRef::Bool => "bool".into(),
            TypeRef::I32 => "i32".into(),
            TypeRef::I64 => "i64".into(),
            TypeRef::F64 => "f64".into(),
            TypeRef::Char => "char".into(),
            TypeRef::String => "string".into(),
            TypeRef::Bytes => "bytes".into(),
            TypeRef::Named(id) => format!("named_{}", id.0),
            TypeRef::Contract(id) => format!("contract_{}", id.0),
            TypeRef::List(inner) => format!("list__{}", self.shape_key(&inner)),
            TypeRef::Option(inner) => format!("option__{}", self.shape_key(&inner)),
            TypeRef::Result { ok, error } => format!(
                "result__{}__{}",
                self.shape_key(&ok),
                self.shape_key(&error)
            ),
        }
    }

    fn composite_types(&self) -> BTreeMap<String, TypeRef> {
        let mut output = BTreeMap::new();
        for declaration in &self.program.module().declarations {
            match declaration {
                Declaration::Constant(item) => self.collect_composites(&item.ty, &mut output),
                Declaration::Alias(item) => self.collect_composites(&item.target, &mut output),
                Declaration::Record(item) => {
                    for field in &item.fields {
                        self.collect_composites(&field.ty, &mut output);
                    }
                }
                Declaration::Enum(item) => {
                    for variant in &item.variants {
                        for field in &variant.fields {
                            self.collect_composites(&field.ty, &mut output);
                        }
                    }
                }
                Declaration::Contract(item) => {
                    for method in &item.methods {
                        for parameter in &method.parameters {
                            self.collect_composites(&parameter.ty, &mut output);
                        }
                        self.collect_composites(&method.return_type, &mut output);
                    }
                }
                Declaration::Implementation(item) => {
                    for method in &item.methods {
                        for parameter in &method.parameters {
                            self.collect_composites(&parameter.ty, &mut output);
                        }
                        self.collect_composites(&method.return_type, &mut output);
                    }
                }
                Declaration::Function(item) => {
                    for parameter in &item.parameters {
                        self.collect_composites(&parameter.ty, &mut output);
                    }
                    self.collect_composites(&item.return_type, &mut output);
                }
                Declaration::Test(_) => {}
            }
        }
        for (_, ty) in self.program.expression_types() {
            self.collect_composites(ty, &mut output);
        }
        output
    }

    fn collect_composites(&self, ty: &TypeRef, output: &mut BTreeMap<String, TypeRef>) {
        match self.resolve_alias(ty) {
            TypeRef::List(inner) => {
                self.collect_composites(&inner, output);
                let ty = TypeRef::List(inner);
                output.insert(self.shape_key(&ty), ty);
            }
            TypeRef::Option(inner) => {
                self.collect_composites(&inner, output);
                let ty = TypeRef::Option(inner);
                output.insert(self.shape_key(&ty), ty);
            }
            TypeRef::Result { ok, error } => {
                self.collect_composites(&ok, output);
                self.collect_composites(&error, output);
                let ty = TypeRef::Result { ok, error };
                output.insert(self.shape_key(&ty), ty);
            }
            _ => {}
        }
    }

    fn definition_order(&self) -> Result<Vec<AbiShape>, BackendError> {
        let mut pending = BTreeMap::new();
        for declaration in &self.program.module().declarations {
            match declaration {
                Declaration::Record(item) => {
                    pending.insert(
                        self.named_name(item.header.node.id),
                        AbiShape::Record(item.header.node.id),
                    );
                }
                Declaration::Enum(item) => {
                    pending.insert(
                        self.named_name(item.header.node.id),
                        AbiShape::Enum(item.header.node.id),
                    );
                }
                _ => {}
            }
        }
        for ty in self.composite_types().into_values() {
            pending.insert(self.ty(&ty).text, AbiShape::Composite(ty));
        }
        let mut defined = BTreeSet::new();
        let mut output = Vec::new();
        while !pending.is_empty() {
            let ready = pending
                .iter()
                .find(|(_, shape)| {
                    self.shape_dependencies(shape)
                        .iter()
                        .all(|dependency| defined.contains(dependency))
                })
                .map(|(name, _)| name.clone());
            let Some(name) = ready else {
                return self
                    .unsupported("owned type definitions contain an irreducible by-value cycle");
            };
            let shape = pending.remove(&name).expect("ready ABI shape");
            defined.insert(name);
            output.push(shape);
        }
        Ok(output)
    }

    fn shape_dependencies(&self, shape: &AbiShape) -> BTreeSet<String> {
        let mut output = BTreeSet::new();
        match shape {
            AbiShape::Record(id) => {
                if let Some(Declaration::Record(item)) = self.declarations.get(id) {
                    for field in &item.fields {
                        self.add_definition_dependency(&field.ty, &mut output);
                    }
                }
            }
            AbiShape::Enum(id) => {
                if let Some(Declaration::Enum(item)) = self.declarations.get(id) {
                    for variant in &item.variants {
                        for field in &variant.fields {
                            self.add_definition_dependency(&field.ty, &mut output);
                        }
                    }
                }
            }
            AbiShape::Composite(TypeRef::List(_)) => {}
            AbiShape::Composite(TypeRef::Option(inner)) => {
                self.add_definition_dependency(inner, &mut output);
            }
            AbiShape::Composite(TypeRef::Result { ok, error }) => {
                self.add_definition_dependency(ok, &mut output);
                self.add_definition_dependency(error, &mut output);
            }
            AbiShape::Composite(_) => {}
        }
        output
    }

    fn add_definition_dependency(&self, ty: &TypeRef, output: &mut BTreeSet<String>) {
        match self.resolve_alias(ty) {
            TypeRef::Named(id) if self.is_record(id) || self.is_enum(id) => {
                output.insert(self.named_name(id));
            }
            TypeRef::List(_) | TypeRef::Option(_) | TypeRef::Result { .. } => {
                output.insert(self.ty(ty).text);
            }
            _ => {}
        }
    }

    fn shape_header(&self, shape: &AbiShape) -> CCode {
        match shape {
            AbiShape::Record(id) => self.record_shape_header(*id),
            AbiShape::Enum(id) => self.enum_shape_header(*id),
            AbiShape::Composite(ty) => self.composite_shape_header(ty),
        }
    }

    fn owned_shape_footer(&self, name: &str) -> CCode {
        CCode::new(format!(
            "bool {name}_clone(poly_allocator allocator, const {name} *source, {name} *output);\nvoid {name}_drop({name} *value);\ntypedef struct {name}_call_result {{ bool ok; {name} value; poly_error error; }} {name}_call_result;\nvoid {name}_call_result_drop({name}_call_result *value);\n\n"
        ))
        .with_system("stdbool.h")
        .with_helper_root("runtime.core")
    }

    fn record_shape_header(&self, id: NodeId) -> CCode {
        let Some(Declaration::Record(item)) = self.declarations.get(&id) else {
            return CCode::default();
        };
        let name = self.named_name(id);
        let mut output = format!("struct {name} {{\n");
        let mut dependencies = Vec::new();
        for field in &item.fields {
            let ty = self.ty(&field.ty);
            output.push_str(&format!("  {ty} {};\n", value_name(&field.header.name)));
            dependencies.push(ty);
        }
        output.push_str("};\n");
        CCode::sequence([
            CCode::new(output).with_text_from(dependencies),
            self.owned_shape_footer(&name),
        ])
    }

    fn enum_shape_header(&self, id: NodeId) -> CCode {
        let Some(Declaration::Enum(item)) = self.declarations.get(&id) else {
            return CCode::default();
        };
        let name = self.named_name(id);
        let tag = format!("{name}_tag");
        let mut output = format!("typedef enum {tag} {{\n");
        let mut dependencies = Vec::new();
        for (index, variant) in item.variants.iter().enumerate() {
            output.push_str(&format!(
                "  {}_{} = {},\n",
                name.to_ascii_uppercase(),
                type_name(&variant.header.name).to_ascii_uppercase(),
                index
            ));
        }
        output.push_str(&format!(
            "}} {tag};\nstruct {name} {{\n  {tag} tag;\n  union {{\n"
        ));
        for variant in &item.variants {
            output.push_str("    struct {\n");
            if variant.fields.is_empty() {
                output.push_str("      uint8_t unused;\n");
            }
            for field in &variant.fields {
                let ty = self.ty(&field.ty);
                output.push_str(&format!("      {ty} {};\n", value_name(&field.header.name)));
                dependencies.push(ty);
            }
            output.push_str(&format!("    }} {};\n", value_name(&variant.header.name)));
        }
        output.push_str("  } payload;\n};\n");
        CCode::sequence([
            CCode::new(output)
                .with_system("stdint.h")
                .with_text_from(dependencies),
            self.owned_shape_footer(&name),
        ])
    }

    pub(crate) fn composite_shape_header(&self, ty: &TypeRef) -> CCode {
        let name = self.ty(ty);
        let body = match self.resolve_alias(ty) {
            TypeRef::List(inner) => {
                let inner = self.ty(&inner);
                CCode::new(format!(
                    "struct {name} {{ {inner} *data; size_t length; size_t capacity; poly_allocator allocator; }};\n"
                ))
                .with_system("stddef.h")
                .with_helper_root("runtime.core")
                .with_text_from([name.clone(), inner])
            }
            TypeRef::Option(inner) => {
                let inner = self.ty(&inner);
                CCode::new(format!(
                    "struct {name} {{ bool has_value; union {{ {inner} value; }} payload; }};\n"
                ))
                .with_system("stdbool.h")
                .with_text_from([name.clone(), inner])
            }
            TypeRef::Result { ok, error } => {
                let ok = self.ty(&ok);
                let error = self.ty(&error);
                CCode::new(format!(
                    "struct {name} {{ bool is_ok; union {{ {ok} ok; {error} error; }} payload; }};\n"
                ))
                .with_system("stdbool.h")
                .with_text_from([name.clone(), ok, error])
            }
            _ => CCode::default(),
        };
        CCode::sequence([body, self.owned_shape_footer(&name.text)])
    }

    pub(crate) fn source(&self) -> Result<CCode, BackendError> {
        let result_drop = CCode::new(format!(
            "void {0}_string_result_drop({0}_string_result *value) {{\n  if (value != NULL) {{ poly_string_drop(&value->value); *value = ({0}_string_result){{0}}; }}\n}}\n\
             void {0}_bytes_result_drop({0}_bytes_result *value) {{\n  if (value != NULL) {{ poly_bytes_drop(&value->value); *value = ({0}_bytes_result){{0}}; }}\n}}\n\n",
            self.prefix
        ))
        .with_helper_root("runtime.core");
        let shapes = CCode::sequence(
            self.definition_order()
                .expect("C ABI graph was validated")
                .iter()
                .map(|shape| self.shape_functions(shape)),
        );
        let declarations = &self.program.module().declarations;
        let forwards = CCode::sequence(
            declarations
                .iter()
                .map(|declaration| self.implementation_forward(declaration)),
        );
        let mut definitions = Vec::new();
        for declaration in declarations {
            match declaration {
                Declaration::Implementation(item) => definitions.push(self.implementation(item)?),
                Declaration::Constant(item) => definitions.push(self.constant_function(item)?),
                Declaration::Function(item) => definitions.push(self.callable(
                    &format!("{}_{}", self.prefix, value_name(&item.header.name)),
                    &item.parameters,
                    &item.return_type,
                    &item.body,
                    None,
                )?),
                _ => {}
            }
        }
        Ok(CCode::sequence([
            result_drop,
            shapes,
            forwards,
            CCode::new("\n"),
            CCode::sequence(definitions),
        ]))
    }

    fn implementation_forward(&self, declaration: &Declaration) -> CCode {
        let Declaration::Implementation(item) = declaration else {
            return CCode::default();
        };
        let record = self.declaration_name(item.record);
        CCode::sequence(item.methods.iter().map(|method| {
            let result = self.result_ty(&method.return_type);
            let parameters = self.parameters(&method.parameters);
            CCode::new(format!(
                "static {result} {}_{}_{}_impl(poly_allocator allocator, const {} *self_value{parameters});\n",
                self.prefix,
                type_name(record),
                value_name(&method.header.name),
                self.record_name(item.record)
            ))
            .with_helper_root("runtime.core")
            .with_text_from([result, parameters])
        }))
    }

    fn shape_functions(&self, shape: &AbiShape) -> CCode {
        match shape {
            AbiShape::Record(id) => match self.declarations.get(id) {
                Some(Declaration::Record(item)) => self.record_functions(item),
                _ => CCode::default(),
            },
            AbiShape::Enum(id) => self.enum_functions(*id),
            AbiShape::Composite(ty) => self.composite_functions(ty),
        }
    }

    fn record_functions(&self, item: &portable_ir::v0::RecordDeclaration) -> CCode {
        let name = self.record_name(item.header.node.id);
        let mut requirements = Vec::new();
        let mut output = format!(
            "bool {name}_clone(poly_allocator allocator, const {name} *source, {name} *output) {{\n  {name} result = {{0}};\n  (void)allocator;\n  if (output == NULL) {{ return false; }}\n  *output = result;\n  if (source == NULL) {{ return false; }}\n"
        );
        for field in &item.fields {
            let field_name = value_name(&field.header.name);
            let clone = self.clone_statement(
                &field.ty,
                &format!("source->{field_name}"),
                &format!("result.{field_name}"),
                &format!("{name}_drop(&result); return false;"),
                2,
            );
            output.push_str(&clone.text);
            requirements.push(clone);
        }
        output.push_str("  *output = result;\n  return true;\n}\n");
        output.push_str(&format!(
            "void {name}_drop({name} *value) {{\n  if (value == NULL) {{ return; }}\n"
        ));
        for field in &item.fields {
            let field_name = value_name(&field.header.name);
            let drop = self.drop_statement(&field.ty, &format!("value->{field_name}"), 2);
            output.push_str(&drop.text);
            requirements.push(drop);
        }
        output.push_str(&format!("  *value = ({name}){{0}};\n}}\n"));
        let call_result_drop = self.call_result_drop_function(&name);
        output.push_str(&call_result_drop.text);
        requirements.push(call_result_drop);
        CCode::new(output)
            .with_helper_root("runtime.core")
            .with_text_from(item.fields.iter().map(|field| self.ty(&field.ty)))
            .with_text_from(requirements)
    }

    fn enum_functions(&self, id: NodeId) -> CCode {
        let Some(Declaration::Enum(item)) = self.declarations.get(&id) else {
            return CCode::default();
        };
        let name = self.named_name(id);
        let mut requirements = Vec::new();
        let mut output = format!(
            "bool {name}_clone(poly_allocator allocator, const {name} *source, {name} *output) {{\n  {name} result = {{0}};\n  if (output == NULL) {{ return false; }}\n  *output = result;\n  if (source == NULL) {{ return false; }}\n  result.tag = source->tag;\n  switch (source->tag) {{\n"
        );
        for variant in &item.variants {
            let variant_name = value_name(&variant.header.name);
            let tag = format!(
                "{}_{}",
                name.to_ascii_uppercase(),
                type_name(&variant.header.name).to_ascii_uppercase()
            );
            output.push_str(&format!("    case {tag}:\n"));
            for field in &variant.fields {
                let field_name = value_name(&field.header.name);
                let clone = self.clone_statement(
                    &field.ty,
                    &format!("source->payload.{variant_name}.{field_name}"),
                    &format!("result.payload.{variant_name}.{field_name}"),
                    &format!("{name}_drop(&result); return false;"),
                    6,
                );
                output.push_str(&clone.text);
                requirements.push(clone);
            }
            output.push_str("      break;\n");
        }
        output
            .push_str("    default: return false;\n  }\n  *output = result;\n  return true;\n}\n");
        output.push_str(&format!(
            "void {name}_drop({name} *value) {{\n  if (value == NULL) {{ return; }}\n  switch (value->tag) {{\n"
        ));
        for variant in &item.variants {
            let variant_name = value_name(&variant.header.name);
            let tag = format!(
                "{}_{}",
                name.to_ascii_uppercase(),
                type_name(&variant.header.name).to_ascii_uppercase()
            );
            output.push_str(&format!("    case {tag}:\n"));
            for field in &variant.fields {
                let field_name = value_name(&field.header.name);
                let drop = self.drop_statement(
                    &field.ty,
                    &format!("value->payload.{variant_name}.{field_name}"),
                    6,
                );
                output.push_str(&drop.text);
                requirements.push(drop);
            }
            output.push_str("      break;\n");
        }
        output.push_str(&format!(
            "    default: break;\n  }}\n  *value = ({name}){{0}};\n}}\n"
        ));
        let call_result_drop = self.call_result_drop_function(&name);
        output.push_str(&call_result_drop.text);
        requirements.push(call_result_drop);
        CCode::new(output)
            .with_helper_root("runtime.core")
            .with_text_from(
                item.variants
                    .iter()
                    .flat_map(|variant| variant.fields.iter())
                    .map(|field| self.ty(&field.ty)),
            )
            .with_text_from(requirements)
    }

    fn composite_functions(&self, ty: &TypeRef) -> CCode {
        let name = self.ty(ty);
        let mut requirements = Vec::new();
        let mut output = match self.resolve_alias(ty) {
            TypeRef::List(inner) => {
                let mut text = format!(
                    "bool {name}_clone(poly_allocator allocator, const {name} *source, {name} *output) {{\n  {name} result = {{0}};\n  size_t index;\n  if (output == NULL) {{ return false; }}\n  *output = result;\n  if (source == NULL) {{ return false; }}\n  result.allocator = allocator;\n  if (source->length != 0U) {{\n    if (source->data == NULL || allocator.allocate == NULL || allocator.deallocate == NULL || source->length > SIZE_MAX / sizeof(*result.data)) {{ return false; }}\n    result.data = allocator.allocate(allocator.context, source->length * sizeof(*result.data));\n    if (result.data == NULL) {{ return false; }}\n    result.capacity = source->length;\n    for (index = 0U; index < source->length; ++index) {{\n"
                );
                let clone = self.clone_statement(
                    &inner,
                    "source->data[index]",
                    "result.data[index]",
                    &format!("result.length = index; {name}_drop(&result); return false;"),
                    6,
                );
                text.push_str(&clone.text);
                requirements.push(clone);
                text.push_str("      result.length = index + 1U;\n    }\n  }\n  *output = result;\n  return true;\n}\n");
                text.push_str(&format!(
                    "void {name}_drop({name} *value) {{\n  size_t index;\n  if (value == NULL) {{ return; }}\n  for (index = 0U; index < value->length; ++index) {{\n"
                ));
                let drop = self.drop_statement(&inner, "value->data[index]", 4);
                text.push_str(&drop.text);
                requirements.push(drop);
                text.push_str(&format!(
                    "  }}\n  if (value->data != NULL && value->allocator.deallocate != NULL) {{ value->allocator.deallocate(value->allocator.context, value->data); }}\n  *value = ({name}){{0}};\n}}\n"
                ));
                text
            }
            TypeRef::Option(inner) => {
                let mut text = format!(
                    "bool {name}_clone(poly_allocator allocator, const {name} *source, {name} *output) {{\n  {name} result = {{0}};\n  (void)allocator;\n  if (output == NULL) {{ return false; }}\n  *output = result;\n  if (source == NULL) {{ return false; }}\n  result.has_value = source->has_value;\n  if (source->has_value) {{\n"
                );
                let clone = self.clone_statement(
                    &inner,
                    "source->payload.value",
                    "result.payload.value",
                    "return false;",
                    4,
                );
                text.push_str(&clone.text);
                requirements.push(clone);
                text.push_str("  }\n  *output = result;\n  return true;\n}\n");
                text.push_str(&format!(
                    "void {name}_drop({name} *value) {{\n  if (value == NULL) {{ return; }}\n  if (value->has_value) {{\n"
                ));
                let drop = self.drop_statement(&inner, "value->payload.value", 4);
                text.push_str(&drop.text);
                requirements.push(drop);
                text.push_str(&format!("  }}\n  *value = ({name}){{0}};\n}}\n"));
                text
            }
            TypeRef::Result { ok, error } => {
                let mut text = format!(
                    "bool {name}_clone(poly_allocator allocator, const {name} *source, {name} *output) {{\n  {name} result = {{0}};\n  if (output == NULL) {{ return false; }}\n  *output = result;\n  if (source == NULL) {{ return false; }}\n  result.is_ok = source->is_ok;\n  if (source->is_ok) {{\n"
                );
                let ok_clone = self.clone_statement(
                    &ok,
                    "source->payload.ok",
                    "result.payload.ok",
                    "return false;",
                    4,
                );
                text.push_str(&ok_clone.text);
                requirements.push(ok_clone);
                text.push_str("  } else {\n");
                let error_clone = self.clone_statement(
                    &error,
                    "source->payload.error",
                    "result.payload.error",
                    "return false;",
                    4,
                );
                text.push_str(&error_clone.text);
                requirements.push(error_clone);
                text.push_str("  }\n  *output = result;\n  return true;\n}\n");
                text.push_str(&format!(
                    "void {name}_drop({name} *value) {{\n  if (value == NULL) {{ return; }}\n  if (value->is_ok) {{\n"
                ));
                let ok_drop = self.drop_statement(&ok, "value->payload.ok", 4);
                text.push_str(&ok_drop.text);
                requirements.push(ok_drop);
                text.push_str("  } else {\n");
                let error_drop = self.drop_statement(&error, "value->payload.error", 4);
                text.push_str(&error_drop.text);
                requirements.push(error_drop);
                text.push_str(&format!("  }}\n  *value = ({name}){{0}};\n}}\n"));
                text
            }
            _ => String::new(),
        };
        let call_result_drop = self.call_result_drop_function(&name.text);
        output.push_str(&call_result_drop.text);
        requirements.push(call_result_drop);
        CCode::new(output)
            .with_helper_root("runtime.core")
            .with_text_from([name])
            .with_text_from(requirements)
    }

    fn clone_statement(
        &self,
        ty: &TypeRef,
        source: &str,
        destination: &str,
        failure: &str,
        spaces: usize,
    ) -> CCode {
        let (statement, runtime) = match self.resolve_alias(ty) {
            TypeRef::String => (
                format!(
                    "if (poly_string_clone(allocator, poly_string_borrow(&{source}), &{destination}) != POLY_OK) {{ {failure} }}"
                ),
                true,
            ),
            TypeRef::Bytes => (
                format!(
                    "if (!poly_bytes_clone(allocator, poly_bytes_borrow(&{source}), &{destination})) {{ {failure} }}"
                ),
                true,
            ),
            TypeRef::Named(id) if self.is_record(id) || self.is_enum(id) => (
                format!(
                    "if (!{}_clone(allocator, &{source}, &{destination})) {{ {failure} }}",
                    self.named_name(id)
                ),
                false,
            ),
            TypeRef::List(_) | TypeRef::Option(_) | TypeRef::Result { .. } => (
                format!(
                    "if (!{}_clone(allocator, &{source}, &{destination})) {{ {failure} }}",
                    self.ty(ty).text
                ),
                false,
            ),
            _ => (format!("{destination} = {source};"), false),
        };
        let code = CCode::new(format!("{}{}\n", " ".repeat(spaces), statement));
        if runtime {
            code.with_helper_root("runtime.core")
        } else {
            code
        }
    }

    fn drop_statement(&self, ty: &TypeRef, value: &str, spaces: usize) -> CCode {
        let (statement, runtime) = match self.resolve_alias(ty) {
            TypeRef::String => (Some(format!("poly_string_drop(&{value});")), true),
            TypeRef::Bytes => (Some(format!("poly_bytes_drop(&{value});")), true),
            TypeRef::Named(id) if self.is_record(id) || self.is_enum(id) => (
                Some(format!("{}_drop(&{value});", self.named_name(id))),
                false,
            ),
            TypeRef::List(_) | TypeRef::Option(_) | TypeRef::Result { .. } => {
                (Some(format!("{}_drop(&{value});", self.ty(ty).text)), false)
            }
            _ => (None, false),
        };
        let code = CCode::new(
            statement
                .map(|statement| format!("{}{}\n", " ".repeat(spaces), statement))
                .unwrap_or_default(),
        );
        if runtime {
            code.with_helper_root("runtime.core")
        } else {
            code
        }
    }

    fn call_result_drop_function(&self, name: &str) -> CCode {
        CCode::new(format!(
            "void {name}_call_result_drop({name}_call_result *value) {{\n  if (value != NULL) {{ {name}_drop(&value->value); *value = ({name}_call_result){{0}}; }}\n}}\n\n"
        ))
        .with_helper_root("runtime.core")
    }

    fn implementation(
        &self,
        item: &portable_ir::v0::ImplementationDeclaration,
    ) -> Result<CCode, BackendError> {
        let record = self.declaration_name(item.record);
        let contract = self.declaration_name(item.contract);
        let record_ty = self.record_name(item.record);
        let contract_ty = self.contract_name(item.contract);
        let mut callables = Vec::new();
        for method in &item.methods {
            callables.push(self.callable(
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
        let mut output = String::new();
        let mut dependencies = Vec::new();
        for signature in &contract_declaration.methods {
            let method = item
                .methods
                .iter()
                .find(|method| method.contract_method == signature.header.node.id)
                .ok_or_else(|| BackendError::Generation {
                    message: "C17 implementation is missing a checked contract method".into(),
                })?;
            let result = self.result_ty(&method.return_type);
            let parameters = self.parameters(&method.parameters);
            output.push_str(&format!(
                "static {result} {}_{}_{}_adapter(poly_allocator allocator, const void *context{parameters}) {{\n  return {}_{}_{}_impl(allocator, (const {record_ty} *)context{});\n}}\n",
                self.prefix,
                type_name(record),
                value_name(&method.header.name),
                self.prefix,
                type_name(record),
                value_name(&method.header.name),
                argument_names(&method.parameters)
            ));
            dependencies.extend([result, parameters]);
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
        Ok(CCode::sequence([
            CCode::sequence(callables),
            CCode::new(output)
                .with_helper_root("runtime.core")
                .with_text_from(dependencies),
        ]))
    }

    fn constant_function(
        &self,
        item: &portable_ir::v0::ConstantDeclaration,
    ) -> Result<CCode, BackendError> {
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
                requirements: CCode::sequence([self.result_ty(&item.ty), self.ty(&item.ty)]),
            },
            _ => return self.unsupported("constant escaped validation"),
        };
        let mut emitter = FunctionEmitter::new(self, &[], false);
        if matches!(
            &item.value,
            ConstantExpression::Literal {
                value: Value::F64(_),
                ..
            }
        ) {
            emitter.require_helper("runtime.feature.f64");
        }
        self.render_callable(
            &format!("{}_{}", self.prefix, value_name(&item.header.name)),
            &[],
            &item.ty,
            expression,
            None,
            emitter,
        )
    }

    fn callable(
        &self,
        name: &str,
        parameters: &[Parameter],
        return_type: &TypeRef,
        block: &Block,
        self_parameter: Option<String>,
    ) -> Result<CCode, BackendError> {
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
    ) -> Result<CCode, BackendError> {
        let mut signature = String::from("poly_allocator allocator");
        if let Some(self_parameter) = self_parameter {
            signature.push_str(", ");
            signature.push_str(&self_parameter);
        }
        let parameter_code = self.parameters(parameters);
        signature.push_str(&parameter_code.text);
        let result_ty = self.result_ty(return_type);
        let expression_requirements = expression.requirements.clone();
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
            TypeRef::List(_) => {
                let list_ty = self.ty(return_type);
                output.push_str(&format!(
                    "  {list_ty} final_value = {{0}};\n  if (!{list_ty}_clone(allocator, {}, &final_value)) {{ error = (poly_error){{POLY_ALLOCATION_FAILED, \"allocation failed\"}}; goto fail; }}\n",
                    expression.value
                ));
            }
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
        let final_type = self.ty(return_type);
        let mut code = CCode::new(output)
            .with_helper_root("runtime.core")
            .with_text_from([
                parameter_code,
                result_ty,
                final_type,
                expression_requirements,
            ]);
        for root in emitter.helper_roots {
            code = code.with_helper_root(root);
        }
        Ok(code)
    }
}

#[derive(Clone)]
struct CExpression {
    prelude: String,
    value: String,
    ty: TypeRef,
    requirements: CCode,
}

struct FunctionEmitter<'generator, 'program> {
    generator: &'generator Generator<'program>,
    locals: BTreeMap<String, CExpression>,
    self_value: Option<CExpression>,
    declarations: Vec<String>,
    cleanups: Vec<String>,
    helper_roots: BTreeSet<String>,
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
                        requirements: generator.parameter_ty(&parameter.ty),
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
                requirements: CCode::default(),
            }),
            declarations: Vec::new(),
            cleanups: Vec::new(),
            helper_roots: BTreeSet::new(),
            next: 0,
        }
    }

    fn require_helper(&mut self, helper: &str) {
        self.helper_roots.insert(helper.to_owned());
    }

    fn temporary(&mut self, declaration: impl FnOnce(&str) -> String) -> String {
        let name = format!("temporary_{}", self.next);
        self.next += 1;
        self.declarations.push(declaration(&name));
        name
    }

    fn expression(&mut self, expression: &Expression) -> Result<CExpression, BackendError> {
        match expression {
            Expression::Literal { value, .. } => {
                if matches!(value, Value::F64(_)) {
                    self.require_helper("runtime.feature.f64");
                }
                self.generator.literal(value)
            }
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
            Expression::ConstructRecord {
                declaration,
                fields,
                ..
            } => self.construct_record(*declaration, fields),
            Expression::ConstructList {
                element_type,
                elements,
                ..
            } => self.construct_list(element_type, elements),
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
                        TypeRef::List(_) => format!("&{access}"),
                        _ => access,
                    },
                    ty: ty.clone(),
                    requirements: CCode::sequence([base.requirements, self.generator.ty(ty)]),
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
        let mut requirements = receiver.requirements;
        let mut values = Vec::new();
        for argument in arguments {
            let argument = self.expression(argument)?;
            prelude.push_str(&argument.prelude);
            requirements = CCode::sequence([requirements, argument.requirements]);
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
        result.requirements = CCode::sequence([requirements, result.requirements]);
        Ok(result)
    }

    fn construct_record(
        &mut self,
        declaration: NodeId,
        fields: &[ExpressionField],
    ) -> Result<CExpression, BackendError> {
        let record_ty = self.generator.record_name(declaration);
        let temporary = self.temporary(|name| format!("{record_ty} {name} = {{0}};"));
        let mut prelude = String::new();
        let mut requirements = CCode::default();
        for field in fields {
            let value = self.expression(&field.value)?;
            prelude.push_str(&value.prelude);
            requirements = CCode::sequence([requirements, value.requirements]);
            let (name, ty) =
                self.generator
                    .field(field.field)
                    .ok_or_else(|| BackendError::Generation {
                        message: "C17 record construction field is missing".into(),
                    })?;
            if !matches!(
                self.generator.resolve_alias(ty),
                TypeRef::Unit
                    | TypeRef::Bool
                    | TypeRef::I32
                    | TypeRef::I64
                    | TypeRef::F64
                    | TypeRef::Char
            ) {
                return self
                    .generator
                    .unsupported("owned record construction fields are not lowered yet");
            }
            prelude.push_str(&format!(
                "{temporary}.{} = {};\n",
                value_name(name),
                value.value
            ));
        }
        Ok(CExpression {
            prelude,
            value: temporary,
            ty: TypeRef::Named(declaration),
            requirements,
        })
    }

    fn construct_list(
        &mut self,
        element_type: &TypeRef,
        elements: &[Expression],
    ) -> Result<CExpression, BackendError> {
        if self.generator.resolve_alias(element_type) != TypeRef::String {
            return self
                .generator
                .unsupported("validated list construction has a non-String element type");
        }
        let ty = TypeRef::List(Box::new(element_type.clone()));
        let list_ty = self.generator.ty(&ty);
        let temporary = self.temporary(|name| format!("{list_ty} {name} = {{0}};"));
        self.cleanups.push(format!("{list_ty}_drop(&{temporary});"));
        let mut requirements = CCode::new("").with_text_from([list_ty.clone()]);
        let mut prelude = format!("{temporary}.allocator = allocator;\n");
        if !elements.is_empty() {
            prelude.push_str(&format!(
                "if (allocator.allocate == NULL || allocator.deallocate == NULL || {0}U > SIZE_MAX / sizeof(*{1}.data)) {{ error = (poly_error){{POLY_ALLOCATION_FAILED, \"allocation failed\"}}; goto fail; }}\n{1}.data = allocator.allocate(allocator.context, {0}U * sizeof(*{1}.data));\nif ({1}.data == NULL) {{ error = (poly_error){{POLY_ALLOCATION_FAILED, \"allocation failed\"}}; goto fail; }}\n{1}.capacity = {0}U;\n",
                elements.len(),
                temporary
            ));
            requirements = requirements.with_system("stdint.h");
        }
        for (index, element) in elements.iter().enumerate() {
            let value = self.expression(element)?;
            prelude.push_str(&value.prelude);
            let status = self.temporary(|name| format!("poly_error_code {name} = POLY_OK;"));
            prelude.push_str(&format!(
                "{status} = poly_string_clone(allocator, {}, &{temporary}.data[{index}]);\nif ({status} != POLY_OK) {{ error = (poly_error){{{status}, {status} == POLY_INVALID_UTF8 ? \"invalid UTF-8\" : \"allocation failed\"}}; goto fail; }}\n{temporary}.length = {}U;\n",
                value.value,
                index + 1
            ));
            requirements = CCode::sequence([requirements, value.requirements]);
        }
        Ok(CExpression {
            prelude,
            value: format!("&{temporary}"),
            ty,
            requirements: requirements.with_helper_root("runtime.core"),
        })
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
                    requirements: CCode::sequence([
                        condition.requirements,
                        then_value.requirements,
                        else_value.requirements,
                    ]),
                })
            }
            TypeRef::List(_) => {
                let list_ty = self.generator.ty(&ty);
                let temporary = self.temporary(|name| format!("{list_ty} {name} = {{0}};"));
                self.cleanups.push(format!("{list_ty}_drop(&{temporary});"));
                let prelude = format!(
                    "{}if ({}) {{\n{}  if (!{list_ty}_clone(allocator, {}, &{temporary})) {{ error = (poly_error){{POLY_ALLOCATION_FAILED, \"allocation failed\"}}; goto fail; }}\n}} else {{\n{}  if (!{list_ty}_clone(allocator, {}, &{temporary})) {{ error = (poly_error){{POLY_ALLOCATION_FAILED, \"allocation failed\"}}; goto fail; }}\n}}\n",
                    condition.prelude,
                    condition.value,
                    indent(&then_value.prelude, 2),
                    then_value.value,
                    indent(&else_value.prelude, 2),
                    else_value.value,
                );
                Ok(CExpression {
                    prelude,
                    value: format!("&{temporary}"),
                    ty,
                    requirements: CCode::sequence([
                        condition.requirements,
                        then_value.requirements,
                        else_value.requirements,
                        list_ty,
                    ]),
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
                    requirements: CCode::sequence([
                        condition.requirements,
                        then_value.requirements,
                        else_value.requirements,
                        c_ty,
                    ]),
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
            TypeRef::List(_) => {
                let list_ty = self.generator.ty(&ty);
                self.cleanups
                    .push(format!("{list_ty}_call_result_drop(&{temporary});"));
                format!("&{temporary}.value")
            }
            _ => format!("{temporary}.value"),
        };
        Ok(CExpression {
            prelude,
            value,
            ty,
            requirements: result_ty,
        })
    }

    fn intrinsic(
        &mut self,
        operation: Intrinsic,
        values: Vec<CExpression>,
        mut prelude: String,
    ) -> Result<CExpression, BackendError> {
        let helper = match operation {
            Intrinsic::FloatTrunc
            | Intrinsic::FloatIsNaN
            | Intrinsic::FloatIsNegativeZero
            | Intrinsic::FloatAbs
            | Intrinsic::FloatRemTrunc => Some("runtime.feature.f64"),
            Intrinsic::StringContains | Intrinsic::StringStartsWith | Intrinsic::StringEndsWith => {
                Some("runtime.feature.string-predicates")
            }
            Intrinsic::StringStripPrefix => Some("runtime.feature.string-strip-prefix"),
            Intrinsic::StringConcat => Some("runtime.feature.string-concat"),
            Intrinsic::StringUtf16Length => Some("runtime.feature.string-utf16-length"),
            Intrinsic::StringIndexOfLiteral => Some("runtime.feature.string-index-of-literal"),
            Intrinsic::StringSliceScalars => Some("runtime.feature.string-slice-scalars"),
            Intrinsic::StringReplaceAll => Some("runtime.feature.string-replace-all"),
            Intrinsic::BytesReplaceAll => Some("runtime.feature.bytes-replace-all"),
            Intrinsic::StringReplaceMany => Some("runtime.feature.string-replace-many"),
            Intrinsic::StringTruncateUtf8Bytes => Some("runtime.feature.string-truncate-utf8"),
            Intrinsic::StringTrimStart | Intrinsic::StringTrimEnd => {
                Some("runtime.feature.string-trim")
            }
            _ => None,
        };
        if let Some(helper) = helper {
            self.require_helper(helper);
        }
        let value = |index: usize| {
            values
                .get(index)
                .map(|item| item.value.as_str())
                .unwrap_or("0")
        };
        let requirements = CCode::sequence(
            values
                .iter()
                .map(|expression| expression.requirements.clone()),
        );
        let scalar = |text: String, ty: TypeRef, prelude: String| CExpression {
            prelude,
            value: text,
            ty,
            requirements: requirements.clone(),
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
            Intrinsic::FloatNeg => scalar(format!("-({})", value(0)), TypeRef::F64, prelude),
            Intrinsic::FloatTrunc => scalar(
                format!("poly_f64_trunc({})", value(0)),
                TypeRef::F64,
                prelude,
            ),
            Intrinsic::FloatIsNaN => scalar(
                format!("poly_f64_is_nan({})", value(0)),
                TypeRef::Bool,
                prelude,
            ),
            Intrinsic::FloatIsNegativeZero => scalar(
                format!("poly_f64_is_negative_zero({})", value(0)),
                TypeRef::Bool,
                prelude,
            ),
            Intrinsic::FloatAbs => {
                scalar(format!("poly_f64_abs({})", value(0)), TypeRef::F64, prelude)
            }
            Intrinsic::FloatAdd
            | Intrinsic::FloatSub
            | Intrinsic::FloatMul
            | Intrinsic::FloatDiv => {
                let operator = match operation {
                    Intrinsic::FloatAdd => "+",
                    Intrinsic::FloatSub => "-",
                    Intrinsic::FloatMul => "*",
                    _ => "/",
                };
                scalar(
                    format!("({}) {operator} ({})", value(0), value(1)),
                    TypeRef::F64,
                    prelude,
                )
            }
            Intrinsic::FloatRemTrunc => scalar(
                format!("poly_f64_rem_trunc({}, {})", value(0), value(1)),
                TypeRef::F64,
                prelude,
            ),
            Intrinsic::StringUtf16Length => {
                let temporary = self.temporary(|name| format!("int64_t {name} = INT64_C(0);"));
                let status = self.temporary(|name| format!("poly_error_code {name} = POLY_OK;"));
                prelude.push_str(&format!(
                    "{status} = poly_string_utf16_length({}, &{temporary});\nif ({status} != POLY_OK) {{ error = (poly_error){{{status}, {status} == POLY_INVALID_UTF8 ? \"invalid UTF-8\" : \"UTF-16 length overflow\"}}; goto fail; }}\n",
                    value(0)
                ));
                scalar(temporary, TypeRef::I64, prelude)
            }
            Intrinsic::StringIndexOfLiteral => {
                let option_ty = TypeRef::Option(Box::new(TypeRef::I64));
                let option = self.generator.ty(&option_ty);
                let temporary = self.temporary(|name| format!("{option} {name} = {{0}};"));
                let status = self.temporary(|name| format!("poly_error_code {name} = POLY_OK;"));
                prelude.push_str(&format!(
                    "{status} = poly_string_index_of_literal({}, {}, &{temporary}.payload.value, &{temporary}.has_value);\nif ({status} != POLY_OK) {{ error = (poly_error){{{status}, {status} == POLY_INVALID_UTF8 ? \"invalid UTF-8\" : \"string index overflow\"}}; goto fail; }}\n",
                    value(0),
                    value(1)
                ));
                let mut result = scalar(temporary, option_ty, prelude);
                result.requirements = CCode::sequence([result.requirements, option]);
                result
            }
            Intrinsic::ListIndexOf => {
                let option_ty = TypeRef::Option(Box::new(TypeRef::I64));
                let option = self.generator.ty(&option_ty);
                let temporary = self.temporary(|name| format!("{option} {name} = {{0}};"));
                let index = self.temporary(|name| format!("size_t {name} = 0U;"));
                let equals = match self.generator.resolve_alias(&values[1].ty) {
                    TypeRef::String => format!(
                        "poly_string_equal(poly_string_borrow(&{}->data[{index}]), {})",
                        value(0),
                        value(1)
                    ),
                    _ => {
                        return self.generator.unsupported(
                            "ListIndexOf C17 lowering currently supports String elements",
                        );
                    }
                };
                prelude.push_str(&format!(
                    "for ({index} = 0U; {index} < {}->length; ++{index}) {{\n  if ({equals}) {{ {temporary}.has_value = true; {temporary}.payload.value = (int64_t){index}; break; }}\n}}\n",
                    value(0)
                ));
                let mut result = scalar(temporary, option_ty, prelude);
                result.requirements = CCode::sequence([result.requirements, option]);
                result
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
            | Intrinsic::StringReplaceMany
            | Intrinsic::StringSliceScalars
            | Intrinsic::StringTruncateUtf8Bytes
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
                    Intrinsic::StringReplaceMany => {
                        let mapping_count = (values.len() - 1) / 2;
                        let needles = self.temporary(|name| {
                            format!("poly_string_view {name}[{mapping_count}] = {{0}};")
                        });
                        let replacements = self.temporary(|name| {
                            format!("poly_string_view {name}[{mapping_count}] = {{0}};")
                        });
                        for index in 0..mapping_count {
                            prelude.push_str(&format!(
                                "{needles}[{index}] = {};\n{replacements}[{index}] = {};\n",
                                value(1 + index * 2),
                                value(2 + index * 2)
                            ));
                        }
                        format!(
                            "poly_string_replace_many(allocator, {}, {needles}, {replacements}, {mapping_count}U, &{temporary})",
                            value(0)
                        )
                    }
                    Intrinsic::StringSliceScalars => format!(
                        "poly_string_slice_scalars(allocator, {}, {}, {}, &{temporary})",
                        value(0),
                        value(1),
                        value(2)
                    ),
                    Intrinsic::StringTruncateUtf8Bytes => format!(
                        "poly_string_truncate_utf8_bytes(allocator, {}, {}, &{temporary})",
                        value(0),
                        value(1)
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
            Intrinsic::BytesReplaceAll => {
                let temporary = self.temporary(|name| format!("poly_bytes {name} = {{0}};"));
                self.cleanups
                    .push(format!("poly_bytes_drop(&{temporary});"));
                let status = self.temporary(|name| format!("poly_error_code {name} = POLY_OK;"));
                prelude.push_str(&format!(
                    "{status} = poly_bytes_replace_all(allocator, {}, {}, {}, &{temporary});\nif ({status} != POLY_OK) {{ error = (poly_error){{{status}, \"allocation failed\"}}; goto fail; }}\n",
                    value(0),
                    value(1),
                    value(2)
                ));
                scalar(
                    format!("poly_bytes_borrow(&{temporary})"),
                    TypeRef::Bytes,
                    prelude,
                )
            }
            Intrinsic::OptionIsSome => {
                scalar(format!("({}).has_value", value(0)), TypeRef::Bool, prelude)
            }
            Intrinsic::OptionIsNone => {
                scalar(format!("!({}).has_value", value(0)), TypeRef::Bool, prelude)
            }
            Intrinsic::OptionUnwrapOr => scalar(
                format!(
                    "({0}).has_value ? ({0}).payload.value : ({1})",
                    value(0),
                    value(1)
                ),
                values[1].ty.clone(),
                prelude,
            ),
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
        let (rendered, ty) = match value {
            Value::Unit => (
                CCode::new(format!("({}_unit){{0}}", self.prefix)),
                TypeRef::Unit,
            ),
            Value::Bool(value) => (CCode::new(value.to_string()), TypeRef::Bool),
            Value::I32(value) => (i32_literal(*value), TypeRef::I32),
            Value::I64(value) => (i64_literal(*value), TypeRef::I64),
            Value::F64(value) => (
                CCode::new(format!("poly_f64_from_bits(UINT64_C(0x{:016x}))", value.0))
                    .with_system("stdint.h")
                    .with_helper_root("runtime.feature.f64"),
                TypeRef::F64,
            ),
            Value::Char(value) => (
                CCode::new(format!("UINT32_C({})", u32::from(*value))).with_system("stdint.h"),
                TypeRef::Char,
            ),
            Value::String(value) => (string_view(value.as_bytes()), TypeRef::String),
            Value::Bytes(value) => (bytes_view(value), TypeRef::Bytes),
            _ => return self.unsupported("aggregate literal escaped validation"),
        };
        let requirements = CCode::sequence([self.ty(&ty), rendered.clone()]);
        Ok(CExpression {
            prelude: String::new(),
            value: rendered.text,
            ty,
            requirements,
        })
    }

    pub(crate) fn tests(&self) -> Result<CCode, BackendError> {
        let mut output = String::from(
            "int main(void) {\n  poly_allocator allocator = poly_default_allocator();\n",
        );
        let mut dependencies = Vec::new();
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
                    let suffix = CCode::joined(rendered, ", ").map_text(|text| {
                        if text.is_empty() {
                            text
                        } else {
                            format!(", {text}")
                        }
                    });
                    let call = CCode::new(format!(
                        "{}_{}(allocator{})",
                        self.prefix,
                        value_name(&function.header.name),
                        suffix.text
                    ))
                    .with_text_from([suffix]);
                    (call, function.return_type.clone(), cleanups)
                }
                TestInvocation::Method { .. } => {
                    return self.unsupported("direct method portable tests are not emitted yet");
                }
            };
            let result_ty = self.result_ty(&return_type);
            output.push_str(&format!("    {} result = {};\n", result_ty.text, call.text));
            dependencies.extend([result_ty, call]);
            match &test.expected {
                ExpectedOutcome::Value(expected) => {
                    let mismatch =
                        self.test_mismatch("result.value", &expected.value, &return_type)?;
                    output.push_str(&format!(
                        "    if (!result.ok || {}) {{ return {}; }}\n",
                        mismatch.text,
                        10 + index
                    ));
                    dependencies.push(mismatch);
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
                TypeRef::List(_) | TypeRef::Option(_) | TypeRef::Result { .. } => {
                    output.push_str(&format!(
                        "    {}_call_result_drop(&result);\n",
                        self.ty(&return_type).text
                    ));
                }
                _ => {}
            }
            for cleanup in cleanups.iter().rev() {
                output.push_str(&format!("    {cleanup}\n"));
            }
            output.push_str("  }\n");
            index += 1;
        }
        output.push_str("  return 0;\n}\n");
        Ok(CCode::new(output)
            .with_helper_root("runtime.core")
            .with_text_from(dependencies))
    }

    fn test_mismatch(
        &self,
        actual: &str,
        expected: &Value,
        ty: &TypeRef,
    ) -> Result<CCode, BackendError> {
        match (self.resolve_alias(ty), expected) {
            (TypeRef::Bool, Value::Bool(value)) => Ok(CCode::new(format!("{actual} != {value}"))),
            (TypeRef::I32, Value::I32(value)) => {
                let expected = i32_literal(*value);
                Ok(CCode::new(format!("{actual} != {}", expected.text)).with_text_from([expected]))
            }
            (TypeRef::I64, Value::I64(value)) => {
                let expected = i64_literal(*value);
                Ok(CCode::new(format!("{actual} != {}", expected.text)).with_text_from([expected]))
            }
            (TypeRef::F64, Value::F64(value)) => Ok(CCode::new(format!(
                "!poly_f64_test_equal({actual}, poly_f64_from_bits(UINT64_C(0x{:016x})))",
                value.0
            ))
            .with_system("stdint.h")
            .with_helper_root("runtime.feature.f64")),
            (TypeRef::String, Value::String(value)) => {
                let expected = string_view(value.as_bytes());
                Ok(CCode::new(format!(
                    "!poly_string_equal(poly_string_borrow(&{actual}), {})",
                    expected.text
                ))
                .with_text_from([expected])
                .with_helper_root("runtime.core"))
            }
            (TypeRef::Bytes, Value::Bytes(value)) => {
                let expected = bytes_view(value);
                Ok(CCode::new(format!(
                    "!poly_bytes_equal(poly_bytes_borrow(&{actual}), {})",
                    expected.text
                ))
                .with_text_from([expected])
                .with_helper_root("runtime.core"))
            }
            (TypeRef::Option(inner), Value::None) if *inner == TypeRef::I64 => {
                Ok(CCode::new(format!("{actual}.has_value")))
            }
            (TypeRef::Option(inner), Value::Some(value)) if *inner == TypeRef::I64 => {
                let Value::I64(expected) = value.as_ref() else {
                    return self.unsupported("Option<I64> test expectation has a non-I64 value");
                };
                let expected = i64_literal(*expected);
                Ok(CCode::new(format!(
                    "!{actual}.has_value || {actual}.payload.value != {}",
                    expected.text
                ))
                .with_text_from([expected]))
            }
            (TypeRef::List(inner), Value::List(values)) if *inner == TypeRef::String => {
                let mut mismatches =
                    vec![CCode::new(format!("{actual}.length != {}U", values.len()))];
                if !values.is_empty() {
                    mismatches.push(CCode::new(format!("{actual}.data == NULL")));
                }
                for (index, value) in values.iter().enumerate() {
                    mismatches.push(self.test_mismatch(
                        &format!("{actual}.data[{index}]"),
                        value,
                        &inner,
                    )?);
                }
                Ok(CCode::joined(mismatches, " || "))
            }
            (
                TypeRef::Named(_),
                Value::Record {
                    declaration,
                    fields,
                },
            ) => {
                let record = match self.declarations.get(declaration) {
                    Some(Declaration::Record(value)) => value,
                    _ => return self.unsupported("portable test record is missing"),
                };
                let mut mismatches = Vec::new();
                for field in fields {
                    let member = record
                        .fields
                        .iter()
                        .find(|candidate| candidate.header.node.id == field.field)
                        .expect("checked portable test field exists");
                    mismatches.push(self.test_mismatch(
                        &format!("{actual}.{}", value_name(&member.header.name)),
                        &field.value,
                        &member.ty,
                    )?);
                }
                Ok(if mismatches.is_empty() {
                    CCode::new("false")
                } else {
                    CCode::joined(mismatches, " || ")
                })
            }
            _ => self.unsupported("this portable-test expectation is not emitted yet"),
        }
    }

    fn test_value(
        &self,
        output: &mut String,
        argument: &TypedValue,
        parameter_type: &TypeRef,
        test_index: usize,
        argument_index: usize,
        cleanups: &mut Vec<String>,
    ) -> Result<CCode, BackendError> {
        match (&argument.value, self.resolve_alias(&argument.ty)) {
            (Value::String(value), TypeRef::String) => Ok(string_view(value.as_bytes())),
            (Value::Bytes(value), TypeRef::Bytes) => Ok(bytes_view(value)),
            (Value::Bool(value), TypeRef::Bool) => Ok(CCode::new(value.to_string())),
            (Value::I32(value), TypeRef::I32) => Ok(i32_literal(*value)),
            (Value::I64(value), TypeRef::I64) => Ok(i64_literal(*value)),
            (Value::F64(value), TypeRef::F64) => Ok(CCode::new(format!(
                "poly_f64_from_bits(UINT64_C(0x{:016x}))",
                value.0
            ))
            .with_system("stdint.h")
            .with_helper_root("runtime.feature.f64")),
            (Value::List(values), TypeRef::List(inner)) if *inner == TypeRef::String => {
                let variable = format!("argument_{test_index}_{argument_index}");
                let items = format!("{variable}_items");
                let list_ty = self.ty(&TypeRef::List(inner.clone()));
                let mut requirements = Vec::new();
                if values.is_empty() {
                    output.push_str(&format!("    {list_ty} {variable} = {{0}};\n"));
                } else {
                    output.push_str(&format!(
                        "    poly_string {items}[{}] = {{0}};\n    {list_ty} {variable} = {{{items}, 0U, {}U, {{0}}}};\n",
                        values.len(),
                        values.len()
                    ));
                    for (value_index, value) in values.iter().enumerate() {
                        let Value::String(value) = value else {
                            return self.unsupported(
                                "List<String> portable-test argument contains a non-string value",
                            );
                        };
                        let view = string_view(value.as_bytes());
                        output.push_str(&format!(
                            "    if (poly_string_clone(allocator, {}, &{items}[{value_index}]) != POLY_OK) {{ {list_ty}_drop(&{variable}); return {}; }}\n    {variable}.length = {}U;\n",
                            view.text,
                            100 + test_index,
                            value_index + 1
                        ));
                        requirements.push(view.with_helper_root("runtime.core"));
                    }
                }
                cleanups.push(format!("{list_ty}_drop(&{variable});"));
                Ok(CCode::new(format!("&{variable}"))
                    .with_text_from([list_ty])
                    .with_text_from(requirements))
            }
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
                let mut requirements = Vec::new();
                output.push_str(&format!("    {record_ty} {variable} = {{0}};\n"));
                for field in fields {
                    let declaration = record
                        .fields
                        .iter()
                        .find(|candidate| candidate.header.node.id == field.field)
                        .expect("checked test field exists");
                    let name = value_name(&declaration.header.name);
                    match (&field.value, self.resolve_alias(&declaration.ty)) {
                        (Value::String(value), TypeRef::String) => {
                            let value = string_view(value.as_bytes());
                            output.push_str(&format!(
                                "    if (poly_string_clone(allocator, {}, &{variable}.{name}) != POLY_OK) {{ return {}; }}\n",
                                value.text,
                                100 + test_index
                            ));
                            requirements.push(value.with_helper_root("runtime.core"));
                        }
                        (Value::I64(value), TypeRef::I64) => {
                            let value = i64_literal(*value);
                            output.push_str(&format!("    {variable}.{name} = {};\n", value.text));
                            requirements.push(value);
                        }
                        (Value::I32(value), TypeRef::I32) => {
                            let value = i32_literal(*value);
                            output.push_str(&format!("    {variable}.{name} = {};\n", value.text));
                            requirements.push(value);
                        }
                        (Value::Bool(value), TypeRef::Bool) => {
                            output.push_str(&format!("    {variable}.{name} = {value};\n"))
                        }
                        (Value::F64(value), TypeRef::F64) => {
                            output.push_str(&format!(
                                "    {variable}.{name} = poly_f64_from_bits(UINT64_C(0x{:016x}));\n",
                                value.0
                            ));
                            requirements.push(
                                CCode::default()
                                    .with_system("stdint.h")
                                    .with_helper_root("runtime.feature.f64"),
                            );
                        }
                        _ => {
                            return self.unsupported(
                                "nested portable-test record fields are not emitted yet",
                            );
                        }
                    }
                }
                cleanups.push(format!("{record_ty}_drop(&{variable});"));
                if let TypeRef::Contract(contract) = self.resolve_alias(parameter_type) {
                    Ok(CCode::new(format!(
                        "{}_{}_as_{}(&{variable})",
                        self.prefix,
                        type_name(&record.header.name),
                        type_name(self.declaration_name(contract))
                    ))
                    .with_text_from(requirements))
                } else {
                    Ok(CCode::new(format!("&{variable}")).with_text_from(requirements))
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

fn i32_literal(value: i32) -> CCode {
    CCode::new(if value == i32::MIN {
        "(-INT32_C(2147483647) - INT32_C(1))".to_owned()
    } else {
        format!("INT32_C({value})")
    })
    .with_system("stdint.h")
}

fn i64_literal(value: i64) -> CCode {
    CCode::new(if value == i64::MIN {
        "(-INT64_C(9223372036854775807) - INT64_C(1))".to_owned()
    } else {
        format!("INT64_C({value})")
    })
    .with_system("stdint.h")
}

fn string_view(bytes: &[u8]) -> CCode {
    view_literal("poly_string_view", bytes)
}

fn bytes_view(bytes: &[u8]) -> CCode {
    view_literal("poly_bytes_view", bytes)
}

fn view_literal(ty: &str, bytes: &[u8]) -> CCode {
    CCode::new(if bytes.is_empty() {
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
    })
    .with_system("stddef.h")
    .with_system("stdint.h")
    .with_helper_root("runtime.core")
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn intrinsic_mappings_own_exact_runtime_roots() {
        let program = fixture();
        let generator = Generator::new(&program);
        for (operation, root) in [
            (Intrinsic::FloatTrunc, "runtime.feature.f64"),
            (Intrinsic::FloatIsNegativeZero, "runtime.feature.f64"),
            (Intrinsic::FloatAbs, "runtime.feature.f64"),
            (
                Intrinsic::StringContains,
                "runtime.feature.string-predicates",
            ),
            (
                Intrinsic::StringStripPrefix,
                "runtime.feature.string-strip-prefix",
            ),
            (Intrinsic::StringConcat, "runtime.feature.string-concat"),
            (
                Intrinsic::StringUtf16Length,
                "runtime.feature.string-utf16-length",
            ),
            (
                Intrinsic::StringReplaceAll,
                "runtime.feature.string-replace-all",
            ),
            (
                Intrinsic::BytesReplaceAll,
                "runtime.feature.bytes-replace-all",
            ),
            (
                Intrinsic::StringReplaceMany,
                "runtime.feature.string-replace-many",
            ),
            (
                Intrinsic::StringTruncateUtf8Bytes,
                "runtime.feature.string-truncate-utf8",
            ),
            (Intrinsic::StringTrimStart, "runtime.feature.string-trim"),
        ] {
            let mut emitter = FunctionEmitter::new(&generator, &[], false);
            let lowered = emitter
                .intrinsic(
                    operation,
                    if matches!(
                        operation,
                        Intrinsic::FloatTrunc
                            | Intrinsic::FloatIsNegativeZero
                            | Intrinsic::FloatAbs
                    ) {
                        vec![float_expression()]
                    } else {
                        vec![
                            string_expression(),
                            string_expression(),
                            string_expression(),
                        ]
                    },
                    String::new(),
                )
                .unwrap();
            assert_eq!(
                emitter.helper_roots,
                BTreeSet::from([root.to_owned()]),
                "{operation:?}"
            );
            if operation == Intrinsic::FloatIsNegativeZero {
                assert_eq!(lowered.value, "poly_f64_is_negative_zero(-0.0)");
            }
            if operation == Intrinsic::FloatAbs {
                assert_eq!(lowered.value, "poly_f64_abs(-0.0)");
            }
        }
    }

    #[test]
    fn literal_mappings_own_standard_headers_and_runtime_roots() {
        let program = fixture();
        let generator = Generator::new(&program);
        let integer = generator.literal(&Value::I64(i64::MIN)).unwrap();
        assert!(integer.requirements.imports.iter().any(|(_, import)| {
            matches!(
                &import.kind,
                super::super::CImportKind::System { path } if path == "stdint.h"
            )
        }));
        let float = generator
            .literal(&Value::F64(portable_ir::v0::F64Bits(0)))
            .unwrap();
        assert!(
            float
                .requirements
                .helper_roots
                .contains("runtime.feature.f64")
        );
    }

    fn string_expression() -> CExpression {
        CExpression {
            prelude: String::new(),
            value: "(poly_string_view){NULL, 0U}".to_owned(),
            ty: TypeRef::String,
            requirements: CCode::default(),
        }
    }

    fn float_expression() -> CExpression {
        CExpression {
            prelude: String::new(),
            value: "-0.0".to_owned(),
            ty: TypeRef::F64,
            requirements: CCode::default(),
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
