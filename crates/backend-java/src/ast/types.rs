//! Java AST: types.

use super::identifiers::JavaIdentifier;
pub use super::known_types::JavaKnownType;
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

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum JavaTypeName {
    Known(JavaKnownType),
    Generated(GeneratedTypeId),
    Imported(crate::dialect::JavaImportedResultType),
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
            symbols.insert(TargetSymbolRef::KnownType((*value).into()));
            if let Some(helper) = value.runtime_helper() {
                symbols.insert(TargetSymbolRef::RuntimeHelper(helper));
            }
        }
        JavaTypeName::Generated(value) => {
            symbols.insert(TargetSymbolRef::Generated(GeneratedSymbolId::Type(*value)));
        }
        JavaTypeName::Imported(value) => {
            symbols.insert(TargetSymbolRef::KnownType(value.clone().into()));
        }
    }
}
