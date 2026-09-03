use std::collections::{BTreeMap, BTreeSet};

use portable_codegen::{
    AstViolation, GeneratedCallableId, GeneratedInterfaceMethodId, GeneratedSymbolId,
    GeneratedTypeId, GeneratedValueId, TargetAstContext, TargetCallableRef, TargetExpressionNode,
    TargetFileItemNode, TargetStatementNode, TargetSymbolRef, TargetTypeRef,
};
use portable_core_ir::CoreFieldId;
use portable_diagnostics::DiagnosticCode;

use crate::dialect::{JavaDialect, JavaInvocationKind, JavaKnownConstructor, JavaRuntimeHelper};

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct JavaIdentifier(String);

impl JavaIdentifier {
    pub fn new(candidate: impl Into<String>) -> Result<Self, AstViolation> {
        let candidate = candidate.into();
        let mut chars = candidate.chars();
        let lexical = chars
            .next()
            .is_some_and(|first| first == '_' || first == '$' || first.is_ascii_alphabetic())
            && chars.all(|ch| ch == '_' || ch == '$' || ch.is_ascii_alphanumeric());
        if lexical && !JAVA_KEYWORDS.contains(&candidate.as_str()) {
            Ok(Self(candidate))
        } else {
            Err(AstViolation::new(
                DiagnosticCode::InvalidIdentifier,
                format!("invalid Java identifier {candidate:?}"),
            ))
        }
    }

