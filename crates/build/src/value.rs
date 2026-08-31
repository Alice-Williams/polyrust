use portable_ir::v0::{
    ConstantExpression, ConstantField, ExpectedOutcome, F64Bits, TestInvocation, TypeRef,
    TypedValue as IrTypedValue, Value as IrValue, ValueField,
};

use crate::{
    AliasId, ContractId, EnumFieldId, EnumId, EnumVariantId, FunctionId, ImplementationId,
    ImplementationMethodId, NamedTypeHandle, RecordFieldId, RecordId,
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Type(TypeRef);

impl Type {
    pub const fn unit() -> Self {
        Self(TypeRef::Unit)
    }

    pub const fn bool() -> Self {
        Self(TypeRef::Bool)
    }

    pub const fn i32() -> Self {
        Self(TypeRef::I32)
    }

    pub const fn i64() -> Self {
        Self(TypeRef::I64)
    }

    pub const fn f64() -> Self {
        Self(TypeRef::F64)
    }

    pub const fn char() -> Self {
        Self(TypeRef::Char)
    }

    pub const fn string() -> Self {
        Self(TypeRef::String)
    }

    pub const fn bytes() -> Self {
        Self(TypeRef::Bytes)
    }

    pub fn list(element: Self) -> Self {
        Self(TypeRef::List(Box::new(element.0)))
    }

    pub fn option(inner: Self) -> Self {
        Self(TypeRef::Option(Box::new(inner.0)))
    }

    pub fn result(ok: Self, error: Self) -> Self {
        Self(TypeRef::Result {
            ok: Box::new(ok.0),
            error: Box::new(error.0),
        })
    }

    pub fn named(handle: impl NamedTypeHandle) -> Self {
        Self(TypeRef::Named(handle.named_node()))
    }

    pub fn contract(contract: ContractId) -> Self {
        Self(TypeRef::Contract(contract.node_id()))
    }

    pub fn as_ir(&self) -> &TypeRef {
        &self.0
    }

    pub(crate) fn into_ir(self) -> TypeRef {
        self.0
    }
}

impl From<RecordId> for Type {
    fn from(value: RecordId) -> Self {
        Self::named(value)
    }
}

impl From<EnumId> for Type {
    fn from(value: EnumId) -> Self {
        Self::named(value)
    }
}

impl From<AliasId> for Type {
    fn from(value: AliasId) -> Self {
        Self::named(value)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Value(IrValue);

impl Value {
    pub const fn unit() -> Self {
        Self(IrValue::Unit)
    }

    pub const fn bool(value: bool) -> Self {
        Self(IrValue::Bool(value))
    }

    pub const fn i32(value: i32) -> Self {
        Self(IrValue::I32(value))
    }

    pub const fn i64(value: i64) -> Self {
        Self(IrValue::I64(value))
    }

    pub fn f64(value: f64) -> Self {
        Self(IrValue::F64(F64Bits::from_f64(value)))
    }

    pub const fn f64_bits(bits: u64) -> Self {
        Self(IrValue::F64(F64Bits(bits)))
    }

    pub const fn char(value: char) -> Self {
        Self(IrValue::Char(value))
    }

    pub fn string(value: impl Into<String>) -> Self {
        Self(IrValue::String(value.into()))
    }

    pub fn bytes(value: impl Into<Vec<u8>>) -> Self {
        Self(IrValue::Bytes(value.into()))
    }

    pub fn list(values: impl IntoIterator<Item = Self>) -> Self {
        Self(IrValue::List(
            values.into_iter().map(|value| value.0).collect(),
        ))
    }

    pub const fn none() -> Self {
        Self(IrValue::None)
    }

    pub fn some(value: Self) -> Self {
        Self(IrValue::Some(Box::new(value.0)))
    }

    pub fn ok(value: Self) -> Self {
        Self(IrValue::Ok(Box::new(value.0)))
    }

    pub fn err(value: Self) -> Self {
        Self(IrValue::Err(Box::new(value.0)))
    }

    pub fn record(
        record: RecordId,
        fields: impl IntoIterator<Item = (RecordFieldId, Self)>,
    ) -> Self {
        Self(IrValue::Record {
            declaration: record.node_id(),
            fields: fields
                .into_iter()
                .map(|(field, value)| ValueField {
                    field: field.node_id(),
                    value: value.0,
                })
                .collect(),
        })
    }

    pub fn enumeration(
        enumeration: EnumId,
        variant: EnumVariantId,
        fields: impl IntoIterator<Item = (EnumFieldId, Self)>,
    ) -> Self {
        Self(IrValue::Enum {
            declaration: enumeration.node_id(),
            variant: variant.node_id(),
            fields: fields
                .into_iter()
                .map(|(field, value)| ValueField {
                    field: field.node_id(),
                    value: value.0,
                })
                .collect(),
        })
    }

    pub fn as_ir(&self) -> &IrValue {
        &self.0
    }

    pub(crate) fn into_ir(self) -> IrValue {
        self.0
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TypedValue(IrTypedValue);

impl TypedValue {
    pub fn new(ty: Type, value: Value) -> Self {
        Self(IrTypedValue {
            ty: ty.into_ir(),
            value: value.into_ir(),
        })
    }

    pub fn as_ir(&self) -> &IrTypedValue {
        &self.0
    }

    pub(crate) fn into_ir(self) -> IrTypedValue {
        self.0
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Expected(ExpectedOutcome);

impl Expected {
    pub fn value(value: TypedValue) -> Self {
        Self(ExpectedOutcome::Value(value.into_ir()))
    }

    pub fn error(value: TypedValue) -> Self {
        Self(ExpectedOutcome::Error(value.into_ir()))
    }

    pub(crate) fn into_ir(self) -> ExpectedOutcome {
        self.0
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Invocation(TestInvocation);

impl Invocation {
    pub fn function(function: FunctionId, arguments: impl IntoIterator<Item = TypedValue>) -> Self {
        Self(TestInvocation::Function {
            function: function.node_id(),
            arguments: arguments.into_iter().map(TypedValue::into_ir).collect(),
        })
    }

    pub fn method(
        implementation: ImplementationId,
        method: ImplementationMethodId,
        receiver: TypedValue,
        arguments: impl IntoIterator<Item = TypedValue>,
    ) -> Self {
        Self(TestInvocation::Method {
            implementation: implementation.node_id(),
            method: method.node_id(),
            receiver: receiver.into_ir(),
            arguments: arguments.into_iter().map(TypedValue::into_ir).collect(),
        })
    }

    pub(crate) fn into_ir(self) -> TestInvocation {
        self.0
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ConstantExpr(ConstantExpression);

impl ConstantExpr {
    pub(crate) fn from_ir(expression: ConstantExpression) -> Self {
        Self(expression)
    }

    pub(crate) fn into_ir(self) -> ConstantExpression {
        self.0
    }
}

pub(crate) fn constant_field(field: RecordFieldId, value: ConstantExpr) -> ConstantField {
    ConstantField {
        field: field.node_id(),
        value: value.into_ir(),
    }
}

pub(crate) fn enum_constant_field(field: EnumFieldId, value: ConstantExpr) -> ConstantField {
    ConstantField {
        field: field.node_id(),
        value: value.into_ir(),
    }
}
