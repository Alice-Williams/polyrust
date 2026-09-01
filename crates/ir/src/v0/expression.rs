use serde::{Deserialize, Serialize};

use super::{NodeId, NodeMeta, TypeRef, Value};

/// Explicit portable semantic operations. Backends may use native punctuation
/// only when it preserves these meanings.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Intrinsic {
    BoolNot,
    BoolAnd,
    BoolOr,
    Equal,
    NotEqual,
    Less,
    LessEqual,
    Greater,
    GreaterEqual,
    IntNegChecked,
    IntAddChecked,
    IntSubChecked,
    IntMulChecked,
    IntDivChecked,
    IntRemChecked,
    IntNegWrapping,
    IntAddWrapping,
    IntSubWrapping,
    IntMulWrapping,
    IntBitNot,
    IntBitAnd,
    IntBitOr,
    IntBitXor,
    IntShiftLeftChecked,
    IntShiftRightChecked,
    FloatNeg,
    FloatAdd,
    FloatSub,
    FloatMul,
    FloatDiv,
    FloatRemTrunc,
    StringConcat,
    StringScalarLength,
    StringIsEmpty,
    StringContains,
    StringStartsWith,
    StringStripPrefix,
    StringEndsWith,
    StringReplaceAll,
    StringReplaceMany,
    StringTruncateUtf8Bytes,
    StringTrimStart,
    StringTrimEnd,
    BytesConcat,
    BytesLength,
    BytesIsEmpty,
    ListLength,
    ListIsEmpty,
    ListGetChecked,
    ListAppend,
    ListConcat,
    ListContains,
    OptionIsSome,
    OptionIsNone,
    OptionUnwrapOr,
    ResultIsOk,
    ResultIsErr,
    WidenI32ToI64,
    NarrowI64ToI32Checked,
    StringToUtf8,
    StringFromUtf8Checked,
}

/// Unchecked v0 expression tree. Each expression carries a stable node identity
/// and source reference; M04 attaches resolved types separately.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(
    tag = "kind",
    content = "data",
    rename_all = "snake_case",
    deny_unknown_fields
)]
pub enum Expression {
    Literal {
        node: NodeMeta,
        value: Value,
    },
    Local {
        node: NodeMeta,
        name: String,
    },
    Constant {
        node: NodeMeta,
        declaration: NodeId,
    },
    SelfValue {
        node: NodeMeta,
    },
    ConstructRecord {
        node: NodeMeta,
        declaration: NodeId,
        fields: Vec<ExpressionField>,
    },
    ConstructEnum {
        node: NodeMeta,
        declaration: NodeId,
        variant: NodeId,
        fields: Vec<ExpressionField>,
    },
    ConstructSome {
        node: NodeMeta,
        value: Box<Expression>,
    },
    ConstructNone {
        node: NodeMeta,
        inner_type: TypeRef,
    },
    ConstructOk {
        node: NodeMeta,
        value: Box<Expression>,
        error_type: TypeRef,
    },
    ConstructErr {
        node: NodeMeta,
        value: Box<Expression>,
        ok_type: TypeRef,
    },
    ConstructList {
        node: NodeMeta,
        element_type: TypeRef,
        elements: Vec<Expression>,
    },
    Field {
        node: NodeMeta,
        base: Box<Expression>,
        field: NodeId,
    },
    Call {
        node: NodeMeta,
        function: NodeId,
        arguments: Vec<Expression>,
    },
    MethodCall {
        node: NodeMeta,
        receiver: Box<Expression>,
        dispatch: MethodDispatch,
        arguments: Vec<Expression>,
    },
    Intrinsic {
        node: NodeMeta,
        operation: Intrinsic,
        arguments: Vec<Expression>,
    },
    If {
        node: NodeMeta,
        condition: Box<Expression>,
        then_block: Box<Block>,
        else_block: Box<Block>,
    },
    Match {
        node: NodeMeta,
        value: Box<Expression>,
        arms: Vec<MatchArm>,
    },
    Block(Box<Block>),
}

impl Expression {
    /// Returns the metadata common to every expression variant.
    pub const fn node(&self) -> &NodeMeta {
        match self {
            Self::Literal { node, .. }
            | Self::Local { node, .. }
            | Self::Constant { node, .. }
            | Self::SelfValue { node }
            | Self::ConstructRecord { node, .. }
            | Self::ConstructEnum { node, .. }
            | Self::ConstructSome { node, .. }
            | Self::ConstructNone { node, .. }
            | Self::ConstructOk { node, .. }
            | Self::ConstructErr { node, .. }
            | Self::ConstructList { node, .. }
            | Self::Field { node, .. }
            | Self::Call { node, .. }
            | Self::MethodCall { node, .. }
            | Self::Intrinsic { node, .. }
            | Self::If { node, .. }
            | Self::Match { node, .. } => node,
            Self::Block(block) => &block.node,
        }
    }
}

/// Record/enum field initializer.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ExpressionField {
    pub field: NodeId,
    pub value: Expression,
}

/// Explicit nominal method-dispatch target.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(
    tag = "kind",
    content = "data",
    rename_all = "snake_case",
    deny_unknown_fields
)]
pub enum MethodDispatch {
    Concrete {
        implementation: NodeId,
        method: NodeId,
    },
    Contract {
        contract: NodeId,
        method: NodeId,
    },
}

/// Expression-valued lexical block.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Block {
    pub node: NodeMeta,
    pub statements: Vec<Statement>,
    pub result: Option<Box<Expression>>,
}

/// v0 statements: immutable binding, bounded list iteration, and explicit
/// return. Mutable locals and unbounded loops are intentionally absent.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(
    tag = "kind",
    content = "data",
    rename_all = "snake_case",
    deny_unknown_fields
)]
pub enum Statement {
    Let {
        node: NodeMeta,
        name: String,
        annotation: Option<TypeRef>,
        value: Expression,
    },
    ForEach {
        node: NodeMeta,
        binding: String,
        iterable: Expression,
        body: Block,
    },
    Return {
        node: NodeMeta,
        value: Option<Expression>,
    },
    Expression {
        node: NodeMeta,
        value: Expression,
    },
}

/// One exhaustive match arm.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MatchArm {
    pub node: NodeMeta,
    pub pattern: Pattern,
    pub body: Block,
}

/// Portable exhaustive patterns for booleans, tagged enums, `Option`, and
/// `Result`.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(
    tag = "kind",
    content = "data",
    rename_all = "snake_case",
    deny_unknown_fields
)]
pub enum Pattern {
    Wildcard {
        node: NodeMeta,
    },
    Bool {
        node: NodeMeta,
        value: bool,
    },
    EnumVariant {
        node: NodeMeta,
        declaration: NodeId,
        variant: NodeId,
        bindings: Vec<FieldBinding>,
    },
    None {
        node: NodeMeta,
    },
    Some {
        node: NodeMeta,
        binding: String,
    },
    Ok {
        node: NodeMeta,
        binding: String,
    },
    Err {
        node: NodeMeta,
        binding: String,
    },
}

/// Binds a record-shaped enum payload field in a match pattern.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FieldBinding {
    pub field: NodeId,
    pub binding: String,
}
