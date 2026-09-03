use std::collections::{BTreeMap, BTreeSet};

use portable_codegen::{
    AstViolation, GeneratedCallableId, GeneratedInterfaceMethodId, GeneratedSymbolId,
    GeneratedTypeId, GeneratedValueId, TargetAstContext, TargetCallableRef, TargetExpressionNode,
    TargetFileItemNode, TargetStatementNode, TargetSymbolRef, TargetTypeRef,
};
use portable_core_ir::CoreFieldId;
use portable_diagnostics::DiagnosticCode;

use crate::dialect::{JavaDialect, JavaInvocationKind};

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
    pub const ALL: [Self; 13] = [
        Self::SemanticEquals,
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
            Self::SemanticEquals => {
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
                        == JavaType::generic(
                            JavaKnownType::List,
                            vec![JavaType::Boxed(JavaPrimitive::Int)],
                        )
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
        let mut violations = self.ty.verify(JavaTypeUse::Value);
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
            JavaExprKind::Value(_) => {}
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
                if let JavaFieldRef::Generated {
                    owner,
                    field,
                    name,
                    ty,
                } = field
                    && !generated_field_matches(*owner, *field, name, ty, context)
                {
                    violations.push(AstViolation::new(
                        DiagnosticCode::UnresolvedReference,
                        "generated Java field reference does not match its declared owner/name/type",
                    ));
                }
            }
            JavaExprKind::Cast { target, value } => {
                violations.extend(target.verify(JavaTypeUse::Value));
                violations.extend(value.verify(context));
                if target != &self.ty {
                    violations.push(type_error("cast target type mismatch"));
                }
            }
            JavaExprKind::InstanceOf { value, target, .. } => {
                violations.extend(value.verify(context));
                violations.extend(target.verify(JavaTypeUse::Value));
                if self.ty != JavaType::Primitive(JavaPrimitive::Boolean) {
                    violations.push(type_error("instanceof result must be boolean"));
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
                (registered.as_ref() == Some(&actual)).then(|| signature.clone())
            }
            Self::Interface { symbol, signature } => context
                .callable_signature(&TargetCallableRef::Interface(*symbol))
                .is_some_and(|registered| registered == JavaDialect.coarse_signature(signature))
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
                        generated_accessor_matches(owner, *field, name, signature, context)
                    }
                    JavaMemberOrigin::Runtime(member) => {
                        name.as_str() == member.name() && member.accepts(signature)
                    }
                    JavaMemberOrigin::GeneratedImplementation(method) => {
                        generated_member_matches(owner, name, signature, Some(*method), context)
                    }
                    JavaMemberOrigin::GeneratedVariant => false,
                };
                (owner_matches && catalogue_matches).then(|| signature.clone())
            }
        }
    }
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
}

impl JavaLexicalScope {
    fn for_method(method: &JavaMethod) -> (Self, Vec<AstViolation>) {
        let mut scope = Self {
            bindings: BTreeMap::new(),
            allows_this: !method.modifiers.contains(&JavaModifier::Static),
        };
        let mut violations = Vec::new();
        for parameter in &method.parameters {
            if scope
                .bindings
                .insert(
                    parameter.name.clone(),
                    JavaLexicalBinding {
                        ty: parameter.ty.clone(),
                        mutable: !parameter.final_parameter,
                    },
                )
                .is_some()
            {
                violations.push(AstViolation::new(
                    DiagnosticCode::DuplicateDeclaration,
                    "Java method parameter is declared more than once",
                ));
            }
        }
        (scope, violations)
    }

    fn for_constructor(constructor: &JavaConstructor) -> (Self, Vec<AstViolation>) {
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
        let (mut scope, violations) = Self::for_method(&method);
        scope.allows_this = true;
        (scope, violations)
    }
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
        JavaExprKind::Value(JavaValueRef::This) if !scope.allows_this => {
            violations.push(AstViolation::new(
                DiagnosticCode::UnresolvedReference,
                "this is unavailable in a static Java scope",
            ));
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
        JavaExprKind::Field { receiver, .. } => {
            violations.extend(verify_expr_scope(receiver, scope));
        }
        JavaExprKind::Cast { value, .. } | JavaExprKind::InstanceOf { value, .. } => {
            violations.extend(verify_expr_scope(value, scope));
        }
        JavaExprKind::Lambda { .. } => {
            // The ordinary AST verifier rejects lambdas before rendering.
        }
    }
    violations
}

