//! Closed standard-library and legacy helper catalogue; not foreign nominal authority.

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
