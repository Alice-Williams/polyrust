#![forbid(unsafe_code)]

//! Target-independent syntax model for the executable prototype.

/// Complete, versioned v0 unchecked IR and canonical JSON representation.
pub mod v0;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Type {
    Bool,
    I64,
    String,
    Named(String),
}

impl Type {
    pub fn named(name: impl Into<String>) -> Self {
        Self::Named(name.into())
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Value {
    Bool(bool),
    I64(i64),
    String(String),
    Record {
        name: String,
        fields: Vec<(String, Value)>,
    },
}

impl Value {
    pub fn record(
        name: impl Into<String>,
        fields: impl IntoIterator<Item = (impl Into<String>, Value)>,
    ) -> Self {
        Self::Record {
            name: name.into(),
            fields: fields
                .into_iter()
                .map(|(name, value)| (name.into(), value))
                .collect(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Parameter {
    pub name: String,
    pub ty: Type,
}

impl Parameter {
    pub fn new(name: impl Into<String>, ty: Type) -> Self {
        Self {
            name: name.into(),
            ty,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Field {
    pub name: String,
    pub ty: Type,
}

impl Field {
    pub fn new(name: impl Into<String>, ty: Type) -> Self {
        Self {
            name: name.into(),
            ty,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Constant {
    pub name: String,
    pub ty: Type,
    pub value: Value,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Record {
    pub name: String,
    pub fields: Vec<Field>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MethodSignature {
    pub name: String,
    pub parameters: Vec<Parameter>,
    pub return_type: Type,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Contract {
    pub name: String,
    pub methods: Vec<MethodSignature>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CompareOperator {
    GreaterThanOrEqual,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Expression {
    Value(Value),
    Local(String),
    Constant(String),
    SelfField(String),
    Field {
        base: Box<Expression>,
        field: String,
    },
    Compare {
        operator: CompareOperator,
        left: Box<Expression>,
        right: Box<Expression>,
    },
    MethodCall {
        receiver: Box<Expression>,
        method: String,
        arguments: Vec<Expression>,
    },
}

impl Expression {
    pub fn local(name: impl Into<String>) -> Self {
        Self::Local(name.into())
    }

    pub fn constant(name: impl Into<String>) -> Self {
        Self::Constant(name.into())
    }

    pub fn self_field(name: impl Into<String>) -> Self {
        Self::SelfField(name.into())
    }

    pub fn field(base: Expression, field: impl Into<String>) -> Self {
        Self::Field {
            base: Box::new(base),
            field: field.into(),
        }
    }

    pub fn greater_than_or_equal(left: Expression, right: Expression) -> Self {
        Self::Compare {
            operator: CompareOperator::GreaterThanOrEqual,
            left: Box::new(left),
            right: Box::new(right),
        }
    }

    pub fn method_call(
        receiver: Expression,
        method: impl Into<String>,
        arguments: impl IntoIterator<Item = Expression>,
    ) -> Self {
        Self::MethodCall {
            receiver: Box::new(receiver),
            method: method.into(),
            arguments: arguments.into_iter().collect(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Function {
    pub name: String,
    pub parameters: Vec<Parameter>,
    pub return_type: Type,
    pub body: Expression,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Implementation {
    pub contract: String,
    pub record: String,
    pub methods: Vec<Function>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PortableTest {
    pub name: String,
    pub function: String,
    pub arguments: Vec<Value>,
    pub expected: Value,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Module {
    pub name: String,
    pub constants: Vec<Constant>,
    pub records: Vec<Record>,
    pub contracts: Vec<Contract>,
    pub implementations: Vec<Implementation>,
    pub functions: Vec<Function>,
    pub tests: Vec<PortableTest>,
}
