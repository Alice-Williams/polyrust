//! Atomic library names are metadata, never executable source fragments.

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum CHeader {
    Stdlib,
    String,
    Math,
    Stdio,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum CSystemLibrary {
    Math,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CKnownCallForm {
    Function,
    DoubleMacro,
}

/// The closed identity, not a generated name or a matching prototype, grants
/// the standard contract. Both native forms admit direct calls only.
///
/// Library macros cannot become generated function-address expressions:
///
/// ```compile_fail
/// use portable_backend_c::{ast::CExpressions, dialect::CKnownCall};
/// fn invalid(ast: &CExpressions<'_>) {
///     ast.function_address(CKnownCall::IsNan);
/// }
/// ```
///
/// Nor can a standard function replace an indirect generated contract:
///
/// ```compile_fail
/// use portable_backend_c::{ast::{CExpressions, CValue}, dialect::CKnownCall};
/// fn invalid(ast: &CExpressions<'_>, pointer: CValue) {
///     ast.indirect(pointer, CKnownCall::Allocate);
/// }
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum CKnownCall {
    Allocate,
    Release,
    CopyBytes,
    CompareBytes,
    FloatRemainder,
    FloatTruncate,
    IsNan,
    SignBit,
    WriteBytes,
    StreamError,
}

impl CKnownCall {
    pub const ALL: [Self; 10] = [
        Self::Allocate,
        Self::Release,
        Self::CopyBytes,
        Self::CompareBytes,
        Self::FloatRemainder,
        Self::FloatTruncate,
        Self::IsNan,
        Self::SignBit,
        Self::WriteBytes,
        Self::StreamError,
    ];

    pub const fn spelling(self) -> &'static str {
        match self {
            Self::Allocate => "malloc",
            Self::Release => "free",
            Self::CopyBytes => "memcpy",
            Self::CompareBytes => "memcmp",
            Self::FloatRemainder => "fmod",
            Self::FloatTruncate => "trunc",
            Self::IsNan => "isnan",
            Self::SignBit => "signbit",
            Self::WriteBytes => "fwrite",
            Self::StreamError => "ferror",
        }
    }

    pub const fn header(self) -> CHeader {
        match self {
            Self::Allocate | Self::Release => CHeader::Stdlib,
            Self::CopyBytes | Self::CompareBytes => CHeader::String,
            Self::FloatRemainder | Self::FloatTruncate | Self::IsNan | Self::SignBit => {
                CHeader::Math
            }
            Self::WriteBytes | Self::StreamError => CHeader::Stdio,
        }
    }

    pub const fn system_library(self) -> Option<CSystemLibrary> {
        match self {
            Self::FloatRemainder | Self::FloatTruncate => Some(CSystemLibrary::Math),
            Self::Allocate
            | Self::Release
            | Self::CopyBytes
            | Self::CompareBytes
            | Self::IsNan
            | Self::SignBit
            | Self::WriteBytes
            | Self::StreamError => None,
        }
    }

    pub const fn form(self) -> CKnownCallForm {
        match self {
            Self::IsNan | Self::SignBit => CKnownCallForm::DoubleMacro,
            Self::Allocate
            | Self::Release
            | Self::CopyBytes
            | Self::CompareBytes
            | Self::FloatRemainder
            | Self::FloatTruncate
            | Self::WriteBytes
            | Self::StreamError => CKnownCallForm::Function,
        }
    }
}