fn verify_assignment_target(target: &JavaExpr, scope: &JavaLexicalScope) -> Vec<AstViolation> {
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
        JavaExprKind::Field { .. } => vec![],
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

fn verify_block_scope(
    block: &JavaBlock,
    scope: &mut JavaLexicalScope,
    expected_return: &JavaType,
    in_loop: bool,
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
                if scope
                    .bindings
                    .insert(
                        name.clone(),
                        JavaLexicalBinding {
                            ty: ty.clone(),
                            mutable: *finality == JavaLocalFinality::Mutable,
                        },
                    )
                    .is_some()
                {
                    violations.push(AstViolation::new(
                        DiagnosticCode::DuplicateDeclaration,
                        "Java local shadows a name already declared in this portable scope",
                    ));
                }
            }
            JavaStmt::Assign { target, value } => {
                violations.extend(verify_expr_scope(target, scope));
                violations.extend(verify_expr_scope(value, scope));
                violations.extend(verify_assignment_target(target, scope));
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
                collect_positive_pattern_bindings(condition, &mut then_scope);
                violations.extend(verify_block_scope(
                    then_block,
                    &mut then_scope,
                    expected_return,
                    in_loop,
                ));
                if let Some(else_block) = else_block {
                    let mut else_scope = scope.clone();
                    violations.extend(verify_block_scope(
                        else_block,
                        &mut else_scope,
                        expected_return,
                        in_loop,
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
                    scope.bindings.insert(
                        binding.clone(),
                        JavaLexicalBinding {
                            ty: target.clone(),
                            mutable: false,
                        },
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
                body_scope.bindings.insert(
                    binding.clone(),
                    JavaLexicalBinding {
                        ty: binding_type.clone(),
                        mutable: false,
                    },
                );
                violations.extend(verify_block_scope(
                    body,
                    &mut body_scope,
                    expected_return,
                    true,
                ));
            }
            JavaStmt::While { condition, body } => {
                violations.extend(verify_expr_scope(condition, scope));
                let mut body_scope = scope.clone();
                violations.extend(verify_block_scope(
                    body,
                    &mut body_scope,
                    expected_return,
                    true,
                ));
            }
            JavaStmt::Switch { value, arms } => {
                violations.extend(verify_expr_scope(value, scope));
                for arm in arms {
                    let mut arm_scope = scope.clone();
                    if let JavaPattern::Type { ty, binding } = &arm.pattern {
                        arm_scope.bindings.insert(
                            binding.clone(),
                            JavaLexicalBinding {
                                ty: ty.clone(),
                                mutable: false,
                            },
                        );
                    }
                    violations.extend(verify_block_scope(
                        &arm.body,
                        &mut arm_scope,
                        expected_return,
                        in_loop,
                    ));
                }
            }
            JavaStmt::TryCatch { try_block, catches } => {
                let mut try_scope = scope.clone();
                violations.extend(verify_block_scope(
                    try_block,
                    &mut try_scope,
                    expected_return,
                    in_loop,
                ));
                for catch in catches {
                    let mut catch_scope = scope.clone();
                    catch_scope.bindings.insert(
                        catch.binding.clone(),
                        JavaLexicalBinding {
                            ty: catch.exception_type.clone(),
                            mutable: false,
                        },
                    );
                    violations.extend(verify_block_scope(
                        &catch.body,
                        &mut catch_scope,
                        expected_return,
                        in_loop,
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

fn collect_positive_pattern_bindings(value: &JavaExpr, scope: &mut JavaLexicalScope) {
    match &value.kind {
        JavaExprKind::InstanceOf {
            target,
            binding: Some(binding),
            ..
        } => {
            scope.bindings.insert(
                binding.clone(),
                JavaLexicalBinding {
                    ty: target.clone(),
                    mutable: false,
                },
            );
        }
        JavaExprKind::Binary {
            operator: JavaBinaryOperator::LogicalAnd,
            left,
            right,
        } => {
            collect_positive_pattern_bindings(left, scope);
            collect_positive_pattern_bindings(right, scope);
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
                for catch in catches {
                    violations.extend(catch.exception_type.verify(JavaTypeUse::Parameter));
                    violations.extend(catch.body.verify(context));
                }
            }
            Self::Break | Self::Continue => {}
        }
        violations
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

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum JavaExternalBase {
    ApprovedFrameworkAdapter,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum JavaHeritage {
    None,
    Interfaces(Vec<JavaType>),
    ExternalAdapter {
        base: JavaExternalBase,
        delegated_field: JavaIdentifier,
    },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct JavaRecordComponent {
    pub origin: JavaRecordComponentOrigin,
    pub ty: JavaType,
    pub name: JavaIdentifier,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
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
        match self {
            Self::Field(field) => {
                let mut violations = verify_modifiers(&field.modifiers);
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
                let mut violations = verify_modifiers(&field.modifiers);
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
                let mut violations = verify_modifiers(&method.modifiers);
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
                    let (mut scope, scope_violations) = JavaLexicalScope::for_method(method);
                    violations.extend(scope_violations);
                    violations.extend(verify_block_scope(
                        body,
                        &mut scope,
                        &method.return_type,
                        false,
                    ));
                    if method.return_type != JavaType::primitive(JavaPrimitive::Void)
                        && !block_guarantees_exit(body)
                    {
                        violations.push(AstViolation::new(
                            DiagnosticCode::InvalidControlFlow,
                            "concrete non-void Java method does not return or throw on every path",
                        ));
                    }
                }
                violations
            }
            Self::Constructor(constructor) => {
                let mut violations = verify_modifiers(&constructor.modifiers);
                for parameter in &constructor.parameters {
                    violations.extend(parameter.ty.verify(JavaTypeUse::Parameter));
                }
                violations.extend(constructor.body.verify(context));
                let (mut scope, scope_violations) = JavaLexicalScope::for_constructor(constructor);
                violations.extend(scope_violations);
                violations.extend(verify_block_scope(
                    &constructor.body,
                    &mut scope,
                    &JavaType::primitive(JavaPrimitive::Void),
                    false,
                ));
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
        let mut violations = verify_modifiers(&self.modifiers);
        let distinct_type_parameters = self.type_parameters.iter().collect::<BTreeSet<_>>();
        if distinct_type_parameters.len() != self.type_parameters.len() {
            violations.push(AstViolation::new(
                DiagnosticCode::DuplicateDeclaration,
                "Java type parameter is declared more than once",
            ));
        }
        if !top_level && self.visibility == JavaVisibility::Package {
            violations.push(AstViolation::new(
                DiagnosticCode::InvalidStructure,
                "nested types require explicit public or private visibility",
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
            JavaHeritage::ExternalAdapter {
                delegated_field, ..
            } => {
                let has_final_component = self.members.iter().any(|member| {
                    matches!(member, JavaMember::Field(field)
                        if field.name == *delegated_field
                            && field.modifiers.contains(&JavaModifier::Final)
                            && !field.modifiers.contains(&JavaModifier::Static))
                });
                if self.kind != JavaDeclarationKind::FinalClass || !has_final_component {
                    violations.push(AstViolation::new(
                        DiagnosticCode::InvalidStructure,
                        "external adapter must be a final leaf with one named final component field",
                    ));
                }
            }
            JavaHeritage::None => {}
        }
        let mut field_names = BTreeSet::new();
        for component in &self.record_components {
            violations.extend(component.ty.verify(JavaTypeUse::Field));
            if !field_names.insert(component.name.clone()) {
                violations.push(AstViolation::new(
                    DiagnosticCode::DuplicateDeclaration,
                    "Java record component is declared more than once",
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
                            .map(|parameter| parameter.ty.clone())
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
                        .map(|parameter| parameter.ty.clone())
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
            violations.extend(member.verify(context));
        }
        violations
    }
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

fn verify_modifiers(modifiers: &[JavaModifier]) -> Vec<AstViolation> {
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

impl TargetFileItemNode<JavaDialect> for JavaFileItem {
    fn verify(&self, context: &TargetAstContext<'_, JavaDialect>) -> Vec<AstViolation> {
        match self {
            Self::Type {
                declared,
                declaration,
            } => {
                let mut violations = declaration.verify(context, true);
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
            Self::RuntimeMembers { members, .. } => members
                .iter()
                .flat_map(|value| value.verify(context))
                .collect(),
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
        assert_eq!(JavaRuntimeMember::ALL.len(), 13);
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
}
