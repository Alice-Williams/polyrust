//! Measured no-call admission policy, enforced before issuing a certificate.
use super::Measurements;
use portable_diagnostics::{Diagnostic, DiagnosticCode, SourceRef};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum CResourceKind {
    Nodes,
    Nesting,
    Parameters,
    Fields,
    IdentifierBytes,
    CommentBytes,
    DiagnosticBytes,
    SourceBytes,
    NativeFrameBytes,
}

impl CResourceKind {
    pub(crate) const fn limit(self) -> u64 {
        match self {
            Self::Nodes => 4096,
            Self::DiagnosticBytes => 4095,
            Self::Nesting => 96,
            Self::Parameters => 127,
            Self::Fields | Self::IdentifierBytes => 256,
            Self::CommentBytes | Self::NativeFrameBytes => 1024 * 1024,
            Self::SourceBytes => 8 * 1024 * 1024,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct CResourceError {
    kind: CResourceKind,
    used: u64,
    source: SourceRef,
}

#[allow(dead_code)]
impl CResourceError {
    pub(crate) fn kind(&self) -> CResourceKind {
        self.kind
    }
    pub(crate) fn used(&self) -> u64 {
        self.used
    }
    pub(crate) fn diagnostic(&self) -> Diagnostic {
        Diagnostic::error(
            DiagnosticCode::TargetResourceLimit,
            format!(
                "C {:?} capacity: used {}, limit {}",
                self.kind,
                self.used,
                self.kind.limit()
            ),
            self.source.clone(),
        )
    }
}

pub(crate) fn check(measured: &Measurements, source: SourceRef) -> Vec<CResourceError> {
    use CResourceKind as K;
    [
        (K::Nodes, measured.nodes),
        (K::Nesting, measured.depth as u64),
        (K::Parameters, measured.max_parameters as u64),
        (K::Fields, measured.max_fields as u64),
        (K::IdentifierBytes, measured.max_identifier_bytes as u64),
        (K::CommentBytes, measured.comment_bytes),
        (K::DiagnosticBytes, measured.max_diagnostic_bytes as u64),
        (K::SourceBytes, measured.source_bound),
        (K::NativeFrameBytes, measured.frame_bound),
    ]
    .into_iter()
    .filter(|(kind, used)| *used > kind.limit())
    .map(|(kind, used)| CResourceError {
        kind,
        used,
        source: source.clone(),
    })
    .collect()
}
