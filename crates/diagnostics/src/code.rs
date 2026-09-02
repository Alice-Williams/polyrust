use std::fmt;

use serde::{Serialize, Serializer};

/// Central registry of stable diagnostic codes.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum DiagnosticCode {
    UnsupportedIrMajor,
    InvalidStructure,
    ExcessiveComplexity,
    InvalidIdentifier,
    UnresolvedReference,
    DuplicateDeclaration,
    AliasCycle,
    TypeMismatch,
    InvalidInvocation,
    InvalidControlFlow,
    NonExhaustiveMatch,
    UnreachablePattern,
    InterfaceNonconformance,
    InvalidInterfacePosition,
    InvalidPortableTest,
    ImpureOperation,
    RecursiveCall,
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
    pub const ALL: [Self; 19] = [
        Self::UnsupportedIrMajor,
        Self::InvalidStructure,
        Self::ExcessiveComplexity,
        Self::InvalidIdentifier,
        Self::UnresolvedReference,
        Self::DuplicateDeclaration,
        Self::AliasCycle,
        Self::TypeMismatch,
        Self::InvalidInvocation,
        Self::InvalidControlFlow,
        Self::NonExhaustiveMatch,
        Self::UnreachablePattern,
        Self::InterfaceNonconformance,
        Self::InvalidInterfacePosition,
        Self::InvalidPortableTest,
        Self::ImpureOperation,
        Self::RecursiveCall,
        Self::UnsupportedCapability,
        Self::UnsafeOutputPath,
    ];

    pub const fn as_str(self) -> &'static str {
        match self {
            Self::UnsupportedIrMajor => "P0001",
            Self::InvalidStructure => "P0002",
            Self::ExcessiveComplexity => "P0003",
            Self::InvalidIdentifier => "P0100",
            Self::UnresolvedReference => "P0101",
            Self::DuplicateDeclaration => "P0102",
            Self::AliasCycle => "P0103",
            Self::TypeMismatch => "P0207",
            Self::InvalidInvocation => "P0208",
            Self::InvalidControlFlow => "P0209",
            Self::NonExhaustiveMatch => "P0214",
            Self::UnreachablePattern => "P0215",
            Self::InterfaceNonconformance => "P0220",
            Self::InvalidInterfacePosition => "P0221",
            Self::InvalidPortableTest => "P0230",
            Self::ImpureOperation => "P0301",
            Self::RecursiveCall => "P0302",
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
        DiagnosticCode::InvalidStructure => Explanation {
            code,
            short: "invalid IR structure",
            long: "The unchecked document has invalid or duplicate structural node identities.",
        },
        DiagnosticCode::ExcessiveComplexity => Explanation {
            code,
            short: "checking complexity limit exceeded",
            long: "The unchecked program exceeds a bounded checker depth or work limit.",
        },
        DiagnosticCode::InvalidIdentifier => Explanation {
            code,
            short: "invalid portable identifier",
            long: "A declaration, member, parameter, or local name is not a portable identifier.",
        },
        DiagnosticCode::UnresolvedReference => Explanation {
            code,
            short: "unresolved reference",
            long: "A type, declaration, member, function, method, field, or local reference does not resolve.",
        },
        DiagnosticCode::DuplicateDeclaration => Explanation {
            code,
            short: "duplicate declaration",
            long: "Two declarations compete for the same portable name or identity in one scope.",
        },
        DiagnosticCode::AliasCycle => Explanation {
            code,
            short: "recursive type alias",
            long: "A type alias directly or indirectly refers back to itself.",
        },
        DiagnosticCode::TypeMismatch => Explanation {
            code,
            short: "type mismatch",
            long: "An expression's inferred portable type differs from the type required at this location.",
        },
        DiagnosticCode::InvalidInvocation => Explanation {
            code,
            short: "invalid invocation",
            long: "A function, method, intrinsic, or constructor has the wrong receiver, arity, or argument types.",
        },
        DiagnosticCode::InvalidControlFlow => Explanation {
            code,
            short: "invalid control flow",
            long: "A block has an invalid return path, unreachable statement, or incompatible branch result.",
        },
        DiagnosticCode::NonExhaustiveMatch => Explanation {
            code,
            short: "non-exhaustive match",
            long: "A match does not cover every value admitted by its portable input type.",
        },
        DiagnosticCode::UnreachablePattern => Explanation {
            code,
            short: "unreachable or duplicate pattern",
            long: "A match arm is duplicated or appears after a pattern that already covers it.",
        },
        DiagnosticCode::InterfaceNonconformance => Explanation {
            code,
            short: "interface implementation does not conform",
            long: "An explicit record implementation is missing or mismatches a required interface method.",
        },
        DiagnosticCode::InvalidInterfacePosition => Explanation {
            code,
            short: "invalid operation on an interface value",
            long: "An interface value is used by an operation such as equality that has no portable interface semantics.",
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
        DiagnosticCode::RecursiveCall => Explanation {
            code,
            short: "recursion is not portable in v0",
            long: "The function or method call graph contains a direct or indirect recursive cycle.",
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
