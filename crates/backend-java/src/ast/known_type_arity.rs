//! One exhaustive generic-arity authority for verification and linking metadata.

use super::types::JavaKnownType;

impl JavaKnownType {
    pub const fn generic_arity(self) -> u16 {
        match self {
            Self::ArrayList | Self::List | Self::RuntimeResult | Self::RuntimeOption => 1,
            Self::LinkedHashMap | Self::Map | Self::RuntimeValueResult => 2,
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
            | Self::BigInteger
            | Self::ByteBuffer
            | Self::CharBuffer
            | Self::CharacterCodingException
            | Self::Charset
            | Self::CharsetDecoder
            | Self::CodingErrorAction
            | Self::StandardCharsets
            | Self::Arrays
            | Self::Objects
            | Self::RuntimeUnit
            | Self::RuntimeError
            | Self::RuntimeBytes
            | Self::RuntimeSemanticValue
            | Self::RuntimeScalar => 0,
        }
    }
}
