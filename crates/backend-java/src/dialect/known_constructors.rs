//! Java dialect: known constructors.

use super::signature_matching::{JavaSignaturePosition, type_pattern_matches};
use crate::ast::{JavaArrayOwnership, JavaIdentifier, JavaKnownType, JavaPrimitive, JavaType};
use std::collections::BTreeMap;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum JavaKnownConstructor {
    AssertionErrorString,
    IllegalArgumentExceptionString,
    IllegalStateExceptionString,
    ArrayList,
    ArrayListFromList,
    RuntimeError,
    RuntimeResult,
    RuntimeOption,
    RuntimeValueResult,
    RuntimeBytes,
    RuntimeScalar,
    RuntimeUnit,
}

impl JavaKnownConstructor {
    pub const ALL: [Self; 12] = [
        Self::AssertionErrorString,
        Self::IllegalArgumentExceptionString,
        Self::IllegalStateExceptionString,
        Self::ArrayList,
        Self::RuntimeError,
        Self::RuntimeResult,
        Self::RuntimeOption,
        Self::RuntimeValueResult,
        Self::RuntimeBytes,
        Self::RuntimeScalar,
        Self::RuntimeUnit,
        Self::ArrayListFromList,
    ];

    pub fn owner(self) -> JavaKnownType {
        match self {
            Self::AssertionErrorString => JavaKnownType::AssertionError,
            Self::IllegalArgumentExceptionString => JavaKnownType::IllegalArgumentException,
            Self::IllegalStateExceptionString => JavaKnownType::IllegalStateException,
            Self::ArrayList | Self::ArrayListFromList => JavaKnownType::ArrayList,
            Self::RuntimeError => JavaKnownType::RuntimeError,
            Self::RuntimeResult => JavaKnownType::RuntimeResult,
            Self::RuntimeOption => JavaKnownType::RuntimeOption,
            Self::RuntimeValueResult => JavaKnownType::RuntimeValueResult,
            Self::RuntimeBytes => JavaKnownType::RuntimeBytes,
            Self::RuntimeScalar => JavaKnownType::RuntimeScalar,
            Self::RuntimeUnit => JavaKnownType::RuntimeUnit,
        }
    }

    pub fn signature(self) -> (JavaType, Vec<JavaType>) {
        let t = JavaType::TypeVariable(JavaIdentifier::from_portable("T"));
        let e = JavaType::TypeVariable(JavaIdentifier::from_portable("E"));
        let owner = match self {
            Self::ArrayList | Self::ArrayListFromList => {
                JavaType::generic(JavaKnownType::ArrayList, vec![t.clone()])
            }
            Self::RuntimeResult => JavaType::generic(JavaKnownType::RuntimeResult, vec![t.clone()]),
            Self::RuntimeOption => JavaType::generic(JavaKnownType::RuntimeOption, vec![t.clone()]),
            Self::RuntimeValueResult => JavaType::generic(
                JavaKnownType::RuntimeValueResult,
                vec![t.clone(), e.clone()],
            ),
            _ => JavaType::known(self.owner()),
        };
        let string = JavaType::known(JavaKnownType::String);
        let boolean = JavaType::primitive(JavaPrimitive::Boolean);
        let parameters = match self {
            Self::AssertionErrorString
            | Self::IllegalArgumentExceptionString
            | Self::IllegalStateExceptionString => vec![string],
            Self::ArrayList => vec![],
            Self::ArrayListFromList => {
                vec![JavaType::generic(JavaKnownType::List, vec![t.clone()])]
            }
            Self::RuntimeError => vec![string.clone(), string],
            Self::RuntimeResult => {
                vec![
                    boolean,
                    t.clone(),
                    JavaType::known(JavaKnownType::RuntimeError),
                ]
            }
            Self::RuntimeOption => vec![boolean, t.clone()],
            Self::RuntimeValueResult => vec![boolean, t, e],
            Self::RuntimeBytes => vec![JavaType::Array {
                component: Box::new(JavaType::primitive(JavaPrimitive::Byte)),
                ownership: JavaArrayOwnership::DefensiveCopyBoundary,
            }],
            Self::RuntimeScalar => vec![JavaType::primitive(JavaPrimitive::Int)],
            Self::RuntimeUnit => vec![],
        };
        (owner, parameters)
    }

    pub fn accepts(self, owner: &JavaType, parameters: &[JavaType]) -> bool {
        let (pattern_owner, pattern_parameters) = self.signature();
        if pattern_parameters.len() != parameters.len() {
            return false;
        }
        let mut bindings = BTreeMap::new();
        type_pattern_matches(
            &pattern_owner,
            owner,
            JavaSignaturePosition::Result,
            &mut bindings,
        ) && pattern_parameters
            .iter()
            .zip(parameters)
            .all(|(pattern, actual)| {
                type_pattern_matches(
                    pattern,
                    actual,
                    JavaSignaturePosition::Parameter,
                    &mut bindings,
                )
            })
    }
}
