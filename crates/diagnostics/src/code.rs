use std::fmt;

use serde::{Serialize, Serializer};

/// Central registry of stable diagnostic codes.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum DiagnosticCode {
    UnsupportedIrMajor,
    DuplicateDeclaration,
    TypeMismatch,
    NonExhaustiveMatch,
    ContractNonconformance,
    InvalidPortableTest,
    ImpureOperation,
    UnsupportedCapability,
    UnsafeOutputPath,
}

/// Short and long explanation for one registered code.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Explanation {
    pub code: DiagnosticCode,
    pub short: &'static str,
    pub long: &'static str,
}

impl DiagnosticCode {
    pub const ALL: [Self; 9] = [
        Self::UnsupportedIrMajor,
        Self::DuplicateDeclaration,
        Self::TypeMismatch,
        Self::NonExhaustiveMatch,
        Self::ContractNonconformance,
        Self::InvalidPortableTest,
        Self::ImpureOperation,
        Self::UnsupportedCapability,
        Self::UnsafeOutputPath,
    ];

    pub const fn as_str(self) -> &'static str {
        match self {
            Self::UnsupportedIrMajor => "P0001",
            Self::DuplicateDeclaration => "P0102",
            Self::TypeMismatch => "P0207",
            Self::NonExhaustiveMatch => "P0214",
            Self::ContractNonconformance => "P0220",
            Self::InvalidPortableTest => "P0230",
            Self::ImpureOperation => "P0301",
            Self::UnsupportedCapability => "P0404",
            Self::UnsafeOutputPath => "P0502",
        }
    }
}

impl fmt::Display for DiagnosticCode {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

impl Serialize for DiagnosticCode {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(self.as_str())
    }
}

pub fn explain(code: DiagnosticCode) -> Explanation {
    match code {
        DiagnosticCode::UnsupportedIrMajor => Explanation {
            code,
            short: "unsupported IR major version",
            long: "The document uses an IR major generation this reader cannot interpret safely.",
        },
        DiagnosticCode::DuplicateDeclaration => Explanation {
            code,
            short: "duplicate declaration",
            long: "Two declarations compete for the same portable name or identity in one scope.",
        },
        DiagnosticCode::TypeMismatch => Explanation {
            code,
            short: "type mismatch",
            long: "An expression's inferred portable type differs from the type required at this location.",
        },
        DiagnosticCode::NonExhaustiveMatch => Explanation {
            code,
            short: "non-exhaustive match",
            long: "A match does not cover every value admitted by its portable input type.",
        },
        DiagnosticCode::ContractNonconformance => Explanation {
            code,
            short: "contract implementation does not conform",
            long: "An explicit record implementation is missing or mismatches a required contract method.",
        },
        DiagnosticCode::InvalidPortableTest => Explanation {
            code,
            short: "invalid portable test",
            long: "A portable test invocation or typed expected outcome is not valid for the referenced callable.",
        },
        DiagnosticCode::ImpureOperation => Explanation {
            code,
            short: "impure operation",
            long: "The program requests an operation outside the pure v0 semantic model.",
        },
        DiagnosticCode::UnsupportedCapability => Explanation {
            code,
            short: "target capability unsupported",
            long: "The selected backend cannot preserve a capability required by the checked program.",
        },
        DiagnosticCode::UnsafeOutputPath => Explanation {
            code,
            short: "unsafe output path",
            long: "A generated path is absolute, escaping, duplicated, reserved, or otherwise unsafe to materialize.",
        },
    }
}
