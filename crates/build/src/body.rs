use portable_ir::v0::{
    Block as IrBlock, ConstantExpression, Expression, ExpressionField, FieldBinding,
    MatchArm as IrMatchArm, MethodDispatch, NodeId, NodeMeta, Pattern as IrPattern, SourceRef,
    Statement,
};

use crate::{
    ConstantExpr, ConstantId, EnumFieldId, EnumId, EnumVariantId, FunctionId, ImplementationId,
    ImplementationMethodId, InterfaceId, InterfaceMethodId, Operation, RecordFieldId, RecordId,
    Type, Value, constant_field, enum_constant_field,
};

pub(crate) struct BuildContext {
    module: String,
    next: u64,
}

impl BuildContext {
    pub(crate) fn new(module: String) -> Self {
        Self { module, next: 1 }
    }

    pub(crate) fn node(&mut self, scope: &[String], role: impl Into<String>) -> NodeMeta {
        let id = self.next;
        self.next += 1;
        let mut segments = Vec::with_capacity(scope.len() + 2);
        segments.push(format!("module({})", self.module));
        segments.extend(scope.iter().cloned());
        segments.push(role.into());
        NodeMeta::new(NodeId(id), SourceRef::logical(segments))
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Expr(Expression);

impl Expr {
    pub fn as_ir(&self) -> &Expression {
        &self.0
    }

    pub(crate) fn into_ir(self) -> Expression {
        self.0
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Stmt(Statement);

impl Stmt {
    pub(crate) fn into_ir(self) -> Statement {
        self.0
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Block(IrBlock);

impl Block {
    pub fn as_ir(&self) -> &IrBlock {
        &self.0
    }

    pub(crate) fn into_ir(self) -> IrBlock {
        self.0
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Pattern(IrPattern);

impl Pattern {
    pub(crate) fn into_ir(self) -> IrPattern {
        self.0
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MatchArm(IrMatchArm);

impl MatchArm {
    pub(crate) fn into_ir(self) -> IrMatchArm {
        self.0
    }
}

pub struct BodyBuilder<'a> {
    pub(crate) context: &'a mut BuildContext,
    pub(crate) scope: Vec<String>,
}

impl BodyBuilder<'_> {
    fn node(&mut self, role: &str) -> NodeMeta {
        self.context.node(&self.scope, role)
    }

    pub fn literal(&mut self, value: Value) -> Expr {
        Expr(Expression::Literal {
            node: self.node("literal"),
            value: value.into_ir(),
        })
    }

    pub fn local(&mut self, name: impl Into<String>) -> Expr {
        let name = name.into();
        Expr(Expression::Local {
            node: self.node(&format!("local({name})")),
            name,
        })
    }

    pub fn constant(&mut self, constant: ConstantId) -> Expr {
        Expr(Expression::Constant {
            node: self.node("constant_reference"),
            declaration: constant.node_id(),
        })
    }

    pub fn self_value(&mut self) -> Expr {
        Expr(Expression::SelfValue {
            node: self.node("self"),
        })
    }

    pub fn record(
        &mut self,
        record: RecordId,
        fields: impl IntoIterator<Item = (RecordFieldId, Expr)>,
    ) -> Expr {
        Expr(Expression::ConstructRecord {
            node: self.node("record"),
            declaration: record.node_id(),
            fields: fields
                .into_iter()
                .map(|(field, value)| ExpressionField {
                    field: field.node_id(),
                    value: value.into_ir(),
                })
                .collect(),
        })
    }

    pub fn enumeration(
        &mut self,
        enumeration: EnumId,
        variant: EnumVariantId,
        fields: impl IntoIterator<Item = (EnumFieldId, Expr)>,
    ) -> Expr {
        Expr(Expression::ConstructEnum {
            node: self.node("enum"),
            declaration: enumeration.node_id(),
            variant: variant.node_id(),
            fields: fields
                .into_iter()
                .map(|(field, value)| ExpressionField {
                    field: field.node_id(),
                    value: value.into_ir(),
                })
                .collect(),
        })
    }

    pub fn some(&mut self, value: Expr) -> Expr {
        Expr(Expression::ConstructSome {
            node: self.node("some"),
            value: Box::new(value.into_ir()),
        })
    }

    pub fn none(&mut self, inner: Type) -> Expr {
        Expr(Expression::ConstructNone {
            node: self.node("none"),
            inner_type: inner.into_ir(),
        })
    }

    pub fn ok(&mut self, value: Expr, error: Type) -> Expr {
        Expr(Expression::ConstructOk {
            node: self.node("ok"),
            value: Box::new(value.into_ir()),
            error_type: error.into_ir(),
        })
    }

    pub fn err(&mut self, value: Expr, ok: Type) -> Expr {
        Expr(Expression::ConstructErr {
            node: self.node("err"),
            value: Box::new(value.into_ir()),
            ok_type: ok.into_ir(),
        })
    }

    pub fn list(&mut self, element: Type, values: impl IntoIterator<Item = Expr>) -> Expr {
        Expr(Expression::ConstructList {
            node: self.node("list"),
            element_type: element.into_ir(),
            elements: values.into_iter().map(Expr::into_ir).collect(),
        })
    }

    /// Constructs an owned interface value with an explicit conformance
    /// witness. The checker proves that `value` is the implementation's record.
    pub fn interface_value(&mut self, implementation: ImplementationId, value: Expr) -> Expr {
        Expr(Expression::CoerceInterface {
            node: self.node("interface_value"),
            implementation: implementation.node_id(),
            value: Box::new(value.into_ir()),
        })
    }

    pub fn field(&mut self, base: Expr, field: RecordFieldId) -> Expr {
        Expr(Expression::Field {
            node: self.node("field"),
            base: Box::new(base.into_ir()),
            field: field.node_id(),
        })
    }

    pub fn call(
        &mut self,
        function: FunctionId,
        arguments: impl IntoIterator<Item = Expr>,
    ) -> Expr {
        Expr(Expression::Call {
            node: self.node("call"),
            function: function.node_id(),
            arguments: arguments.into_iter().map(Expr::into_ir).collect(),
        })
    }

    pub fn concrete_method(
        &mut self,
        receiver: Expr,
        implementation: ImplementationId,
        method: ImplementationMethodId,
        arguments: impl IntoIterator<Item = Expr>,
    ) -> Expr {
        Expr(Expression::MethodCall {
            node: self.node("concrete_method"),
            receiver: Box::new(receiver.into_ir()),
            dispatch: MethodDispatch::Concrete {
                implementation: implementation.node_id(),
                method: method.node_id(),
            },
            arguments: arguments.into_iter().map(Expr::into_ir).collect(),
        })
    }

    pub fn interface_method(
        &mut self,
        receiver: Expr,
        interface: InterfaceId,
        method: InterfaceMethodId,
        arguments: impl IntoIterator<Item = Expr>,
    ) -> Expr {
        Expr(Expression::MethodCall {
            node: self.node("interface_method"),
            receiver: Box::new(receiver.into_ir()),
            dispatch: MethodDispatch::Interface {
                interface: interface.node_id(),
                method: method.node_id(),
            },
            arguments: arguments.into_iter().map(Expr::into_ir).collect(),
        })
    }

    pub fn intrinsic(
        &mut self,
        operation: Operation,
        arguments: impl IntoIterator<Item = Expr>,
    ) -> Expr {
        Expr(Expression::Intrinsic {
            node: self.node("intrinsic"),
            operation,
            arguments: arguments.into_iter().map(Expr::into_ir).collect(),
        })
    }

    pub fn if_else(&mut self, condition: Expr, then_block: Block, else_block: Block) -> Expr {
        Expr(Expression::If {
            node: self.node("if"),
            condition: Box::new(condition.into_ir()),
            then_block: Box::new(then_block.into_ir()),
            else_block: Box::new(else_block.into_ir()),
        })
    }

    pub fn match_value(&mut self, value: Expr, arms: impl IntoIterator<Item = MatchArm>) -> Expr {
        Expr(Expression::Match {
            node: self.node("match"),
            value: Box::new(value.into_ir()),
            arms: arms.into_iter().map(MatchArm::into_ir).collect(),
        })
    }

    pub fn block_expression(&mut self, block: Block) -> Expr {
        Expr(Expression::Block(Box::new(block.into_ir())))
    }

    pub fn block(
        &mut self,
        statements: impl IntoIterator<Item = Stmt>,
        result: Option<Expr>,
    ) -> Block {
        Block(IrBlock {
            node: self.node("block"),
            statements: statements.into_iter().map(Stmt::into_ir).collect(),
            result: result.map(|value| Box::new(value.into_ir())),
        })
    }

    pub fn let_statement(
        &mut self,
        name: impl Into<String>,
        annotation: Option<Type>,
        value: Expr,
    ) -> Stmt {
        let name = name.into();
        Stmt(Statement::Let {
            node: self.node(&format!("let({name})")),
            name,
            annotation: annotation.map(Type::into_ir),
            value: value.into_ir(),
        })
    }

    pub fn for_each(&mut self, binding: impl Into<String>, iterable: Expr, body: Block) -> Stmt {
        let binding = binding.into();
        Stmt(Statement::ForEach {
            node: self.node(&format!("for_each({binding})")),
            binding,
            iterable: iterable.into_ir(),
            body: body.into_ir(),
        })
    }

    pub fn return_statement(&mut self, value: Option<Expr>) -> Stmt {
        Stmt(Statement::Return {
            node: self.node("return"),
            value: value.map(Expr::into_ir),
        })
    }

    pub fn expression_statement(&mut self, value: Expr) -> Stmt {
        Stmt(Statement::Expression {
            node: self.node("expression_statement"),
            value: value.into_ir(),
        })
    }

    pub fn wildcard_pattern(&mut self) -> Pattern {
        Pattern(IrPattern::Wildcard {
            node: self.node("pattern(wildcard)"),
        })
    }

    pub fn bool_pattern(&mut self, value: bool) -> Pattern {
        Pattern(IrPattern::Bool {
            node: self.node("pattern(bool)"),
            value,
        })
    }

    pub fn enum_pattern(
        &mut self,
        enumeration: EnumId,
        variant: EnumVariantId,
        bindings: impl IntoIterator<Item = (EnumFieldId, String)>,
    ) -> Pattern {
        Pattern(IrPattern::EnumVariant {
            node: self.node("pattern(enum)"),
            declaration: enumeration.node_id(),
            variant: variant.node_id(),
            bindings: bindings
                .into_iter()
                .map(|(field, binding)| FieldBinding {
                    field: field.node_id(),
                    binding,
                })
                .collect(),
        })
    }

    pub fn none_pattern(&mut self) -> Pattern {
        Pattern(IrPattern::None {
            node: self.node("pattern(none)"),
        })
    }

    pub fn some_pattern(&mut self, binding: impl Into<String>) -> Pattern {
        Pattern(IrPattern::Some {
            node: self.node("pattern(some)"),
            binding: binding.into(),
        })
    }

    pub fn ok_pattern(&mut self, binding: impl Into<String>) -> Pattern {
        Pattern(IrPattern::Ok {
            node: self.node("pattern(ok)"),
            binding: binding.into(),
        })
    }

    pub fn err_pattern(&mut self, binding: impl Into<String>) -> Pattern {
        Pattern(IrPattern::Err {
            node: self.node("pattern(err)"),
            binding: binding.into(),
        })
    }

    pub fn match_arm(&mut self, pattern: Pattern, body: Block) -> MatchArm {
        MatchArm(IrMatchArm {
            node: self.node("match_arm"),
            pattern: pattern.into_ir(),
            body: body.into_ir(),
        })
    }

    pub fn constant_literal(&mut self, value: Value) -> ConstantExpr {
        ConstantExpr::from_ir(ConstantExpression::Literal {
            node: self.node("constant(literal)"),
            value: value.into_ir(),
        })
    }

    pub fn constant_reference(&mut self, constant: ConstantId) -> ConstantExpr {
        ConstantExpr::from_ir(ConstantExpression::Reference {
            node: self.node("constant(reference)"),
            declaration: constant.node_id(),
        })
    }

    pub fn constant_record(
        &mut self,
        record: RecordId,
        fields: impl IntoIterator<Item = (RecordFieldId, ConstantExpr)>,
    ) -> ConstantExpr {
        ConstantExpr::from_ir(ConstantExpression::Record {
            node: self.node("constant(record)"),
            declaration: record.node_id(),
            fields: fields
                .into_iter()
                .map(|(field, value)| constant_field(field, value))
                .collect(),
        })
    }

    pub fn constant_enum(
        &mut self,
        enumeration: EnumId,
        variant: EnumVariantId,
        fields: impl IntoIterator<Item = (EnumFieldId, ConstantExpr)>,
    ) -> ConstantExpr {
        ConstantExpr::from_ir(ConstantExpression::Enum {
            node: self.node("constant(enum)"),
            declaration: enumeration.node_id(),
            variant: variant.node_id(),
            fields: fields
                .into_iter()
                .map(|(field, value)| enum_constant_field(field, value))
                .collect(),
        })
    }

    pub fn constant_some(&mut self, value: ConstantExpr) -> ConstantExpr {
        ConstantExpr::from_ir(ConstantExpression::Some {
            node: self.node("constant(some)"),
            value: Box::new(value.into_ir()),
        })
    }

    pub fn constant_none(&mut self, inner: Type) -> ConstantExpr {
        ConstantExpr::from_ir(ConstantExpression::None {
            node: self.node("constant(none)"),
            inner_type: inner.into_ir(),
        })
    }

    pub fn constant_ok(&mut self, value: ConstantExpr, error: Type) -> ConstantExpr {
        ConstantExpr::from_ir(ConstantExpression::Ok {
            node: self.node("constant(ok)"),
            value: Box::new(value.into_ir()),
            error_type: error.into_ir(),
        })
    }

    pub fn constant_err(&mut self, value: ConstantExpr, ok: Type) -> ConstantExpr {
        ConstantExpr::from_ir(ConstantExpression::Err {
            node: self.node("constant(err)"),
            value: Box::new(value.into_ir()),
            ok_type: ok.into_ir(),
        })
    }

    pub fn constant_list(
        &mut self,
        element: Type,
        values: impl IntoIterator<Item = ConstantExpr>,
    ) -> ConstantExpr {
        ConstantExpr::from_ir(ConstantExpression::List {
            node: self.node("constant(list)"),
            element_type: element.into_ir(),
            elements: values.into_iter().map(ConstantExpr::into_ir).collect(),
        })
    }

    pub fn constant_intrinsic(
        &mut self,
        operation: Operation,
        arguments: impl IntoIterator<Item = ConstantExpr>,
    ) -> ConstantExpr {
        ConstantExpr::from_ir(ConstantExpression::Intrinsic {
            node: self.node("constant(intrinsic)"),
            operation,
            arguments: arguments.into_iter().map(ConstantExpr::into_ir).collect(),
        })
    }
}
