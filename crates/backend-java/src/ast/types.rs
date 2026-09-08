//! Java AST: types.

use super::identifiers::JavaIdentifier;
use crate::dialect::JavaDialect;
use portable_codegen::{AstViolation, GeneratedSymbolId, GeneratedTypeId, TargetSymbolRef};
use portable_diagnostics::DiagnosticCode;
use std::collections::BTreeSet;

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
                ownership: _,
            } => {
                violations.extend(component.verify(JavaTypeUse::Value));
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
                    if matches!(bound.as_ref(), Self::Primitive(_)) {
                        violations.push(type_error("Java wildcard bounds must be reference types"));
                    }
                    violations.extend(bound.verify(JavaTypeUse::TypeBound));
                }
            }
            Self::Primitive(_) | Self::Boxed(_) | Self::Reference(_) | Self::TypeVariable(_) => {}
        }
        if matches!(
            usage,
            JavaTypeUse::Parameter | JavaTypeUse::Return | JavaTypeUse::Field
        ) && let Some(hazard) = super::value_boundaries::hazard(self)
        {
            violations.push(type_error(hazard.message()));
        }
        violations
    }
}

pub(super) fn type_error(message: &str) -> AstViolation {
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
