//! Java AST: expression model.

use super::identifiers::JavaIdentifier;
use super::runtime_members::JavaRuntimeMember;
use super::types::{JavaKnownType, JavaType};
use portable_codegen::{
    GeneratedCallableId, GeneratedInterfaceMethodId, GeneratedSymbolId, GeneratedTypeId,
    GeneratedValueId,
};
use portable_core_ir::CoreFieldId;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum JavaPrecedence {
    Assignment,
    Conditional,
    LogicalOr,
    LogicalAnd,
    BitOr,
    BitXor,
    BitAnd,
    Equality,
    Relational,
    Shift,
    Additive,
    Multiplicative,
    Unary,
    Primary,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum JavaUnaryOperator {
    Not,
    Negate,
    BitNot,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum JavaBinaryOperator {
    LogicalAnd,
    LogicalOr,
    Equal,
    NotEqual,
    Less,
    LessEqual,
    Greater,
    GreaterEqual,
    Add,
    Subtract,
    Multiply,
    Divide,
    Remainder,
    BitAnd,
    BitOr,
    BitXor,
    ShiftLeft,
    ShiftRight,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum JavaLiteral {
    Boolean(bool),
    I32(i32),
    I64(i64),
    CharScalar(u32),
    String(String),
    /// Exact UTF-16 code units used only by generated boundary conformance tests.
    Utf16Units(Vec<u16>),
    InternalNull(JavaNullPurpose),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum JavaNullPurpose {
    AbsentTaggedPayload,
    InternalSentinel,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum JavaValueRef {
    Local(JavaIdentifier),
    This,
    Generated(GeneratedSymbolId),
    EnumVariant {
        enumeration: GeneratedTypeId,
        variant: GeneratedValueId,
    },
    KnownField(crate::dialect::JavaKnownField),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct JavaMethodSignature {
    pub receiver: Option<JavaType>,
    pub parameters: Vec<JavaType>,
    pub result: JavaType,
    pub checked_exceptions: Vec<JavaKnownType>,
    pub nullable_result: bool,
    pub pure: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum JavaCallableRef {
    Known {
        callable: crate::dialect::JavaKnownCallable,
        signature: JavaMethodSignature,
    },
    Runtime {
        callable: crate::dialect::JavaRuntimeCallable,
        signature: JavaMethodSignature,
    },
    Generated {
        symbol: GeneratedCallableId,
        signature: JavaMethodSignature,
    },
    Interface {
        symbol: GeneratedInterfaceMethodId,
        signature: JavaMethodSignature,
    },
    Member {
        owner: JavaType,
        name: JavaIdentifier,
        signature: JavaMethodSignature,
        origin: JavaMemberOrigin,
    },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum JavaMemberOrigin {
    Known(crate::dialect::JavaKnownMethod),
    GeneratedField(CoreFieldId),
    GeneratedVariant,
    Runtime(JavaRuntimeMember),
    GeneratedImplementation(portable_core_ir::CoreImplementationMethodId),
}
