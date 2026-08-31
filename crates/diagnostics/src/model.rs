use serde::Serialize;

use crate::{DiagnosticCode, SourceRef};

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Severity {
    Error,
    Warning,
    Note,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum LabelStyle {
    Primary,
    Secondary,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct Label {
    pub style: LabelStyle,
    pub source: SourceRef,
    pub message: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct RelatedLocation {
    pub source: SourceRef,
    pub message: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct Diagnostic {
    pub code: DiagnosticCode,
    pub severity: Severity,
    pub message: String,
    pub labels: Vec<Label>,
    pub related: Vec<RelatedLocation>,
    pub notes: Vec<String>,
    pub hint: Option<String>,
    pub target: Option<String>,
}

impl Diagnostic {
    pub fn error(code: DiagnosticCode, message: impl Into<String>, source: SourceRef) -> Self {
        Self {
            code,
            severity: Severity::Error,
            message: message.into(),
            labels: vec![Label {
                style: LabelStyle::Primary,
                source,
                message: String::new(),
            }],
            related: vec![],
            notes: vec![],
            hint: None,
            target: None,
        }
    }

    pub fn sort_key(&self) -> (String, DiagnosticCode) {
        (
            self.labels
                .first()
                .map(|label| source_key(&label.source))
                .unwrap_or_default(),
            self.code,
        )
    }
}

pub fn sort_diagnostics(diagnostics: &mut [Diagnostic]) {
    diagnostics.sort_by_key(Diagnostic::sort_key);
}

pub(crate) fn source_key(source: &SourceRef) -> String {
    match source {
        SourceRef::File(span) => format!("F:{}:{:020}:{:020}", span.file, span.start, span.end),
        SourceRef::Logical(path) => format!("L:{}", path.segments.join("/")),
    }
}
