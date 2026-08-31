use serde::{Deserialize, Serialize};

use super::{
    Block, DeclarationHeader, Intrinsic, MemberHeader, NodeId, NodeMeta, TypeRef, TypedValue, Value,
};

/// Portable module namespace and its unordered declaration set.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Module {
    pub name: String,
    pub declarations: Vec<Declaration>,
}

/// Every top-level v0 declaration category.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(
    tag = "kind",
    content = "data",
    rename_all = "snake_case",
    deny_unknown_fields
)]
pub enum Declaration {
    Constant(ConstantDeclaration),
    Alias(AliasDeclaration),
    Record(RecordDeclaration),
    Enum(EnumDeclaration),
    Contract(ContractDeclaration),
    Implementation(ImplementationDeclaration),
    Function(FunctionDeclaration),
    Test(TestDeclaration),
}

impl Declaration {
    /// Returns the shared declaration metadata.
    pub const fn header(&self) -> &DeclarationHeader {
        match self {
            Self::Constant(declaration) => &declaration.header,
            Self::Alias(declaration) => &declaration.header,
            Self::Record(declaration) => &declaration.header,
            Self::Enum(declaration) => &declaration.header,
            Self::Contract(declaration) => &declaration.header,
            Self::Implementation(declaration) => &declaration.header,
            Self::Function(declaration) => &declaration.header,
            Self::Test(declaration) => &declaration.header,
        }
    }
}

/// Immutable module value and its explicit type.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ConstantDeclaration {
    pub header: DeclarationHeader,
    pub ty: TypeRef,
    pub value: ConstantExpression,
}

/// Restricted expression set permitted during deterministic constant
/// initialization. The checker validates acyclicity and operation eligibility.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(
    tag = "kind",
    content = "data",
    rename_all = "snake_case",
    deny_unknown_fields
)]
pub enum ConstantExpression {
    Literal {
        node: NodeMeta,
        value: Value,
    },
    Reference {
        node: NodeMeta,
        declaration: NodeId,
    },
    Record {
        node: NodeMeta,
        declaration: NodeId,
        fields: Vec<ConstantField>,
    },
    Enum {
        node: NodeMeta,
        declaration: NodeId,
        variant: NodeId,
        fields: Vec<ConstantField>,
    },
    Some {
        node: NodeMeta,
        value: Box<ConstantExpression>,
    },
    None {
        node: NodeMeta,
        inner_type: TypeRef,
    },
    Ok {
        node: NodeMeta,
        value: Box<ConstantExpression>,
        error_type: TypeRef,
    },
    Err {
        node: NodeMeta,
        value: Box<ConstantExpression>,
        ok_type: TypeRef,
    },
    List {
        node: NodeMeta,
        element_type: TypeRef,
        elements: Vec<ConstantExpression>,
    },
    Intrinsic {
        node: NodeMeta,
        operation: Intrinsic,
        arguments: Vec<ConstantExpression>,
    },
}

/// Field initializer in an immutable constant aggregate.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ConstantField {
    pub field: NodeId,
    pub value: ConstantExpression,
}

/// Non-recursive alternative name for another portable type.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AliasDeclaration {
    pub header: DeclarationHeader,
    pub target: TypeRef,
}

/// Immutable named product type.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RecordDeclaration {
    pub header: DeclarationHeader,
    pub fields: Vec<FieldDeclaration>,
}

/// Record or record-shaped enum payload field.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FieldDeclaration {
    pub header: MemberHeader,
    pub ty: TypeRef,
}

/// Closed portable sum type.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EnumDeclaration {
    pub header: DeclarationHeader,
    pub variants: Vec<EnumVariant>,
}

/// Unit or record-shaped tagged enum variant.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EnumVariant {
    pub header: MemberHeader,
    pub fields: Vec<FieldDeclaration>,
}

/// Restricted immutable instance-method contract.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ContractDeclaration {
    pub header: DeclarationHeader,
    pub methods: Vec<MethodSignature>,
}

/// Required contract method signature.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MethodSignature {
    pub header: MemberHeader,
    pub parameters: Vec<Parameter>,
    pub return_type: TypeRef,
}

/// Explicitly typed immutable function/method parameter.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Parameter {
    pub header: MemberHeader,
    pub ty: TypeRef,
}

/// Explicit nominal record-to-contract conformance.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ImplementationDeclaration {
    pub header: DeclarationHeader,
    pub contract: NodeId,
    pub record: NodeId,
    pub methods: Vec<MethodImplementation>,
}

/// Pure immutable-self contract method body.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MethodImplementation {
    pub header: MemberHeader,
    pub contract_method: NodeId,
    pub parameters: Vec<Parameter>,
    pub return_type: TypeRef,
    pub body: Block,
}

/// Pure top-level function.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FunctionDeclaration {
    pub header: DeclarationHeader,
    pub parameters: Vec<Parameter>,
    pub return_type: TypeRef,
    pub body: Block,
}

/// First-class portable behavioral test.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TestDeclaration {
    pub header: DeclarationHeader,
    pub invocation: TestInvocation,
    pub expected: ExpectedOutcome,
}

/// Canonically typed function or method invocation used by a portable test.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(
    tag = "kind",
    content = "data",
    rename_all = "snake_case",
    deny_unknown_fields
)]
pub enum TestInvocation {
    Function {
        function: NodeId,
        arguments: Vec<TypedValue>,
    },
    Method {
        implementation: NodeId,
        method: NodeId,
        receiver: TypedValue,
        arguments: Vec<TypedValue>,
    },
}

/// Typed expected normal value or structured error value.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(
    tag = "kind",
    content = "data",
    rename_all = "snake_case",
    deny_unknown_fields
)]
pub enum ExpectedOutcome {
    Value(TypedValue),
    Error(TypedValue),
}
