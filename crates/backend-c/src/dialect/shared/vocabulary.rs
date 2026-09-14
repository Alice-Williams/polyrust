//! Closed vocabulary for the first no-heap C shared-graph profile.

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
pub struct CDialect;

/// Categories not admitted by this profile have no constructible sentinel.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum CUnavailable {}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum CStdType {
    I32,
    Size,
}

impl CStdType {
    pub(super) const fn spelling(self) -> &'static str {
        match self {
            Self::I32 => "int32_t",
            Self::Size => "size_t",
        }
    }
    pub(super) const fn header(self) -> crate::dialect::CHeader {
        match self {
            Self::I32 => crate::dialect::CHeader::Stdint,
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
