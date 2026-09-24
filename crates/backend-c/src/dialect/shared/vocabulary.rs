//! Closed vocabulary for the first no-heap C shared-graph profile.

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
pub struct CDialect;

/// Shared type vocabulary distinguishes a callable result from C object types.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum CPrimitiveType {
    Scalar(crate::ast::CScalarType),
    Void,
}

/// Categories not admitted by this profile have no constructible sentinel.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum CUnavailable {}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum CStdType {
    I32,
    I64,
    U32,
    U64,
    Size,
}

/// Language-owned references distinguish standard typedefs from certified tags.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum CReferencedType {
    Standard(CStdType),
    Certified(super::CDependencyStruct),
}

impl CStdType {
    pub(super) const fn spelling(self) -> &'static str {
        match self {
            Self::I32 => "int32_t",
            Self::I64 => "int64_t",
            Self::U32 => "uint32_t",
            Self::U64 => "uint64_t",
            Self::Size => "size_t",
        }
    }
    pub(super) const fn header(self) -> crate::dialect::CHeader {
        match self {
            Self::I32 | Self::I64 | Self::U32 | Self::U64 => crate::dialect::CHeader::Stdint,
            Self::Size => crate::dialect::CHeader::Stddef,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum CInvocation {
    Function,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum CSharedTypeKind {
    Struct,
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum CFileGrammar {
    TranslationUnit,
    Header(super::CHeaderGuard),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum CVisibility {
    Exported,
    Private,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum CNamespace {
    Tag,
    Ordinary,
}
