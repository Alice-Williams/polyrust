//! Java dialect: runtime helpers.

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum JavaRuntimeHelper {
    Core,
    TaggedValues,
    CheckedIntegers,
    FloatBits,
    Unicode,
    Bytes,
    ImmutableLists,
    StringOperations,
    Interfaces,
}

impl JavaRuntimeHelper {
    pub const ALL: [Self; 9] = [
        Self::Core,
        Self::TaggedValues,
        Self::CheckedIntegers,
        Self::FloatBits,
        Self::Unicode,
        Self::Bytes,
        Self::ImmutableLists,
        Self::StringOperations,
        Self::Interfaces,
    ];

    pub const fn name(self) -> &'static str {
        match self {
            Self::Core => "java.runtime.core",
            Self::TaggedValues => "java.runtime.tagged-values",
            Self::CheckedIntegers => "java.runtime.checked-integers",
            Self::FloatBits => "java.runtime.float-bits",
            Self::Unicode => "java.runtime.unicode",
            Self::Bytes => "java.runtime.bytes",
            Self::ImmutableLists => "java.runtime.immutable-lists",
            Self::StringOperations => "java.runtime.string-operations",
            Self::Interfaces => "java.runtime.interfaces",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum JavaHelperCapability {
    Failures,
    TaggedValues,
    CheckedArithmetic,
    ExactFloatBits,
    UnicodeScalars,
    ImmutableBytes,
    ImmutableLists,
    StringOperations,
    InterfaceDispatch,
}

impl JavaHelperCapability {
    pub const fn name(self) -> &'static str {
        match self {
            Self::Failures => "failures",
            Self::TaggedValues => "tagged_values",
            Self::CheckedArithmetic => "checked_arithmetic",
            Self::ExactFloatBits => "exact_float_bits",
            Self::UnicodeScalars => "unicode_scalars",
            Self::ImmutableBytes => "immutable_bytes",
            Self::ImmutableLists => "immutable_lists",
            Self::StringOperations => "string_operations",
            Self::InterfaceDispatch => "interface_dispatch",
        }
    }
}