    pub fn from_portable(candidate: &str) -> Self {
        let mut value = candidate
            .chars()
            .map(|ch| {
                if ch == '_' || ch == '$' || ch.is_ascii_alphanumeric() {
                    ch
                } else {
                    '_'
                }
            })
            .collect::<String>();
        if value.is_empty()
            || !value
                .chars()
                .next()
                .is_some_and(|ch| ch == '_' || ch == '$' || ch.is_ascii_alphabetic())
        {
            value.insert(0, '_');
        }
        if JAVA_KEYWORDS.contains(&value.as_str()) {
            value.push('_');
        }
        if value.starts_with("__polyrust_") {
            value.push_str("_user");
        }
        Self::new(value).expect("portable identifier normalization is valid")
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

const JAVA_KEYWORDS: &[&str] = &[
    "abstract",
    "assert",
    "boolean",
    "break",
    "byte",
    "case",
    "catch",
    "char",
    "class",
    "const",
    "continue",
    "default",
    "do",
    "double",
    "else",
    "enum",
    "exports",
    "extends",
    "false",
    "final",
    "finally",
    "float",
    "for",
    "goto",
    "if",
    "implements",
    "import",
    "instanceof",
    "int",
    "interface",
    "long",
    "module",
    "native",
    "new",
    "non-sealed",
    "null",
    "open",
    "opens",
    "package",
    "permits",
    "private",
    "protected",
    "provides",
    "public",
    "record",
    "requires",
    "return",
    "sealed",
    "short",
    "static",
    "strictfp",
    "super",
    "switch",
    "synchronized",
    "this",
    "throw",
    "throws",
    "to",
    "transient",
    "transitive",
    "true",
    "try",
    "uses",
    "var",
    "void",
    "volatile",
    "when",
    "while",
    "with",
    "yield",
    "_",
];

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum JavaPrimitive {
    Boolean,
    Byte,
    Char,
    Int,
    Long,
    Double,
    Void,
}

impl JavaPrimitive {
    pub const fn keyword(self) -> &'static str {
        match self {
            Self::Boolean => "boolean",
            Self::Byte => "byte",
            Self::Char => "char",
            Self::Int => "int",
            Self::Long => "long",
            Self::Double => "double",
            Self::Void => "void",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum JavaKnownType {
    Object,
    String,
    Boolean,
    Byte,
    Character,
    Integer,
    Long,
    Double,
    Math,
    AssertionError,
    IllegalArgumentException,
    IllegalStateException,
    RuntimeException,
    BigInteger,
    ByteBuffer,
    CharBuffer,
    CharacterCodingException,
    Charset,
    CharsetDecoder,
    CodingErrorAction,
    StandardCharsets,
    ArrayList,
    Arrays,
    LinkedHashMap,
    List,
    Map,
    Objects,
    RuntimeUnit,
    RuntimeError,
    RuntimeResult,
    RuntimeOption,
    RuntimeValueResult,
    RuntimeBytes,
    RuntimeSemanticValue,
    RuntimeScalar,
}

impl JavaKnownType {
    pub const ALL: [Self; 35] = [
        Self::Object,
        Self::String,
        Self::Boolean,
        Self::Byte,
        Self::Character,
        Self::Integer,
        Self::Long,
        Self::Double,
        Self::Math,
        Self::AssertionError,
        Self::IllegalArgumentException,
        Self::IllegalStateException,
        Self::RuntimeException,
        Self::BigInteger,
        Self::ByteBuffer,
        Self::CharBuffer,
        Self::CharacterCodingException,
        Self::Charset,
        Self::CharsetDecoder,
        Self::CodingErrorAction,
        Self::StandardCharsets,
        Self::ArrayList,
        Self::Arrays,
        Self::LinkedHashMap,
        Self::List,
        Self::Map,
        Self::Objects,
        Self::RuntimeUnit,
        Self::RuntimeError,
        Self::RuntimeResult,
        Self::RuntimeOption,
        Self::RuntimeValueResult,
        Self::RuntimeBytes,
        Self::RuntimeSemanticValue,
        Self::RuntimeScalar,
    ];

    pub const fn qualified_name(self) -> &'static str {
        match self {
            Self::Object => "java.lang.Object",
            Self::String => "java.lang.String",
            Self::Boolean => "java.lang.Boolean",
            Self::Byte => "java.lang.Byte",
            Self::Character => "java.lang.Character",
            Self::Integer => "java.lang.Integer",
            Self::Long => "java.lang.Long",
            Self::Double => "java.lang.Double",
            Self::Math => "java.lang.Math",
            Self::AssertionError => "java.lang.AssertionError",
            Self::IllegalArgumentException => "java.lang.IllegalArgumentException",
            Self::IllegalStateException => "java.lang.IllegalStateException",
            Self::RuntimeException => "java.lang.RuntimeException",
            Self::BigInteger => "java.math.BigInteger",
            Self::ByteBuffer => "java.nio.ByteBuffer",
            Self::CharBuffer => "java.nio.CharBuffer",
            Self::CharacterCodingException => "java.nio.charset.CharacterCodingException",
            Self::Charset => "java.nio.charset.Charset",
            Self::CharsetDecoder => "java.nio.charset.CharsetDecoder",
            Self::CodingErrorAction => "java.nio.charset.CodingErrorAction",
            Self::StandardCharsets => "java.nio.charset.StandardCharsets",
            Self::ArrayList => "java.util.ArrayList",
            Self::Arrays => "java.util.Arrays",
            Self::LinkedHashMap => "java.util.LinkedHashMap",
            Self::List => "java.util.List",
            Self::Map => "java.util.Map",
            Self::Objects => "java.util.Objects",
            Self::RuntimeUnit => "org.polyrust.generated.Runtime.Unit",
            Self::RuntimeError => "org.polyrust.generated.Runtime.PolyError",
            Self::RuntimeResult => "org.polyrust.generated.Runtime.PolyResult",
            Self::RuntimeOption => "org.polyrust.generated.Runtime.PolyOption",
            Self::RuntimeValueResult => "org.polyrust.generated.Runtime.PolyValueResult",
            Self::RuntimeBytes => "org.polyrust.generated.Runtime.Bytes",
            Self::RuntimeSemanticValue => "org.polyrust.generated.Runtime.SemanticValue",
            Self::RuntimeScalar => "org.polyrust.generated.Runtime.Scalar",
        }
    }

    pub fn simple_name(self) -> &'static str {
        self.qualified_name()
            .rsplit('.')
            .next()
            .expect("known type")
    }

    pub const fn implicit(self) -> bool {
        matches!(
            self,
            Self::Object
                | Self::String
                | Self::Boolean
                | Self::Byte
                | Self::Character
                | Self::Integer
                | Self::Long
                | Self::Double
                | Self::Math
                | Self::AssertionError
                | Self::IllegalArgumentException
                | Self::IllegalStateException
                | Self::RuntimeException
        )
    }

    pub const fn runtime_nested(self) -> bool {
        matches!(
            self,
            Self::RuntimeUnit
                | Self::RuntimeError
                | Self::RuntimeResult
                | Self::RuntimeOption
                | Self::RuntimeValueResult
                | Self::RuntimeBytes
                | Self::RuntimeSemanticValue
                | Self::RuntimeScalar
        )
    }

    pub const fn runtime_helper(self) -> Option<crate::dialect::JavaRuntimeHelper> {
        match self {
            Self::RuntimeUnit
            | Self::RuntimeError
            | Self::RuntimeResult
            | Self::RuntimeSemanticValue
            | Self::RuntimeScalar => Some(crate::dialect::JavaRuntimeHelper::Core),
            Self::RuntimeOption | Self::RuntimeValueResult => {
                Some(crate::dialect::JavaRuntimeHelper::TaggedValues)
            }
            Self::RuntimeBytes => Some(crate::dialect::JavaRuntimeHelper::Bytes),
            _ => None,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum JavaTypeName {
    Known(JavaKnownType),
    Generated(GeneratedTypeId),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum JavaWildcardBound {
    Extends,
    Super,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum JavaArrayOwnership {
    InternalMutable,
    DefensiveCopyBoundary,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum JavaArrayOwnershipTransition {
    FreshCopyToBoundary,
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum JavaType {
    Primitive(JavaPrimitive),
    Boxed(JavaPrimitive),
    Reference(JavaTypeName),
    Array {
        component: Box<JavaType>,
        ownership: JavaArrayOwnership,
    },
    Generic {
        raw: JavaTypeName,
        arguments: Vec<JavaType>,
    },
    Wildcard {
        bound: Option<(JavaWildcardBound, Box<JavaType>)>,
    },
    TypeVariable(JavaIdentifier),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum JavaTypeUse {
    Value,
    Parameter,
    Return,
    Field,
    GenericArgument,
    TypeBound,
}

impl JavaType {
    pub const fn primitive(value: JavaPrimitive) -> Self {
        Self::Primitive(value)
    }
    pub const fn known(value: JavaKnownType) -> Self {
        Self::Reference(JavaTypeName::Known(value))
    }
    pub fn generic(raw: JavaKnownType, arguments: Vec<Self>) -> Self {
        Self::Generic {
            raw: JavaTypeName::Known(raw),
            arguments,
        }
    }
    pub fn boxed(self) -> Self {
        match self {
            Self::Primitive(value) => Self::Boxed(value),
            value => value,
        }
    }

    pub fn symbols(&self, symbols: &mut BTreeSet<TargetSymbolRef<JavaDialect>>) {
        match self {
            Self::Reference(name) => insert_type_name(name, symbols),
            Self::Array { component, .. } => component.symbols(symbols),
            Self::Generic { raw, arguments } => {
                insert_type_name(raw, symbols);
                for argument in arguments {
                    argument.symbols(symbols);
                }
            }
            Self::Wildcard {
                bound: Some((_, ty)),
            } => ty.symbols(symbols),
            Self::Wildcard { bound: None }
            | Self::Primitive(_)
            | Self::Boxed(_)
            | Self::TypeVariable(_) => {}
        }
    }

    pub fn verify(&self, usage: JavaTypeUse) -> Vec<AstViolation> {
        let mut violations = Vec::new();
        match self {
            Self::Primitive(JavaPrimitive::Void) if usage != JavaTypeUse::Return => {
                violations.push(type_error("Java void is valid only as a return type"));
            }
            Self::Primitive(_) if usage == JavaTypeUse::GenericArgument => {
                violations.push(type_error("primitive Java generic arguments must be boxed"));
            }
            Self::Boxed(JavaPrimitive::Void) => {
                violations.push(type_error("Java void has no boxed portable representation"));
            }
            Self::Array {
                component,
                ownership,
            } => {
                violations.extend(component.verify(JavaTypeUse::Value));
                if matches!(
                    usage,
                    JavaTypeUse::Parameter | JavaTypeUse::Return | JavaTypeUse::Field
                ) && *ownership != JavaArrayOwnership::DefensiveCopyBoundary
                {
                    violations.push(type_error("mutable Java array escapes a value boundary"));
                }
            }
            Self::Generic { arguments, .. } => {
                if arguments.is_empty() {
                    violations.push(type_error("raw generics are forbidden"));
                }
                for argument in arguments {
                    violations.extend(argument.verify(JavaTypeUse::GenericArgument));
                }
            }
            Self::Wildcard { bound } => {
                if usage != JavaTypeUse::GenericArgument {
                    violations.push(type_error("wildcards require generic-argument position"));
                }
                if let Some((_, bound)) = bound {
                    violations.extend(bound.verify(JavaTypeUse::TypeBound));
                }
            }
            Self::Primitive(_) | Self::Boxed(_) | Self::Reference(_) | Self::TypeVariable(_) => {}
        }
        violations
    }
}

fn type_error(message: &str) -> AstViolation {
    AstViolation::new(DiagnosticCode::TypeMismatch, message)
}

fn insert_type_name(name: &JavaTypeName, symbols: &mut BTreeSet<TargetSymbolRef<JavaDialect>>) {
    match name {
        JavaTypeName::Known(value) => {
            symbols.insert(TargetSymbolRef::KnownType(*value));
            if let Some(helper) = value.runtime_helper() {
                symbols.insert(TargetSymbolRef::RuntimeHelper(helper));
            }
        }
        JavaTypeName::Generated(value) => {
            symbols.insert(TargetSymbolRef::Generated(GeneratedSymbolId::Type(*value)));
        }
    }
}

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

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum JavaRuntimeMember {
    SemanticEquals,
    DeepEquals,
    ScalarValue,
    ErrorCode,
    ErrorMessage,
    ResultOk,
    ResultValue,
    ResultError,
    OptionSome,
    OptionValue,
    ValueResultOk,
    ValueResultValue,
    ValueResultError,
    BytesValues,
}

impl JavaRuntimeMember {
    pub const ALL: [Self; 14] = [
        Self::SemanticEquals,
        Self::DeepEquals,
        Self::ScalarValue,
        Self::ErrorCode,
        Self::ErrorMessage,
        Self::ResultOk,
        Self::ResultValue,
        Self::ResultError,
        Self::OptionSome,
        Self::OptionValue,
        Self::ValueResultOk,
        Self::ValueResultValue,
        Self::ValueResultError,
        Self::BytesValues,
    ];

    pub const fn name(self) -> &'static str {
        match self {
            Self::SemanticEquals => "semanticEquals",
            Self::DeepEquals => "deepEquals",
            Self::ScalarValue | Self::ResultValue | Self::OptionValue | Self::ValueResultValue => {
                "value"
            }
            Self::ErrorCode => "code",
            Self::ErrorMessage => "message",
            Self::ResultOk | Self::ValueResultOk => "ok",
            Self::ResultError | Self::ValueResultError => "error",
            Self::OptionSome => "some",
            Self::BytesValues => "values",
        }
    }

    fn accepts(self, signature: &JavaMethodSignature) -> bool {
        if !signature.checked_exceptions.is_empty() || signature.nullable_result || !signature.pure
        {
            return false;
        }
        let Some(receiver) = signature.receiver.as_ref() else {
            return false;
        };
        let boolean = JavaType::primitive(JavaPrimitive::Boolean);
        let int = JavaType::primitive(JavaPrimitive::Int);
        let string = JavaType::known(JavaKnownType::String);
        let object = JavaType::known(JavaKnownType::Object);
        let error = JavaType::known(JavaKnownType::RuntimeError);
        let no_arguments = signature.parameters.is_empty();
        match self {
            Self::SemanticEquals | Self::DeepEquals => {
                *receiver == JavaType::known(JavaKnownType::RuntimeSemanticValue)
                    && signature.parameters == [object]
                    && signature.result == boolean
            }
            Self::ScalarValue => {
                *receiver == JavaType::known(JavaKnownType::RuntimeScalar)
                    && no_arguments
                    && signature.result == int
            }
            Self::ErrorCode | Self::ErrorMessage => {
                *receiver == error && no_arguments && signature.result == string
            }
            Self::ResultOk => {
                runtime_generic_arguments(receiver, JavaKnownType::RuntimeResult, 1).is_some()
                    && no_arguments
                    && signature.result == boolean
            }
            Self::ResultValue => {
                runtime_generic_arguments(receiver, JavaKnownType::RuntimeResult, 1).is_some_and(
                    |arguments| {
                        no_arguments && invocation_types_match(&arguments[0], &signature.result)
                    },
                )
            }
            Self::ResultError => {
                runtime_generic_arguments(receiver, JavaKnownType::RuntimeResult, 1).is_some()
                    && no_arguments
                    && signature.result == error
            }
            Self::OptionSome => {
                runtime_generic_arguments(receiver, JavaKnownType::RuntimeOption, 1).is_some()
                    && no_arguments
                    && signature.result == boolean
            }
            Self::OptionValue => {
                runtime_generic_arguments(receiver, JavaKnownType::RuntimeOption, 1).is_some_and(
                    |arguments| {
                        no_arguments && invocation_types_match(&arguments[0], &signature.result)
                    },
                )
            }
            Self::ValueResultOk => {
                runtime_generic_arguments(receiver, JavaKnownType::RuntimeValueResult, 2).is_some()
                    && no_arguments
                    && signature.result == boolean
            }
            Self::ValueResultValue => {
                runtime_generic_arguments(receiver, JavaKnownType::RuntimeValueResult, 2)
                    .is_some_and(|arguments| {
                        no_arguments && invocation_types_match(&arguments[0], &signature.result)
                    })
            }
            Self::ValueResultError => {
                runtime_generic_arguments(receiver, JavaKnownType::RuntimeValueResult, 2)
                    .is_some_and(|arguments| {
                        no_arguments && invocation_types_match(&arguments[1], &signature.result)
                    })
            }
            Self::BytesValues => {
                *receiver == JavaType::known(JavaKnownType::RuntimeBytes)
                    && no_arguments
                    && signature.result
                        == JavaType::Array {
                            component: Box::new(JavaType::primitive(JavaPrimitive::Byte)),
                            ownership: JavaArrayOwnership::DefensiveCopyBoundary,
                        }
            }
        }
    }
}

fn runtime_generic_arguments(
    receiver: &JavaType,
    expected: JavaKnownType,
    arity: usize,
) -> Option<&[JavaType]> {
    match receiver {
        JavaType::Generic {
            raw: JavaTypeName::Known(actual),
            arguments,
        } if *actual == expected && arguments.len() == arity => Some(arguments),
        _ => None,
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum JavaConstructorRef {
    Known {
        constructor: crate::dialect::JavaKnownConstructor,
        owner: JavaType,
        parameters: Vec<JavaType>,
    },
    Generated {
        owner: GeneratedTypeId,
        parameters: Vec<JavaType>,
    },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum JavaFieldRef {
    Known(crate::dialect::JavaKnownField),
    Structural {
        name: JavaIdentifier,
        ty: JavaType,
    },
    Generated {
        owner: GeneratedTypeId,
        field: CoreFieldId,
        name: JavaIdentifier,
        ty: JavaType,
    },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct JavaExpr {
    pub ty: JavaType,
    pub precedence: JavaPrecedence,
    pub kind: JavaExprKind,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum JavaExprKind {
    Literal(JavaLiteral),
    Value(JavaValueRef),
    Unary {
        operator: JavaUnaryOperator,
        operand: Box<JavaExpr>,
    },
    Binary {
        operator: JavaBinaryOperator,
        left: Box<JavaExpr>,
        right: Box<JavaExpr>,
    },
    Conditional {
        condition: Box<JavaExpr>,
        when_true: Box<JavaExpr>,
        when_false: Box<JavaExpr>,
    },
    Call {
        callable: JavaCallableRef,
        receiver: Option<Box<JavaExpr>>,
        arguments: Vec<JavaExpr>,
    },
    New {
        constructor: JavaConstructorRef,
        arguments: Vec<JavaExpr>,
    },
    NewArray {
        component: JavaType,
        length: Box<JavaExpr>,
    },
    ArrayIndex {
        array: Box<JavaExpr>,
        index: Box<JavaExpr>,
    },
    Field {
        receiver: Box<JavaExpr>,
        field: JavaFieldRef,
    },
    Cast {
        target: JavaType,
        value: Box<JavaExpr>,
    },
    ArrayOwnershipTransition {
        transition: JavaArrayOwnershipTransition,
        value: Box<JavaExpr>,
    },
    InstanceOf {
        value: Box<JavaExpr>,
        target: JavaType,
        binding: Option<JavaIdentifier>,
    },
    Lambda {
        parameters: Vec<JavaParameter>,
        body: JavaBlock,
    },
}

impl JavaExpr {
    pub fn literal(ty: JavaType, literal: JavaLiteral) -> Self {
        Self {
            ty,
            precedence: JavaPrecedence::Primary,
            kind: JavaExprKind::Literal(literal),
        }
    }

    pub fn local(ty: JavaType, name: JavaIdentifier) -> Self {
        Self {
            ty,
            precedence: JavaPrecedence::Primary,
            kind: JavaExprKind::Value(JavaValueRef::Local(name)),
        }
    }

    pub fn symbols(&self, symbols: &mut BTreeSet<TargetSymbolRef<JavaDialect>>) {
        self.ty.symbols(symbols);
        match &self.kind {
            JavaExprKind::Literal(_) => {}
            JavaExprKind::Value(value) => match value {
                JavaValueRef::Generated(value) => {
                    symbols.insert(TargetSymbolRef::Generated(*value));
                }
                JavaValueRef::KnownField(value) => {
                    symbols.insert(TargetSymbolRef::KnownField(*value));
                    symbols.insert(TargetSymbolRef::KnownType(value.owner()));
                }
                JavaValueRef::Local(_) | JavaValueRef::This => {}
            },
            JavaExprKind::Unary { operand, .. } => operand.symbols(symbols),
            JavaExprKind::Binary { left, right, .. } => {
                left.symbols(symbols);
                right.symbols(symbols);
            }
            JavaExprKind::Conditional {
                condition,
                when_true,
                when_false,
            } => {
                condition.symbols(symbols);
                when_true.symbols(symbols);
                when_false.symbols(symbols);
            }
            JavaExprKind::Call {
                callable,
                receiver,
                arguments,
            } => {
                match callable {
                    JavaCallableRef::Known { callable, .. } => {
                        symbols.insert(TargetSymbolRef::KnownCallable(*callable));
                        symbols.insert(TargetSymbolRef::KnownType(callable.owner()));
                    }
                    JavaCallableRef::Runtime { callable, .. } => {
                        symbols.insert(TargetSymbolRef::RuntimeCallable(*callable));
                        symbols.insert(TargetSymbolRef::RuntimeHelper(callable.helper()));
                    }
                    JavaCallableRef::Generated { symbol, .. } => {
                        symbols.insert(TargetSymbolRef::Generated(GeneratedSymbolId::Callable(
                            *symbol,
                        )));
                    }
                    JavaCallableRef::Interface { symbol, .. } => {
                        symbols.insert(TargetSymbolRef::Generated(
                            GeneratedSymbolId::InterfaceMethod(*symbol),
                        ));
                    }
                    JavaCallableRef::Member { owner, origin, .. } => {
                        owner.symbols(symbols);
                        if let JavaMemberOrigin::Known(value) = origin {
                            symbols.insert(TargetSymbolRef::KnownMethod(*value));
                        }
                    }
                }
                if let Some(receiver) = receiver {
                    receiver.symbols(symbols);
                }
                for argument in arguments {
                    argument.symbols(symbols);
                }
            }
            JavaExprKind::New {
                constructor,
                arguments,
            } => {
                match constructor {
                    JavaConstructorRef::Known {
                        constructor,
                        owner,
                        parameters,
                    } => {
                        symbols.insert(TargetSymbolRef::KnownConstructor(*constructor));
                        owner.symbols(symbols);
                        for parameter in parameters {
                            parameter.symbols(symbols);
                        }
                    }
                    JavaConstructorRef::Generated { owner, .. } => {
                        symbols.insert(TargetSymbolRef::Generated(GeneratedSymbolId::Type(*owner)));
                    }
                }
                for argument in arguments {
                    argument.symbols(symbols);
                }
            }
            JavaExprKind::NewArray { component, length } => {
                component.symbols(symbols);
                length.symbols(symbols);
            }
            JavaExprKind::ArrayIndex { array, index } => {
                array.symbols(symbols);
                index.symbols(symbols);
            }
            JavaExprKind::Field { receiver, field } => {
                receiver.symbols(symbols);
                match field {
                    JavaFieldRef::Known(value) => {
                        symbols.insert(TargetSymbolRef::KnownField(*value));
                        symbols.insert(TargetSymbolRef::KnownType(value.owner()));
                    }
                    JavaFieldRef::Structural { ty, .. } => ty.symbols(symbols),
                    JavaFieldRef::Generated { owner, ty, .. } => {
                        symbols.insert(TargetSymbolRef::Generated(GeneratedSymbolId::Type(*owner)));
                        ty.symbols(symbols);
                    }
                }
            }
            JavaExprKind::Cast { target, value } => {
                target.symbols(symbols);
                value.symbols(symbols);
            }
            JavaExprKind::ArrayOwnershipTransition { value, .. } => value.symbols(symbols),
            JavaExprKind::InstanceOf { value, target, .. } => {
                value.symbols(symbols);
                target.symbols(symbols);
            }
            JavaExprKind::Lambda { parameters, body } => {
                for parameter in parameters {
                    parameter.ty.symbols(symbols);
                }
                body.symbols(symbols);
            }
        }
    }

    pub fn verify(&self, context: &TargetAstContext<'_, JavaDialect>) -> Vec<AstViolation> {
        let expression_type_use = if matches!(self.kind, JavaExprKind::Call { .. })
            && matches!(self.ty, JavaType::Wildcard { .. })
        {
            JavaTypeUse::GenericArgument
        } else {
            JavaTypeUse::Value
        };
        let mut violations = self.ty.verify(expression_type_use);
        match &self.kind {
            JavaExprKind::Literal(JavaLiteral::CharScalar(value))
                if char::from_u32(*value).is_none() =>
            {
                violations.push(type_error("character literal is not a Unicode scalar"));
            }
            JavaExprKind::Literal(literal) => {
                if !literal_matches_type(literal, &self.ty) {
                    violations.push(type_error("literal does not match its declared Java type"));
                }
            }
            JavaExprKind::Value(value) => match value {
                JavaValueRef::Generated(GeneratedSymbolId::Value(id)) => {
                    match generated_value_matches(*id, &self.ty, context) {
                        Some(true) => {}
                        Some(false) => violations.push(type_error(
                            "generated Java value reference type disagrees with its authoritative registration",
                        )),
                        None => violations.push(AstViolation::new(
                        DiagnosticCode::UnresolvedReference,
                        "generated Java value reference is not registered",
                        )),
                    }
                }
                JavaValueRef::Generated(_) => violations.push(AstViolation::new(
                    DiagnosticCode::UnresolvedReference,
                    "Java value expression references a generated symbol from the wrong category",
                )),
                JavaValueRef::KnownField(field) if self.ty != field.ty() => {
                    violations.push(type_error(
                        "known Java field reference type disagrees with its catalogue entry",
                    ));
                }
                JavaValueRef::KnownField(_) | JavaValueRef::Local(_) | JavaValueRef::This => {}
            },
            JavaExprKind::Unary { operator, operand } => {
                violations.extend(operand.verify(context));
                if !unary_signature_matches(*operator, &operand.ty, &self.ty) {
                    violations.push(type_error("unary operator type mismatch"));
                }
            }
            JavaExprKind::Binary {
                operator,
                left,
                right,
            } => {
                violations.extend(left.verify(context));
                violations.extend(right.verify(context));
                if !binary_signature_matches(*operator, &left.ty, &right.ty, &self.ty) {
                    violations.push(type_error("binary operator type mismatch"));
                }
            }
            JavaExprKind::Conditional {
                condition,
                when_true,
                when_false,
            } => {
                violations.extend(condition.verify(context));
                violations.extend(when_true.verify(context));
                violations.extend(when_false.verify(context));
                if condition.ty != JavaType::Primitive(JavaPrimitive::Boolean)
                    || when_true.ty != self.ty
                    || when_false.ty != self.ty
                {
                    violations.push(type_error("conditional operand type mismatch"));
                }
            }
            JavaExprKind::Call {
                callable,
                receiver,
                arguments,
            } => {
                match callable.signature(context) {
                    Some(signature) => verify_call(
                        &mut violations,
                        &signature,
                        receiver.as_deref(),
                        arguments,
                        &self.ty,
                        context,
                    ),
                    None => violations.push(AstViolation::new(
                        DiagnosticCode::UnresolvedReference,
                        format!(
                            "callable is absent or violates its catalogue signature: {callable:?}"
                        ),
                    )),
                }
                if let Some(receiver) = receiver {
                    violations.extend(receiver.verify(context));
                }
                for argument in arguments {
                    violations.extend(argument.verify(context));
                }
            }
            JavaExprKind::New {
                constructor,
                arguments,
            } => {
                match constructor.signature(context) {
                    Some((owner, parameters))
                        if owner == self.ty
                            && parameters.len() == arguments.len()
                            && parameters.iter().zip(arguments).all(|(expected, actual)| {
                                invocation_types_match(expected, &actual.ty)
                            }) => {}
                    _ => {
                        violations.push(AstViolation::new(
                            DiagnosticCode::InvalidInvocation,
                            "constructor arguments do not match its authoritative signature",
                        ));
                    }
                }
                for argument in arguments {
                    violations.extend(argument.verify(context));
                }
            }
            JavaExprKind::NewArray { component, length } => {
                violations.extend(component.verify(JavaTypeUse::Value));
                violations.extend(length.verify(context));
                let expected = JavaType::Array {
                    component: Box::new(component.clone()),
                    ownership: JavaArrayOwnership::InternalMutable,
                };
                if self.ty != expected || length.ty != JavaType::Primitive(JavaPrimitive::Int) {
                    violations.push(type_error(
                        "new-array component, length, or result type mismatch",
                    ));
                }
            }
            JavaExprKind::ArrayIndex { array, index } => {
                violations.extend(array.verify(context));
                violations.extend(index.verify(context));
                let component = match &array.ty {
                    JavaType::Array { component, .. } => Some(component.as_ref()),
                    _ => None,
                };
                if component != Some(&self.ty)
                    || index.ty != JavaType::Primitive(JavaPrimitive::Int)
                {
                    violations.push(type_error("array-index operand or result type mismatch"));
                }
            }
            JavaExprKind::Field { receiver, field } => {
                violations.extend(receiver.verify(context));
                if field.ty() != self.ty {
                    violations.push(type_error("field result type mismatch"));
                }
                match field {
                    JavaFieldRef::Known(value) => {
                        if receiver.ty != JavaType::known(value.owner()) {
                            violations.push(AstViolation::new(
                                DiagnosticCode::UnresolvedReference,
                                "known Java field receiver does not match its catalogue owner",
                            ));
                        }
                    }
                    JavaFieldRef::Structural { name, ty } => {
                        let metadata = structural_field_metadata(&receiver.ty, name, context);
                        let deferred_runtime_owner = metadata.is_none()
                            && matches!(
                                receiver.ty,
                                JavaType::Reference(JavaTypeName::Known(known))
                                    | JavaType::Generic {
                                        raw: JavaTypeName::Known(known),
                                        ..
                                    } if known.runtime_helper().is_some()
                            );
                        if metadata
                            .as_ref()
                            .is_some_and(|metadata| metadata.ty != *ty)
                            || (metadata.is_none() && !deferred_runtime_owner)
                        {
                            violations.push(AstViolation::new(
                                DiagnosticCode::UnresolvedReference,
                                format!(
                                    "structural Java field reference {:?}.{} does not name a field with type {ty:?}",
                                    receiver.ty,
                                    name.as_str()
                                ),
                            ));
                        }
                    }
                    JavaFieldRef::Generated {
                        owner,
                        field,
                        name,
                        ty,
                    } => {
                        if receiver.ty
                            != JavaType::Reference(JavaTypeName::Generated(*owner))
                            || !generated_field_matches(*owner, *field, name, ty, context)
                        {
                            violations.push(AstViolation::new(
                                DiagnosticCode::UnresolvedReference,
                                "generated Java field reference does not match its declared receiver/owner/name/type",
                            ));
                        }
                    }
                }
            }
            JavaExprKind::Cast { target, value } => {
                violations.extend(target.verify(JavaTypeUse::Value));
                violations.extend(value.verify(context));
                if target != &self.ty {
                    violations.push(type_error("cast target type mismatch"));
                }
                if !java_cast_is_legal(target, &value.ty, context) {
                    violations.push(type_error(
                        "Java cast is not legal between the declared source and target types",
                    ));
                }
            }
            JavaExprKind::ArrayOwnershipTransition { transition, value } => {
                violations.extend(value.verify(context));
                let valid = match transition {
                    JavaArrayOwnershipTransition::FreshCopyToBoundary => matches!(
                        (&value.ty, &self.ty),
                        (
                            JavaType::Array {
                                component: source,
                                ownership: JavaArrayOwnership::InternalMutable,
                            },
                            JavaType::Array {
                                component: target,
                                ownership: JavaArrayOwnership::DefensiveCopyBoundary,
                            },
                        ) if source == target
                    ),
                };
                if !valid {
                    violations.push(type_error("invalid Java array ownership transition"));
                }
            }
            JavaExprKind::InstanceOf { value, target, .. } => {
                violations.extend(value.verify(context));
                violations.extend(target.verify(JavaTypeUse::Value));
                if self.ty != JavaType::Primitive(JavaPrimitive::Boolean) {
                    violations.push(type_error("instanceof result must be boolean"));
                }
                if !java_type_is_reifiable(target) {
                    violations.push(type_error("Java instanceof target must be reifiable"));
                }
                if !java_instanceof_is_legal(&value.ty, target, context) {
                    violations.push(type_error(&format!(
                        "Java instanceof is not legal from {:?} to {target:?}",
                        value.ty
                    )));
                }
            }
            JavaExprKind::Lambda { parameters, body } => {
                for parameter in parameters {
                    violations.extend(parameter.ty.verify(JavaTypeUse::Parameter));
                }
                violations.extend(body.verify(context));
                violations.push(type_error(
                    "lambda targets are not part of the portable Java dialect",
                ));
            }
        }
        violations
    }
}

fn literal_matches_type(literal: &JavaLiteral, ty: &JavaType) -> bool {
    match literal {
        JavaLiteral::Boolean(_) => *ty == JavaType::Primitive(JavaPrimitive::Boolean),
        JavaLiteral::I32(_) => *ty == JavaType::Primitive(JavaPrimitive::Int),
        JavaLiteral::I64(_) => *ty == JavaType::Primitive(JavaPrimitive::Long),
        JavaLiteral::CharScalar(_) => *ty == JavaType::primitive(JavaPrimitive::Int),
        JavaLiteral::String(_) | JavaLiteral::Utf16Units(_) => {
            *ty == JavaType::known(JavaKnownType::String)
        }
        JavaLiteral::InternalNull(_) => matches!(
            ty,
            JavaType::Reference(_)
                | JavaType::Array { .. }
                | JavaType::Generic { .. }
                | JavaType::Wildcard { .. }
                | JavaType::TypeVariable(_)
        ),
    }
}

fn unary_signature_matches(
    operator: JavaUnaryOperator,
    operand: &JavaType,
    result: &JavaType,
) -> bool {
    match operator {
        JavaUnaryOperator::Not => {
            *operand == JavaType::Primitive(JavaPrimitive::Boolean) && operand == result
        }
        JavaUnaryOperator::Negate => is_numeric_primitive(operand) && operand == result,
        JavaUnaryOperator::BitNot => is_integral_primitive(operand) && operand == result,
    }
}

fn binary_signature_matches(
    operator: JavaBinaryOperator,
    left: &JavaType,
    right: &JavaType,
    result: &JavaType,
) -> bool {
    let boolean = JavaType::Primitive(JavaPrimitive::Boolean);
    match operator {
        JavaBinaryOperator::LogicalAnd | JavaBinaryOperator::LogicalOr => {
            left == &boolean && right == &boolean && result == &boolean
        }
        JavaBinaryOperator::Equal | JavaBinaryOperator::NotEqual => {
            invocation_types_match(left, right) && result == &boolean
        }
        JavaBinaryOperator::Less
        | JavaBinaryOperator::LessEqual
        | JavaBinaryOperator::Greater
        | JavaBinaryOperator::GreaterEqual => {
            is_numeric_primitive(left) && left == right && result == &boolean
        }
        JavaBinaryOperator::Add
            if left == &JavaType::known(JavaKnownType::String)
                && right == &JavaType::known(JavaKnownType::String) =>
        {
            result == left
        }
        JavaBinaryOperator::Add
        | JavaBinaryOperator::Subtract
        | JavaBinaryOperator::Multiply
        | JavaBinaryOperator::Divide
        | JavaBinaryOperator::Remainder => {
            is_numeric_primitive(left) && left == right && result == left
        }
        JavaBinaryOperator::BitAnd | JavaBinaryOperator::BitOr | JavaBinaryOperator::BitXor => {
            (is_integral_primitive(left) || left == &boolean) && left == right && result == left
        }
        JavaBinaryOperator::ShiftLeft | JavaBinaryOperator::ShiftRight => {
            is_integral_primitive(left)
                && *right == JavaType::Primitive(JavaPrimitive::Int)
                && result == left
        }
    }
}

fn is_numeric_primitive(ty: &JavaType) -> bool {
    matches!(
        ty,
        JavaType::Primitive(JavaPrimitive::Int | JavaPrimitive::Long | JavaPrimitive::Double)
    )
}

fn is_integral_primitive(ty: &JavaType) -> bool {
    matches!(
        ty,
        JavaType::Primitive(JavaPrimitive::Int | JavaPrimitive::Long)
    )
}

fn invocation_types_match(left: &JavaType, right: &JavaType) -> bool {
    left == right
        || matches!(
            (left, right),
            (JavaType::Primitive(left), JavaType::Boxed(right))
                | (JavaType::Boxed(left), JavaType::Primitive(right))
                if left == right
        )
}

fn verify_call(
    violations: &mut Vec<AstViolation>,
    signature: &JavaMethodSignature,
    receiver: Option<&JavaExpr>,
    arguments: &[JavaExpr],
    result: &JavaType,
    context: &TargetAstContext<'_, JavaDialect>,
) {
    let receiver_valid = match (&signature.receiver, receiver) {
        (None, None) => true,
        (Some(expected), Some(actual)) => {
            invocation_types_match_in_context(expected, &actual.ty, context)
        }
        _ => false,
    };
    if !receiver_valid
        || signature.parameters.len() != arguments.len()
        || signature
            .parameters
            .iter()
            .zip(arguments)
            .any(|(a, b)| !invocation_types_match_in_context(a, &b.ty, context))
        || &signature.result != result
    {
        violations.push(AstViolation::new(
            DiagnosticCode::InvalidInvocation,
            "call does not match its authoritative owner/receiver/parameter/result signature",
        ));
    }
}

fn invocation_types_match_in_context(
    expected: &JavaType,
    actual: &JavaType,
    context: &TargetAstContext<'_, JavaDialect>,
) -> bool {
    invocation_types_match(expected, actual)
        || matches!(
            (expected, actual),
            (
                JavaType::Reference(JavaTypeName::Generated(expected_interface)),
                JavaType::Reference(JavaTypeName::Generated(actual_type)),
            ) if generated_type_implements(*actual_type, *expected_interface, context)
        )
}

fn generated_type_implements(
    actual: GeneratedTypeId,
    expected_interface: GeneratedTypeId,
    context: &TargetAstContext<'_, JavaDialect>,
) -> bool {
    context.files().any(|file| {
        file.items().iter().any(|item| {
            matches!(item, JavaFileItem::Type { declaration, .. }
            if declaration_contains_conformance(
                declaration,
                actual,
                expected_interface,
            ))
        })
    })
}

fn declaration_contains_conformance(
    declaration: &JavaTypeDeclaration,
    actual: GeneratedTypeId,
    expected_interface: GeneratedTypeId,
) -> bool {
    (declaration.declared == Some(actual)
        && matches!(&declaration.heritage, JavaHeritage::Interfaces(values)
            if values.contains(&JavaType::Reference(JavaTypeName::Generated(expected_interface)))))
        || declaration.members.iter().any(|member| {
            matches!(member, JavaMember::NestedType(nested)
                if declaration_contains_conformance(nested, actual, expected_interface))
        })
}

fn java_cast_is_legal(
    target: &JavaType,
    source: &JavaType,
    context: &TargetAstContext<'_, JavaDialect>,
) -> bool {
    if target == source {
        return !matches!(target, JavaType::Primitive(JavaPrimitive::Void));
    }
    match (target, source) {
        (JavaType::Primitive(target), JavaType::Primitive(source)) => {
            java_numeric_primitive(*target) && java_numeric_primitive(*source)
        }
        (JavaType::Primitive(target), JavaType::Boxed(source))
        | (JavaType::Boxed(source), JavaType::Primitive(target)) => {
            (*target == *source && *target != JavaPrimitive::Void)
                || (java_numeric_primitive(*target) && java_numeric_primitive(*source))
        }
        (JavaType::Reference(JavaTypeName::Known(JavaKnownType::Object)), source) => {
            !matches!(source, JavaType::Primitive(JavaPrimitive::Void))
        }
        (target, JavaType::Reference(JavaTypeName::Known(JavaKnownType::Object))) => !matches!(
            target,
            JavaType::Primitive(JavaPrimitive::Void)
                | JavaType::Wildcard { .. }
                | JavaType::TypeVariable(_)
        ),
        (
            JavaType::Array {
                component: target, ..
            },
            JavaType::Array {
                component: source, ..
            },
        ) => target == source || java_reference_cast_is_legal(target, source, context),
        (target, source) => java_reference_cast_is_legal(target, source, context),
    }
}

fn java_reference_cast_is_legal(
    target: &JavaType,
    source: &JavaType,
    context: &TargetAstContext<'_, JavaDialect>,
) -> bool {
    if !java_type_is_reference(target) || !java_type_is_reference(source) {
        return false;
    }
    if matches!(source, JavaType::TypeVariable(_)) && !matches!(target, JavaType::TypeVariable(_)) {
        return true;
    }
    if erased_java_type(target) == erased_java_type(source) {
        return target == source;
    }
    invocation_types_match_in_context(target, source, context)
        || invocation_types_match_in_context(source, target, context)
        || generated_and_known_interface_related(target, source, context)
        || generated_and_known_interface_related(source, target, context)
}

fn generated_and_known_interface_related(
    expected_interface: &JavaType,
    actual: &JavaType,
    context: &TargetAstContext<'_, JavaDialect>,
) -> bool {
    let JavaType::Reference(JavaTypeName::Known(JavaKnownType::RuntimeSemanticValue)) =
        expected_interface
    else {
        return false;
    };
    find_type_declaration(actual, context).is_some_and(|declaration| {
        matches!(
            declaration.heritage,
            JavaHeritage::Interfaces(ref interfaces)
                if interfaces.contains(expected_interface)
        )
    })
}

fn java_instanceof_is_legal(
    source: &JavaType,
    target: &JavaType,
    context: &TargetAstContext<'_, JavaDialect>,
) -> bool {
    java_type_is_reference(source)
        && java_type_is_reference(target)
        && java_cast_is_legal(target, source, context)
}

fn java_type_is_reference(ty: &JavaType) -> bool {
    matches!(
        ty,
        JavaType::Boxed(_)
            | JavaType::Reference(_)
            | JavaType::Array { .. }
            | JavaType::Generic { .. }
            | JavaType::TypeVariable(_)
    )
}

fn java_type_is_reifiable(ty: &JavaType) -> bool {
    match ty {
        JavaType::Boxed(_) | JavaType::Reference(_) => true,
        JavaType::Array { component, .. } => java_type_is_reifiable(component),
        JavaType::Generic { arguments, .. } => arguments
            .iter()
            .all(|argument| matches!(argument, JavaType::Wildcard { bound: None })),
        JavaType::Primitive(_) | JavaType::Wildcard { .. } | JavaType::TypeVariable(_) => false,
    }
}

fn java_numeric_primitive(value: JavaPrimitive) -> bool {
    matches!(
        value,
        JavaPrimitive::Byte
            | JavaPrimitive::Char
            | JavaPrimitive::Int
            | JavaPrimitive::Long
            | JavaPrimitive::Double
    )
}

impl JavaCallableRef {
    fn signature(
        &self,
        context: &TargetAstContext<'_, JavaDialect>,
    ) -> Option<JavaMethodSignature> {
        match self {
            Self::Known {
                callable,
                signature,
            } => callable.accepts(signature).then(|| signature.clone()),
            Self::Runtime {
                callable,
                signature,
            } => callable.accepts(signature).then(|| signature.clone()),
            Self::Generated { symbol, signature } => {
                let registered = context.callable_signature(&TargetCallableRef::Generated(*symbol));
                let actual = JavaDialect.coarse_signature(signature);
                (registered.as_ref() == Some(&actual)
                    && signature.checked_exceptions.is_empty()
                    && !signature.nullable_result
                    && signature.pure)
                    .then(|| signature.clone())
            }
            Self::Interface { symbol, signature } => context
                .callable_signature(&TargetCallableRef::Interface(*symbol))
                .is_some_and(|registered| {
                    registered == JavaDialect.coarse_signature(signature)
                        && signature.checked_exceptions.is_empty()
                        && !signature.nullable_result
                        && signature.pure
                })
                .then(|| signature.clone()),
            Self::Member {
                owner,
                name,
                signature,
                origin,
            } => {
                let owner_matches = signature.receiver.as_ref() == Some(owner);
                let catalogue_matches = match origin {
                    JavaMemberOrigin::Known(method) => method.accepts(signature),
                    JavaMemberOrigin::GeneratedField(field) => {
                        portable_member_metadata_matches(signature)
                            && generated_accessor_matches(owner, *field, name, signature, context)
                    }
                    JavaMemberOrigin::Runtime(member) => {
                        name.as_str() == member.name() && member.accepts(signature)
                    }
                    JavaMemberOrigin::GeneratedImplementation(method) => {
                        portable_member_metadata_matches(signature)
                            && generated_member_matches(
                                owner,
                                name,
                                signature,
                                Some(*method),
                                context,
                            )
                    }
                    JavaMemberOrigin::GeneratedVariant => false,
                };
                (owner_matches && catalogue_matches).then(|| signature.clone())
            }
        }
    }
}

fn portable_member_metadata_matches(signature: &JavaMethodSignature) -> bool {
    signature.checked_exceptions.is_empty() && !signature.nullable_result && signature.pure
}

impl JavaConstructorRef {
    fn signature(
        &self,
        context: &TargetAstContext<'_, JavaDialect>,
    ) -> Option<(JavaType, Vec<JavaType>)> {
        match self {
            Self::Known {
                constructor,
                owner,
                parameters,
            } => constructor
                .accepts(owner, parameters)
                .then(|| (owner.clone(), parameters.clone())),
            Self::Generated { owner, parameters }
                if generated_constructor_matches(*owner, parameters, context) =>
            {
                Some((
                    JavaType::Reference(JavaTypeName::Generated(*owner)),
                    parameters.clone(),
                ))
            }
            Self::Generated { .. } => None,
        }
    }
}

impl JavaFieldRef {
    fn ty(&self) -> JavaType {
        match self {
            Self::Known(value) => value.ty(),
            Self::Structural { ty, .. } => ty.clone(),
            Self::Generated { ty, .. } => ty.clone(),
        }
    }
}

#[derive(Clone, Debug)]
struct JavaFieldMetadata {
    ty: JavaType,
    final_field: bool,
    blank_final: bool,
}

fn structural_field_metadata(
    owner: &JavaType,
    name: &JavaIdentifier,
    context: &TargetAstContext<'_, JavaDialect>,
) -> Option<JavaFieldMetadata> {
    if matches!(owner, JavaType::Array { .. }) && name.as_str() == "length" {
        return Some(JavaFieldMetadata {
            ty: JavaType::primitive(JavaPrimitive::Int),
            final_field: true,
            blank_final: false,
        });
    }
    let declaration = find_type_declaration(owner, context)?;
    let substitutions = declaration_type_substitutions(&declaration, owner)?;
    if let Some(component) = declaration
        .record_components
        .iter()
        .find(|component| component.name == *name)
    {
        return Some(JavaFieldMetadata {
            ty: substitute_type_variables(&component.ty, &substitutions),
            final_field: true,
            blank_final: true,
        });
    }
    declaration.members.iter().find_map(|member| match member {
        JavaMember::Field(field)
            if field.name == *name && !field.modifiers.contains(&JavaModifier::Static) =>
        {
            Some(JavaFieldMetadata {
                ty: substitute_type_variables(&field.ty, &substitutions),
                final_field: field.modifiers.contains(&JavaModifier::Final),
                blank_final: field.modifiers.contains(&JavaModifier::Final)
                    && field.initializer.is_none(),
            })
        }
        _ => None,
    })
}

fn declaration_type_substitutions(
    declaration: &JavaTypeDeclaration,
    owner: &JavaType,
) -> Option<BTreeMap<JavaIdentifier, JavaType>> {
    match owner {
        JavaType::Generic { arguments, .. }
            if arguments.len() == declaration.type_parameters.len() =>
        {
            Some(
                declaration
                    .type_parameters
                    .iter()
                    .cloned()
                    .zip(arguments.iter().cloned())
                    .collect(),
            )
        }
        JavaType::Reference(_) if declaration.type_parameters.is_empty() => Some(BTreeMap::new()),
        _ => None,
    }
}

fn substitute_type_variables(
    ty: &JavaType,
    substitutions: &BTreeMap<JavaIdentifier, JavaType>,
) -> JavaType {
    match ty {
        JavaType::TypeVariable(name) => substitutions
            .get(name)
            .cloned()
            .unwrap_or_else(|| ty.clone()),
        JavaType::Array {
            component,
            ownership,
        } => JavaType::Array {
            component: Box::new(substitute_type_variables(component, substitutions)),
            ownership: *ownership,
        },
        JavaType::Generic { raw, arguments } => JavaType::Generic {
            raw: raw.clone(),
            arguments: arguments
                .iter()
                .map(|argument| substitute_type_variables(argument, substitutions))
                .collect(),
        },
        JavaType::Wildcard {
            bound: Some((kind, bound)),
        } => JavaType::Wildcard {
            bound: Some((
                *kind,
                Box::new(substitute_type_variables(bound, substitutions)),
            )),
        },
        JavaType::Primitive(_)
        | JavaType::Boxed(_)
        | JavaType::Reference(_)
        | JavaType::Wildcard { bound: None } => ty.clone(),
    }
}

fn find_type_declaration(
    owner: &JavaType,
    context: &TargetAstContext<'_, JavaDialect>,
) -> Option<JavaTypeDeclaration> {
    context.files().find_map(|file| {
        file.items().iter().find_map(|item| match item {
            JavaFileItem::Type { declaration, .. } => find_type_declaration_in(declaration, owner),
            JavaFileItem::RuntimeMembers { members, .. } => {
                members.iter().find_map(|member| match member {
                    JavaMember::NestedType(nested) => find_type_declaration_in(nested, owner),
                    _ => None,
                })
            }
        })
    })
}

fn find_type_declaration_in(
    declaration: &JavaTypeDeclaration,
    owner: &JavaType,
) -> Option<JavaTypeDeclaration> {
    if declaration_represents_type(declaration, owner) {
        return Some(declaration.clone());
    }
    declaration.members.iter().find_map(|member| match member {
        JavaMember::NestedType(nested) => find_type_declaration_in(nested, owner),
        _ => None,
    })
}

fn declaration_represents_type(declaration: &JavaTypeDeclaration, owner: &JavaType) -> bool {
    match owner {
        JavaType::Reference(JavaTypeName::Generated(id))
        | JavaType::Generic {
            raw: JavaTypeName::Generated(id),
            ..
        } => declaration.declared == Some(*id),
        JavaType::Reference(JavaTypeName::Known(known))
        | JavaType::Generic {
            raw: JavaTypeName::Known(known),
            ..
        } if known.runtime_helper().is_some() => known.simple_name() == declaration.name.as_str(),
        _ => false,
    }
}

fn generated_constructor_matches(
    owner: GeneratedTypeId,
    parameters: &[JavaType],
    context: &TargetAstContext<'_, JavaDialect>,
) -> bool {
    context.files().any(|file| {
        file.items().iter().any(|item| {
            matches!(item, JavaFileItem::Type { declaration, .. }
                if declaration_has_constructor(declaration, owner, parameters))
        })
    })
}

fn generated_value_matches(
    value: GeneratedValueId,
    ty: &JavaType,
    context: &TargetAstContext<'_, JavaDialect>,
) -> Option<bool> {
    let registered = context.value(value)?;
    let declared_type = context.files().find_map(|file| {
        file.items().iter().find_map(|item| {
            let JavaFileItem::Type { declaration, .. } = item else {
                return None;
            };
            find_generated_value_type(declaration, value)
        })
    });
    Some(
        registered.ty == JavaDialect.coarse_type(ty)
            && declared_type.is_some_and(|declared| declared == *ty),
    )
}

fn find_generated_value_type(
    declaration: &JavaTypeDeclaration,
    value: GeneratedValueId,
) -> Option<JavaType> {
    declaration.members.iter().find_map(|member| match member {
        JavaMember::Field(field) if field.declared == Some(value) => Some(field.ty.clone()),
        JavaMember::NestedType(nested) => find_generated_value_type(nested, value),
        _ => None,
    })
}

fn declaration_has_constructor(
    declaration: &JavaTypeDeclaration,
    owner: GeneratedTypeId,
    parameters: &[JavaType],
) -> bool {
    (declaration.declared == Some(owner)
        && declaration.members.iter().any(|member| {
            matches!(member, JavaMember::Constructor(constructor)
                if constructor.name == declaration.name
                    && constructor.parameters.iter().map(|value| &value.ty).eq(parameters.iter()))
        }))
        || declaration.members.iter().any(|member| {
            matches!(member, JavaMember::NestedType(nested)
                if declaration_has_constructor(nested, owner, parameters))
        })
}

fn generated_field_matches(
    owner: GeneratedTypeId,
    field: CoreFieldId,
    name: &JavaIdentifier,
    ty: &JavaType,
    context: &TargetAstContext<'_, JavaDialect>,
) -> bool {
    context.files().any(|file| {
        file.items().iter().any(|item| {
            matches!(item, JavaFileItem::Type { declaration, .. }
                if declaration_has_generated_field(declaration, owner, field, name, ty))
        })
    })
}

fn declaration_has_generated_field(
    declaration: &JavaTypeDeclaration,
    owner: GeneratedTypeId,
    field: CoreFieldId,
    name: &JavaIdentifier,
    ty: &JavaType,
) -> bool {
    (declaration.declared == Some(owner)
        && declaration.record_components.iter().any(|component| {
            component.origin == JavaRecordComponentOrigin::Core(field)
                && &component.name == name
                && &component.ty == ty
        }))
        || declaration.members.iter().any(|member| {
            matches!(member, JavaMember::NestedType(nested)
                if declaration_has_generated_field(nested, owner, field, name, ty))
        })
}

fn generated_accessor_matches(
    owner: &JavaType,
    field: CoreFieldId,
    name: &JavaIdentifier,
    signature: &JavaMethodSignature,
    context: &TargetAstContext<'_, JavaDialect>,
) -> bool {
    let JavaType::Reference(JavaTypeName::Generated(owner_id)) = owner else {
        return false;
    };
    signature.receiver.as_ref() == Some(owner)
        && signature.parameters.is_empty()
        && generated_field_matches(*owner_id, field, name, &signature.result, context)
}

fn generated_member_matches(
    owner: &JavaType,
    name: &JavaIdentifier,
    signature: &JavaMethodSignature,
    implementation: Option<portable_core_ir::CoreImplementationMethodId>,
    context: &TargetAstContext<'_, JavaDialect>,
) -> bool {
    context.files().any(|file| {
        file.items().iter().any(|item| {
            matches!(item, JavaFileItem::Type { declaration, .. }
            if declaration_has_member(
                declaration,
                owner,
                name,
                signature,
                implementation,
            ))
        })
    })
}

fn declaration_has_member(
    declaration: &JavaTypeDeclaration,
    owner: &JavaType,
    name: &JavaIdentifier,
    signature: &JavaMethodSignature,
    implementation: Option<portable_core_ir::CoreImplementationMethodId>,
) -> bool {
    let represents_owner = match owner {
        JavaType::Reference(JavaTypeName::Generated(id)) => declaration.declared == Some(*id),
        JavaType::Reference(JavaTypeName::Known(known))
        | JavaType::Generic {
            raw: JavaTypeName::Known(known),
            ..
        } if known.runtime_helper().is_some() => known
            .qualified_name()
            .rsplit('.')
            .next()
            .is_some_and(|simple| simple == declaration.name.as_str()),
        _ => false,
    };
    let member_matches = represents_owner
        && declaration.members.iter().any(|member| match member {
            JavaMember::Method(method) => {
                let origin_matches = match (implementation, method.declared) {
                    (Some(expected), JavaMethodDeclaration::Implementation { method, .. }) => {
                        expected == method
                    }
                    (None, JavaMethodDeclaration::Structural) => true,
                    _ => false,
                };
                origin_matches
                    && method.name == *name
                    && !method.modifiers.contains(&JavaModifier::Static)
                    && method
                        .parameters
                        .iter()
                        .map(|value| &value.ty)
                        .eq(signature.parameters.iter())
                    && method.return_type == signature.result
            }
            _ => false,
        });
    let implicit_record_accessor = represents_owner
        && implementation.is_none()
        && signature.parameters.is_empty()
        && declaration
            .record_components
            .iter()
            .any(|component| component.name == *name && component.ty == signature.result);
    member_matches
        || implicit_record_accessor
        || declaration.members.iter().any(|member| {
            matches!(member, JavaMember::NestedType(nested)
            if declaration_has_member(
                nested,
                owner,
                name,
                signature,
                implementation,
            ))
        })
}

impl TargetExpressionNode<JavaDialect> for JavaExpr {
    fn child_expressions(&self) -> Vec<portable_codegen::TargetExprId> {
        vec![]
    }

    fn verify(
        &self,
        stored_type: &TargetTypeRef<JavaDialect>,
        context: &TargetAstContext<'_, JavaDialect>,
    ) -> Vec<AstViolation> {
        let mut violations = self.verify(context);
        if &context.dialect().coarse_type(&self.ty) != stored_type {
            violations.push(type_error(
                "Java type disagrees with shared target-AST type",
            ));
        }
        violations
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum JavaLocalFinality {
    Final,
    Mutable,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct JavaBlock {
    pub statements: Vec<JavaStmt>,
}

impl JavaBlock {
    pub fn new(statements: Vec<JavaStmt>) -> Self {
        Self { statements }
    }
    pub fn symbols(&self, symbols: &mut BTreeSet<TargetSymbolRef<JavaDialect>>) {
        for statement in &self.statements {
            statement.symbols(symbols);
        }
    }
    pub fn verify(&self, context: &TargetAstContext<'_, JavaDialect>) -> Vec<AstViolation> {
        self.statements
            .iter()
            .flat_map(|value| value.verify(context))
            .collect()
    }
}

#[derive(Clone, Debug)]
struct JavaLexicalBinding {
    ty: JavaType,
    mutable: bool,
}

#[derive(Clone, Debug)]
struct JavaLexicalScope {
    bindings: BTreeMap<JavaIdentifier, JavaLexicalBinding>,
    allows_this: bool,
    owner: Option<JavaType>,
    constructor: bool,
    owner_fields: BTreeMap<JavaIdentifier, JavaFieldMetadata>,
}

impl JavaLexicalScope {
    fn bind(
        &mut self,
        name: JavaIdentifier,
        binding: JavaLexicalBinding,
        duplicate_message: &'static str,
        violations: &mut Vec<AstViolation>,
    ) -> bool {
        if self.bindings.contains_key(&name) {
            violations.push(AstViolation::new(
                DiagnosticCode::DuplicateDeclaration,
                duplicate_message,
            ));
            return false;
        }
        self.bindings.insert(name, binding);
        true
    }

    #[cfg(test)]
    fn for_method(method: &JavaMethod) -> (Self, Vec<AstViolation>) {
        Self::for_method_in_owner(method, None)
    }

    #[cfg(test)]
    fn for_method_in_owner(
        method: &JavaMethod,
        owner: Option<JavaType>,
    ) -> (Self, Vec<AstViolation>) {
        Self::for_method_in_declaration(method, owner, None)
    }

    fn for_method_in_declaration(
        method: &JavaMethod,
        owner: Option<JavaType>,
        declaration: Option<&JavaTypeDeclaration>,
    ) -> (Self, Vec<AstViolation>) {
        let mut scope = Self {
            bindings: BTreeMap::new(),
            allows_this: !method.modifiers.contains(&JavaModifier::Static),
            owner,
            constructor: false,
            owner_fields: declaration
                .map(declared_instance_fields)
                .unwrap_or_default(),
        };
        let mut violations = Vec::new();
        for parameter in &method.parameters {
            scope.bind(
                parameter.name.clone(),
                JavaLexicalBinding {
                    ty: parameter.ty.clone(),
                    mutable: !parameter.final_parameter,
                },
                "Java method parameter is declared more than once",
                &mut violations,
            );
        }
        (scope, violations)
    }

    fn for_constructor_in_declaration(
        constructor: &JavaConstructor,
        owner: Option<JavaType>,
        declaration: Option<&JavaTypeDeclaration>,
    ) -> (Self, Vec<AstViolation>) {
        let method = JavaMethod {
            declared: JavaMethodDeclaration::Structural,
            annotations: vec![],
            modifiers: constructor.modifiers.clone(),
            type_parameters: vec![],
            return_type: JavaType::primitive(JavaPrimitive::Void),
            name: constructor.name.clone(),
            parameters: constructor.parameters.clone(),
            body: None,
        };
        let (mut scope, violations) = Self::for_method_in_declaration(&method, owner, declaration);
        scope.allows_this = true;
        scope.constructor = true;
        (scope, violations)
    }
}

fn declared_instance_fields(
    declaration: &JavaTypeDeclaration,
) -> BTreeMap<JavaIdentifier, JavaFieldMetadata> {
    let mut fields = declaration
        .record_components
        .iter()
        .map(|component| {
            (
                component.name.clone(),
                JavaFieldMetadata {
                    ty: component.ty.clone(),
                    final_field: true,
                    blank_final: true,
                },
            )
        })
        .collect::<BTreeMap<_, _>>();
    for member in &declaration.members {
        if let JavaMember::Field(field) = member
            && !field.modifiers.contains(&JavaModifier::Static)
        {
            fields.insert(
                field.name.clone(),
                JavaFieldMetadata {
                    ty: field.ty.clone(),
                    final_field: field.modifiers.contains(&JavaModifier::Final),
                    blank_final: field.modifiers.contains(&JavaModifier::Final)
                        && field.initializer.is_none(),
                },
            );
        }
    }
    fields
}

fn verify_expr_scope(value: &JavaExpr, scope: &JavaLexicalScope) -> Vec<AstViolation> {
    let mut violations = Vec::new();
    match &value.kind {
        JavaExprKind::Value(JavaValueRef::Local(name)) => match scope.bindings.get(name) {
            Some(binding) if binding.ty == value.ty => {}
            Some(_) => violations.push(type_error(
                "local reference type disagrees with its lexical declaration",
            )),
            None => violations.push(AstViolation::new(
                DiagnosticCode::UnresolvedReference,
                format!(
                    "local reference {:?} is outside its lexical scope",
                    name.as_str()
                ),
            )),
        },
        JavaExprKind::Value(JavaValueRef::This) => {
            if !scope.allows_this {
                violations.push(AstViolation::new(
                    DiagnosticCode::UnresolvedReference,
                    "this is unavailable in a static Java scope",
                ));
            } else if scope.owner.as_ref() != Some(&value.ty) {
                violations.push(AstViolation::new(
                    DiagnosticCode::UnresolvedReference,
                    "this type does not match its declaring Java owner",
                ));
            }
        }
        JavaExprKind::Value(_) | JavaExprKind::Literal(_) => {}
        JavaExprKind::Unary { operand, .. } => {
            violations.extend(verify_expr_scope(operand, scope));
        }
        JavaExprKind::Binary { left, right, .. } => {
            violations.extend(verify_expr_scope(left, scope));
            violations.extend(verify_expr_scope(right, scope));
        }
        JavaExprKind::Conditional {
            condition,
            when_true,
            when_false,
        } => {
            violations.extend(verify_expr_scope(condition, scope));
            violations.extend(verify_expr_scope(when_true, scope));
            violations.extend(verify_expr_scope(when_false, scope));
        }
        JavaExprKind::Call {
            receiver,
            arguments,
            ..
        } => {
            if let Some(receiver) = receiver {
                violations.extend(verify_expr_scope(receiver, scope));
            }
            for argument in arguments {
                violations.extend(verify_expr_scope(argument, scope));
            }
        }
        JavaExprKind::New { arguments, .. } => {
            for argument in arguments {
                violations.extend(verify_expr_scope(argument, scope));
            }
        }
        JavaExprKind::NewArray { length, .. } => {
            violations.extend(verify_expr_scope(length, scope));
        }
        JavaExprKind::ArrayIndex { array, index } => {
            violations.extend(verify_expr_scope(array, scope));
            violations.extend(verify_expr_scope(index, scope));
        }
        JavaExprKind::Field { receiver, field } => {
            violations.extend(verify_expr_scope(receiver, scope));
            if let JavaFieldRef::Structural { name, ty } = field {
                let valid =
                    if matches!(receiver.ty, JavaType::Array { .. }) && name.as_str() == "length" {
                        *ty == JavaType::primitive(JavaPrimitive::Int)
                    } else {
                        scope.owner.as_ref() == Some(&receiver.ty)
                            && scope
                                .owner_fields
                                .get(name)
                                .is_some_and(|metadata| metadata.ty == *ty)
                    };
                if !valid {
                    violations.push(AstViolation::new(
                        DiagnosticCode::UnresolvedReference,
                        "structural Java field is absent from its lexical owner declaration",
                    ));
                }
            }
        }
        JavaExprKind::Cast { value, .. }
        | JavaExprKind::ArrayOwnershipTransition { value, .. }
        | JavaExprKind::InstanceOf { value, .. } => {
            violations.extend(verify_expr_scope(value, scope));
        }
        JavaExprKind::Lambda { .. } => {
            // The ordinary AST verifier rejects lambdas before rendering.
        }
    }
    violations
}

fn verify_assignment_target(
    target: &JavaExpr,
    scope: &JavaLexicalScope,
    context: Option<&TargetAstContext<'_, JavaDialect>>,
) -> Vec<AstViolation> {
    match &target.kind {
        JavaExprKind::Value(JavaValueRef::Local(name)) => match scope.bindings.get(name) {
            Some(binding) if binding.mutable && binding.ty == target.ty => vec![],
            Some(binding) if !binding.mutable => vec![AstViolation::new(
                DiagnosticCode::InvalidControlFlow,
                "cannot assign to a final Java local or parameter",
            )],
            Some(_) => vec![type_error(
                "assignment target type disagrees with its lexical declaration",
            )],
            None => vec![AstViolation::new(
                DiagnosticCode::UnresolvedReference,
                "assignment target is outside its lexical scope",
            )],
        },
        JavaExprKind::Field { receiver, field } => {
            let metadata = match (field, context) {
                (JavaFieldRef::Known(_), _) => Some(JavaFieldMetadata {
                    ty: field.ty(),
                    final_field: true,
                    blank_final: false,
                }),
                (JavaFieldRef::Structural { name, .. }, Some(context)) => {
                    structural_field_metadata(&receiver.ty, name, context).or_else(|| {
                        (scope.owner.as_ref() == Some(&receiver.ty))
                            .then(|| scope.owner_fields.get(name).cloned())
                            .flatten()
                    })
                }
                (
                    JavaFieldRef::Generated {
                        owner,
                        field,
                        name,
                        ty,
                    },
                    Some(context),
                ) if receiver.ty == JavaType::Reference(JavaTypeName::Generated(*owner))
                    && generated_field_matches(*owner, *field, name, ty, context) =>
                {
                    Some(JavaFieldMetadata {
                        ty: ty.clone(),
                        final_field: true,
                        blank_final: true,
                    })
                }
                _ => None,
            };
            match metadata {
                Some(metadata) if metadata.ty != target.ty => vec![type_error(
                    "assignment field type disagrees with its declaration",
                )],
                Some(metadata) if !metadata.final_field => vec![],
                Some(metadata) if metadata.final_field && !metadata.blank_final => {
                    vec![AstViolation::new(
                        DiagnosticCode::InvalidControlFlow,
                        "initialized final Java fields cannot be assigned",
                    )]
                }
                Some(_)
                    if scope.constructor
                        && matches!(receiver.kind, JavaExprKind::Value(JavaValueRef::This))
                        && scope.owner.as_ref() == Some(&receiver.ty) =>
                {
                    vec![]
                }
                Some(_) => vec![AstViolation::new(
                    DiagnosticCode::InvalidControlFlow,
                    "final Java fields may be assigned only through this in their declaring constructor",
                )],
                None => vec![AstViolation::new(
                    DiagnosticCode::UnresolvedReference,
                    format!(
                        "assignment target does not resolve to a declared Java field on {:?}",
                        receiver.ty
                    ),
                )],
            }
        }
        JavaExprKind::ArrayIndex { array, .. }
            if matches!(
                array.ty,
                JavaType::Array {
                    ownership: JavaArrayOwnership::InternalMutable,
                    ..
                }
            ) =>
        {
            vec![]
        }
        JavaExprKind::ArrayIndex { .. } => vec![AstViolation::new(
            DiagnosticCode::InvalidControlFlow,
            "cannot assign through a defensive-copy Java array boundary",
        )],
        _ => vec![AstViolation::new(
            DiagnosticCode::InvalidControlFlow,
            "Java assignment target is not an lvalue",
        )],
    }
}

fn expr_checked_exceptions(
    value: &JavaExpr,
    context: &TargetAstContext<'_, JavaDialect>,
) -> BTreeSet<JavaKnownType> {
    let mut exceptions = BTreeSet::new();
    match &value.kind {
        JavaExprKind::Literal(_) | JavaExprKind::Value(_) => {}
        JavaExprKind::Unary { operand, .. } => {
            exceptions.extend(expr_checked_exceptions(operand, context));
        }
        JavaExprKind::Binary { left, right, .. } => {
            exceptions.extend(expr_checked_exceptions(left, context));
            exceptions.extend(expr_checked_exceptions(right, context));
        }
        JavaExprKind::Conditional {
            condition,
            when_true,
            when_false,
        } => {
            exceptions.extend(expr_checked_exceptions(condition, context));
            exceptions.extend(expr_checked_exceptions(when_true, context));
            exceptions.extend(expr_checked_exceptions(when_false, context));
        }
        JavaExprKind::Call {
            callable,
            receiver,
            arguments,
        } => {
            if let Some(signature) = callable.signature(context) {
                exceptions.extend(signature.checked_exceptions);
            }
            if let Some(receiver) = receiver {
                exceptions.extend(expr_checked_exceptions(receiver, context));
            }
            for argument in arguments {
                exceptions.extend(expr_checked_exceptions(argument, context));
            }
        }
        JavaExprKind::New { arguments, .. } => {
            for argument in arguments {
                exceptions.extend(expr_checked_exceptions(argument, context));
            }
        }
        JavaExprKind::NewArray { length, .. } => {
            exceptions.extend(expr_checked_exceptions(length, context));
        }
        JavaExprKind::ArrayIndex { array, index } => {
            exceptions.extend(expr_checked_exceptions(array, context));
            exceptions.extend(expr_checked_exceptions(index, context));
        }
        JavaExprKind::Field { receiver, .. } => {
            exceptions.extend(expr_checked_exceptions(receiver, context));
        }
        JavaExprKind::Cast { value, .. }
        | JavaExprKind::ArrayOwnershipTransition { value, .. }
        | JavaExprKind::InstanceOf { value, .. } => {
            exceptions.extend(expr_checked_exceptions(value, context));
        }
        JavaExprKind::Lambda { body, .. } => {
            exceptions.extend(block_checked_exceptions(body, context));
        }
    }
    exceptions
}

fn block_checked_exceptions(
    block: &JavaBlock,
    context: &TargetAstContext<'_, JavaDialect>,
) -> BTreeSet<JavaKnownType> {
    let mut exceptions = BTreeSet::new();
    for statement in &block.statements {
        match statement {
            JavaStmt::Local { value, .. } | JavaStmt::Return(value) => {
                if let Some(value) = value {
                    exceptions.extend(expr_checked_exceptions(value, context));
                }
            }
            JavaStmt::Assign { target, value } => {
                exceptions.extend(expr_checked_exceptions(target, context));
                exceptions.extend(expr_checked_exceptions(value, context));
            }
            JavaStmt::Expression(value)
            | JavaStmt::Throw(value)
            | JavaStmt::ThrowAssertion(value) => {
                exceptions.extend(expr_checked_exceptions(value, context));
                if let JavaStmt::Throw(_) = statement
                    && let Some(exception) = throwable_known_type(&value.ty)
                    && is_checked_exception(exception)
                {
                    exceptions.insert(exception);
                }
            }
            JavaStmt::If {
                condition,
                then_block,
                else_block,
            } => {
                exceptions.extend(expr_checked_exceptions(condition, context));
                exceptions.extend(block_checked_exceptions(then_block, context));
                if let Some(else_block) = else_block {
                    exceptions.extend(block_checked_exceptions(else_block, context));
                }
            }
            JavaStmt::ForEach { iterable, body, .. } => {
                exceptions.extend(expr_checked_exceptions(iterable, context));
                exceptions.extend(block_checked_exceptions(body, context));
            }
            JavaStmt::While { condition, body } => {
                exceptions.extend(expr_checked_exceptions(condition, context));
                exceptions.extend(block_checked_exceptions(body, context));
            }
            JavaStmt::Switch { value, arms } => {
                exceptions.extend(expr_checked_exceptions(value, context));
                for arm in arms {
                    exceptions.extend(block_checked_exceptions(&arm.body, context));
                }
            }
            JavaStmt::TryCatch { try_block, catches } => {
                let mut try_exceptions = block_checked_exceptions(try_block, context);
                for catch in catches {
                    if let Some(caught) = throwable_known_type(&catch.exception_type) {
                        try_exceptions.remove(&caught);
                    }
                    exceptions.extend(block_checked_exceptions(&catch.body, context));
                }
                exceptions.extend(try_exceptions);
            }
            JavaStmt::Break | JavaStmt::Continue => {}
        }
    }
    exceptions
}

fn throwable_known_type(ty: &JavaType) -> Option<JavaKnownType> {
    match ty {
        JavaType::Reference(JavaTypeName::Known(value))
            if matches!(
                value,
                JavaKnownType::AssertionError
                    | JavaKnownType::IllegalArgumentException
                    | JavaKnownType::IllegalStateException
                    | JavaKnownType::RuntimeException
                    | JavaKnownType::CharacterCodingException
            ) =>
        {
            Some(*value)
        }
        _ => None,
    }
}

fn is_checked_exception(value: JavaKnownType) -> bool {
    matches!(value, JavaKnownType::CharacterCodingException)
}

fn admitted_throwable_is_supertype_of(supertype: JavaKnownType, subtype: JavaKnownType) -> bool {
    supertype == subtype
        || matches!(
            (supertype, subtype),
            (
                JavaKnownType::RuntimeException,
                JavaKnownType::IllegalArgumentException | JavaKnownType::IllegalStateException
            )
        )
}

#[cfg(test)]
fn verify_block_scope(
    block: &JavaBlock,
    scope: &mut JavaLexicalScope,
    expected_return: &JavaType,
    in_loop: bool,
) -> Vec<AstViolation> {
    verify_block_scope_in_context(block, scope, expected_return, in_loop, None)
}

fn verify_block_scope_in_context(
    block: &JavaBlock,
    scope: &mut JavaLexicalScope,
    expected_return: &JavaType,
    in_loop: bool,
    context: Option<&TargetAstContext<'_, JavaDialect>>,
) -> Vec<AstViolation> {
    let mut violations = Vec::new();
    for statement in &block.statements {
        match statement {
            JavaStmt::Local {
                finality,
                ty,
                name,
                value,
            } => {
                if let Some(value) = value {
                    violations.extend(verify_expr_scope(value, scope));
                }
                scope.bind(
                    name.clone(),
                    JavaLexicalBinding {
                        ty: ty.clone(),
                        mutable: *finality == JavaLocalFinality::Mutable,
                    },
                    "Java local shadows a name already declared in this portable scope",
                    &mut violations,
                );
            }
            JavaStmt::Assign { target, value } => {
                violations.extend(verify_expr_scope(target, scope));
                violations.extend(verify_expr_scope(value, scope));
                violations.extend(verify_assignment_target(target, scope, context));
            }
            JavaStmt::Expression(value)
            | JavaStmt::Throw(value)
            | JavaStmt::ThrowAssertion(value) => {
                violations.extend(verify_expr_scope(value, scope));
            }
            JavaStmt::Return(value) => match value {
                Some(value) => {
                    violations.extend(verify_expr_scope(value, scope));
                    if !invocation_types_match(expected_return, &value.ty) {
                        violations.push(type_error(
                            "Java return value does not match the declared return type",
                        ));
                    }
                }
                None if *expected_return != JavaType::primitive(JavaPrimitive::Void) => {
                    violations.push(type_error(
                        "non-void Java method cannot return without a value",
                    ));
                }
                None => {}
            },
            JavaStmt::If {
                condition,
                then_block,
                else_block,
            } => {
                violations.extend(verify_expr_scope(condition, scope));
                let mut then_scope = scope.clone();
                collect_positive_pattern_bindings(condition, &mut then_scope, &mut violations);
                violations.extend(verify_block_scope_in_context(
                    then_block,
                    &mut then_scope,
                    expected_return,
                    in_loop,
                    context,
                ));
                if let Some(else_block) = else_block {
                    let mut else_scope = scope.clone();
                    violations.extend(verify_block_scope_in_context(
                        else_block,
                        &mut else_scope,
                        expected_return,
                        in_loop,
                        context,
                    ));
                }
                if else_block.is_none()
                    && block_guarantees_exit(then_block)
                    && let JavaExprKind::Unary {
                        operator: JavaUnaryOperator::Not,
                        operand,
                    } = &condition.kind
                    && let JavaExprKind::InstanceOf {
                        target,
                        binding: Some(binding),
                        ..
                    } = &operand.kind
                {
                    scope.bind(
                        binding.clone(),
                        JavaLexicalBinding {
                            ty: target.clone(),
                            mutable: false,
                        },
                        "Java instanceof flow binding conflicts with an overlapping lexical binding",
                        &mut violations,
                    );
                }
            }
            JavaStmt::ForEach {
                binding_type,
                binding,
                iterable,
                body,
            } => {
                violations.extend(verify_expr_scope(iterable, scope));
                let mut body_scope = scope.clone();
                body_scope.bind(
                    binding.clone(),
                    JavaLexicalBinding {
                        ty: binding_type.clone(),
                        mutable: false,
                    },
                    "Java foreach binding conflicts with an overlapping lexical binding",
                    &mut violations,
                );
                violations.extend(verify_block_scope_in_context(
                    body,
                    &mut body_scope,
                    expected_return,
                    true,
                    context,
                ));
            }
            JavaStmt::While { condition, body } => {
                violations.extend(verify_expr_scope(condition, scope));
                let mut body_scope = scope.clone();
                collect_positive_pattern_bindings(condition, &mut body_scope, &mut violations);
                violations.extend(verify_block_scope_in_context(
                    body,
                    &mut body_scope,
                    expected_return,
                    true,
                    context,
                ));
            }
            JavaStmt::Switch { value, arms } => {
                violations.extend(verify_expr_scope(value, scope));
                for arm in arms {
                    let mut arm_scope = scope.clone();
                    if let JavaPattern::Type { ty, binding } = &arm.pattern {
                        arm_scope.bind(
                            binding.clone(),
                            JavaLexicalBinding {
                                ty: ty.clone(),
                                mutable: false,
                            },
                            "Java switch type-pattern binding conflicts with an overlapping lexical binding",
                            &mut violations,
                        );
                    }
                    violations.extend(verify_block_scope_in_context(
                        &arm.body,
                        &mut arm_scope,
                        expected_return,
                        in_loop,
                        context,
                    ));
                }
            }
            JavaStmt::TryCatch { try_block, catches } => {
                let mut try_scope = scope.clone();
                violations.extend(verify_block_scope_in_context(
                    try_block,
                    &mut try_scope,
                    expected_return,
                    in_loop,
                    context,
                ));
                for catch in catches {
                    let mut catch_scope = scope.clone();
                    catch_scope.bind(
                        catch.binding.clone(),
                        JavaLexicalBinding {
                            ty: catch.exception_type.clone(),
                            mutable: false,
                        },
                        "Java catch binding conflicts with an overlapping lexical binding",
                        &mut violations,
                    );
                    violations.extend(verify_block_scope_in_context(
                        &catch.body,
                        &mut catch_scope,
                        expected_return,
                        in_loop,
                        context,
                    ));
                }
            }
            JavaStmt::Break | JavaStmt::Continue if !in_loop => violations.push(AstViolation::new(
                DiagnosticCode::InvalidControlFlow,
                "Java break/continue is outside a loop",
            )),
            JavaStmt::Break | JavaStmt::Continue => {}
        }
    }
    violations
}

fn collect_positive_pattern_bindings(
    value: &JavaExpr,
    scope: &mut JavaLexicalScope,
    violations: &mut Vec<AstViolation>,
) {
    match &value.kind {
        JavaExprKind::InstanceOf {
            target,
            binding: Some(binding),
            ..
        } => {
            scope.bind(
                binding.clone(),
                JavaLexicalBinding {
                    ty: target.clone(),
                    mutable: false,
                },
                "Java instanceof flow binding conflicts with an overlapping lexical binding",
                violations,
            );
        }
        JavaExprKind::Binary {
            operator: JavaBinaryOperator::LogicalAnd,
            left,
            right,
        } => {
            collect_positive_pattern_bindings(left, scope, violations);
            collect_positive_pattern_bindings(right, scope, violations);
        }
        _ => {}
    }
}

fn block_guarantees_exit(block: &JavaBlock) -> bool {
    block
        .statements
        .last()
        .is_some_and(statement_guarantees_exit)
}

fn statement_guarantees_exit(statement: &JavaStmt) -> bool {
    match statement {
        JavaStmt::Return(_) | JavaStmt::Throw(_) | JavaStmt::ThrowAssertion(_) => true,
        JavaStmt::If {
            then_block,
            else_block: Some(else_block),
            ..
        } => block_guarantees_exit(then_block) && block_guarantees_exit(else_block),
        JavaStmt::Switch { arms, .. } => {
            !arms.is_empty() && arms.iter().all(|arm| block_guarantees_exit(&arm.body))
        }
        JavaStmt::TryCatch { try_block, catches } => {
            block_guarantees_exit(try_block)
                && !catches.is_empty()
                && catches
                    .iter()
                    .all(|catch| block_guarantees_exit(&catch.body))
        }
        _ => false,
    }
}

type JavaFinalAssignmentState = BTreeSet<JavaIdentifier>;
type JavaFinalAssignmentStates = BTreeSet<JavaFinalAssignmentState>;

#[derive(Default)]
struct JavaFinalAssignmentOutcome {
    fallthrough: JavaFinalAssignmentStates,
    constructor_returns: JavaFinalAssignmentStates,
}

fn declaration_blank_instance_finals(
    declaration: &JavaTypeDeclaration,
) -> BTreeSet<JavaIdentifier> {
    let mut fields = declaration
        .record_components
        .iter()
        .map(|component| component.name.clone())
        .collect::<BTreeSet<_>>();
    fields.extend(
        declaration
            .members
            .iter()
            .filter_map(|member| match member {
                JavaMember::Field(field)
                    if !field.modifiers.contains(&JavaModifier::Static)
                        && field.modifiers.contains(&JavaModifier::Final)
                        && field.initializer.is_none() =>
                {
                    Some(field.name.clone())
                }
                _ => None,
            }),
    );
    fields
}

fn assigned_blank_final<'a>(
    target: &'a JavaExpr,
    blank_finals: &BTreeSet<JavaIdentifier>,
) -> Option<&'a JavaIdentifier> {
    let JavaExprKind::Field { receiver, field } = &target.kind else {
        return None;
    };
    if !matches!(receiver.kind, JavaExprKind::Value(JavaValueRef::This)) {
        return None;
    }
    let name = match field {
        JavaFieldRef::Structural { name, .. } | JavaFieldRef::Generated { name, .. } => name,
        JavaFieldRef::Known(_) => return None,
    };
    blank_finals.contains(name).then_some(name)
}

fn block_assigns_blank_final(block: &JavaBlock, blank_finals: &BTreeSet<JavaIdentifier>) -> bool {
    block.statements.iter().any(|statement| match statement {
        JavaStmt::Assign { target, .. } => assigned_blank_final(target, blank_finals).is_some(),
        JavaStmt::If {
            then_block,
            else_block,
            ..
        } => {
            block_assigns_blank_final(then_block, blank_finals)
                || else_block
                    .as_ref()
                    .is_some_and(|block| block_assigns_blank_final(block, blank_finals))
        }
        JavaStmt::ForEach { body, .. } | JavaStmt::While { body, .. } => {
            block_assigns_blank_final(body, blank_finals)
        }
        JavaStmt::Switch { arms, .. } => arms
            .iter()
            .any(|arm| block_assigns_blank_final(&arm.body, blank_finals)),
        JavaStmt::TryCatch { try_block, catches } => {
            block_assigns_blank_final(try_block, blank_finals)
                || catches
                    .iter()
                    .any(|catch| block_assigns_blank_final(&catch.body, blank_finals))
        }
        JavaStmt::Local { .. }
        | JavaStmt::Expression(_)
        | JavaStmt::Return(_)
        | JavaStmt::Throw(_)
        | JavaStmt::ThrowAssertion(_)
        | JavaStmt::Break
        | JavaStmt::Continue => false,
    })
}

fn analyze_constructor_final_assignments(
    block: &JavaBlock,
    incoming: JavaFinalAssignmentStates,
    blank_finals: &BTreeSet<JavaIdentifier>,
    violations: &mut Vec<AstViolation>,
) -> JavaFinalAssignmentOutcome {
    let mut fallthrough = incoming;
    let mut constructor_returns = JavaFinalAssignmentStates::new();
    for statement in &block.statements {
        if fallthrough.is_empty() {
            break;
        }
        let outcome = analyze_constructor_final_assignment_statement(
            statement,
            fallthrough,
            blank_finals,
            violations,
        );
        fallthrough = outcome.fallthrough;
        constructor_returns.extend(outcome.constructor_returns);
    }
    JavaFinalAssignmentOutcome {
        fallthrough,
        constructor_returns,
    }
}

fn analyze_constructor_final_assignment_statement(
    statement: &JavaStmt,
    incoming: JavaFinalAssignmentStates,
    blank_finals: &BTreeSet<JavaIdentifier>,
    violations: &mut Vec<AstViolation>,
) -> JavaFinalAssignmentOutcome {
    match statement {
        JavaStmt::Assign { target, .. } => {
            let mut fallthrough = JavaFinalAssignmentStates::new();
            if let Some(field) = assigned_blank_final(target, blank_finals) {
                let duplicate = incoming.iter().any(|state| state.contains(field));
                if duplicate {
                    violations.push(AstViolation::new(
                        DiagnosticCode::InvalidControlFlow,
                        format!(
                            "blank final Java field `{}` can be assigned more than once on a constructor path",
                            field.as_str()
                        ),
                    ));
                }
                for mut state in incoming {
                    state.insert(field.clone());
                    fallthrough.insert(state);
                }
            } else {
                fallthrough = incoming;
            }
            JavaFinalAssignmentOutcome {
                fallthrough,
                ..JavaFinalAssignmentOutcome::default()
            }
        }
        JavaStmt::Return(_) => JavaFinalAssignmentOutcome {
            constructor_returns: incoming,
            ..JavaFinalAssignmentOutcome::default()
        },
        JavaStmt::Throw(_) | JavaStmt::ThrowAssertion(_) => JavaFinalAssignmentOutcome::default(),
        JavaStmt::If {
            then_block,
            else_block,
            ..
        } => {
            let then_outcome = analyze_constructor_final_assignments(
                then_block,
                incoming.clone(),
                blank_finals,
                violations,
            );
            let else_outcome = if let Some(block) = else_block {
                analyze_constructor_final_assignments(block, incoming, blank_finals, violations)
            } else {
                JavaFinalAssignmentOutcome {
                    fallthrough: incoming,
                    ..JavaFinalAssignmentOutcome::default()
                }
            };
            JavaFinalAssignmentOutcome {
                fallthrough: then_outcome
                    .fallthrough
                    .union(&else_outcome.fallthrough)
                    .cloned()
                    .collect(),
                constructor_returns: then_outcome
                    .constructor_returns
                    .union(&else_outcome.constructor_returns)
                    .cloned()
                    .collect(),
            }
        }
        JavaStmt::ForEach { body, .. } | JavaStmt::While { body, .. } => {
            if block_assigns_blank_final(body, blank_finals) {
                violations.push(AstViolation::new(
                    DiagnosticCode::InvalidControlFlow,
                    "blank final Java fields cannot be assigned in a portable constructor loop",
                ));
            }
            let body_outcome = analyze_constructor_final_assignments(
                body,
                incoming.clone(),
                blank_finals,
                violations,
            );
            JavaFinalAssignmentOutcome {
                fallthrough: incoming,
                constructor_returns: body_outcome.constructor_returns,
            }
        }
        JavaStmt::Switch { arms, .. } => {
            let mut outcome = JavaFinalAssignmentOutcome::default();
            for arm in arms {
                let arm_outcome = analyze_constructor_final_assignments(
                    &arm.body,
                    incoming.clone(),
                    blank_finals,
                    violations,
                );
                outcome.fallthrough.extend(arm_outcome.fallthrough);
                outcome
                    .constructor_returns
                    .extend(arm_outcome.constructor_returns);
            }
            if !arms
                .iter()
                .any(|arm| matches!(arm.pattern, JavaPattern::Default))
            {
                outcome.fallthrough.extend(incoming);
            }
            outcome
        }
        JavaStmt::TryCatch { try_block, catches } => {
            if block_assigns_blank_final(try_block, blank_finals)
                || catches
                    .iter()
                    .any(|catch| block_assigns_blank_final(&catch.body, blank_finals))
            {
                violations.push(AstViolation::new(
                    DiagnosticCode::InvalidControlFlow,
                    "blank final Java assignment inside portable try/catch is not supported by sound definite-assignment analysis",
                ));
            }
            let mut outcome = analyze_constructor_final_assignments(
                try_block,
                incoming.clone(),
                blank_finals,
                violations,
            );
            for catch in catches {
                let catch_outcome = analyze_constructor_final_assignments(
                    &catch.body,
                    incoming.clone(),
                    blank_finals,
                    violations,
                );
                outcome.fallthrough.extend(catch_outcome.fallthrough);
                outcome
                    .constructor_returns
                    .extend(catch_outcome.constructor_returns);
            }
            outcome
        }
        JavaStmt::Break | JavaStmt::Continue => JavaFinalAssignmentOutcome::default(),
        JavaStmt::Local { .. } | JavaStmt::Expression(_) => JavaFinalAssignmentOutcome {
            fallthrough: incoming,
            ..JavaFinalAssignmentOutcome::default()
        },
    }
}

fn verify_constructor_final_assignments(
    constructor: &JavaConstructor,
    declaration: &JavaTypeDeclaration,
) -> Vec<AstViolation> {
    let blank_finals = declaration_blank_instance_finals(declaration);
    if blank_finals.is_empty() {
        return vec![];
    }
    let mut initial = JavaFinalAssignmentStates::new();
    initial.insert(JavaFinalAssignmentState::new());
    let mut violations = Vec::new();
    let outcome = analyze_constructor_final_assignments(
        &constructor.body,
        initial,
        &blank_finals,
        &mut violations,
    );
    let normally_completing = outcome
        .fallthrough
        .union(&outcome.constructor_returns)
        .cloned()
        .collect::<JavaFinalAssignmentStates>();
    for state in normally_completing {
        for missing in blank_finals.difference(&state) {
            violations.push(AstViolation::new(
                DiagnosticCode::InvalidControlFlow,
                format!(
                    "Java constructor can complete normally without assigning blank final field `{}`",
                    missing.as_str()
                ),
            ));
        }
    }
    violations
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum JavaPattern {
    Default,
    Literal(JavaLiteral),
    Type {
        ty: JavaType,
        binding: JavaIdentifier,
    },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct JavaSwitchArm {
    pub pattern: JavaPattern,
    pub body: JavaBlock,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct JavaCatch {
    pub exception_type: JavaType,
    pub binding: JavaIdentifier,
    pub body: JavaBlock,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum JavaStmt {
    Local {
        finality: JavaLocalFinality,
        ty: JavaType,
        name: JavaIdentifier,
        value: Option<JavaExpr>,
    },
    Assign {
        target: JavaExpr,
        value: JavaExpr,
    },
    Expression(JavaExpr),
    Return(Option<JavaExpr>),
    If {
        condition: JavaExpr,
        then_block: JavaBlock,
        else_block: Option<JavaBlock>,
    },
    ForEach {
        binding_type: JavaType,
        binding: JavaIdentifier,
        iterable: JavaExpr,
        body: JavaBlock,
    },
    While {
        condition: JavaExpr,
        body: JavaBlock,
    },
    Switch {
        value: JavaExpr,
        arms: Vec<JavaSwitchArm>,
    },
    TryCatch {
        try_block: JavaBlock,
        catches: Vec<JavaCatch>,
    },
    Throw(JavaExpr),
    ThrowAssertion(JavaExpr),
    Break,
    Continue,
}

impl JavaStmt {
    pub fn symbols(&self, symbols: &mut BTreeSet<TargetSymbolRef<JavaDialect>>) {
        match self {
            Self::Local { ty, value, .. } => {
                ty.symbols(symbols);
                if let Some(value) = value {
                    value.symbols(symbols);
                }
            }
            Self::Assign { target, value } => {
                target.symbols(symbols);
                value.symbols(symbols);
            }
            Self::Expression(value) | Self::Throw(value) | Self::ThrowAssertion(value) => {
                value.symbols(symbols)
            }
            Self::Return(value) => {
                if let Some(value) = value {
                    value.symbols(symbols);
                }
            }
            Self::If {
                condition,
                then_block,
                else_block,
            } => {
                condition.symbols(symbols);
                then_block.symbols(symbols);
                if let Some(block) = else_block {
                    block.symbols(symbols);
                }
            }
            Self::ForEach {
                binding_type,
                iterable,
                body,
                ..
            } => {
                binding_type.symbols(symbols);
                iterable.symbols(symbols);
                body.symbols(symbols);
            }
            Self::While { condition, body } => {
                condition.symbols(symbols);
                body.symbols(symbols);
            }
            Self::Switch { value, arms } => {
                value.symbols(symbols);
                for arm in arms {
                    if let JavaPattern::Type { ty, .. } = &arm.pattern {
                        ty.symbols(symbols);
                    }
                    arm.body.symbols(symbols);
                }
            }
            Self::TryCatch { try_block, catches } => {
                try_block.symbols(symbols);
                for catch in catches {
                    catch.exception_type.symbols(symbols);
                    catch.body.symbols(symbols);
                }
            }
            Self::Break | Self::Continue => {}
        }
    }

    pub fn verify(&self, context: &TargetAstContext<'_, JavaDialect>) -> Vec<AstViolation> {
        let mut violations = Vec::new();
        match self {
            Self::Local { ty, value, .. } => {
                violations.extend(ty.verify(JavaTypeUse::Value));
                if let Some(value) = value {
                    violations.extend(value.verify(context));
                    if &value.ty != ty {
                        violations.push(type_error("local initializer type mismatch"));
                    }
                }
            }
            Self::Assign { target, value } => {
                violations.extend(target.verify(context));
                violations.extend(value.verify(context));
                if target.ty != value.ty {
                    violations.push(type_error("assignment type mismatch"));
                }
            }
            Self::Expression(value) | Self::Throw(value) | Self::ThrowAssertion(value) => {
                violations.extend(value.verify(context));
                if matches!(self, Self::Expression(_)) && !is_java_statement_expression(value) {
                    violations.push(AstViolation::new(
                        DiagnosticCode::InvalidStructure,
                        "Java expression statement must be a method invocation or class instance creation",
                    ));
                }
                if matches!(self, Self::Throw(_)) && throwable_known_type(&value.ty).is_none() {
                    violations.push(type_error(
                        "Java throw expression must have a known throwable type",
                    ));
                }
            }
            Self::Return(value) => {
                if let Some(value) = value {
                    violations.extend(value.verify(context));
                }
            }
            Self::If {
                condition,
                then_block,
                else_block,
            } => {
                violations.extend(condition.verify(context));
                if condition.ty != JavaType::Primitive(JavaPrimitive::Boolean) {
                    violations.push(type_error("if condition must be boolean"));
                }
                violations.extend(then_block.verify(context));
                if let Some(block) = else_block {
                    violations.extend(block.verify(context));
                }
            }
            Self::ForEach {
                binding_type,
                iterable,
                body,
                ..
            } => {
                violations.extend(binding_type.verify(JavaTypeUse::Value));
                violations.extend(iterable.verify(context));
                match iterable_element_type(&iterable.ty) {
                    Some(element) if invocation_types_match(binding_type, element) => {}
                    Some(_) => violations.push(type_error(
                        "Java foreach binding type does not match its iterable element type",
                    )),
                    None => violations.push(type_error(
                        "Java foreach expression must be an array or typed List",
                    )),
                }
                violations.extend(body.verify(context));
            }
            Self::While { condition, body } => {
                violations.extend(condition.verify(context));
                if condition.ty != JavaType::Primitive(JavaPrimitive::Boolean) {
                    violations.push(type_error("while condition must be boolean"));
                }
                violations.extend(body.verify(context));
            }
            Self::Switch { value, arms } => {
                violations.extend(value.verify(context));
                violations.extend(verify_switch_patterns(value, arms, context));
                for arm in arms {
                    violations.extend(arm.body.verify(context));
                }
            }
            Self::TryCatch { try_block, catches } => {
                violations.extend(try_block.verify(context));
                if catches.is_empty() {
                    violations.push(AstViolation::new(
                        DiagnosticCode::InvalidStructure,
                        "typed try statement requires at least one catch",
                    ));
                }
                let thrown = block_checked_exceptions(try_block, context);
                let mut caught_types = BTreeSet::new();
                let mut prior_caught_types = Vec::new();
                for catch in catches {
                    violations.extend(catch.exception_type.verify(JavaTypeUse::Parameter));
                    match throwable_known_type(&catch.exception_type) {
                        Some(caught) => {
                            if prior_caught_types.iter().any(|prior| {
                                *prior != caught
                                    && admitted_throwable_is_supertype_of(*prior, caught)
                            }) {
                                violations.push(AstViolation::new(
                                    DiagnosticCode::InvalidControlFlow,
                                    "Java catch clause is dominated by an earlier admitted throwable supertype",
                                ));
                            }
                            if !caught_types.insert(caught) {
                                violations.push(AstViolation::new(
                                    DiagnosticCode::DuplicateDeclaration,
                                    "Java catch type is repeated",
                                ));
                            }
                            if is_checked_exception(caught) && !thrown.contains(&caught) {
                                violations.push(AstViolation::new(
                                    DiagnosticCode::InvalidStructure,
                                    "Java checked catch cannot be reached from its try block",
                                ));
                            }
                            prior_caught_types.push(caught);
                        }
                        None => violations.push(type_error(
                            "Java catch parameter must have a known throwable type",
                        )),
                    }
                    violations.extend(catch.body.verify(context));
                }
            }
            Self::Break | Self::Continue => {}
        }
        violations
    }
}

fn is_java_statement_expression(value: &JavaExpr) -> bool {
    matches!(
        value.kind,
        JavaExprKind::Call { .. } | JavaExprKind::New { .. }
    )
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum JavaSwitchConstant {
    Integral(i64),
    String(String),
}

fn verify_switch_patterns(
    selector: &JavaExpr,
    arms: &[JavaSwitchArm],
    context: &TargetAstContext<'_, JavaDialect>,
) -> Vec<AstViolation> {
    let mut violations = Vec::new();
    if !java_switch_selector_is_legal(&selector.ty) {
        violations.push(type_error(
            "Java switch selector must be an int-compatible primitive or reference type",
        ));
    }
    if arms
        .iter()
        .filter(|arm| matches!(arm.pattern, JavaPattern::Default))
        .count()
        != 1
    {
        violations.push(AstViolation::new(
            DiagnosticCode::NonExhaustiveMatch,
            "statement switch must have exactly one typed default arm",
        ));
    }

    let mut constants = Vec::new();
    let mut prior_types = Vec::<JavaType>::new();
    for arm in arms {
        match &arm.pattern {
            JavaPattern::Default => {}
            JavaPattern::Literal(literal) => {
                if !java_switch_literal_is_compatible(literal, &selector.ty) {
                    violations.push(type_error(
                        "Java switch literal is not compatible with its selector type",
                    ));
                    continue;
                }
                if prior_types
                    .iter()
                    .any(|prior| java_type_pattern_dominates(prior, &selector.ty, context))
                {
                    violations.push(AstViolation::new(
                        DiagnosticCode::InvalidControlFlow,
                        "Java switch literal is dominated by an earlier type pattern",
                    ));
                }
                if let Some(constant) = java_switch_constant(literal)
                    && !constants.contains(&constant)
                {
                    constants.push(constant);
                } else {
                    violations.push(AstViolation::new(
                        DiagnosticCode::DuplicateDeclaration,
                        "Java switch constant label is repeated",
                    ));
                }
            }
            JavaPattern::Type { ty, .. } => {
                if !java_type_is_reifiable(ty)
                    || !java_instanceof_is_legal(&selector.ty, ty, context)
                {
                    violations.push(type_error(
                        "Java switch type pattern is not reifiable or selector-compatible",
                    ));
                }
                if prior_types
                    .iter()
                    .any(|prior| java_type_pattern_dominates(prior, ty, context))
                {
                    violations.push(AstViolation::new(
                        DiagnosticCode::InvalidControlFlow,
                        "Java switch type pattern is dominated by an earlier pattern",
                    ));
                }
                if java_type_pattern_dominates(ty, &selector.ty, context) {
                    violations.push(AstViolation::new(
                        DiagnosticCode::InvalidStructure,
                        "Java switch cannot combine an unconditional type pattern with its required default arm",
                    ));
                }
                prior_types.push(ty.clone());
            }
        }
    }
    violations
}

fn java_switch_selector_is_legal(ty: &JavaType) -> bool {
    match ty {
        JavaType::Primitive(value) => matches!(
            value,
            JavaPrimitive::Byte | JavaPrimitive::Char | JavaPrimitive::Int
        ),
        _ => java_type_is_reference(ty),
    }
}

fn java_switch_literal_is_compatible(literal: &JavaLiteral, selector: &JavaType) -> bool {
    match literal {
        JavaLiteral::I32(value) => match selector {
            JavaType::Primitive(JavaPrimitive::Byte) | JavaType::Boxed(JavaPrimitive::Byte) => {
                i8::try_from(*value).is_ok()
            }
            JavaType::Primitive(JavaPrimitive::Char) | JavaType::Boxed(JavaPrimitive::Char) => {
                u16::try_from(*value).is_ok()
            }
            JavaType::Primitive(JavaPrimitive::Int) | JavaType::Boxed(JavaPrimitive::Int) => true,
            _ => false,
        },
        JavaLiteral::CharScalar(value) => match selector {
            JavaType::Primitive(JavaPrimitive::Byte) | JavaType::Boxed(JavaPrimitive::Byte) => {
                *value <= i8::MAX as u32
            }
            JavaType::Primitive(JavaPrimitive::Char) | JavaType::Boxed(JavaPrimitive::Char) => {
                u16::try_from(*value).is_ok()
            }
            JavaType::Primitive(JavaPrimitive::Int) | JavaType::Boxed(JavaPrimitive::Int) => true,
            _ => false,
        },
        JavaLiteral::String(_) => *selector == JavaType::known(JavaKnownType::String),
        JavaLiteral::Boolean(_)
        | JavaLiteral::I64(_)
        | JavaLiteral::Utf16Units(_)
        | JavaLiteral::InternalNull(_) => false,
    }
}

fn java_switch_constant(literal: &JavaLiteral) -> Option<JavaSwitchConstant> {
    match literal {
        JavaLiteral::I32(value) => Some(JavaSwitchConstant::Integral(i64::from(*value))),
        JavaLiteral::CharScalar(value) => Some(JavaSwitchConstant::Integral(i64::from(*value))),
        JavaLiteral::String(value) => Some(JavaSwitchConstant::String(value.clone())),
        JavaLiteral::Boolean(_)
        | JavaLiteral::I64(_)
        | JavaLiteral::Utf16Units(_)
        | JavaLiteral::InternalNull(_) => None,
    }
}

fn java_type_pattern_dominates(
    earlier: &JavaType,
    later: &JavaType,
    context: &TargetAstContext<'_, JavaDialect>,
) -> bool {
    if erased_java_type(earlier) == erased_java_type(later) {
        return true;
    }
    if *earlier == JavaType::known(JavaKnownType::Object) && java_type_is_reference(later) {
        return true;
    }
    match (earlier, later) {
        (
            JavaType::Reference(JavaTypeName::Generated(expected)),
            JavaType::Reference(JavaTypeName::Generated(actual)),
        ) => generated_type_implements(*actual, *expected, context),
        (
            JavaType::Array {
                component: expected,
                ..
            },
            JavaType::Array {
                component: actual, ..
            },
        ) => java_type_pattern_dominates(expected, actual, context),
        _ => generated_and_known_interface_related(earlier, later, context),
    }
}

fn iterable_element_type(ty: &JavaType) -> Option<&JavaType> {
    match ty {
        JavaType::Array { component, .. } => Some(component),
        JavaType::Generic {
            raw: JavaTypeName::Known(JavaKnownType::List),
            arguments,
        } if arguments.len() == 1 => arguments.first(),
        _ => None,
    }
}

impl TargetStatementNode<JavaDialect> for JavaStmt {
    fn child_expressions(&self) -> Vec<portable_codegen::TargetExprId> {
        vec![]
    }
    fn verify(&self, context: &TargetAstContext<'_, JavaDialect>) -> Vec<AstViolation> {
        self.verify(context)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum JavaVisibility {
    Public,
    Package,
    Private,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum JavaModifier {
    Public,
    Private,
    Static,
    Final,
    Transient,
    Sealed,
    NonSealed,
    Abstract,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum JavaAnnotation {
    Override,
    SafeVarargs,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct JavaParameter {
    pub ty: JavaType,
    pub name: JavaIdentifier,
    pub final_parameter: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct JavaField {
    pub declared: Option<GeneratedValueId>,
    pub modifiers: Vec<JavaModifier>,
    pub ty: JavaType,
    pub name: JavaIdentifier,
    pub initializer: Option<JavaExpr>,
}

/// A deliberately ill-typed field used only to prove the native compiler
/// rejects mappings which the portable surface forbids.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct JavaCompileFailField {
    pub modifiers: Vec<JavaModifier>,
    pub expected_type: JavaType,
    pub name: JavaIdentifier,
    pub initializer: JavaExpr,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum JavaMethodDeclaration {
    Structural,
    Callable(GeneratedCallableId),
    Interface(GeneratedInterfaceMethodId),
    Implementation {
        method: portable_core_ir::CoreImplementationMethodId,
        interface: GeneratedInterfaceMethodId,
    },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct JavaMethod {
    pub declared: JavaMethodDeclaration,
    pub annotations: Vec<JavaAnnotation>,
    pub modifiers: Vec<JavaModifier>,
    pub type_parameters: Vec<JavaIdentifier>,
    pub return_type: JavaType,
    pub name: JavaIdentifier,
    pub parameters: Vec<JavaParameter>,
    pub body: Option<JavaBlock>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct JavaConstructor {
    pub modifiers: Vec<JavaModifier>,
    pub name: JavaIdentifier,
    pub parameters: Vec<JavaParameter>,
    pub body: JavaBlock,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum JavaMember {
    Field(JavaField),
    CompileFailField(JavaCompileFailField),
    Method(JavaMethod),
    Constructor(JavaConstructor),
    NestedType(JavaTypeDeclaration),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum JavaDeclarationKind {
    FinalClass,
    Record,
    Interface,
    SealedInterface,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum JavaHeritage {
    None,
    Interfaces(Vec<JavaType>),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct JavaRecordComponent {
    pub origin: JavaRecordComponentOrigin,
    pub ty: JavaType,
    pub name: JavaIdentifier,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum JavaRecordComponentOrigin {
    Core(CoreFieldId),
    Runtime(JavaRuntimeMember),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct JavaTypeDeclaration {
    pub declared: Option<GeneratedTypeId>,
    pub kind: JavaDeclarationKind,
    pub visibility: JavaVisibility,
    pub modifiers: Vec<JavaModifier>,
    pub name: JavaIdentifier,
    pub type_parameters: Vec<JavaIdentifier>,
    pub record_components: Vec<JavaRecordComponent>,
    pub heritage: JavaHeritage,
    pub permits: Vec<JavaType>,
    pub members: Vec<JavaMember>,
}

fn declaration_owner_type(declaration: &JavaTypeDeclaration) -> Option<JavaType> {
    let raw = if let Some(id) = declaration.declared {
        JavaTypeName::Generated(id)
    } else {
        JavaKnownType::ALL
            .into_iter()
            .find(|known| {
                known.runtime_helper().is_some() && known.simple_name() == declaration.name.as_str()
            })
            .map(JavaTypeName::Known)?
    };
    if declaration.type_parameters.is_empty() {
        Some(JavaType::Reference(raw))
    } else {
        Some(JavaType::Generic {
            raw,
            arguments: declaration
                .type_parameters
                .iter()
                .cloned()
                .map(JavaType::TypeVariable)
                .collect(),
        })
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum JavaFileItem {
    Type {
        declared: Vec<GeneratedSymbolId>,
        declaration: JavaTypeDeclaration,
    },
    RuntimeMembers {
        helper: crate::dialect::JavaRuntimeHelper,
        members: Vec<JavaMember>,
    },
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
enum JavaErasedType {
    Primitive(JavaPrimitive),
    Reference(JavaTypeName),
    Array(Box<JavaErasedType>),
}

fn erased_java_type(ty: &JavaType) -> JavaErasedType {
    match ty {
        JavaType::Primitive(value) => JavaErasedType::Primitive(*value),
        JavaType::Boxed(value) => JavaErasedType::Reference(JavaTypeName::Known(match value {
            JavaPrimitive::Boolean => JavaKnownType::Boolean,
            JavaPrimitive::Byte => JavaKnownType::Byte,
            JavaPrimitive::Char => JavaKnownType::Character,
            JavaPrimitive::Int => JavaKnownType::Integer,
            JavaPrimitive::Long => JavaKnownType::Long,
            JavaPrimitive::Double => JavaKnownType::Double,
            JavaPrimitive::Void => JavaKnownType::Object,
        })),
        JavaType::Reference(name) | JavaType::Generic { raw: name, .. } => {
            JavaErasedType::Reference(name.clone())
        }
        JavaType::Array { component, .. } => {
            JavaErasedType::Array(Box::new(erased_java_type(component)))
        }
        JavaType::Wildcard { .. } | JavaType::TypeVariable(_) => {
            JavaErasedType::Reference(JavaTypeName::Known(JavaKnownType::Object))
        }
    }
}

fn known_generic_arity(known: JavaKnownType) -> usize {
    match known {
        JavaKnownType::ArrayList
        | JavaKnownType::List
        | JavaKnownType::RuntimeResult
        | JavaKnownType::RuntimeOption => 1,
        JavaKnownType::LinkedHashMap | JavaKnownType::Map | JavaKnownType::RuntimeValueResult => 2,
        _ => 0,
    }
}

fn verify_contextual_type(
    ty: &JavaType,
    variables: &BTreeSet<JavaIdentifier>,
    context: &TargetAstContext<'_, JavaDialect>,
) -> Vec<AstViolation> {
    let mut violations = Vec::new();
    match ty {
        JavaType::Reference(JavaTypeName::Known(known)) => {
            if known_generic_arity(*known) != 0 {
                violations.push(type_error(
                    "generic Java known type cannot be used as a raw reference",
                ));
            }
        }
        JavaType::Reference(JavaTypeName::Generated(id)) => {
            match find_type_declaration(ty, context) {
                Some(declaration) if declaration.type_parameters.is_empty() => {}
                Some(_) => violations.push(type_error(
                    "generic generated Java type cannot be used as a raw reference",
                )),
                None => violations.push(AstViolation::new(
                    DiagnosticCode::UnresolvedReference,
                    format!("generated Java type {id:?} has no AST declaration"),
                )),
            }
        }
        JavaType::Generic { raw, arguments } => {
            let expected = match raw {
                JavaTypeName::Known(known) => Some(known_generic_arity(*known)),
                JavaTypeName::Generated(_) => find_type_declaration(ty, context)
                    .map(|declaration| declaration.type_parameters.len()),
            };
            match expected {
                Some(arity) if arity == arguments.len() && arity != 0 => {}
                Some(arity) => violations.push(type_error(&format!(
                    "Java generic type requires exactly {arity} type arguments"
                ))),
                None => violations.push(AstViolation::new(
                    DiagnosticCode::UnresolvedReference,
                    "generic generated Java type has no AST declaration",
                )),
            }
            for argument in arguments {
                violations.extend(verify_contextual_type(argument, variables, context));
            }
        }
        JavaType::Array { component, .. } => {
            violations.extend(verify_contextual_type(component, variables, context));
        }
        JavaType::Wildcard {
            bound: Some((_, bound)),
        } => {
            violations.extend(verify_contextual_type(bound, variables, context));
        }
        JavaType::TypeVariable(name) if !variables.contains(name) => {
            violations.push(AstViolation::new(
                DiagnosticCode::UnresolvedReference,
                format!(
                    "Java type variable {:?} is not declared in this type/member scope",
                    name.as_str()
                ),
            ));
        }
        JavaType::Primitive(_)
        | JavaType::Boxed(_)
        | JavaType::Wildcard { bound: None }
        | JavaType::TypeVariable(_) => {}
    }
    violations
}

fn verify_expression_type_context(
    expression: &JavaExpr,
    variables: &BTreeSet<JavaIdentifier>,
    context: &TargetAstContext<'_, JavaDialect>,
) -> Vec<AstViolation> {
    let mut violations = verify_contextual_type(&expression.ty, variables, context);
    match &expression.kind {
        JavaExprKind::Literal(_) | JavaExprKind::Value(_) => {}
        JavaExprKind::Unary { operand, .. } => {
            violations.extend(verify_expression_type_context(operand, variables, context));
        }
        JavaExprKind::Binary { left, right, .. } => {
            violations.extend(verify_expression_type_context(left, variables, context));
            violations.extend(verify_expression_type_context(right, variables, context));
        }
        JavaExprKind::Conditional {
            condition,
            when_true,
            when_false,
        } => {
            violations.extend(verify_expression_type_context(
                condition, variables, context,
            ));
            violations.extend(verify_expression_type_context(
                when_true, variables, context,
            ));
            violations.extend(verify_expression_type_context(
                when_false, variables, context,
            ));
        }
        JavaExprKind::Call {
            callable,
            receiver,
            arguments,
        } => {
            let signature = match callable {
                JavaCallableRef::Known { signature, .. }
                | JavaCallableRef::Runtime { signature, .. }
                | JavaCallableRef::Generated { signature, .. }
                | JavaCallableRef::Interface { signature, .. }
                | JavaCallableRef::Member { signature, .. } => signature,
            };
            if let JavaCallableRef::Member { owner, .. } = callable {
                violations.extend(verify_contextual_type(owner, variables, context));
            }
            if let Some(receiver_type) = &signature.receiver {
                violations.extend(verify_contextual_type(receiver_type, variables, context));
            }
            for parameter in &signature.parameters {
                violations.extend(verify_contextual_type(parameter, variables, context));
            }
            violations.extend(verify_contextual_type(
                &signature.result,
                variables,
                context,
            ));
            if let Some(receiver) = receiver {
                violations.extend(verify_expression_type_context(receiver, variables, context));
            }
            for argument in arguments {
                violations.extend(verify_expression_type_context(argument, variables, context));
            }
        }
        JavaExprKind::New {
            constructor,
            arguments,
        } => {
            match constructor {
                JavaConstructorRef::Known {
                    owner, parameters, ..
                } => {
                    violations.extend(verify_contextual_type(owner, variables, context));
                    for parameter in parameters {
                        violations.extend(verify_contextual_type(parameter, variables, context));
                    }
                }
                JavaConstructorRef::Generated { owner, parameters } => {
                    violations.extend(verify_contextual_type(
                        &JavaType::Reference(JavaTypeName::Generated(*owner)),
                        variables,
                        context,
                    ));
                    for parameter in parameters {
                        violations.extend(verify_contextual_type(parameter, variables, context));
                    }
                }
            }
            for argument in arguments {
                violations.extend(verify_expression_type_context(argument, variables, context));
            }
        }
        JavaExprKind::NewArray { component, length } => {
            violations.extend(verify_contextual_type(component, variables, context));
            violations.extend(verify_expression_type_context(length, variables, context));
        }
        JavaExprKind::ArrayIndex { array, index } => {
            violations.extend(verify_expression_type_context(array, variables, context));
            violations.extend(verify_expression_type_context(index, variables, context));
        }
        JavaExprKind::Field { receiver, field } => {
            violations.extend(verify_expression_type_context(receiver, variables, context));
            violations.extend(verify_contextual_type(&field.ty(), variables, context));
        }
        JavaExprKind::Cast { target, value }
        | JavaExprKind::InstanceOf {
            target,
            value,
            binding: _,
        } => {
            violations.extend(verify_contextual_type(target, variables, context));
            violations.extend(verify_expression_type_context(value, variables, context));
        }
        JavaExprKind::ArrayOwnershipTransition { value, .. } => {
            violations.extend(verify_expression_type_context(value, variables, context));
        }
        JavaExprKind::Lambda { parameters, body } => {
            for parameter in parameters {
                violations.extend(verify_contextual_type(&parameter.ty, variables, context));
            }
            violations.extend(verify_block_type_context(body, variables, context));
        }
    }
    violations
}

fn verify_block_type_context(
    block: &JavaBlock,
    variables: &BTreeSet<JavaIdentifier>,
    context: &TargetAstContext<'_, JavaDialect>,
) -> Vec<AstViolation> {
    let mut violations = Vec::new();
    for statement in &block.statements {
        match statement {
            JavaStmt::Local { ty, value, .. } => {
                violations.extend(verify_contextual_type(ty, variables, context));
                if let Some(value) = value {
                    violations.extend(verify_expression_type_context(value, variables, context));
                }
            }
            JavaStmt::Assign { target, value } => {
                violations.extend(verify_expression_type_context(target, variables, context));
                violations.extend(verify_expression_type_context(value, variables, context));
            }
            JavaStmt::Expression(value)
            | JavaStmt::Throw(value)
            | JavaStmt::ThrowAssertion(value) => {
                violations.extend(verify_expression_type_context(value, variables, context));
            }
            JavaStmt::Return(value) => {
                if let Some(value) = value {
                    violations.extend(verify_expression_type_context(value, variables, context));
                }
            }
            JavaStmt::If {
                condition,
                then_block,
                else_block,
            } => {
                violations.extend(verify_expression_type_context(
                    condition, variables, context,
                ));
                violations.extend(verify_block_type_context(then_block, variables, context));
                if let Some(else_block) = else_block {
                    violations.extend(verify_block_type_context(else_block, variables, context));
                }
            }
            JavaStmt::ForEach {
                binding_type,
                iterable,
                body,
                ..
            } => {
                violations.extend(verify_contextual_type(binding_type, variables, context));
                violations.extend(verify_expression_type_context(iterable, variables, context));
                violations.extend(verify_block_type_context(body, variables, context));
            }
            JavaStmt::While { condition, body } => {
                violations.extend(verify_expression_type_context(
                    condition, variables, context,
                ));
                violations.extend(verify_block_type_context(body, variables, context));
            }
            JavaStmt::Switch { value, arms } => {
                violations.extend(verify_expression_type_context(value, variables, context));
                for arm in arms {
                    if let JavaPattern::Type { ty, .. } = &arm.pattern {
                        violations.extend(verify_contextual_type(ty, variables, context));
                    }
                    violations.extend(verify_block_type_context(&arm.body, variables, context));
                }
            }
            JavaStmt::TryCatch { try_block, catches } => {
                violations.extend(verify_block_type_context(try_block, variables, context));
                for catch in catches {
                    violations.extend(verify_contextual_type(
                        &catch.exception_type,
                        variables,
                        context,
                    ));
                    violations.extend(verify_block_type_context(&catch.body, variables, context));
                }
            }
            JavaStmt::Break | JavaStmt::Continue => {}
        }
    }
    violations
}

fn verify_member_type_context(
    member: &JavaMember,
    declaration_variables: &BTreeSet<JavaIdentifier>,
    context: &TargetAstContext<'_, JavaDialect>,
) -> Vec<AstViolation> {
    let mut violations = Vec::new();
    match member {
        JavaMember::Field(field) => {
            let variables = if field.modifiers.contains(&JavaModifier::Static) {
                BTreeSet::new()
            } else {
                declaration_variables.clone()
            };
            violations.extend(verify_contextual_type(&field.ty, &variables, context));
            if let Some(initializer) = &field.initializer {
                violations.extend(verify_expression_type_context(
                    initializer,
                    &variables,
                    context,
                ));
            }
        }
        JavaMember::CompileFailField(field) => {
            let variables = if field.modifiers.contains(&JavaModifier::Static) {
                BTreeSet::new()
            } else {
                declaration_variables.clone()
            };
            violations.extend(verify_contextual_type(
                &field.expected_type,
                &variables,
                context,
            ));
            violations.extend(verify_expression_type_context(
                &field.initializer,
                &variables,
                context,
            ));
        }
        JavaMember::Method(method) => {
            let mut variables = method
                .type_parameters
                .iter()
                .cloned()
                .collect::<BTreeSet<_>>();
            if !method.modifiers.contains(&JavaModifier::Static) {
                variables.extend(declaration_variables.iter().cloned());
            }
            violations.extend(verify_contextual_type(
                &method.return_type,
                &variables,
                context,
            ));
            for parameter in &method.parameters {
                violations.extend(verify_contextual_type(&parameter.ty, &variables, context));
            }
            if let Some(body) = &method.body {
                violations.extend(verify_block_type_context(body, &variables, context));
            }
        }
        JavaMember::Constructor(constructor) => {
            for parameter in &constructor.parameters {
                violations.extend(verify_contextual_type(
                    &parameter.ty,
                    declaration_variables,
                    context,
                ));
            }
            violations.extend(verify_block_type_context(
                &constructor.body,
                declaration_variables,
                context,
            ));
        }
        JavaMember::NestedType(_) => {}
    }
    violations
}

fn verify_declaration_type_context(
    declaration: &JavaTypeDeclaration,
    context: &TargetAstContext<'_, JavaDialect>,
) -> Vec<AstViolation> {
    let variables = declaration
        .type_parameters
        .iter()
        .cloned()
        .collect::<BTreeSet<_>>();
    let mut violations = Vec::new();
    for component in &declaration.record_components {
        violations.extend(verify_contextual_type(&component.ty, &variables, context));
    }
    if let JavaHeritage::Interfaces(interfaces) = &declaration.heritage {
        for interface in interfaces {
            violations.extend(verify_contextual_type(interface, &variables, context));
        }
    }
    for permitted in &declaration.permits {
        violations.extend(verify_contextual_type(permitted, &variables, context));
    }
    for member in &declaration.members {
        violations.extend(verify_member_type_context(member, &variables, context));
    }
    violations
}

impl JavaMember {
    pub fn symbols(&self, symbols: &mut BTreeSet<TargetSymbolRef<JavaDialect>>) {
        match self {
            Self::Field(field) => {
                if let Some(value) = field.declared {
                    symbols.insert(TargetSymbolRef::Generated(GeneratedSymbolId::Value(value)));
                }
                field.ty.symbols(symbols);
                if let Some(value) = &field.initializer {
                    value.symbols(symbols);
                }
            }
            Self::CompileFailField(field) => {
                field.expected_type.symbols(symbols);
                field.initializer.symbols(symbols);
            }
            Self::Method(method) => {
                match method.declared {
                    JavaMethodDeclaration::Callable(value) => {
                        symbols.insert(TargetSymbolRef::Generated(GeneratedSymbolId::Callable(
                            value,
                        )));
                    }
                    JavaMethodDeclaration::Interface(value)
                    | JavaMethodDeclaration::Implementation {
                        interface: value, ..
                    } => {
                        symbols.insert(TargetSymbolRef::Generated(
                            GeneratedSymbolId::InterfaceMethod(value),
                        ));
                    }
                    JavaMethodDeclaration::Structural => {}
                }
                method.return_type.symbols(symbols);
                for parameter in &method.parameters {
                    parameter.ty.symbols(symbols);
                }
                if let Some(body) = &method.body {
                    body.symbols(symbols);
                }
            }
            Self::Constructor(constructor) => {
                for parameter in &constructor.parameters {
                    parameter.ty.symbols(symbols);
                }
                constructor.body.symbols(symbols);
            }
            Self::NestedType(value) => value.symbols(symbols),
        }
    }

    pub fn verify(&self, context: &TargetAstContext<'_, JavaDialect>) -> Vec<AstViolation> {
        self.verify_with_owner(context, None, None)
    }

    fn verify_with_owner(
        &self,
        context: &TargetAstContext<'_, JavaDialect>,
        owner: Option<&JavaType>,
        declaration: Option<&JavaTypeDeclaration>,
    ) -> Vec<AstViolation> {
        match self {
            Self::Field(field) => {
                let mut violations =
                    verify_modifiers_for(&field.modifiers, JavaModifierSite::Field);
                violations.extend(field.ty.verify(JavaTypeUse::Field));
                if let Some(value) = &field.initializer {
                    violations.extend(value.verify(context));
                    if value.ty != field.ty {
                        violations.push(type_error("field initializer type mismatch"));
                    }
                }
                violations
            }
            Self::CompileFailField(field) => {
                let mut violations =
                    verify_modifiers_for(&field.modifiers, JavaModifierSite::Field);
                violations.extend(field.expected_type.verify(JavaTypeUse::Field));
                violations.extend(field.initializer.verify(context));
                if invocation_types_match(&field.expected_type, &field.initializer.ty) {
                    violations.push(AstViolation::new(
                        DiagnosticCode::InvalidStructure,
                        "compile-fail field must contain a deliberate type mismatch",
                    ));
                }
                violations
            }
            Self::Method(method) => {
                let mut violations =
                    verify_modifiers_for(&method.modifiers, JavaModifierSite::Method);
                violations.extend(method.return_type.verify(JavaTypeUse::Return));
                let distinct_type_parameters =
                    method.type_parameters.iter().collect::<BTreeSet<_>>();
                if distinct_type_parameters.len() != method.type_parameters.len() {
                    violations.push(AstViolation::new(
                        DiagnosticCode::DuplicateDeclaration,
                        "Java method type parameter is declared more than once",
                    ));
                }
                for parameter in &method.parameters {
                    violations.extend(parameter.ty.verify(JavaTypeUse::Parameter));
                }
                if method.body.is_none() != method.modifiers.contains(&JavaModifier::Abstract) {
                    violations.push(AstViolation::new(
                        DiagnosticCode::InvalidStructure,
                        "abstract methods have no body and concrete methods have a body",
                    ));
                }
                if let Some(body) = &method.body {
                    violations.extend(body.verify(context));
                    let (mut scope, scope_violations) = JavaLexicalScope::for_method_in_declaration(
                        method,
                        owner.cloned(),
                        declaration,
                    );
                    violations.extend(scope_violations);
                    violations.extend(verify_block_scope_in_context(
                        body,
                        &mut scope,
                        &method.return_type,
                        false,
                        Some(context),
                    ));
                    if method.return_type != JavaType::primitive(JavaPrimitive::Void)
                        && !block_guarantees_exit(body)
                    {
                        violations.push(AstViolation::new(
                            DiagnosticCode::InvalidControlFlow,
                            "concrete non-void Java method does not return or throw on every path",
                        ));
                    }
                    let unhandled = block_checked_exceptions(body, context);
                    if !unhandled.is_empty() {
                        violations.push(AstViolation::new(
                            DiagnosticCode::InvalidInvocation,
                            format!("Java method has unhandled checked exceptions: {unhandled:?}"),
                        ));
                    }
                }
                violations
            }
            Self::Constructor(constructor) => {
                let mut violations =
                    verify_modifiers_for(&constructor.modifiers, JavaModifierSite::Constructor);
                for parameter in &constructor.parameters {
                    violations.extend(parameter.ty.verify(JavaTypeUse::Parameter));
                }
                violations.extend(constructor.body.verify(context));
                let (mut scope, scope_violations) =
                    JavaLexicalScope::for_constructor_in_declaration(
                        constructor,
                        owner.cloned(),
                        declaration,
                    );
                violations.extend(scope_violations);
                violations.extend(verify_block_scope_in_context(
                    &constructor.body,
                    &mut scope,
                    &JavaType::primitive(JavaPrimitive::Void),
                    false,
                    Some(context),
                ));
                if let Some(declaration) = declaration {
                    violations.extend(verify_constructor_final_assignments(
                        constructor,
                        declaration,
                    ));
                }
                let unhandled = block_checked_exceptions(&constructor.body, context);
                if !unhandled.is_empty() {
                    violations.push(AstViolation::new(
                        DiagnosticCode::InvalidInvocation,
                        format!("Java constructor has unhandled checked exceptions: {unhandled:?}"),
                    ));
                }
                violations
            }
            Self::NestedType(value) => value.verify(context, false),
        }
    }
}

impl JavaTypeDeclaration {
    pub fn contains_compile_fail_member(&self) -> bool {
        self.members.iter().any(|member| match member {
            JavaMember::CompileFailField(_) => true,
            JavaMember::NestedType(value) => value.contains_compile_fail_member(),
            JavaMember::Field(_) | JavaMember::Method(_) | JavaMember::Constructor(_) => false,
        })
    }

    pub fn symbols(&self, symbols: &mut BTreeSet<TargetSymbolRef<JavaDialect>>) {
        if let Some(value) = self.declared {
            symbols.insert(TargetSymbolRef::Generated(GeneratedSymbolId::Type(value)));
        }
        for component in &self.record_components {
            component.ty.symbols(symbols);
        }
        if let JavaHeritage::Interfaces(values) = &self.heritage {
            for value in values {
                value.symbols(symbols);
            }
        }
        for value in &self.permits {
            value.symbols(symbols);
        }
        for member in &self.members {
            member.symbols(symbols);
        }
    }

    pub fn verify(
        &self,
        context: &TargetAstContext<'_, JavaDialect>,
        top_level: bool,
    ) -> Vec<AstViolation> {
        let mut violations =
            verify_modifiers_for(&self.modifiers, JavaModifierSite::Type { top_level });
        let owner = declaration_owner_type(self);
        let distinct_type_parameters = self.type_parameters.iter().collect::<BTreeSet<_>>();
        if distinct_type_parameters.len() != self.type_parameters.len() {
            violations.push(AstViolation::new(
                DiagnosticCode::DuplicateDeclaration,
                "Java type parameter is declared more than once",
            ));
        }
        violations.extend(verify_declaration_type_context(self, context));
        if !top_level && self.visibility == JavaVisibility::Package {
            violations.push(AstViolation::new(
                DiagnosticCode::InvalidStructure,
                "nested types require explicit public or private visibility",
            ));
        }
        if top_level && self.visibility == JavaVisibility::Private {
            violations.push(AstViolation::new(
                DiagnosticCode::InvalidStructure,
                "top-level Java types cannot be private",
            ));
        }
        if self.kind != JavaDeclarationKind::Record && !self.record_components.is_empty() {
            violations.push(AstViolation::new(
                DiagnosticCode::InvalidStructure,
                "only records may have record components",
            ));
        }
        if self.kind != JavaDeclarationKind::SealedInterface && !self.permits.is_empty() {
            violations.push(AstViolation::new(
                DiagnosticCode::InvalidStructure,
                "only sealed interfaces may have permits clauses",
            ));
        }
        if matches!(
            self.kind,
            JavaDeclarationKind::Interface | JavaDeclarationKind::SealedInterface
        ) && !matches!(self.heritage, JavaHeritage::None)
        {
            violations.push(AstViolation::new(
                DiagnosticCode::InvalidStructure,
                "portable Java interfaces are flat and cannot extend an interface",
            ));
        }
        violations.extend(verify_declaration_kind_grammar(self));
        violations.extend(verify_sealed_permits(self, context));
        match &self.heritage {
            JavaHeritage::Interfaces(values) => {
                if values.is_empty() {
                    violations.push(AstViolation::new(
                        DiagnosticCode::InvalidStructure,
                        "Java interface conformance list cannot be empty",
                    ));
                }
                let mut seen = BTreeSet::new();
                for value in values {
                    if !seen.insert(value) {
                        violations.push(AstViolation::new(
                            DiagnosticCode::DuplicateDeclaration,
                            "Java interface conformance is listed more than once",
                        ));
                    }
                    let interface = match value {
                        JavaType::Reference(JavaTypeName::Known(
                            JavaKnownType::RuntimeSemanticValue,
                        )) => true,
                        JavaType::Reference(JavaTypeName::Generated(id)) => {
                            context.generated_type(*id).is_some_and(|item| {
                                matches!(
                                    item.kind,
                                    JavaDeclarationKind::Interface
                                        | JavaDeclarationKind::SealedInterface
                                )
                            })
                        }
                        _ => false,
                    };
                    if !interface {
                        violations.push(AstViolation::new(
                            DiagnosticCode::InterfaceNonconformance,
                            "Java implements clause must reference a declared interface",
                        ));
                    }
                }
            }
            JavaHeritage::None => {}
        }
        let mut field_names = BTreeSet::new();
        let mut component_origins = BTreeSet::new();
        for component in &self.record_components {
            violations.extend(component.ty.verify(JavaTypeUse::Field));
            if !field_names.insert(component.name.clone()) {
                violations.push(AstViolation::new(
                    DiagnosticCode::DuplicateDeclaration,
                    "Java record component is declared more than once",
                ));
            }
            if !component_origins.insert(component.origin) {
                violations.push(AstViolation::new(
                    DiagnosticCode::DuplicateDeclaration,
                    "Java record component origin is declared more than once",
                ));
            }
        }
        if let JavaHeritage::Interfaces(values) = &self.heritage {
            for value in values {
                violations.extend(value.verify(JavaTypeUse::TypeBound));
            }
        }
        let mut method_signatures = BTreeSet::new();
        let mut constructor_signatures = BTreeSet::new();
        let mut nested_type_names = BTreeSet::new();
        for member in &self.members {
            match member {
                JavaMember::Field(field) => {
                    if !field_names.insert(field.name.clone()) {
                        violations.push(AstViolation::new(
                            DiagnosticCode::DuplicateDeclaration,
                            "Java field conflicts with another field or record component",
                        ));
                    }
                }
                JavaMember::CompileFailField(field) => {
                    if !field_names.insert(field.name.clone()) {
                        violations.push(AstViolation::new(
                            DiagnosticCode::DuplicateDeclaration,
                            "Java compile-fail field conflicts with another field",
                        ));
                    }
                }
                JavaMember::Method(method) => {
                    let key = (
                        method.name.clone(),
                        method
                            .parameters
                            .iter()
                            .map(|parameter| erased_java_type(&parameter.ty))
                            .collect::<Vec<_>>(),
                    );
                    if !method_signatures.insert(key) {
                        violations.push(AstViolation::new(
                            DiagnosticCode::DuplicateDeclaration,
                            "Java method has a duplicate erased declaration signature",
                        ));
                    }
                    violations.extend(verify_method_registration(self, method, context));
                }
                JavaMember::Constructor(constructor) => {
                    let key = constructor
                        .parameters
                        .iter()
                        .map(|parameter| erased_java_type(&parameter.ty))
                        .collect::<Vec<_>>();
                    if !constructor_signatures.insert(key) {
                        violations.push(AstViolation::new(
                            DiagnosticCode::DuplicateDeclaration,
                            "Java constructor has a duplicate declaration signature",
                        ));
                    }
                    if constructor.name != self.name {
                        violations.push(AstViolation::new(
                            DiagnosticCode::InvalidStructure,
                            "Java constructor name does not match its declaring type",
                        ));
                    }
                }
                JavaMember::NestedType(nested) => {
                    if !nested_type_names.insert(nested.name.clone()) {
                        violations.push(AstViolation::new(
                            DiagnosticCode::DuplicateDeclaration,
                            "Java nested type is declared more than once",
                        ));
                    }
                }
            }
            violations.extend(member.verify_with_owner(context, owner.as_ref(), Some(self)));
        }
        if self.kind == JavaDeclarationKind::FinalClass
            && !declaration_blank_instance_finals(self).is_empty()
            && !self
                .members
                .iter()
                .any(|member| matches!(member, JavaMember::Constructor(_)))
        {
            violations.push(AstViolation::new(
                DiagnosticCode::InvalidControlFlow,
                "implicit Java constructor cannot initialize declared blank final instance fields",
            ));
        }
        violations.extend(verify_interface_conformance(self, context));
        violations
    }
}

fn verify_declaration_kind_grammar(declaration: &JavaTypeDeclaration) -> Vec<AstViolation> {
    let mut violations = Vec::new();
    let interface = matches!(
        declaration.kind,
        JavaDeclarationKind::Interface | JavaDeclarationKind::SealedInterface
    );
    let mut record_constructor_count = 0usize;
    for member in &declaration.members {
        match member {
            JavaMember::Field(_) | JavaMember::CompileFailField(_) if interface => {
                violations.push(AstViolation::new(
                    DiagnosticCode::InvalidStructure,
                    "portable Java interfaces cannot declare fields",
                ));
            }
            JavaMember::Constructor(_) if interface => {
                violations.push(AstViolation::new(
                    DiagnosticCode::InvalidStructure,
                    "Java interfaces cannot declare constructors",
                ));
            }
            JavaMember::NestedType(_) if interface => {
                violations.push(AstViolation::new(
                    DiagnosticCode::InvalidStructure,
                    "portable Java interfaces are flat and cannot declare nested types",
                ));
            }
            JavaMember::Method(method) if interface => {
                if !matches!(
                    method.declared,
                    JavaMethodDeclaration::Structural | JavaMethodDeclaration::Interface(_)
                ) || method.body.is_some()
                    || !method.modifiers.contains(&JavaModifier::Abstract)
                    || method.modifiers.contains(&JavaModifier::Static)
                {
                    violations.push(AstViolation::new(
                        DiagnosticCode::InvalidStructure,
                        "portable Java interface members must be abstract instance method declarations",
                    ));
                }
            }
            JavaMember::Method(method) => {
                if matches!(method.declared, JavaMethodDeclaration::Interface(_)) {
                    violations.push(AstViolation::new(
                        DiagnosticCode::InvalidStructure,
                        "Java interface method declarations must belong to an interface",
                    ));
                }
                if method.modifiers.contains(&JavaModifier::Abstract) {
                    violations.push(AstViolation::new(
                        DiagnosticCode::InvalidStructure,
                        "records and final Java classes cannot declare abstract methods",
                    ));
                }
            }
            JavaMember::Field(field) if declaration.kind == JavaDeclarationKind::Record => {
                if !field.modifiers.contains(&JavaModifier::Static) {
                    violations.push(AstViolation::new(
                        DiagnosticCode::InvalidStructure,
                        "Java records cannot declare additional instance fields",
                    ));
                }
            }
            JavaMember::CompileFailField(field)
                if declaration.kind == JavaDeclarationKind::Record =>
            {
                if !field.modifiers.contains(&JavaModifier::Static) {
                    violations.push(AstViolation::new(
                        DiagnosticCode::InvalidStructure,
                        "Java records cannot declare additional instance fields",
                    ));
                }
            }
            JavaMember::Constructor(constructor)
                if declaration.kind == JavaDeclarationKind::Record =>
            {
                record_constructor_count += 1;
                let canonical = constructor.parameters.len() == declaration.record_components.len()
                    && constructor
                        .parameters
                        .iter()
                        .zip(&declaration.record_components)
                        .all(|(parameter, component)| {
                            parameter.name == component.name && parameter.ty == component.ty
                        });
                if !canonical {
                    violations.push(AstViolation::new(
                        DiagnosticCode::InvalidStructure,
                        "explicit Java record constructor must have the canonical component signature",
                    ));
                }
            }
            JavaMember::Field(_)
            | JavaMember::CompileFailField(_)
            | JavaMember::Constructor(_)
            | JavaMember::NestedType(_) => {}
        }
    }
    if declaration.kind == JavaDeclarationKind::Record && record_constructor_count > 1 {
        violations.push(AstViolation::new(
            DiagnosticCode::InvalidStructure,
            "portable Java record may declare at most one canonical constructor",
        ));
    }
    violations
}

fn verify_sealed_permits(
    declaration: &JavaTypeDeclaration,
    context: &TargetAstContext<'_, JavaDialect>,
) -> Vec<AstViolation> {
    if declaration.kind != JavaDeclarationKind::SealedInterface {
        return vec![];
    }
    let Some(interface) = declaration.declared else {
        return vec![AstViolation::new(
            DiagnosticCode::InvalidStructure,
            "sealed Java interface must have a generated declaration identity",
        )];
    };
    let actual = generated_implementors(interface, context);
    let mut permitted = BTreeSet::new();
    let mut violations = Vec::new();
    for value in &declaration.permits {
        let JavaType::Reference(JavaTypeName::Generated(id)) = value else {
            violations.push(AstViolation::new(
                DiagnosticCode::InvalidStructure,
                "sealed Java permits entries must be non-generic generated declaration types",
            ));
            continue;
        };
        if !permitted.insert(*id) {
            violations.push(AstViolation::new(
                DiagnosticCode::DuplicateDeclaration,
                "sealed Java permits entry is repeated",
            ));
        }
        let valid_declaration = find_type_declaration(value, context).is_some_and(|candidate| {
            matches!(
                candidate.kind,
                JavaDeclarationKind::FinalClass | JavaDeclarationKind::Record
            ) && matches!(
                candidate.heritage,
                JavaHeritage::Interfaces(ref interfaces)
                    if interfaces.contains(&JavaType::Reference(JavaTypeName::Generated(interface)))
            )
        });
        if !valid_declaration {
            violations.push(AstViolation::new(
                DiagnosticCode::InterfaceNonconformance,
                "sealed Java permits entry does not name an actual final implementing declaration",
            ));
        }
    }
    if actual.is_empty() || permitted != actual {
        violations.push(AstViolation::new(
            DiagnosticCode::InterfaceNonconformance,
            "sealed Java permits set must exactly name every implementing declaration once",
        ));
    }
    violations
}

fn generated_implementors(
    interface: GeneratedTypeId,
    context: &TargetAstContext<'_, JavaDialect>,
) -> BTreeSet<GeneratedTypeId> {
    let mut output = BTreeSet::new();
    for file in context.files() {
        for item in file.items() {
            if let JavaFileItem::Type { declaration, .. } = item {
                collect_generated_implementors(declaration, interface, &mut output);
            }
        }
    }
    output
}

fn collect_generated_implementors(
    declaration: &JavaTypeDeclaration,
    interface: GeneratedTypeId,
    output: &mut BTreeSet<GeneratedTypeId>,
) {
    if let Some(id) = declaration.declared
        && matches!(
            declaration.kind,
            JavaDeclarationKind::FinalClass | JavaDeclarationKind::Record
        )
        && matches!(
            declaration.heritage,
            JavaHeritage::Interfaces(ref interfaces)
                if interfaces.contains(&JavaType::Reference(JavaTypeName::Generated(interface)))
        )
    {
        output.insert(id);
    }
    for member in &declaration.members {
        if let JavaMember::NestedType(nested) = member {
            collect_generated_implementors(nested, interface, output);
        }
    }
}

fn verify_interface_conformance(
    declaration: &JavaTypeDeclaration,
    context: &TargetAstContext<'_, JavaDialect>,
) -> Vec<AstViolation> {
    let JavaHeritage::Interfaces(interfaces) = &declaration.heritage else {
        return vec![];
    };
    let mut violations = Vec::new();
    for interface in interfaces {
        match interface {
            JavaType::Reference(JavaTypeName::Generated(interface_id)) => {
                let Some(required) = declared_interface_methods(*interface_id, context) else {
                    violations.push(AstViolation::new(
                        DiagnosticCode::InterfaceNonconformance,
                        "implemented generated Java interface has no matching AST declaration",
                    ));
                    continue;
                };
                for required_method in required {
                    let implementations = declaration
                        .members
                        .iter()
                        .filter(|member| {
                            matches!(
                                member,
                                JavaMember::Method(JavaMethod {
                                    declared: JavaMethodDeclaration::Implementation {
                                        interface,
                                        ..
                                    },
                                    ..
                                }) if *interface == required_method
                            )
                        })
                        .count();
                    if implementations != 1 {
                        violations.push(AstViolation::new(
                            DiagnosticCode::InterfaceNonconformance,
                            format!(
                                "Java conformance must implement interface method {required_method:?} exactly once"
                            ),
                        ));
                    }
                }
            }
            JavaType::Reference(JavaTypeName::Known(JavaKnownType::RuntimeSemanticValue)) => {
                let semantic_method = declaration.members.iter().any(|member| {
                    matches!(
                        member,
                        JavaMember::Method(JavaMethod {
                            declared: JavaMethodDeclaration::Structural,
                            name,
                            parameters,
                            return_type,
                            body: Some(_),
                            ..
                        }) if name.as_str() == JavaRuntimeMember::SemanticEquals.name()
                            && parameters.len() == 1
                            && parameters[0].ty == JavaType::known(JavaKnownType::Object)
                            && *return_type == JavaType::primitive(JavaPrimitive::Boolean)
                    )
                });
                if !semantic_method {
                    violations.push(AstViolation::new(
                        DiagnosticCode::InterfaceNonconformance,
                        "Java runtime semantic value must implement semanticEquals(Object)",
                    ));
                }
                let deep_method = declaration.members.iter().any(|member| {
                    matches!(
                        member,
                        JavaMember::Method(JavaMethod {
                            declared: JavaMethodDeclaration::Structural,
                            name,
                            parameters,
                            return_type,
                            body: Some(_),
                            ..
                        }) if name.as_str() == JavaRuntimeMember::DeepEquals.name()
                            && parameters.len() == 1
                            && parameters[0].ty == JavaType::known(JavaKnownType::Object)
                            && *return_type == JavaType::primitive(JavaPrimitive::Boolean)
                    )
                });
                if !deep_method {
                    violations.push(AstViolation::new(
                        DiagnosticCode::InterfaceNonconformance,
                        "Java runtime semantic value must implement deepEquals(Object)",
                    ));
                }
            }
            _ => {}
        }
    }
    violations
}

fn declared_interface_methods(
    interface: GeneratedTypeId,
    context: &TargetAstContext<'_, JavaDialect>,
) -> Option<Vec<GeneratedInterfaceMethodId>> {
    context.files().find_map(|file| {
        file.items().iter().find_map(|item| {
            let JavaFileItem::Type { declaration, .. } = item else {
                return None;
            };
            find_declared_interface_methods(declaration, interface)
        })
    })
}

fn find_declared_interface_methods(
    declaration: &JavaTypeDeclaration,
    interface: GeneratedTypeId,
) -> Option<Vec<GeneratedInterfaceMethodId>> {
    if declaration.declared == Some(interface)
        && matches!(
            declaration.kind,
            JavaDeclarationKind::Interface | JavaDeclarationKind::SealedInterface
        )
    {
        return Some(
            declaration
                .members
                .iter()
                .filter_map(|member| match member {
                    JavaMember::Method(JavaMethod {
                        declared: JavaMethodDeclaration::Interface(method),
                        ..
                    }) => Some(*method),
                    _ => None,
                })
                .collect(),
        );
    }
    declaration.members.iter().find_map(|member| match member {
        JavaMember::NestedType(nested) => find_declared_interface_methods(nested, interface),
        _ => None,
    })
}

fn verify_method_registration(
    declaration: &JavaTypeDeclaration,
    method: &JavaMethod,
    context: &TargetAstContext<'_, JavaDialect>,
) -> Vec<AstViolation> {
    let (registered, expected_name, receiver_owner) = match method.declared {
        JavaMethodDeclaration::Structural => return vec![],
        JavaMethodDeclaration::Callable(id) => (
            context.callable(id).map(|value| &value.signature),
            context.callable(id).map(|value| value.name.as_str()),
            None,
        ),
        JavaMethodDeclaration::Interface(id)
        | JavaMethodDeclaration::Implementation { interface: id, .. } => (
            context.interface_method(id).map(|value| &value.signature),
            context
                .interface_method(id)
                .map(|value| value.name.as_str()),
            context.interface_method(id).map(|value| value.owner),
        ),
    };
    let Some(registered) = registered else {
        return vec![AstViolation::new(
            DiagnosticCode::UnresolvedReference,
            "Java method declaration references an unregistered callable",
        )];
    };
    let actual_parameters = method
        .parameters
        .iter()
        .map(|parameter| JavaDialect.coarse_type(&parameter.ty))
        .collect::<Vec<_>>();
    let actual_return = JavaDialect.coarse_type(&method.return_type);
    let name_matches =
        expected_name.map(JavaIdentifier::from_portable).as_ref() == Some(&method.name);
    let static_method = method.modifiers.contains(&JavaModifier::Static);
    let signature_matches =
        registered.parameters == actual_parameters && registered.return_type == actual_return;
    let declaration_matches = match method.declared {
        JavaMethodDeclaration::Structural => true,
        JavaMethodDeclaration::Callable(_) => {
            registered.invocation == JavaInvocationKind::Static
                && registered.receiver.is_none()
                && static_method
        }
        JavaMethodDeclaration::Interface(_) => {
            registered.invocation == JavaInvocationKind::Instance
                && !static_method
                && receiver_owner == declaration.declared
        }
        JavaMethodDeclaration::Implementation { .. } => {
            registered.invocation == JavaInvocationKind::Instance
                && !static_method
                && receiver_owner.is_some_and(|owner| {
                    matches!(&declaration.heritage, JavaHeritage::Interfaces(values)
                        if values.contains(&JavaType::Reference(JavaTypeName::Generated(owner))))
                })
        }
    };
    if name_matches && signature_matches && declaration_matches {
        vec![]
    } else {
        vec![AstViolation::new(
            DiagnosticCode::InvalidInvocation,
            "Java method declaration does not match its authoritative registered callable",
        )]
    }
}

#[derive(Clone, Copy)]
enum JavaModifierSite {
    Type { top_level: bool },
    Field,
    Method,
    Constructor,
}

fn verify_modifiers_for(modifiers: &[JavaModifier], site: JavaModifierSite) -> Vec<AstViolation> {
    let distinct = modifiers.iter().copied().collect::<BTreeSet<_>>();
    let mut violations = Vec::new();
    if distinct.len() != modifiers.len() {
        violations.push(AstViolation::new(
            DiagnosticCode::DuplicateDeclaration,
            "Java modifier is repeated",
        ));
    }
    for (left, right, message) in [
        (
            JavaModifier::Public,
            JavaModifier::Private,
            "Java declaration cannot be both public and private",
        ),
        (
            JavaModifier::Abstract,
            JavaModifier::Final,
            "Java declaration cannot be both abstract and final",
        ),
        (
            JavaModifier::Sealed,
            JavaModifier::NonSealed,
            "Java declaration cannot be both sealed and non-sealed",
        ),
    ] {
        if distinct.contains(&left) && distinct.contains(&right) {
            violations.push(AstViolation::new(DiagnosticCode::InvalidStructure, message));
        }
    }
    let allowed = |modifier| match site {
        JavaModifierSite::Type { top_level } => modifier == JavaModifier::Static && !top_level,
        JavaModifierSite::Field => matches!(
            modifier,
            JavaModifier::Public
                | JavaModifier::Private
                | JavaModifier::Static
                | JavaModifier::Final
                | JavaModifier::Transient
        ),
        JavaModifierSite::Method => matches!(
            modifier,
            JavaModifier::Public
                | JavaModifier::Private
                | JavaModifier::Static
                | JavaModifier::Final
                | JavaModifier::Abstract
        ),
        JavaModifierSite::Constructor => {
            matches!(modifier, JavaModifier::Public | JavaModifier::Private)
        }
    };
    for modifier in &distinct {
        if !allowed(*modifier) {
            violations.push(AstViolation::new(
                DiagnosticCode::InvalidStructure,
                format!("Java modifier {modifier:?} is illegal in this declaration context"),
            ));
        }
    }
    if matches!(site, JavaModifierSite::Method)
        && distinct.contains(&JavaModifier::Abstract)
        && (distinct.contains(&JavaModifier::Static)
            || distinct.contains(&JavaModifier::Private)
            || distinct.contains(&JavaModifier::Final))
    {
        violations.push(AstViolation::new(
            DiagnosticCode::InvalidStructure,
            "abstract Java methods cannot be static, private, or final",
        ));
    }
    violations
}

impl JavaFileItem {
    pub fn declared_symbols(&self) -> Vec<GeneratedSymbolId> {
        match self {
            Self::Type { declared, .. } => declared.clone(),
            Self::RuntimeMembers { .. } => vec![],
        }
    }

    pub fn symbols(&self) -> Vec<TargetSymbolRef<JavaDialect>> {
        let mut symbols = BTreeSet::new();
        match self {
            Self::Type { declaration, .. } => declaration.symbols(&mut symbols),
            Self::RuntimeMembers { helper, members } => {
                for member in members {
                    member.symbols(&mut symbols);
                }
                symbols.remove(&TargetSymbolRef::RuntimeHelper(*helper));
            }
        }
        symbols.into_iter().collect()
    }
}

#[derive(Clone, Copy)]
struct JavaPrivilegedLiteralScope {
    tagged_storage_helper: Option<JavaRuntimeHelper>,
    tagged_constructor: Option<JavaKnownType>,
}

impl JavaPrivilegedLiteralScope {
    const FORBIDDEN: Self = Self {
        tagged_storage_helper: None,
        tagged_constructor: None,
    };
}

fn verify_privileged_literals_in_declaration(
    declaration: &JavaTypeDeclaration,
    helper: Option<JavaRuntimeHelper>,
) -> Vec<AstViolation> {
    let tagged_owner =
        helper.and_then(|helper| registered_tagged_runtime_type(helper, declaration));
    declaration
        .members
        .iter()
        .flat_map(|member| verify_privileged_literals_in_member(member, helper, tagged_owner))
        .collect()
}

fn registered_tagged_runtime_type(
    helper: JavaRuntimeHelper,
    declaration: &JavaTypeDeclaration,
) -> Option<JavaKnownType> {
    let candidate = match helper {
        JavaRuntimeHelper::Core => JavaKnownType::RuntimeResult,
        JavaRuntimeHelper::TaggedValues
            if declaration.name.as_str() == JavaKnownType::RuntimeOption.simple_name() =>
        {
            JavaKnownType::RuntimeOption
        }
        JavaRuntimeHelper::TaggedValues => JavaKnownType::RuntimeValueResult,
        _ => return None,
    };
    (declaration.name.as_str() == candidate.simple_name()
        && declaration.kind == JavaDeclarationKind::FinalClass)
        .then_some(candidate)
}

fn verify_privileged_literals_in_member(
    member: &JavaMember,
    helper: Option<JavaRuntimeHelper>,
    tagged_owner: Option<JavaKnownType>,
) -> Vec<AstViolation> {
    let tagged_storage_helper = helper.filter(|helper| {
        matches!(
            helper,
            JavaRuntimeHelper::Core | JavaRuntimeHelper::TaggedValues
        )
    });
    match member {
        JavaMember::Field(field) => field.initializer.as_ref().map_or_else(Vec::new, |value| {
            verify_privileged_literals_in_expression(value, JavaPrivilegedLiteralScope::FORBIDDEN)
        }),
        JavaMember::CompileFailField(field) => verify_privileged_literals_in_expression(
            &field.initializer,
            JavaPrivilegedLiteralScope::FORBIDDEN,
        ),
        JavaMember::Method(method) => method.body.as_ref().map_or_else(Vec::new, |body| {
            verify_privileged_literals_in_block(
                body,
                JavaPrivilegedLiteralScope {
                    tagged_storage_helper,
                    tagged_constructor: None,
                },
            )
        }),
        JavaMember::Constructor(constructor) => verify_privileged_literals_in_block(
            &constructor.body,
            JavaPrivilegedLiteralScope {
                tagged_storage_helper,
                tagged_constructor: tagged_owner,
            },
        ),
        JavaMember::NestedType(declaration) => {
            verify_privileged_literals_in_declaration(declaration, helper)
        }
    }
}

fn verify_privileged_literals_in_block(
    block: &JavaBlock,
    scope: JavaPrivilegedLiteralScope,
) -> Vec<AstViolation> {
    let mut violations = Vec::new();
    for statement in &block.statements {
        match statement {
            JavaStmt::Local { value, .. } | JavaStmt::Return(value) => {
                if let Some(value) = value {
                    violations.extend(verify_privileged_literals_in_expression(value, scope));
                }
            }
            JavaStmt::Assign { target, value } => {
                violations.extend(verify_privileged_literals_in_expression(target, scope));
                violations.extend(verify_privileged_literals_in_expression(value, scope));
            }
            JavaStmt::Expression(value)
            | JavaStmt::Throw(value)
            | JavaStmt::ThrowAssertion(value) => {
                violations.extend(verify_privileged_literals_in_expression(value, scope));
            }
            JavaStmt::If {
                condition,
                then_block,
                else_block,
            } => {
                violations.extend(verify_privileged_literals_in_expression(condition, scope));
                violations.extend(verify_privileged_literals_in_block(then_block, scope));
                if let Some(else_block) = else_block {
                    violations.extend(verify_privileged_literals_in_block(else_block, scope));
                }
            }
            JavaStmt::ForEach { iterable, body, .. } => {
                violations.extend(verify_privileged_literals_in_expression(iterable, scope));
                violations.extend(verify_privileged_literals_in_block(body, scope));
            }
            JavaStmt::While { condition, body } => {
                violations.extend(verify_privileged_literals_in_expression(condition, scope));
                violations.extend(verify_privileged_literals_in_block(body, scope));
            }
            JavaStmt::Switch { value, arms } => {
                violations.extend(verify_privileged_literals_in_expression(value, scope));
                for arm in arms {
                    if matches!(
                        arm.pattern,
                        JavaPattern::Literal(
                            JavaLiteral::Utf16Units(_) | JavaLiteral::InternalNull(_)
                        )
                    ) {
                        violations.push(privileged_literal_error(
                            "privileged Java literal cannot be used as a switch label",
                        ));
                    }
                    violations.extend(verify_privileged_literals_in_block(&arm.body, scope));
                }
            }
            JavaStmt::TryCatch { try_block, catches } => {
                violations.extend(verify_privileged_literals_in_block(try_block, scope));
                for catch in catches {
                    violations.extend(verify_privileged_literals_in_block(&catch.body, scope));
                }
            }
            JavaStmt::Break | JavaStmt::Continue => {}
        }
    }
    violations
}

fn verify_privileged_literals_in_expression(
    expression: &JavaExpr,
    scope: JavaPrivilegedLiteralScope,
) -> Vec<AstViolation> {
    let mut violations = Vec::new();
    match &expression.kind {
        JavaExprKind::Literal(JavaLiteral::Utf16Units(_)) => {
            violations.push(privileged_literal_error(
                "raw UTF-16-unit literal is not admitted in verified executable Java AST",
            ))
        }
        JavaExprKind::Literal(JavaLiteral::InternalNull(_)) => {
            violations.push(privileged_literal_error(
                "internal null literal is outside exact registered tagged runtime storage",
            ))
        }
        JavaExprKind::Literal(_) | JavaExprKind::Value(_) => {}
        JavaExprKind::Unary { operand, .. } => {
            violations.extend(verify_privileged_literals_in_expression(operand, scope));
        }
        JavaExprKind::Binary {
            operator,
            left,
            right,
        } => {
            let tagged_null_check = matches!(
                operator,
                JavaBinaryOperator::Equal | JavaBinaryOperator::NotEqual
            ) && scope.tagged_constructor.is_some();
            if !(tagged_null_check
                && registered_tagged_null_check(left, right, scope.tagged_constructor.unwrap()))
            {
                violations.extend(verify_privileged_literals_in_expression(left, scope));
            }
            if !(tagged_null_check
                && registered_tagged_null_check(right, left, scope.tagged_constructor.unwrap()))
            {
                violations.extend(verify_privileged_literals_in_expression(right, scope));
            }
        }
        JavaExprKind::Conditional {
            condition,
            when_true,
            when_false,
        } => {
            violations.extend(verify_privileged_literals_in_expression(condition, scope));
            violations.extend(verify_privileged_literals_in_expression(when_true, scope));
            violations.extend(verify_privileged_literals_in_expression(when_false, scope));
        }
        JavaExprKind::Call {
            receiver,
            arguments,
            ..
        } => {
            if let Some(receiver) = receiver {
                violations.extend(verify_privileged_literals_in_expression(receiver, scope));
            }
            for argument in arguments {
                violations.extend(verify_privileged_literals_in_expression(argument, scope));
            }
        }
        JavaExprKind::New {
            constructor,
            arguments,
        } => {
            for (index, argument) in arguments.iter().enumerate() {
                if !scope.tagged_storage_helper.is_some_and(|helper| {
                    registered_inactive_tagged_storage_argument(
                        helper,
                        constructor,
                        arguments,
                        index,
                        argument,
                    )
                }) {
                    violations.extend(verify_privileged_literals_in_expression(argument, scope));
                }
            }
        }
        JavaExprKind::NewArray { length, .. } => {
            violations.extend(verify_privileged_literals_in_expression(length, scope));
        }
        JavaExprKind::ArrayIndex { array, index } => {
            violations.extend(verify_privileged_literals_in_expression(array, scope));
            violations.extend(verify_privileged_literals_in_expression(index, scope));
        }
        JavaExprKind::Field { receiver, .. } => {
            violations.extend(verify_privileged_literals_in_expression(receiver, scope));
        }
        JavaExprKind::Cast { value, .. }
        | JavaExprKind::ArrayOwnershipTransition { value, .. }
        | JavaExprKind::InstanceOf { value, .. } => {
            violations.extend(verify_privileged_literals_in_expression(value, scope));
        }
        JavaExprKind::Lambda { body, .. } => {
            violations.extend(verify_privileged_literals_in_block(body, scope));
        }
    }
    violations
}

fn registered_tagged_null_check(null: &JavaExpr, payload: &JavaExpr, owner: JavaKnownType) -> bool {
    matches!(
        &null.kind,
        JavaExprKind::Literal(JavaLiteral::InternalNull(
            JavaNullPurpose::AbsentTaggedPayload
        ))
    ) && null.ty == payload.ty
        && matches!(
            &payload.kind,
            JavaExprKind::Value(JavaValueRef::Local(name))
                if match owner {
                    JavaKnownType::RuntimeOption => name.as_str() == "value",
                    JavaKnownType::RuntimeResult | JavaKnownType::RuntimeValueResult => {
                        matches!(name.as_str(), "value" | "error")
                    }
                    _ => false,
                }
        )
}

fn registered_inactive_tagged_storage_argument(
    helper: JavaRuntimeHelper,
    constructor: &JavaConstructorRef,
    arguments: &[JavaExpr],
    index: usize,
    argument: &JavaExpr,
) -> bool {
    if !matches!(
        argument.kind,
        JavaExprKind::Literal(JavaLiteral::InternalNull(
            JavaNullPurpose::AbsentTaggedPayload
        ))
    ) {
        return false;
    }
    let Some(JavaExpr {
        kind: JavaExprKind::Literal(JavaLiteral::Boolean(active)),
        ..
    }) = arguments.first()
    else {
        return false;
    };
    let JavaConstructorRef::Known { constructor, .. } = constructor else {
        return false;
    };
    if !matches!(
        (helper, constructor),
        (JavaRuntimeHelper::Core, JavaKnownConstructor::RuntimeResult)
            | (
                JavaRuntimeHelper::TaggedValues,
                JavaKnownConstructor::RuntimeOption | JavaKnownConstructor::RuntimeValueResult
            )
    ) {
        return false;
    }
    match constructor {
        JavaKnownConstructor::RuntimeOption => !active && index == 1 && arguments.len() == 2,
        JavaKnownConstructor::RuntimeResult | JavaKnownConstructor::RuntimeValueResult => {
            ((*active && index == 2) || (!active && index == 1)) && arguments.len() == 3
        }
        _ => false,
    }
}

fn privileged_literal_error(message: &str) -> AstViolation {
    AstViolation::new(DiagnosticCode::InvalidStructure, message)
}

impl TargetFileItemNode<JavaDialect> for JavaFileItem {
    fn verify(&self, context: &TargetAstContext<'_, JavaDialect>) -> Vec<AstViolation> {
        match self {
            Self::Type {
                declared,
                declaration,
            } => {
                let mut violations = declaration.verify(context, true);
                violations.extend(verify_privileged_literals_in_declaration(declaration, None));
                for symbol in declared {
                    let present = match symbol {
                        GeneratedSymbolId::Type(id) => context.generated_type(*id).is_some(),
                        GeneratedSymbolId::Callable(id) => context.callable(*id).is_some(),
                        GeneratedSymbolId::InterfaceMethod(id) => {
                            context.interface_method(*id).is_some()
                        }
                        GeneratedSymbolId::Value(id) => context.value(*id).is_some(),
                    };
                    if !present {
                        violations.push(AstViolation::new(
                            DiagnosticCode::UnresolvedReference,
                            "file item declares an unknown generated symbol",
                        ));
                    }
                }
                violations
            }
            Self::RuntimeMembers { helper, members } => {
                let variables = BTreeSet::new();
                let mut violations = members
                    .iter()
                    .flat_map(|value| {
                        let mut violations = value.verify(context);
                        violations.extend(verify_member_type_context(value, &variables, context));
                        violations
                    })
                    .collect::<Vec<_>>();
                for member in members {
                    violations.extend(verify_privileged_literals_in_member(
                        member,
                        Some(*helper),
                        None,
                    ));
                }
                violations
            }
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ResolvedJavaFileItem {
    pub item: JavaFileItem,
    pub names: BTreeMap<TargetSymbolRef<JavaDialect>, JavaResolvedName>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum JavaResolvedName {
    Local(JavaIdentifier),
    Qualified(crate::dialect::JavaQualifiedName),
    GeneratedMember {
        owner: crate::dialect::JavaGeneratedContainer,
        member: JavaIdentifier,
    },
    Member {
        owner: crate::dialect::JavaQualifiedName,
        member: crate::dialect::JavaMemberName,
    },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum JavaPackage {
    Generated,
}

impl JavaPackage {
    pub const fn name(self) -> &'static str {
        match self {
            Self::Generated => "org.polyrust.generated",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum JavaFilePlacement {
    Main,
    Runtime,
    NativeTest,
    Conformance,
    NegativeTest,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum JavaTemplateId {
    CompilationUnit,
    Package,
    Import,
    Annotation,
    Heritage,
    Class,
    Record,
    Interface,
    SealedInterface,
    Field,
    Method,
    AbstractMethod,
    Constructor,
    Parameter,
    Block,
    Local,
    Assign,
    ExpressionStatement,
    Return,
    If,
    ForEach,
    While,
    Switch,
    SwitchArm,
    Throw,
    Break,
    Continue,
    Literal,
    Name,
    Unary,
    Binary,
    Conditional,
    Call,
    New,
    FieldAccess,
    Cast,
    InstanceOf,
    NewArray,
    ArrayIndex,
    Lambda,
    TryCatch,
    Catch,
    ThrowValue,
    Comment,
}

impl JavaTemplateId {
    pub const ALL: [Self; 44] = [
        Self::CompilationUnit,
        Self::Package,
        Self::Import,
        Self::Annotation,
        Self::Heritage,
        Self::Class,
        Self::Record,
        Self::Interface,
        Self::SealedInterface,
        Self::Field,
        Self::Method,
        Self::AbstractMethod,
        Self::Constructor,
        Self::Parameter,
        Self::Block,
        Self::Local,
        Self::Assign,
        Self::ExpressionStatement,
        Self::Return,
        Self::If,
        Self::ForEach,
        Self::While,
        Self::Switch,
        Self::SwitchArm,
        Self::Throw,
        Self::Break,
        Self::Continue,
        Self::Literal,
        Self::Name,
        Self::Unary,
        Self::Binary,
        Self::Conditional,
        Self::Call,
        Self::New,
        Self::FieldAccess,
        Self::Cast,
        Self::InstanceOf,
        Self::NewArray,
        Self::ArrayIndex,
        Self::Lambda,
        Self::TryCatch,
        Self::Catch,
        Self::ThrowValue,
        Self::Comment,
    ];
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct JavaCompilationUnit {
    pub package: JavaPackage,
    pub imports: Vec<crate::dialect::JavaImportKind>,
    pub declarations: Vec<JavaTypeDeclaration>,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn verifier_source(label: &str) -> portable_diagnostics::SourceRef {
        portable_diagnostics::SourceRef::logical(["java-ast-verifier-test", label])
    }

    fn fixture_declaration(members: Vec<JavaMember>) -> JavaTypeDeclaration {
        JavaTypeDeclaration {
            declared: None,
            kind: JavaDeclarationKind::FinalClass,
            visibility: JavaVisibility::Package,
            modifiers: vec![],
            name: JavaIdentifier::from_portable("Fixture"),
            type_parameters: vec![],
            record_components: vec![],
            heritage: JavaHeritage::None,
            permits: vec![],
            members,
        }
    }

    fn verify_fixture(
        builder: portable_codegen::TargetAstBuilder<JavaDialect>,
        declarations: Vec<(Vec<GeneratedSymbolId>, JavaTypeDeclaration)>,
    ) -> Result<(), Vec<portable_diagnostics::Diagnostic>> {
        verify_file_items(
            builder,
            portable_codegen::SourceRole::PublicApi,
            JavaFilePlacement::Main,
            declarations
                .into_iter()
                .map(|(declared, declaration)| JavaFileItem::Type {
                    declared,
                    declaration,
                })
                .collect(),
        )
    }

    fn verify_file_items(
        mut builder: portable_codegen::TargetAstBuilder<JavaDialect>,
        role: portable_codegen::SourceRole,
        placement: JavaFilePlacement,
        items: Vec<JavaFileItem>,
    ) -> Result<(), Vec<portable_diagnostics::Diagnostic>> {
        let group_role = match role {
            portable_codegen::SourceRole::PublicApi => portable_codegen::FileGroupRole::PublicApi,
            portable_codegen::SourceRole::Implementation => {
                portable_codegen::FileGroupRole::Implementation
            }
            portable_codegen::SourceRole::Runtime => portable_codegen::FileGroupRole::Runtime,
            portable_codegen::SourceRole::NativeTest => {
                portable_codegen::FileGroupRole::NativeTests
            }
            portable_codegen::SourceRole::Conformance => {
                portable_codegen::FileGroupRole::Conformance
            }
            portable_codegen::SourceRole::NegativeTest => {
                portable_codegen::FileGroupRole::NegativeTests
            }
        };
        let file = builder.file(portable_codegen::TargetFile::new(
            portable_codegen::RelativeOutputPath::new("Fixture.java").unwrap(),
            role,
            JavaPackage::Generated,
            placement,
            items,
            JavaTemplateId::CompilationUnit,
            verifier_source("file"),
        ));
        builder.group(portable_codegen::TargetFileGroup::new(
            group_role,
            vec![portable_codegen::TargetFileMember::Source(file)],
            verifier_source("group"),
        ));
        portable_codegen::verify_target_ast(&builder.build())
    }

    fn structural_method(
        name: &str,
        return_type: JavaType,
        parameters: Vec<JavaParameter>,
        body: JavaBlock,
    ) -> JavaMember {
        JavaMember::Method(JavaMethod {
            declared: JavaMethodDeclaration::Structural,
            annotations: vec![],
            modifiers: vec![JavaModifier::Public, JavaModifier::Static],
            type_parameters: vec![],
            return_type,
            name: JavaIdentifier::from_portable(name),
            parameters,
            body: Some(body),
        })
    }

    fn parameter(ty: JavaType, name: &str) -> JavaParameter {
        JavaParameter {
            ty,
            name: JavaIdentifier::from_portable(name),
            final_parameter: true,
        }
    }

    fn instanceof(value: JavaExpr, target: JavaType, binding: &str) -> JavaExpr {
        JavaExpr {
            ty: JavaType::primitive(JavaPrimitive::Boolean),
            precedence: JavaPrecedence::Relational,
            kind: JavaExprKind::InstanceOf {
                value: Box::new(value),
                target,
                binding: Some(JavaIdentifier::from_portable(binding)),
            },
        }
    }

    fn this_field(owner: JavaType, ty: JavaType, name: &str) -> JavaExpr {
        JavaExpr {
            ty: ty.clone(),
            precedence: JavaPrecedence::Primary,
            kind: JavaExprKind::Field {
                receiver: Box::new(JavaExpr {
                    ty: owner,
                    precedence: JavaPrecedence::Primary,
                    kind: JavaExprKind::Value(JavaValueRef::This),
                }),
                field: JavaFieldRef::Structural {
                    name: JavaIdentifier::from_portable(name),
                    ty,
                },
            },
        }
    }

    fn fixture_core_field() -> CoreFieldId {
        let checked = portable_check::v0::check_program(
            portable_ir::v0::from_json(include_bytes!(
                "../../build/testdata/registration.poly.json"
            ))
            .expect("fixture parses"),
        )
        .expect("fixture checks");
        let core = portable_core_ir::lower_checked(&checked).expect("fixture lowers to CoreIR");
        core.records()
            .first()
            .and_then(|record| record.fields.first())
            .copied()
            .expect("fixture contains a record field")
    }

    #[test]
    fn identifiers_and_type_positions_fail_closed() {
        assert!(JavaIdentifier::new("valid_name").is_ok());
        assert!(JavaIdentifier::new("class").is_err());
        assert!(JavaIdentifier::new("false").is_err());
        assert!(JavaIdentifier::new("_").is_err());
        assert!(JavaIdentifier::new("9invalid").is_err());
        assert_eq!(JavaIdentifier::from_portable("false").as_str(), "false_");
        assert_eq!(JavaIdentifier::from_portable("_").as_str(), "__");
        assert_eq!(
            JavaIdentifier::from_portable("__polyrust_callResult_0").as_str(),
            "__polyrust_callResult_0_user"
        );
        assert_eq!(
            JavaIdentifier::from_portable("match-value").as_str(),
            "match_value"
        );
        assert_eq!(
            JAVA_KEYWORDS.iter().copied().collect::<BTreeSet<_>>().len(),
            JAVA_KEYWORDS.len()
        );
        for keyword in JAVA_KEYWORDS {
            assert!(
                JavaIdentifier::new(*keyword).is_err(),
                "protected Java spelling was accepted: {keyword}"
            );
        }

        assert!(
            !JavaType::primitive(JavaPrimitive::Void)
                .verify(JavaTypeUse::Value)
                .is_empty()
        );
        assert!(
            !JavaType::primitive(JavaPrimitive::Int)
                .verify(JavaTypeUse::GenericArgument)
                .is_empty()
        );
        assert!(
            !JavaType::Wildcard { bound: None }
                .verify(JavaTypeUse::Value)
                .is_empty()
        );
        assert!(
            JavaType::Wildcard { bound: None }
                .verify(JavaTypeUse::GenericArgument)
                .is_empty()
        );
        assert!(
            !JavaType::Generic {
                raw: JavaTypeName::Known(JavaKnownType::List),
                arguments: vec![],
            }
            .verify(JavaTypeUse::Value)
            .is_empty()
        );
    }

    #[test]
    fn literal_and_operator_signatures_reject_false_type_claims() {
        let boolean = JavaType::primitive(JavaPrimitive::Boolean);
        let int = JavaType::primitive(JavaPrimitive::Int);
        let long = JavaType::primitive(JavaPrimitive::Long);
        let string = JavaType::known(JavaKnownType::String);

        assert!(literal_matches_type(&JavaLiteral::Boolean(true), &boolean));
        assert!(!literal_matches_type(&JavaLiteral::Boolean(true), &int));
        assert!(literal_matches_type(
            &JavaLiteral::Utf16Units(vec![0xd800]),
            &string
        ));
        assert!(!literal_matches_type(
            &JavaLiteral::InternalNull(JavaNullPurpose::AbsentTaggedPayload),
            &long
        ));
        assert!(unary_signature_matches(
            JavaUnaryOperator::Not,
            &boolean,
            &boolean
        ));
        assert!(!unary_signature_matches(
            JavaUnaryOperator::Not,
            &int,
            &boolean
        ));
        assert!(binary_signature_matches(
            JavaBinaryOperator::Add,
            &int,
            &int,
            &int
        ));
        assert!(!binary_signature_matches(
            JavaBinaryOperator::Add,
            &int,
            &long,
            &long
        ));
        assert!(binary_signature_matches(
            JavaBinaryOperator::Add,
            &string,
            &string,
            &string
        ));
        assert!(!binary_signature_matches(
            JavaBinaryOperator::ShiftLeft,
            &long,
            &long,
            &long
        ));
    }

    #[test]
    fn privileged_literals_fail_closed_outside_registered_runtime_storage() {
        let string = JavaType::known(JavaKnownType::String);
        let declaration = fixture_declaration(vec![
            JavaMember::Field(JavaField {
                declared: None,
                modifiers: vec![JavaModifier::Static, JavaModifier::Final],
                ty: string.clone(),
                name: JavaIdentifier::from_portable("rawSurrogate"),
                initializer: Some(JavaExpr::literal(
                    string.clone(),
                    JavaLiteral::Utf16Units(vec![0xd800]),
                )),
            }),
            structural_method(
                "leakNull",
                string.clone(),
                vec![],
                JavaBlock::new(vec![JavaStmt::Return(Some(JavaExpr::literal(
                    string,
                    JavaLiteral::InternalNull(JavaNullPurpose::AbsentTaggedPayload),
                )))]),
            ),
        ]);
        let diagnostics = verify_fixture(
            portable_codegen::TargetAstBuilder::new(JavaDialect),
            vec![(vec![], declaration)],
        )
        .unwrap_err();
        assert!(diagnostics.iter().any(|value| {
            value.code == DiagnosticCode::InvalidStructure
                && value.message.contains("raw UTF-16-unit literal")
        }));
        assert!(diagnostics.iter().any(|value| {
            value.code == DiagnosticCode::InvalidStructure
                && value
                    .message
                    .contains("outside exact registered tagged runtime storage")
        }));
    }

    #[test]
    fn registered_runtime_inactive_tagged_storage_remains_valid() {
        let items = [JavaRuntimeHelper::Core, JavaRuntimeHelper::TaggedValues]
            .into_iter()
            .flat_map(crate::runtime::helper_items)
            .collect();
        let verification = verify_file_items(
            portable_codegen::TargetAstBuilder::new(JavaDialect),
            portable_codegen::SourceRole::Runtime,
            JavaFilePlacement::Runtime,
            items,
        );
        assert!(verification.is_ok(), "{verification:?}");
    }

    #[test]
    fn statement_expressions_and_switch_patterns_fail_closed() {
        let int = JavaType::primitive(JavaPrimitive::Int);
        let object = JavaType::known(JavaKnownType::Object);
        let string = JavaType::known(JavaKnownType::String);
        let declaration = fixture_declaration(vec![
            structural_method(
                "badStatement",
                JavaType::primitive(JavaPrimitive::Void),
                vec![],
                JavaBlock::new(vec![JavaStmt::Expression(JavaExpr::literal(
                    int.clone(),
                    JavaLiteral::I32(1),
                ))]),
            ),
            structural_method(
                "badConstants",
                JavaType::primitive(JavaPrimitive::Void),
                vec![parameter(int.clone(), "selector")],
                JavaBlock::new(vec![JavaStmt::Switch {
                    value: JavaExpr::local(int.clone(), JavaIdentifier::from_portable("selector")),
                    arms: vec![
                        JavaSwitchArm {
                            pattern: JavaPattern::Literal(JavaLiteral::I32(1)),
                            body: JavaBlock::new(vec![]),
                        },
                        JavaSwitchArm {
                            pattern: JavaPattern::Literal(JavaLiteral::CharScalar(1)),
                            body: JavaBlock::new(vec![]),
                        },
                        JavaSwitchArm {
                            pattern: JavaPattern::Literal(JavaLiteral::String("one".to_owned())),
                            body: JavaBlock::new(vec![]),
                        },
                        JavaSwitchArm {
                            pattern: JavaPattern::Default,
                            body: JavaBlock::new(vec![]),
                        },
                    ],
                }]),
            ),
            structural_method(
                "badDominance",
                JavaType::primitive(JavaPrimitive::Void),
                vec![parameter(object.clone(), "selector")],
                JavaBlock::new(vec![JavaStmt::Switch {
                    value: JavaExpr::local(object, JavaIdentifier::from_portable("selector")),
                    arms: vec![
                        JavaSwitchArm {
                            pattern: JavaPattern::Type {
                                ty: JavaType::known(JavaKnownType::Object),
                                binding: JavaIdentifier::from_portable("anything"),
                            },
                            body: JavaBlock::new(vec![]),
                        },
                        JavaSwitchArm {
                            pattern: JavaPattern::Type {
                                ty: string,
                                binding: JavaIdentifier::from_portable("text"),
                            },
                            body: JavaBlock::new(vec![]),
                        },
                        JavaSwitchArm {
                            pattern: JavaPattern::Default,
                            body: JavaBlock::new(vec![]),
                        },
                    ],
                }]),
            ),
        ]);
        let diagnostics = verify_fixture(
            portable_codegen::TargetAstBuilder::new(JavaDialect),
            vec![(vec![], declaration)],
        )
        .unwrap_err();
        assert!(diagnostics.iter().any(|value| {
            value.code == DiagnosticCode::InvalidStructure
                && value.message.contains("expression statement")
        }));
        assert!(diagnostics.iter().any(|value| {
            value.code == DiagnosticCode::DuplicateDeclaration
                && value.message.contains("constant label")
        }));
        assert!(diagnostics.iter().any(|value| {
            value.code == DiagnosticCode::TypeMismatch
                && value.message.contains("literal is not compatible")
        }));
        assert!(diagnostics.iter().any(|value| {
            value.code == DiagnosticCode::InvalidControlFlow
                && value.message.contains("dominated by an earlier pattern")
        }));
    }

    #[test]
    fn valid_statement_expressions_and_switch_patterns_verify() {
        let int = JavaType::primitive(JavaPrimitive::Int);
        let string = JavaType::known(JavaKnownType::String);
        let assertion = JavaType::known(JavaKnownType::AssertionError);
        let constructor = JavaKnownConstructor::AssertionErrorString;
        let expression = JavaExpr {
            ty: assertion.clone(),
            precedence: JavaPrecedence::Primary,
            kind: JavaExprKind::New {
                constructor: JavaConstructorRef::Known {
                    constructor,
                    owner: assertion,
                    parameters: vec![string.clone()],
                },
                arguments: vec![JavaExpr::literal(
                    string,
                    JavaLiteral::String("discarded".to_owned()),
                )],
            },
        };
        let declaration = fixture_declaration(vec![structural_method(
            "validGrammar",
            JavaType::primitive(JavaPrimitive::Void),
            vec![parameter(int.clone(), "selector")],
            JavaBlock::new(vec![
                JavaStmt::Expression(expression),
                JavaStmt::Switch {
                    value: JavaExpr::local(int, JavaIdentifier::from_portable("selector")),
                    arms: vec![
                        JavaSwitchArm {
                            pattern: JavaPattern::Literal(JavaLiteral::I32(1)),
                            body: JavaBlock::new(vec![]),
                        },
                        JavaSwitchArm {
                            pattern: JavaPattern::Default,
                            body: JavaBlock::new(vec![]),
                        },
                    ],
                },
            ]),
        )]);
        assert!(
            verify_fixture(
                portable_codegen::TargetAstBuilder::new(JavaDialect),
                vec![(vec![], declaration)],
            )
            .is_ok()
        );
    }

    #[test]
    fn array_ownership_transitions_require_a_fresh_internal_copy() {
        let byte = JavaType::primitive(JavaPrimitive::Byte);
        let internal = JavaType::Array {
            component: Box::new(byte.clone()),
            ownership: JavaArrayOwnership::InternalMutable,
        };
        let boundary = JavaType::Array {
            component: Box::new(byte.clone()),
            ownership: JavaArrayOwnership::DefensiveCopyBoundary,
        };
        let fresh = JavaExpr {
            ty: internal,
            precedence: JavaPrecedence::Primary,
            kind: JavaExprKind::NewArray {
                component: byte,
                length: Box::new(JavaExpr::literal(
                    JavaType::primitive(JavaPrimitive::Int),
                    JavaLiteral::I32(1),
                )),
            },
        };
        let promoted = JavaExpr {
            ty: boundary.clone(),
            precedence: JavaPrecedence::Primary,
            kind: JavaExprKind::ArrayOwnershipTransition {
                transition: JavaArrayOwnershipTransition::FreshCopyToBoundary,
                value: Box::new(fresh),
            },
        };
        assert!(
            verify_fixture(
                portable_codegen::TargetAstBuilder::new(JavaDialect),
                vec![(
                    vec![],
                    fixture_declaration(vec![structural_method(
                        "validCopy",
                        boundary.clone(),
                        vec![],
                        JavaBlock::new(vec![JavaStmt::Return(Some(promoted))]),
                    )]),
                )],
            )
            .is_ok()
        );

        let invalid = JavaExpr {
            ty: boundary.clone(),
            precedence: JavaPrecedence::Primary,
            kind: JavaExprKind::ArrayOwnershipTransition {
                transition: JavaArrayOwnershipTransition::FreshCopyToBoundary,
                value: Box::new(JavaExpr::local(
                    boundary.clone(),
                    JavaIdentifier::from_portable("value"),
                )),
            },
        };
        assert!(
            verify_fixture(
                portable_codegen::TargetAstBuilder::new(JavaDialect),
                vec![(
                    vec![],
                    fixture_declaration(vec![structural_method(
                        "invalidCopy",
                        boundary.clone(),
                        vec![parameter(boundary, "value")],
                        JavaBlock::new(vec![JavaStmt::Return(Some(invalid))]),
                    )]),
                )],
            )
            .is_err()
        );
    }

    #[test]
    fn compile_fail_members_are_explicit_and_discoverable() {
        let declaration = JavaTypeDeclaration {
            declared: None,
            kind: JavaDeclarationKind::FinalClass,
            visibility: JavaVisibility::Package,
            modifiers: vec![],
            name: JavaIdentifier::from_portable("InvalidTypes"),
            type_parameters: vec![],
            record_components: vec![],
            heritage: JavaHeritage::None,
            permits: vec![],
            members: vec![JavaMember::CompileFailField(JavaCompileFailField {
                modifiers: vec![JavaModifier::Final],
                expected_type: JavaType::generic(
                    JavaKnownType::RuntimeOption,
                    vec![JavaType::Boxed(JavaPrimitive::Int)],
                ),
                name: JavaIdentifier::from_portable("invalid"),
                initializer: JavaExpr::literal(
                    JavaType::known(JavaKnownType::String),
                    JavaLiteral::String("missing".to_owned()),
                ),
            })],
        };
        assert!(declaration.contains_compile_fail_member());
    }

    #[test]
    fn lexical_verifier_rejects_unresolved_final_assignment_and_wrong_return() {
        let int = JavaType::primitive(JavaPrimitive::Int);
        let boolean = JavaType::primitive(JavaPrimitive::Boolean);
        let method = JavaMethod {
            declared: JavaMethodDeclaration::Structural,
            annotations: vec![],
            modifiers: vec![JavaModifier::Static],
            type_parameters: vec![],
            return_type: boolean.clone(),
            name: JavaIdentifier::from_portable("invalid"),
            parameters: vec![JavaParameter {
                ty: int.clone(),
                name: JavaIdentifier::from_portable("fixed"),
                final_parameter: true,
            }],
            body: None,
        };
        let (mut scope, initial) = JavaLexicalScope::for_method(&method);
        assert!(initial.is_empty());
        let block = JavaBlock::new(vec![
            JavaStmt::Assign {
                target: JavaExpr::local(int.clone(), JavaIdentifier::from_portable("fixed")),
                value: JavaExpr::literal(int.clone(), JavaLiteral::I32(1)),
            },
            JavaStmt::Expression(JavaExpr::local(
                int.clone(),
                JavaIdentifier::from_portable("missing"),
            )),
            JavaStmt::Return(Some(JavaExpr::literal(int, JavaLiteral::I32(2)))),
        ]);
        let violations = verify_block_scope(&block, &mut scope, &boolean, false);
        assert!(
            violations
                .iter()
                .any(|value| value.code == DiagnosticCode::InvalidControlFlow)
        );
        assert!(
            violations
                .iter()
                .any(|value| value.code == DiagnosticCode::UnresolvedReference)
        );
        assert!(
            violations
                .iter()
                .any(|value| value.code == DiagnosticCode::TypeMismatch)
        );
        assert!(block_guarantees_exit(&block));
        assert!(!block_guarantees_exit(&JavaBlock::new(vec![])));
    }

    #[test]
    fn runtime_member_catalogue_rejects_false_owner_and_result_claims() {
        assert_eq!(JavaRuntimeMember::ALL.len(), 14);
        let valid = JavaMethodSignature {
            receiver: Some(JavaType::known(JavaKnownType::RuntimeScalar)),
            parameters: vec![],
            result: JavaType::primitive(JavaPrimitive::Int),
            checked_exceptions: vec![],
            nullable_result: false,
            pure: true,
        };
        assert!(JavaRuntimeMember::ScalarValue.accepts(&valid));
        let mut wrong = valid.clone();
        wrong.result = JavaType::primitive(JavaPrimitive::Long);
        assert!(!JavaRuntimeMember::ScalarValue.accepts(&wrong));
        wrong = valid;
        wrong.receiver = Some(JavaType::known(JavaKnownType::RuntimeError));
        assert!(!JavaRuntimeMember::ScalarValue.accepts(&wrong));
    }

    #[test]
    fn value_references_match_authoritative_registered_and_known_field_types() {
        let mut builder = portable_codegen::TargetAstBuilder::new(JavaDialect);
        let generated = builder.value(portable_codegen::GeneratedValue {
            name: "number".to_owned(),
            ty: TargetTypeRef::Primitive(JavaPrimitive::Int),
            origin: portable_codegen::GeneratedOrigin::Synthesized(
                portable_codegen::SynthesisReason::TestHarness,
            ),
            source: verifier_source("value"),
        });
        let string = JavaType::known(JavaKnownType::String);
        let forged_generated = JavaExpr {
            ty: string.clone(),
            precedence: JavaPrecedence::Primary,
            kind: JavaExprKind::Value(JavaValueRef::Generated(GeneratedSymbolId::Value(generated))),
        };
        let declaration = fixture_declaration(vec![JavaMember::Field(JavaField {
            declared: Some(generated),
            modifiers: vec![JavaModifier::Static, JavaModifier::Final],
            ty: string.clone(),
            name: JavaIdentifier::from_portable("forged"),
            initializer: Some(forged_generated),
        })]);
        let diagnostics = verify_fixture(
            builder,
            vec![(vec![GeneratedSymbolId::Value(generated)], declaration)],
        )
        .unwrap_err();
        assert!(diagnostics.iter().any(|value| {
            value.code == DiagnosticCode::TypeMismatch
                && value.message.contains("authoritative registration")
        }));

        let builder = portable_codegen::TargetAstBuilder::new(JavaDialect);
        let forged_known = JavaExpr {
            ty: string.clone(),
            precedence: JavaPrecedence::Primary,
            kind: JavaExprKind::Value(JavaValueRef::KnownField(
                crate::dialect::JavaKnownField::IntegerMaxValue,
            )),
        };
        let declaration = fixture_declaration(vec![JavaMember::Field(JavaField {
            declared: None,
            modifiers: vec![JavaModifier::Static, JavaModifier::Final],
            ty: string,
            name: JavaIdentifier::from_portable("forged"),
            initializer: Some(forged_known),
        })]);
        let diagnostics = verify_fixture(builder, vec![(vec![], declaration)]).unwrap_err();
        assert!(diagnostics.iter().any(|value| {
            value.code == DiagnosticCode::TypeMismatch && value.message.contains("catalogue entry")
        }));

        let builder = portable_codegen::TargetAstBuilder::new(JavaDialect);
        let int = JavaType::primitive(JavaPrimitive::Int);
        let valid_known = JavaExpr {
            ty: int.clone(),
            precedence: JavaPrecedence::Primary,
            kind: JavaExprKind::Value(JavaValueRef::KnownField(
                crate::dialect::JavaKnownField::IntegerMaxValue,
            )),
        };
        let declaration = fixture_declaration(vec![JavaMember::Field(JavaField {
            declared: None,
            modifiers: vec![JavaModifier::Static, JavaModifier::Final],
            ty: int,
            name: JavaIdentifier::from_portable("valid"),
            initializer: Some(valid_known),
        })]);
        assert!(verify_fixture(builder, vec![(vec![], declaration)]).is_ok());
    }

    fn decoder_decode_call() -> (JavaExpr, Vec<JavaParameter>) {
        let decoder = JavaType::known(JavaKnownType::CharsetDecoder);
        let buffer = JavaType::known(JavaKnownType::ByteBuffer);
        let signature = crate::dialect::JavaKnownMethod::DecoderDecode.signature();
        (
            JavaExpr {
                ty: signature.result.clone(),
                precedence: JavaPrecedence::Primary,
                kind: JavaExprKind::Call {
                    callable: JavaCallableRef::Member {
                        owner: decoder.clone(),
                        name: JavaIdentifier::from_portable("decode"),
                        signature,
                        origin: JavaMemberOrigin::Known(
                            crate::dialect::JavaKnownMethod::DecoderDecode,
                        ),
                    },
                    receiver: Some(Box::new(JavaExpr::local(
                        decoder.clone(),
                        JavaIdentifier::from_portable("decoder"),
                    ))),
                    arguments: vec![JavaExpr::local(
                        buffer.clone(),
                        JavaIdentifier::from_portable("buffer"),
                    )],
                },
            },
            vec![parameter(decoder, "decoder"), parameter(buffer, "buffer")],
        )
    }

    #[test]
    fn checked_calls_require_a_matching_catch_when_throws_are_not_modelled() {
        let (call, parameters) = decoder_decode_call();
        let result = call.ty.clone();
        let uncaught = fixture_declaration(vec![structural_method(
            "decode",
            result.clone(),
            parameters.clone(),
            JavaBlock::new(vec![JavaStmt::Return(Some(call))]),
        )]);
        let diagnostics = verify_fixture(
            portable_codegen::TargetAstBuilder::new(JavaDialect),
            vec![(vec![], uncaught)],
        )
        .unwrap_err();
        assert!(diagnostics.iter().any(|value| {
            value.code == DiagnosticCode::InvalidInvocation
                && value.message.contains("unhandled checked exceptions")
        }));

        let (call, parameters) = decoder_decode_call();
        let caught = fixture_declaration(vec![structural_method(
            "decode",
            result.clone(),
            parameters,
            JavaBlock::new(vec![JavaStmt::TryCatch {
                try_block: JavaBlock::new(vec![JavaStmt::Return(Some(call))]),
                catches: vec![JavaCatch {
                    exception_type: JavaType::known(JavaKnownType::CharacterCodingException),
                    binding: JavaIdentifier::from_portable("failure"),
                    body: JavaBlock::new(vec![JavaStmt::ThrowAssertion(JavaExpr::literal(
                        JavaType::known(JavaKnownType::String),
                        JavaLiteral::String("decode failed".to_owned()),
                    ))]),
                }],
            }]),
        )]);
        let verification = verify_fixture(
            portable_codegen::TargetAstBuilder::new(JavaDialect),
            vec![(vec![], caught)],
        );
        assert!(verification.is_ok(), "{verification:?}");
    }

    #[test]
    fn nested_bindings_reject_outer_collisions_and_never_replace_the_outer_binding() {
        let boolean = JavaType::primitive(JavaPrimitive::Boolean);
        let object = JavaType::known(JavaKnownType::Object);
        let string = JavaType::known(JavaKnownType::String);
        let runtime_exception = JavaType::known(JavaKnownType::RuntimeException);
        let strings = JavaType::generic(JavaKnownType::List, vec![string.clone()]);
        let preserve_outer_string = |name: &str| JavaStmt::Local {
            finality: JavaLocalFinality::Final,
            ty: string.clone(),
            name: JavaIdentifier::from_portable("preserved"),
            value: Some(JavaExpr::local(
                string.clone(),
                JavaIdentifier::from_portable(name),
            )),
        };
        let preserve_outer_object = |name: &str| JavaStmt::Local {
            finality: JavaLocalFinality::Final,
            ty: object.clone(),
            name: JavaIdentifier::from_portable("preserved"),
            value: Some(JavaExpr::local(
                object.clone(),
                JavaIdentifier::from_portable(name),
            )),
        };

        let declaration = fixture_declaration(vec![
            structural_method(
                "foreachCollision",
                JavaType::primitive(JavaPrimitive::Void),
                vec![
                    parameter(strings.clone(), "values"),
                    parameter(string.clone(), "item"),
                ],
                JavaBlock::new(vec![JavaStmt::ForEach {
                    binding_type: string.clone(),
                    binding: JavaIdentifier::from_portable("item"),
                    iterable: JavaExpr::local(strings, JavaIdentifier::from_portable("values")),
                    body: JavaBlock::new(vec![preserve_outer_string("item")]),
                }]),
            ),
            structural_method(
                "switchCollision",
                JavaType::primitive(JavaPrimitive::Void),
                vec![parameter(object.clone(), "selector")],
                JavaBlock::new(vec![JavaStmt::Switch {
                    value: JavaExpr::local(
                        object.clone(),
                        JavaIdentifier::from_portable("selector"),
                    ),
                    arms: vec![
                        JavaSwitchArm {
                            pattern: JavaPattern::Type {
                                ty: string.clone(),
                                binding: JavaIdentifier::from_portable("selector"),
                            },
                            body: JavaBlock::new(vec![preserve_outer_object("selector")]),
                        },
                        JavaSwitchArm {
                            pattern: JavaPattern::Default,
                            body: JavaBlock::new(vec![]),
                        },
                    ],
                }]),
            ),
            structural_method(
                "catchCollision",
                JavaType::primitive(JavaPrimitive::Void),
                vec![parameter(string.clone(), "failure")],
                JavaBlock::new(vec![JavaStmt::TryCatch {
                    try_block: JavaBlock::new(vec![]),
                    catches: vec![JavaCatch {
                        exception_type: runtime_exception,
                        binding: JavaIdentifier::from_portable("failure"),
                        body: JavaBlock::new(vec![preserve_outer_string("failure")]),
                    }],
                }]),
            ),
            structural_method(
                "outerFlowCollision",
                JavaType::primitive(JavaPrimitive::Void),
                vec![
                    parameter(object.clone(), "input"),
                    parameter(string.clone(), "text"),
                ],
                JavaBlock::new(vec![JavaStmt::If {
                    condition: instanceof(
                        JavaExpr::local(object.clone(), JavaIdentifier::from_portable("input")),
                        string.clone(),
                        "text",
                    ),
                    then_block: JavaBlock::new(vec![preserve_outer_string("text")]),
                    else_block: None,
                }]),
            ),
            structural_method(
                "duplicateFlowCollision",
                JavaType::primitive(JavaPrimitive::Void),
                vec![
                    parameter(object.clone(), "left"),
                    parameter(object.clone(), "right"),
                ],
                JavaBlock::new(vec![JavaStmt::If {
                    condition: JavaExpr {
                        ty: boolean,
                        precedence: JavaPrecedence::LogicalAnd,
                        kind: JavaExprKind::Binary {
                            operator: JavaBinaryOperator::LogicalAnd,
                            left: Box::new(instanceof(
                                JavaExpr::local(
                                    object.clone(),
                                    JavaIdentifier::from_portable("left"),
                                ),
                                string.clone(),
                                "text",
                            )),
                            right: Box::new(instanceof(
                                JavaExpr::local(object, JavaIdentifier::from_portable("right")),
                                string,
                                "text",
                            )),
                        },
                    },
                    then_block: JavaBlock::new(vec![]),
                    else_block: None,
                }]),
            ),
        ]);
        let diagnostics = verify_fixture(
            portable_codegen::TargetAstBuilder::new(JavaDialect),
            vec![(vec![], declaration)],
        )
        .unwrap_err();
        let duplicates = diagnostics
            .iter()
            .filter(|value| value.code == DiagnosticCode::DuplicateDeclaration)
            .count();
        assert_eq!(duplicates, 5, "{diagnostics:?}");
        assert!(
            diagnostics
                .iter()
                .all(|value| value.code != DiagnosticCode::TypeMismatch),
            "a rejected nested binding replaced its authoritative outer binding: {diagnostics:?}"
        );
    }

    #[test]
    fn catch_order_uses_the_admitted_throwable_hierarchy() {
        assert!(admitted_throwable_is_supertype_of(
            JavaKnownType::RuntimeException,
            JavaKnownType::IllegalArgumentException,
        ));
        assert!(admitted_throwable_is_supertype_of(
            JavaKnownType::RuntimeException,
            JavaKnownType::IllegalStateException,
        ));
        assert!(!admitted_throwable_is_supertype_of(
            JavaKnownType::IllegalArgumentException,
            JavaKnownType::RuntimeException,
        ));
        assert!(!admitted_throwable_is_supertype_of(
            JavaKnownType::RuntimeException,
            JavaKnownType::CharacterCodingException,
        ));

        let catch = |exception, binding: &str| JavaCatch {
            exception_type: JavaType::known(exception),
            binding: JavaIdentifier::from_portable(binding),
            body: JavaBlock::new(vec![]),
        };
        let method = |catches| {
            structural_method(
                "catchOrder",
                JavaType::primitive(JavaPrimitive::Void),
                vec![],
                JavaBlock::new(vec![JavaStmt::TryCatch {
                    try_block: JavaBlock::new(vec![]),
                    catches,
                }]),
            )
        };
        let invalid = fixture_declaration(vec![method(vec![
            catch(JavaKnownType::RuntimeException, "runtime"),
            catch(JavaKnownType::IllegalArgumentException, "argument"),
        ])]);
        let diagnostics = verify_fixture(
            portable_codegen::TargetAstBuilder::new(JavaDialect),
            vec![(vec![], invalid)],
        )
        .unwrap_err();
        assert!(diagnostics.iter().any(|value| {
            value.code == DiagnosticCode::InvalidControlFlow
                && value.message.contains("dominated by an earlier")
        }));

        let valid = fixture_declaration(vec![method(vec![
            catch(JavaKnownType::IllegalArgumentException, "argument"),
            catch(JavaKnownType::RuntimeException, "runtime"),
        ])]);
        let verification = verify_fixture(
            portable_codegen::TargetAstBuilder::new(JavaDialect),
            vec![(vec![], valid)],
        );
        assert!(verification.is_ok(), "{verification:?}");
    }

    #[test]
    fn blank_final_fields_are_assigned_exactly_once_on_every_normal_constructor_exit() {
        let int = JavaType::primitive(JavaPrimitive::Int);
        let boolean = JavaType::primitive(JavaPrimitive::Boolean);
        let owner = JavaType::known(JavaKnownType::RuntimeError);
        let assignment = || JavaStmt::Assign {
            target: this_field(owner.clone(), int.clone(), "value"),
            value: JavaExpr::literal(int.clone(), JavaLiteral::I32(1)),
        };
        let constructor = |parameters, statements| {
            JavaMember::Constructor(JavaConstructor {
                modifiers: vec![],
                name: JavaIdentifier::from_portable("PolyError"),
                parameters,
                body: JavaBlock::new(statements),
            })
        };
        let declaration = |initializer: Option<JavaExpr>, constructor: Option<JavaMember>| {
            let mut members = vec![JavaMember::Field(JavaField {
                declared: None,
                modifiers: vec![JavaModifier::Private, JavaModifier::Final],
                ty: int.clone(),
                name: JavaIdentifier::from_portable("value"),
                initializer,
            })];
            members.extend(constructor);
            let mut declaration = fixture_declaration(members);
            declaration.name = JavaIdentifier::from_portable("PolyError");
            declaration
        };
        let verify = |declaration| {
            verify_fixture(
                portable_codegen::TargetAstBuilder::new(JavaDialect),
                vec![(vec![], declaration)],
            )
        };

        let straight_line = declaration(None, Some(constructor(vec![], vec![assignment()])));
        let verification = verify(straight_line);
        assert!(verification.is_ok(), "{verification:?}");

        let both_branches = declaration(
            None,
            Some(constructor(
                vec![parameter(boolean.clone(), "condition")],
                vec![JavaStmt::If {
                    condition: JavaExpr::local(
                        boolean.clone(),
                        JavaIdentifier::from_portable("condition"),
                    ),
                    then_block: JavaBlock::new(vec![assignment()]),
                    else_block: Some(JavaBlock::new(vec![assignment()])),
                }],
            )),
        );
        let verification = verify(both_branches);
        assert!(verification.is_ok(), "{verification:?}");

        let conditional_missing = declaration(
            None,
            Some(constructor(
                vec![parameter(boolean.clone(), "condition")],
                vec![JavaStmt::If {
                    condition: JavaExpr::local(
                        boolean.clone(),
                        JavaIdentifier::from_portable("condition"),
                    ),
                    then_block: JavaBlock::new(vec![assignment()]),
                    else_block: None,
                }],
            )),
        );
        let diagnostics = verify(conditional_missing).unwrap_err();
        assert!(diagnostics.iter().any(|value| {
            value.code == DiagnosticCode::InvalidControlFlow
                && value
                    .message
                    .contains("without assigning blank final field")
        }));

        let duplicate = declaration(
            None,
            Some(constructor(vec![], vec![assignment(), assignment()])),
        );
        let diagnostics = verify(duplicate).unwrap_err();
        assert!(diagnostics.iter().any(|value| {
            value.code == DiagnosticCode::InvalidControlFlow
                && value.message.contains("more than once")
        }));

        let early_return = declaration(
            None,
            Some(constructor(
                vec![parameter(boolean.clone(), "condition")],
                vec![
                    JavaStmt::If {
                        condition: JavaExpr::local(
                            boolean.clone(),
                            JavaIdentifier::from_portable("condition"),
                        ),
                        then_block: JavaBlock::new(vec![JavaStmt::Return(None)]),
                        else_block: None,
                    },
                    assignment(),
                ],
            )),
        );
        let diagnostics = verify(early_return).unwrap_err();
        assert!(diagnostics.iter().any(|value| {
            value.code == DiagnosticCode::InvalidControlFlow
                && value
                    .message
                    .contains("without assigning blank final field")
        }));

        let loop_assignment = declaration(
            None,
            Some(constructor(
                vec![parameter(boolean.clone(), "condition")],
                vec![JavaStmt::While {
                    condition: JavaExpr::local(boolean, JavaIdentifier::from_portable("condition")),
                    body: JavaBlock::new(vec![assignment()]),
                }],
            )),
        );
        let diagnostics = verify(loop_assignment).unwrap_err();
        assert!(diagnostics.iter().any(|value| {
            value.code == DiagnosticCode::InvalidControlFlow
                && value.message.contains("constructor loop")
        }));

        let initialized = declaration(
            Some(JavaExpr::literal(int.clone(), JavaLiteral::I32(0))),
            Some(constructor(vec![], vec![assignment()])),
        );
        let diagnostics = verify(initialized).unwrap_err();
        assert!(diagnostics.iter().any(|value| {
            value.code == DiagnosticCode::InvalidControlFlow
                && value.message.contains("initialized final")
        }));

        let implicit = declaration(None, None);
        let diagnostics = verify(implicit).unwrap_err();
        assert!(diagnostics.iter().any(|value| {
            value.code == DiagnosticCode::InvalidControlFlow
                && value.message.contains("implicit Java constructor")
        }));
    }

    #[test]
    fn modifier_context_rejects_static_constructors() {
        let invalid = fixture_declaration(vec![JavaMember::Constructor(JavaConstructor {
            modifiers: vec![JavaModifier::Static],
            name: JavaIdentifier::from_portable("Fixture"),
            parameters: vec![],
            body: JavaBlock::new(vec![]),
        })]);
        let diagnostics = verify_fixture(
            portable_codegen::TargetAstBuilder::new(JavaDialect),
            vec![(vec![], invalid)],
        )
        .unwrap_err();
        assert!(diagnostics.iter().any(|value| {
            value.code == DiagnosticCode::InvalidStructure
                && value.message.contains("declaration context")
        }));

        let valid = fixture_declaration(vec![JavaMember::Constructor(JavaConstructor {
            modifiers: vec![JavaModifier::Private],
            name: JavaIdentifier::from_portable("Fixture"),
            parameters: vec![],
            body: JavaBlock::new(vec![]),
        })]);
        assert!(
            verify_fixture(
                portable_codegen::TargetAstBuilder::new(JavaDialect),
                vec![(vec![], valid)],
            )
            .is_ok()
        );
    }

    #[test]
    fn foreach_binding_matches_list_or_array_element_type() {
        let integer = JavaType::Boxed(JavaPrimitive::Int);
        let list = JavaType::generic(JavaKnownType::List, vec![integer.clone()]);
        let loop_statement = |binding_type| JavaStmt::ForEach {
            binding_type,
            binding: JavaIdentifier::from_portable("item"),
            iterable: JavaExpr::local(list.clone(), JavaIdentifier::from_portable("values")),
            body: JavaBlock::new(vec![]),
        };
        let invalid = fixture_declaration(vec![structural_method(
            "visit",
            JavaType::primitive(JavaPrimitive::Void),
            vec![parameter(list.clone(), "values")],
            JavaBlock::new(vec![loop_statement(JavaType::known(JavaKnownType::String))]),
        )]);
        let diagnostics = verify_fixture(
            portable_codegen::TargetAstBuilder::new(JavaDialect),
            vec![(vec![], invalid)],
        )
        .unwrap_err();
        assert!(diagnostics.iter().any(|value| {
            value.code == DiagnosticCode::TypeMismatch
                && value.message.contains("iterable element type")
        }));

        let valid = fixture_declaration(vec![structural_method(
            "visit",
            JavaType::primitive(JavaPrimitive::Void),
            vec![parameter(list.clone(), "values")],
            JavaBlock::new(vec![loop_statement(integer)]),
        )]);
        assert!(
            verify_fixture(
                portable_codegen::TargetAstBuilder::new(JavaDialect),
                vec![(vec![], valid)],
            )
            .is_ok()
        );
    }

    #[test]
    fn java_erasure_rejects_generic_overloads_with_the_same_raw_signature() {
        let method = |argument: JavaType, parameter_name: &str| {
            structural_method(
                "accept",
                JavaType::primitive(JavaPrimitive::Void),
                vec![parameter(argument, parameter_name)],
                JavaBlock::new(vec![]),
            )
        };
        let invalid = fixture_declaration(vec![
            method(
                JavaType::generic(
                    JavaKnownType::List,
                    vec![JavaType::known(JavaKnownType::String)],
                ),
                "strings",
            ),
            method(
                JavaType::generic(
                    JavaKnownType::List,
                    vec![JavaType::Boxed(JavaPrimitive::Int)],
                ),
                "integers",
            ),
        ]);
        let diagnostics = verify_fixture(
            portable_codegen::TargetAstBuilder::new(JavaDialect),
            vec![(vec![], invalid)],
        )
        .unwrap_err();
        assert!(diagnostics.iter().any(|value| {
            value.code == DiagnosticCode::DuplicateDeclaration
                && value.message.contains("erased declaration signature")
        }));

        let valid = fixture_declaration(vec![
            method(
                JavaType::generic(
                    JavaKnownType::List,
                    vec![JavaType::known(JavaKnownType::String)],
                ),
                "strings",
            ),
            method(JavaType::known(JavaKnownType::String), "string"),
        ]);
        assert!(
            verify_fixture(
                portable_codegen::TargetAstBuilder::new(JavaDialect),
                vec![(vec![], valid)],
            )
            .is_ok()
        );
    }

    #[test]
    fn generated_interface_conformance_requires_every_declared_method() {
        let mut builder = portable_codegen::TargetAstBuilder::new(JavaDialect);
        let interface = builder.generated_type(portable_codegen::GeneratedType {
            name: "Service".to_owned(),
            kind: JavaDeclarationKind::Interface,
            visibility: JavaVisibility::Package,
            origin: portable_codegen::GeneratedOrigin::Synthesized(
                portable_codegen::SynthesisReason::InterfaceAdapter,
            ),
            source: verifier_source("interface"),
        });
        let record = builder.generated_type(portable_codegen::GeneratedType {
            name: "Implementation".to_owned(),
            kind: JavaDeclarationKind::Record,
            visibility: JavaVisibility::Package,
            origin: portable_codegen::GeneratedOrigin::Synthesized(
                portable_codegen::SynthesisReason::InterfaceAdapter,
            ),
            source: verifier_source("record"),
        });
        let method = builder.interface_method(portable_codegen::GeneratedInterfaceMethod {
            owner: interface,
            name: "value".to_owned(),
            signature: portable_codegen::TargetCallableSignature {
                invocation: JavaInvocationKind::Instance,
                receiver: Some(TargetTypeRef::Generated(interface)),
                parameters: vec![],
                return_type: TargetTypeRef::Primitive(JavaPrimitive::Int),
            },
            origin: portable_codegen::GeneratedOrigin::Synthesized(
                portable_codegen::SynthesisReason::InterfaceAdapter,
            ),
            source: verifier_source("method"),
        });
        let interface_declaration = JavaTypeDeclaration {
            declared: Some(interface),
            kind: JavaDeclarationKind::Interface,
            visibility: JavaVisibility::Package,
            modifiers: vec![],
            name: JavaIdentifier::from_portable("Service"),
            type_parameters: vec![],
            record_components: vec![],
            heritage: JavaHeritage::None,
            permits: vec![],
            members: vec![JavaMember::Method(JavaMethod {
                declared: JavaMethodDeclaration::Interface(method),
                annotations: vec![],
                modifiers: vec![JavaModifier::Public, JavaModifier::Abstract],
                type_parameters: vec![],
                return_type: JavaType::primitive(JavaPrimitive::Int),
                name: JavaIdentifier::from_portable("value"),
                parameters: vec![],
                body: None,
            })],
        };
        let record_declaration = JavaTypeDeclaration {
            declared: Some(record),
            kind: JavaDeclarationKind::Record,
            visibility: JavaVisibility::Package,
            modifiers: vec![],
            name: JavaIdentifier::from_portable("Implementation"),
            type_parameters: vec![],
            record_components: vec![],
            heritage: JavaHeritage::Interfaces(vec![JavaType::Reference(JavaTypeName::Generated(
                interface,
            ))]),
            permits: vec![],
            members: vec![],
        };
        let diagnostics = verify_fixture(
            builder,
            vec![
                (
                    vec![
                        GeneratedSymbolId::Type(interface),
                        GeneratedSymbolId::InterfaceMethod(method),
                    ],
                    interface_declaration,
                ),
                (vec![GeneratedSymbolId::Type(record)], record_declaration),
            ],
        )
        .unwrap_err();
        assert!(diagnostics.iter().any(|value| {
            value.code == DiagnosticCode::InterfaceNonconformance
                && value.message.contains("exactly once")
        }));
    }

    #[test]
    fn contextual_types_require_exact_known_arity_and_declared_variables() {
        let invalid_arity = fixture_declaration(vec![JavaMember::Field(JavaField {
            declared: None,
            modifiers: vec![JavaModifier::Static],
            ty: JavaType::generic(
                JavaKnownType::List,
                vec![
                    JavaType::known(JavaKnownType::String),
                    JavaType::known(JavaKnownType::Object),
                ],
            ),
            name: JavaIdentifier::from_portable("values"),
            initializer: None,
        })]);
        let diagnostics = verify_fixture(
            portable_codegen::TargetAstBuilder::new(JavaDialect),
            vec![(vec![], invalid_arity)],
        )
        .unwrap_err();
        assert!(diagnostics.iter().any(|value| {
            value.code == DiagnosticCode::TypeMismatch
                && value.message.contains("exactly 1 type arguments")
        }));

        let variable = JavaIdentifier::from_portable("T");
        let undeclared = fixture_declaration(vec![structural_method(
            "identity",
            JavaType::TypeVariable(variable.clone()),
            vec![parameter(JavaType::TypeVariable(variable.clone()), "value")],
            JavaBlock::new(vec![JavaStmt::Return(Some(JavaExpr::local(
                JavaType::TypeVariable(variable.clone()),
                JavaIdentifier::from_portable("value"),
            )))]),
        )]);
        let diagnostics = verify_fixture(
            portable_codegen::TargetAstBuilder::new(JavaDialect),
            vec![(vec![], undeclared)],
        )
        .unwrap_err();
        assert!(diagnostics.iter().any(|value| {
            value.code == DiagnosticCode::UnresolvedReference
                && value.message.contains("type variable")
        }));

        let declared = fixture_declaration(vec![JavaMember::Method(JavaMethod {
            declared: JavaMethodDeclaration::Structural,
            annotations: vec![],
            modifiers: vec![JavaModifier::Public, JavaModifier::Static],
            type_parameters: vec![variable.clone()],
            return_type: JavaType::TypeVariable(variable.clone()),
            name: JavaIdentifier::from_portable("identity"),
            parameters: vec![parameter(JavaType::TypeVariable(variable.clone()), "value")],
            body: Some(JavaBlock::new(vec![JavaStmt::Return(Some(
                JavaExpr::local(
                    JavaType::TypeVariable(variable),
                    JavaIdentifier::from_portable("value"),
                ),
            ))])),
        })]);
        assert!(
            verify_fixture(
                portable_codegen::TargetAstBuilder::new(JavaDialect),
                vec![(vec![], declared)],
            )
            .is_ok()
        );
    }

    #[test]
    fn casts_and_instanceof_require_java_legality_and_reifiable_targets() {
        let string = JavaType::known(JavaKnownType::String);
        let object = JavaType::known(JavaKnownType::Object);
        let int = JavaType::primitive(JavaPrimitive::Int);
        let boolean = JavaType::primitive(JavaPrimitive::Boolean);
        let invalid_cast = JavaExpr {
            ty: int.clone(),
            precedence: JavaPrecedence::Unary,
            kind: JavaExprKind::Cast {
                target: int.clone(),
                value: Box::new(JavaExpr::local(
                    string.clone(),
                    JavaIdentifier::from_portable("value"),
                )),
            },
        };
        let invalid_primitive_test = JavaExpr {
            ty: boolean.clone(),
            precedence: JavaPrecedence::Relational,
            kind: JavaExprKind::InstanceOf {
                value: Box::new(JavaExpr::local(
                    int.clone(),
                    JavaIdentifier::from_portable("number"),
                )),
                target: string.clone(),
                binding: None,
            },
        };
        let non_reifiable = JavaType::generic(JavaKnownType::List, vec![string.clone()]);
        let invalid_generic_test = JavaExpr {
            ty: boolean.clone(),
            precedence: JavaPrecedence::Relational,
            kind: JavaExprKind::InstanceOf {
                value: Box::new(JavaExpr::local(
                    object.clone(),
                    JavaIdentifier::from_portable("value"),
                )),
                target: non_reifiable,
                binding: None,
            },
        };
        let invalid = fixture_declaration(vec![
            structural_method(
                "cast",
                int.clone(),
                vec![parameter(string, "value")],
                JavaBlock::new(vec![JavaStmt::Return(Some(invalid_cast))]),
            ),
            structural_method(
                "primitiveTest",
                boolean.clone(),
                vec![parameter(int, "number")],
                JavaBlock::new(vec![JavaStmt::Return(Some(invalid_primitive_test))]),
            ),
            structural_method(
                "genericTest",
                boolean.clone(),
                vec![parameter(object.clone(), "value")],
                JavaBlock::new(vec![JavaStmt::Return(Some(invalid_generic_test))]),
            ),
        ]);
        let diagnostics = verify_fixture(
            portable_codegen::TargetAstBuilder::new(JavaDialect),
            vec![(vec![], invalid)],
        )
        .unwrap_err();
        assert!(diagnostics.iter().any(|value| {
            value.code == DiagnosticCode::TypeMismatch
                && value.message.contains("cast is not legal")
        }));
        assert!(diagnostics.iter().any(|value| {
            value.code == DiagnosticCode::TypeMismatch
                && value.message.contains("instanceof is not legal")
        }));
        assert!(diagnostics.iter().any(|value| {
            value.code == DiagnosticCode::TypeMismatch
                && value
                    .message
                    .contains("instanceof target must be reifiable")
        }));

        let list_any = JavaType::generic(
            JavaKnownType::List,
            vec![JavaType::Wildcard { bound: None }],
        );
        let valid = fixture_declaration(vec![structural_method(
            "listTest",
            boolean,
            vec![parameter(object.clone(), "value")],
            JavaBlock::new(vec![JavaStmt::Return(Some(JavaExpr {
                ty: JavaType::primitive(JavaPrimitive::Boolean),
                precedence: JavaPrecedence::Relational,
                kind: JavaExprKind::InstanceOf {
                    value: Box::new(JavaExpr::local(
                        object,
                        JavaIdentifier::from_portable("value"),
                    )),
                    target: list_any,
                    binding: Some(JavaIdentifier::from_portable("values")),
                },
            }))]),
        )]);
        assert!(
            verify_fixture(
                portable_codegen::TargetAstBuilder::new(JavaDialect),
                vec![(vec![], valid)],
            )
            .is_ok()
        );
    }

    #[test]
    fn this_references_must_match_the_lexical_owner() {
        let mut builder = portable_codegen::TargetAstBuilder::new(JavaDialect);
        let owner = builder.generated_type(portable_codegen::GeneratedType {
            name: "Owner".to_owned(),
            kind: JavaDeclarationKind::FinalClass,
            visibility: JavaVisibility::Package,
            origin: portable_codegen::GeneratedOrigin::Synthesized(
                portable_codegen::SynthesisReason::TestHarness,
            ),
            source: verifier_source("owner"),
        });
        let mut declaration = fixture_declaration(vec![JavaMember::Method(JavaMethod {
            declared: JavaMethodDeclaration::Structural,
            annotations: vec![],
            modifiers: vec![JavaModifier::Public],
            type_parameters: vec![],
            return_type: JavaType::known(JavaKnownType::String),
            name: JavaIdentifier::from_portable("selfValue"),
            parameters: vec![],
            body: Some(JavaBlock::new(vec![JavaStmt::Return(Some(JavaExpr {
                ty: JavaType::known(JavaKnownType::String),
                precedence: JavaPrecedence::Primary,
                kind: JavaExprKind::Value(JavaValueRef::This),
            }))])),
        })]);
        declaration.declared = Some(owner);
        declaration.name = JavaIdentifier::from_portable("Owner");
        let diagnostics = verify_fixture(
            builder,
            vec![(vec![GeneratedSymbolId::Type(owner)], declaration)],
        )
        .unwrap_err();
        assert!(diagnostics.iter().any(|value| {
            value.code == DiagnosticCode::UnresolvedReference
                && value.message.contains("declaring Java owner")
        }));

        let mut builder = portable_codegen::TargetAstBuilder::new(JavaDialect);
        let owner = builder.generated_type(portable_codegen::GeneratedType {
            name: "Owner".to_owned(),
            kind: JavaDeclarationKind::FinalClass,
            visibility: JavaVisibility::Package,
            origin: portable_codegen::GeneratedOrigin::Synthesized(
                portable_codegen::SynthesisReason::TestHarness,
            ),
            source: verifier_source("owner-positive"),
        });
        let owner_type = JavaType::Reference(JavaTypeName::Generated(owner));
        let mut declaration = fixture_declaration(vec![JavaMember::Method(JavaMethod {
            declared: JavaMethodDeclaration::Structural,
            annotations: vec![],
            modifiers: vec![JavaModifier::Public],
            type_parameters: vec![],
            return_type: owner_type.clone(),
            name: JavaIdentifier::from_portable("selfValue"),
            parameters: vec![],
            body: Some(JavaBlock::new(vec![JavaStmt::Return(Some(JavaExpr {
                ty: owner_type,
                precedence: JavaPrecedence::Primary,
                kind: JavaExprKind::Value(JavaValueRef::This),
            }))])),
        })]);
        declaration.declared = Some(owner);
        declaration.name = JavaIdentifier::from_portable("Owner");
        assert!(
            verify_fixture(
                builder,
                vec![(vec![GeneratedSymbolId::Type(owner)], declaration,)],
            )
            .is_ok()
        );
    }

    #[test]
    fn structural_fields_require_declared_type_and_final_assignment_context() {
        let int = JavaType::primitive(JavaPrimitive::Int);
        let owner = JavaType::known(JavaKnownType::RuntimeError);
        let string = JavaType::known(JavaKnownType::String);
        let this = || JavaExpr {
            ty: owner.clone(),
            precedence: JavaPrecedence::Primary,
            kind: JavaExprKind::Value(JavaValueRef::This),
        };
        let field = |name: &str, ty: JavaType| JavaExpr {
            ty: ty.clone(),
            precedence: JavaPrecedence::Primary,
            kind: JavaExprKind::Field {
                receiver: Box::new(this()),
                field: JavaFieldRef::Structural {
                    name: JavaIdentifier::from_portable(name),
                    ty,
                },
            },
        };
        let valid = JavaTypeDeclaration {
            declared: None,
            kind: JavaDeclarationKind::Record,
            visibility: JavaVisibility::Package,
            modifiers: vec![],
            name: JavaIdentifier::from_portable("PolyError"),
            type_parameters: vec![],
            record_components: vec![JavaRecordComponent {
                origin: JavaRecordComponentOrigin::Runtime(JavaRuntimeMember::ErrorCode),
                ty: string.clone(),
                name: JavaIdentifier::from_portable("code"),
            }],
            heritage: JavaHeritage::None,
            permits: vec![],
            members: vec![JavaMember::Constructor(JavaConstructor {
                modifiers: vec![],
                name: JavaIdentifier::from_portable("PolyError"),
                parameters: vec![parameter(string.clone(), "code")],
                body: JavaBlock::new(vec![JavaStmt::Assign {
                    target: field("code", string.clone()),
                    value: JavaExpr::local(string.clone(), JavaIdentifier::from_portable("code")),
                }]),
            })],
        };
        assert!(
            verify_fixture(
                portable_codegen::TargetAstBuilder::new(JavaDialect),
                vec![(vec![], valid)],
            )
            .is_ok()
        );

        let mut invalid = fixture_declaration(vec![
            JavaMember::Field(JavaField {
                declared: None,
                modifiers: vec![JavaModifier::Private, JavaModifier::Final],
                ty: int.clone(),
                name: JavaIdentifier::from_portable("count"),
                initializer: None,
            }),
            JavaMember::Method(JavaMethod {
                declared: JavaMethodDeclaration::Structural,
                annotations: vec![],
                modifiers: vec![JavaModifier::Public],
                type_parameters: vec![],
                return_type: JavaType::primitive(JavaPrimitive::Void),
                name: JavaIdentifier::from_portable("mutate"),
                parameters: vec![],
                body: Some(JavaBlock::new(vec![JavaStmt::Assign {
                    target: JavaExpr {
                        ty: int.clone(),
                        precedence: JavaPrecedence::Primary,
                        kind: JavaExprKind::Field {
                            receiver: Box::new(JavaExpr {
                                ty: JavaType::known(JavaKnownType::String),
                                precedence: JavaPrecedence::Primary,
                                kind: JavaExprKind::Value(JavaValueRef::This),
                            }),
                            field: JavaFieldRef::Structural {
                                name: JavaIdentifier::from_portable("missing"),
                                ty: int.clone(),
                            },
                        },
                    },
                    value: JavaExpr::literal(int, JavaLiteral::I32(1)),
                }])),
            }),
        ]);
        invalid.name = JavaIdentifier::from_portable("Fixture");
        let diagnostics = verify_fixture(
            portable_codegen::TargetAstBuilder::new(JavaDialect),
            vec![(vec![], invalid)],
        )
        .unwrap_err();
        assert!(diagnostics.iter().any(|value| {
            value.code == DiagnosticCode::UnresolvedReference
                && value.message.contains("structural Java field")
        }));
    }

    #[test]
    fn generated_record_fields_are_final_outside_their_canonical_constructor() {
        let mut builder = portable_codegen::TargetAstBuilder::new(JavaDialect);
        let record = builder.generated_type(portable_codegen::GeneratedType {
            name: "Value".to_owned(),
            kind: JavaDeclarationKind::Record,
            visibility: JavaVisibility::Package,
            origin: portable_codegen::GeneratedOrigin::Synthesized(
                portable_codegen::SynthesisReason::TestHarness,
            ),
            source: verifier_source("field-record"),
        });
        let field_id = fixture_core_field();
        let int = JavaType::primitive(JavaPrimitive::Int);
        let owner = JavaType::Reference(JavaTypeName::Generated(record));
        let generated_field = || JavaExpr {
            ty: int.clone(),
            precedence: JavaPrecedence::Primary,
            kind: JavaExprKind::Field {
                receiver: Box::new(JavaExpr {
                    ty: owner.clone(),
                    precedence: JavaPrecedence::Primary,
                    kind: JavaExprKind::Value(JavaValueRef::This),
                }),
                field: JavaFieldRef::Generated {
                    owner: record,
                    field: field_id,
                    name: JavaIdentifier::from_portable("value"),
                    ty: int.clone(),
                },
            },
        };
        let declaration = JavaTypeDeclaration {
            declared: Some(record),
            kind: JavaDeclarationKind::Record,
            visibility: JavaVisibility::Package,
            modifiers: vec![],
            name: JavaIdentifier::from_portable("Value"),
            type_parameters: vec![],
            record_components: vec![JavaRecordComponent {
                origin: JavaRecordComponentOrigin::Core(field_id),
                ty: int.clone(),
                name: JavaIdentifier::from_portable("value"),
            }],
            heritage: JavaHeritage::None,
            permits: vec![],
            members: vec![
                JavaMember::Constructor(JavaConstructor {
                    modifiers: vec![],
                    name: JavaIdentifier::from_portable("Value"),
                    parameters: vec![parameter(int.clone(), "value")],
                    body: JavaBlock::new(vec![JavaStmt::Assign {
                        target: generated_field(),
                        value: JavaExpr::local(int.clone(), JavaIdentifier::from_portable("value")),
                    }]),
                }),
                JavaMember::Method(JavaMethod {
                    declared: JavaMethodDeclaration::Structural,
                    annotations: vec![],
                    modifiers: vec![JavaModifier::Public],
                    type_parameters: vec![],
                    return_type: JavaType::primitive(JavaPrimitive::Void),
                    name: JavaIdentifier::from_portable("mutate"),
                    parameters: vec![],
                    body: Some(JavaBlock::new(vec![JavaStmt::Assign {
                        target: generated_field(),
                        value: JavaExpr::literal(int, JavaLiteral::I32(2)),
                    }])),
                }),
                JavaMember::Method(JavaMethod {
                    declared: JavaMethodDeclaration::Structural,
                    annotations: vec![],
                    modifiers: vec![JavaModifier::Public],
                    type_parameters: vec![],
                    return_type: JavaType::primitive(JavaPrimitive::Int),
                    name: JavaIdentifier::from_portable("missing"),
                    parameters: vec![],
                    body: Some(JavaBlock::new(vec![JavaStmt::Return(Some(JavaExpr {
                        ty: JavaType::primitive(JavaPrimitive::Int),
                        precedence: JavaPrecedence::Primary,
                        kind: JavaExprKind::Field {
                            receiver: Box::new(JavaExpr {
                                ty: owner,
                                precedence: JavaPrecedence::Primary,
                                kind: JavaExprKind::Value(JavaValueRef::This),
                            }),
                            field: JavaFieldRef::Generated {
                                owner: record,
                                field: field_id,
                                name: JavaIdentifier::from_portable("missing"),
                                ty: JavaType::primitive(JavaPrimitive::Int),
                            },
                        },
                    }))])),
                }),
            ],
        };
        let diagnostics = verify_fixture(
            builder,
            vec![(vec![GeneratedSymbolId::Type(record)], declaration)],
        )
        .unwrap_err();
        assert!(diagnostics.iter().any(|value| {
            value.code == DiagnosticCode::InvalidControlFlow
                && value.message.contains("declaring constructor")
        }));
        assert!(diagnostics.iter().any(|value| {
            value.code == DiagnosticCode::UnresolvedReference
                && value.message.contains("generated Java field reference")
        }));
    }

    #[test]
    fn declaration_kinds_enforce_interface_members_and_canonical_records() {
        let int = JavaType::primitive(JavaPrimitive::Int);
        let interface = JavaTypeDeclaration {
            declared: None,
            kind: JavaDeclarationKind::Interface,
            visibility: JavaVisibility::Package,
            modifiers: vec![],
            name: JavaIdentifier::from_portable("InvalidInterface"),
            type_parameters: vec![],
            record_components: vec![],
            heritage: JavaHeritage::None,
            permits: vec![],
            members: vec![JavaMember::Field(JavaField {
                declared: None,
                modifiers: vec![JavaModifier::Static, JavaModifier::Final],
                ty: int.clone(),
                name: JavaIdentifier::from_portable("value"),
                initializer: Some(JavaExpr::literal(int.clone(), JavaLiteral::I32(1))),
            })],
        };
        let record = JavaTypeDeclaration {
            declared: None,
            kind: JavaDeclarationKind::Record,
            visibility: JavaVisibility::Package,
            modifiers: vec![],
            name: JavaIdentifier::from_portable("InvalidRecord"),
            type_parameters: vec![],
            record_components: vec![JavaRecordComponent {
                origin: JavaRecordComponentOrigin::Core(fixture_core_field()),
                ty: int,
                name: JavaIdentifier::from_portable("value"),
            }],
            heritage: JavaHeritage::None,
            permits: vec![],
            members: vec![JavaMember::Constructor(JavaConstructor {
                modifiers: vec![],
                name: JavaIdentifier::from_portable("InvalidRecord"),
                parameters: vec![parameter(JavaType::known(JavaKnownType::String), "wrong")],
                body: JavaBlock::new(vec![]),
            })],
        };
        let diagnostics = verify_fixture(
            portable_codegen::TargetAstBuilder::new(JavaDialect),
            vec![(vec![], interface), (vec![], record)],
        )
        .unwrap_err();
        assert!(diagnostics.iter().any(|value| {
            value.code == DiagnosticCode::InvalidStructure
                && value.message.contains("interfaces cannot declare fields")
        }));
        assert!(diagnostics.iter().any(|value| {
            value.code == DiagnosticCode::InvalidStructure
                && value.message.contains("canonical component signature")
        }));
    }

    #[test]
    fn sealed_permits_are_unique_and_exactly_match_implementors() {
        let setup = |valid: bool| {
            let mut builder = portable_codegen::TargetAstBuilder::new(JavaDialect);
            let interface = builder.generated_type(portable_codegen::GeneratedType {
                name: "Shape".to_owned(),
                kind: JavaDeclarationKind::SealedInterface,
                visibility: JavaVisibility::Package,
                origin: portable_codegen::GeneratedOrigin::Synthesized(
                    portable_codegen::SynthesisReason::TestHarness,
                ),
                source: verifier_source("sealed-interface"),
            });
            let implementation = builder.generated_type(portable_codegen::GeneratedType {
                name: "Circle".to_owned(),
                kind: JavaDeclarationKind::Record,
                visibility: JavaVisibility::Package,
                origin: portable_codegen::GeneratedOrigin::Synthesized(
                    portable_codegen::SynthesisReason::TestHarness,
                ),
                source: verifier_source("sealed-implementation"),
            });
            let unrelated = builder.generated_type(portable_codegen::GeneratedType {
                name: "Square".to_owned(),
                kind: JavaDeclarationKind::Record,
                visibility: JavaVisibility::Package,
                origin: portable_codegen::GeneratedOrigin::Synthesized(
                    portable_codegen::SynthesisReason::TestHarness,
                ),
                source: verifier_source("sealed-unrelated"),
            });
            let interface_type = JavaType::Reference(JavaTypeName::Generated(interface));
            let implementation_type = JavaType::Reference(JavaTypeName::Generated(implementation));
            let unrelated_type = JavaType::Reference(JavaTypeName::Generated(unrelated));
            let interface_declaration = JavaTypeDeclaration {
                declared: Some(interface),
                kind: JavaDeclarationKind::SealedInterface,
                visibility: JavaVisibility::Package,
                modifiers: vec![],
                name: JavaIdentifier::from_portable("Shape"),
                type_parameters: vec![],
                record_components: vec![],
                heritage: JavaHeritage::None,
                permits: if valid {
                    vec![implementation_type]
                } else {
                    vec![unrelated_type.clone(), unrelated_type]
                },
                members: vec![],
            };
            let implementation_declaration = JavaTypeDeclaration {
                declared: Some(implementation),
                kind: JavaDeclarationKind::Record,
                visibility: JavaVisibility::Package,
                modifiers: vec![],
                name: JavaIdentifier::from_portable("Circle"),
                type_parameters: vec![],
                record_components: vec![],
                heritage: JavaHeritage::Interfaces(vec![interface_type]),
                permits: vec![],
                members: vec![],
            };
            let unrelated_declaration = JavaTypeDeclaration {
                declared: Some(unrelated),
                kind: JavaDeclarationKind::Record,
                visibility: JavaVisibility::Package,
                modifiers: vec![],
                name: JavaIdentifier::from_portable("Square"),
                type_parameters: vec![],
                record_components: vec![],
                heritage: JavaHeritage::None,
                permits: vec![],
                members: vec![],
            };
            (
                builder,
                vec![
                    (
                        vec![GeneratedSymbolId::Type(interface)],
                        interface_declaration,
                    ),
                    (
                        vec![GeneratedSymbolId::Type(implementation)],
                        implementation_declaration,
                    ),
                    (
                        vec![GeneratedSymbolId::Type(unrelated)],
                        unrelated_declaration,
                    ),
                ],
            )
        };
        let (builder, declarations) = setup(false);
        let diagnostics = verify_fixture(builder, declarations).unwrap_err();
        assert!(diagnostics.iter().any(|value| {
            value.code == DiagnosticCode::DuplicateDeclaration
                && value.message.contains("permits entry is repeated")
        }));
        assert!(diagnostics.iter().any(|value| {
            value.code == DiagnosticCode::InterfaceNonconformance
                && value.message.contains("exactly name every implementing")
        }));

        let (builder, declarations) = setup(true);
        assert!(verify_fixture(builder, declarations).is_ok());
    }
}
