use std::fmt::Write as _;

use crate::{Diagnostic, LabelStyle, Severity, SourceRef};

pub trait SourceProvider {
    fn source(&self, file: &str) -> Option<String>;
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Color {
    Never,
    Ansi,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LineEnding {
    Lf,
    CrLf,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RenderOptions {
    pub color: Color,
    pub line_ending: LineEnding,
}

impl Default for RenderOptions {
    fn default() -> Self {
        Self {
            color: Color::Never,
            line_ending: LineEnding::Lf,
        }
    }
}

pub fn render_json(diagnostics: &[Diagnostic]) -> Result<String, serde_json::Error> {
    serde_json::to_string(diagnostics)
}

pub fn render_terminal(
    diagnostic: &Diagnostic,
    sources: &impl SourceProvider,
    color: Color,
) -> String {
    render_terminal_with_options(
        diagnostic,
        sources,
        RenderOptions {
            color,
            ..RenderOptions::default()
        },
    )
}

pub fn render_terminal_with_options(
    diagnostic: &Diagnostic,
    sources: &impl SourceProvider,
    options: RenderOptions,
) -> String {
    let severity = severity_name(diagnostic.severity);
    let (prefix, reset) = color_codes(diagnostic.severity, options.color);
    let mut output = format!(
        "{prefix}{severity}[{}]{reset}: {}\n",
        diagnostic.code, diagnostic.message
    );
    for label in &diagnostic.labels {
        let style = match label.style {
            LabelStyle::Primary => "primary",
            LabelStyle::Secondary => "secondary",
        };
        match &label.source {
            SourceRef::Logical(path) => {
                let path = if path.segments.is_empty() {
                    "<root>".to_owned()
                } else {
                    path.segments.join(" > ")
                };
                writeln!(output, " --> logical: {path}").expect("writing to String cannot fail");
                if !label.message.is_empty() {
                    writeln!(output, "  = {style}: {}", label.message)
                        .expect("writing to String cannot fail");
                }
            }
            SourceRef::File(span) => {
                writeln!(output, " --> {}:{}..{}", span.file, span.start, span.end)
                    .expect("writing to String cannot fail");
                if let Some(text) = sources.source(&span.file) {
                    render_source_frame(
                        &mut output,
                        &text,
                        span.start,
                        span.end,
                        label.style,
                        &label.message,
                    );
                } else {
                    output.push_str("  = source unavailable\n");
                    if !label.message.is_empty() {
                        writeln!(output, "  = {style}: {}", label.message)
                            .expect("writing to String cannot fail");
                    }
                }
            }
        }
    }
    for related in &diagnostic.related {
        writeln!(
            output,
            "  = related: {}: {}",
            source_display(&related.source),
            related.message
        )
        .expect("writing to String cannot fail");
    }
    for note in &diagnostic.notes {
        writeln!(output, "  = note: {note}").expect("writing to String cannot fail");
    }
    if let Some(hint) = &diagnostic.hint {
        writeln!(output, "  = help: {hint}").expect("writing to String cannot fail");
    }
    if let Some(target) = &diagnostic.target {
        writeln!(output, "  = target: {target}").expect("writing to String cannot fail");
    }
    apply_line_ending(&output, options.line_ending)
}

fn render_source_frame(
    output: &mut String,
    text: &str,
    requested_start: u64,
    requested_end: u64,
    style: LabelStyle,
    message: &str,
) {
    let start = safe_boundary(text, requested_start);
    let end = safe_boundary(text, requested_end).max(start);
    let line_start = text[..start].rfind('\n').map_or(0, |index| index + 1);
    let line_end = text[start..]
        .find('\n')
        .map_or(text.len(), |offset| start + offset);
    let display_line_end = line_end
        .checked_sub(1)
        .filter(|index| text.as_bytes().get(*index) == Some(&b'\r'))
        .unwrap_or(line_end);
    let line_number = text[..line_start]
        .bytes()
        .filter(|byte| *byte == b'\n')
        .count()
        + 1;
    let column = text[line_start..start].chars().count();
    let marked_end = end.min(display_line_end).max(start);
    let marker_count = text[start..marked_end].chars().count().max(1);
    let marker = match style {
        LabelStyle::Primary => '^',
        LabelStyle::Secondary => '-',
    };
    let width = line_number.to_string().len().max(3);

    writeln!(output, " {:width$} |", "", width = width).expect("writing to String cannot fail");
    writeln!(
        output,
        " {line_number:>width$} | {}",
        &text[line_start..display_line_end],
        width = width
    )
    .expect("writing to String cannot fail");
    write!(
        output,
        " {:width$} | {}{}",
        "",
        " ".repeat(column),
        marker.to_string().repeat(marker_count),
        width = width
    )
    .expect("writing to String cannot fail");
    if !message.is_empty() {
        write!(output, " {message}").expect("writing to String cannot fail");
    }
    output.push('\n');
}

fn safe_boundary(text: &str, requested: u64) -> usize {
    let requested = usize::try_from(requested).unwrap_or(usize::MAX);
    let mut index = requested.min(text.len());
    while index > 0 && !text.is_char_boundary(index) {
        index -= 1;
    }
    index
}

fn severity_name(severity: Severity) -> &'static str {
    match severity {
        Severity::Error => "error",
        Severity::Warning => "warning",
        Severity::Note => "note",
    }
}

fn color_codes(severity: Severity, color: Color) -> (&'static str, &'static str) {
    match (severity, color) {
        (_, Color::Never) => ("", ""),
        (Severity::Error, Color::Ansi) => ("\u{1b}[31m", "\u{1b}[0m"),
        (Severity::Warning, Color::Ansi) => ("\u{1b}[33m", "\u{1b}[0m"),
        (Severity::Note, Color::Ansi) => ("\u{1b}[34m", "\u{1b}[0m"),
    }
}

fn source_display(source: &SourceRef) -> String {
    match source {
        SourceRef::File(span) => format!("{}:{}..{}", span.file, span.start, span.end),
        SourceRef::Logical(path) if path.segments.is_empty() => "logical:<root>".to_owned(),
        SourceRef::Logical(path) => format!("logical:{}", path.segments.join(" > ")),
    }
}

fn apply_line_ending(output: &str, line_ending: LineEnding) -> String {
    let normalized = output.replace("\r\n", "\n").replace('\r', "\n");
    match line_ending {
        LineEnding::Lf => normalized,
        LineEnding::CrLf => normalized.replace('\n', "\r\n"),
    }
}
