//! Typed hazards which cannot cross portable value boundaries or be erased.

use super::types::{JavaArrayOwnership, JavaKnownType, JavaType, JavaTypeName};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum JavaBoundaryHazard {
    InternalArray,
    MutableCollection(JavaKnownType),
}

impl JavaBoundaryHazard {
    pub(super) fn message(self) -> &'static str {
        match self {
            Self::InternalArray => "mutable Java array escapes a value boundary",
            Self::MutableCollection(_) => "mutable Java collection escapes a value boundary",
        }
    }
}

pub(super) fn hazard(ty: &JavaType) -> Option<JavaBoundaryHazard> {
    find_hazard(ty, HazardFilter::All)
}

pub(super) fn requires_internal_ownership(ty: &JavaType) -> bool {
    find_hazard(ty, HazardFilter::InternalOwnership).is_some()
}

#[derive(Clone, Copy)]
enum HazardFilter {
    All,
    InternalOwnership,
}

fn find_hazard(ty: &JavaType, filter: HazardFilter) -> Option<JavaBoundaryHazard> {
    let collection = |name| match filter {
        HazardFilter::All => collection_hazard(name),
        HazardFilter::InternalOwnership => None,
    };
    match ty {
        JavaType::Array {
            component,
            ownership,
        } => {
            if *ownership == JavaArrayOwnership::InternalMutable {
                Some(JavaBoundaryHazard::InternalArray)
            } else {
                find_hazard(component, filter)
            }
        }
        JavaType::Generic { raw, arguments } => {
            collection(raw).or_else(|| arguments.iter().find_map(|ty| find_hazard(ty, filter)))
        }
        JavaType::Reference(name) => collection(name),
        JavaType::Wildcard {
            bound: Some((_, bound)),
        } => find_hazard(bound, filter),
        JavaType::Primitive(_)
        | JavaType::Boxed(_)
        | JavaType::Wildcard { bound: None }
        | JavaType::TypeVariable(_) => None,
    }
}

fn collection_hazard(name: &JavaTypeName) -> Option<JavaBoundaryHazard> {
    let JavaTypeName::Known(known) = name else {
        return None;
    };
    match known {
        JavaKnownType::ArrayList | JavaKnownType::LinkedHashMap => {
            Some(JavaBoundaryHazard::MutableCollection(*known))
        }
        JavaKnownType::Object
        | JavaKnownType::String
        | JavaKnownType::Boolean
        | JavaKnownType::Byte
        | JavaKnownType::Character
        | JavaKnownType::Integer
        | JavaKnownType::Long
        | JavaKnownType::Double
        | JavaKnownType::Math
        | JavaKnownType::AssertionError
        | JavaKnownType::IllegalArgumentException
        | JavaKnownType::IllegalStateException
        | JavaKnownType::RuntimeException
        | JavaKnownType::BigInteger
        | JavaKnownType::ByteBuffer
        | JavaKnownType::CharBuffer
        | JavaKnownType::CharacterCodingException
        | JavaKnownType::Charset
        | JavaKnownType::CharsetDecoder
        | JavaKnownType::CodingErrorAction
        | JavaKnownType::StandardCharsets
        | JavaKnownType::Arrays
        | JavaKnownType::List
        | JavaKnownType::Map
        | JavaKnownType::Objects
        | JavaKnownType::RuntimeUnit
        | JavaKnownType::RuntimeError
        | JavaKnownType::RuntimeResult
        | JavaKnownType::RuntimeOption
        | JavaKnownType::RuntimeValueResult
        | JavaKnownType::RuntimeBytes
        | JavaKnownType::RuntimeSemanticValue
        | JavaKnownType::RuntimeScalar => None,
    }
}
