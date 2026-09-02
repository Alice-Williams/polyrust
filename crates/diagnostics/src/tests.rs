use std::collections::{BTreeMap, BTreeSet};

use super::*;

#[derive(Default)]
struct Sources(BTreeMap<String, String>);

impl Sources {
    fn with(mut self, file: &str, source: &str) -> Self {
        self.0.insert(file.to_owned(), source.to_owned());
        self
    }
}

impl SourceProvider for Sources {
    fn source(&self, file: &str) -> Option<String> {
        self.0.get(file).cloned()
    }
}

fn file_source(file: &str, start: u64, end: u64) -> SourceRef {
    SourceRef::File(FileSpan {
        file: file.to_owned(),
        start,
        end,
    })
}

fn snapshot_diagnostic() -> Diagnostic {
    let mut diagnostic = Diagnostic::error(
        DiagnosticCode::TypeMismatch,
        "expected String",
        file_source("Chloë.poly", 4, 9),
    );
    diagnostic.labels[0].message = "found integer".to_owned();
    diagnostic
        .notes
        .push("portable strings are distinct from integers".to_owned());
    diagnostic.hint = Some("convert the value before returning it".to_owned());
    diagnostic.target = Some("rust".to_owned());
    diagnostic
}

#[test]
fn registry_is_unique_and_explained() {
    let mut codes = BTreeSet::new();
    for code in DiagnosticCode::ALL {
        assert!(codes.insert(code.as_str()), "duplicate code {code}");
        let explanation = explain(code);
        assert_eq!(explanation.code, code);
        assert!(!explanation.short.trim().is_empty());
        assert!(!explanation.long.trim().is_empty());
        assert_ne!(explanation.short, explanation.long);
    }
}

#[test]
fn plain_and_colored_terminal_snapshots_are_stable() {
    let diagnostic = snapshot_diagnostic();
    let sources = Sources::default().with("Chloë.poly", "let café = 1;\n");
    let plain = concat!(
        "error[P0207]: expected String\n",
        " --> Chloë.poly:4..9\n",
        "     |\n",
        "   1 | let café = 1;\n",
        "     |     ^^^^ found integer\n",
        "  = note: portable strings are distinct from integers\n",
        "  = help: convert the value before returning it\n",
        "  = target: rust\n",
    );
    assert_eq!(render_terminal(&diagnostic, &sources, Color::Never), plain);

    let colored = plain.replacen("error[P0207]", "\u{1b}[31merror[P0207]\u{1b}[0m", 1);
    assert_eq!(render_terminal(&diagnostic, &sources, Color::Ansi), colored);
}

#[test]
fn multi_label_and_related_location_snapshot_is_stable() {
    let mut diagnostic = Diagnostic::error(
        DiagnosticCode::DuplicateDeclaration,
        "duplicate declaration",
        file_source("main.poly", 0, 3),
    );
    diagnostic.labels[0].message = "second declaration".to_owned();
    diagnostic.labels.push(Label {
        style: LabelStyle::Secondary,
        source: file_source("main.poly", 8, 11),
        message: "first declaration".to_owned(),
    });
    diagnostic.related.push(RelatedLocation {
        source: SourceRef::logical(["module(example)", "constant(ONE)"]),
        message: "builder created this name".to_owned(),
    });
    let sources = Sources::default().with("main.poly", "one = 1\none = 2\n");

    assert_eq!(
        render_terminal(&diagnostic, &sources, Color::Never),
        concat!(
            "error[P0102]: duplicate declaration\n",
            " --> main.poly:0..3\n",
            "     |\n",
            "   1 | one = 1\n",
            "     | ^^^ second declaration\n",
            " --> main.poly:8..11\n",
            "     |\n",
            "   2 | one = 2\n",
            "     | --- first declaration\n",
            "  = related: logical:module(example) > constant(ONE): builder created this name\n",
        )
    );
}

#[test]
fn logical_path_and_missing_source_snapshots_are_stable() {
    let logical = Diagnostic::error(
        DiagnosticCode::InterfaceNonconformance,
        "missing method",
        SourceRef::logical(["module(example)", "record(User)", "impl(Display)"]),
    );
    assert_eq!(
        render_terminal(&logical, &Sources::default(), Color::Never),
        concat!(
            "error[P0220]: missing method\n",
            " --> logical: module(example) > record(User) > impl(Display)\n",
        )
    );

    let mut missing = Diagnostic::error(
        DiagnosticCode::TypeMismatch,
        "expected String",
        file_source("Chloë.poly", 99, u64::MAX),
    );
    missing.labels[0].message = "source was not retained".to_owned();
    assert_eq!(
        render_terminal(&missing, &Sources::default(), Color::Never),
        concat!(
            "error[P0207]: expected String\n",
            " --> Chloë.poly:99..18446744073709551615\n",
            "  = source unavailable\n",
            "  = primary: source was not retained\n",
        )
    );
}

#[test]
fn json_schema_and_snapshot_are_stable_and_ansi_free() {
    let diagnostic = snapshot_diagnostic();
    let json = render_json(std::slice::from_ref(&diagnostic)).unwrap();
    assert_eq!(
        json,
        concat!(
            r#"[{"code":"P0207","severity":"error","message":"expected String","#,
            r#""labels":[{"style":"primary","source":{"kind":"file","data":{"file":"Chloë.poly","start":4,"end":9}},"message":"found integer"}],"#,
            r#""related":[],"notes":["portable strings are distinct from integers"],"#,
            r#""hint":"convert the value before returning it","target":"rust"}]"#,
        )
    );
    assert!(!json.as_bytes().contains(&0x1b));

    let value: serde_json::Value = serde_json::from_str(&json).unwrap();
    let object = value[0].as_object().unwrap();
    let keys = object.keys().map(String::as_str).collect::<BTreeSet<_>>();
    assert_eq!(
        keys,
        BTreeSet::from([
            "code", "hint", "labels", "message", "notes", "related", "severity", "target",
        ])
    );
}

#[test]
fn windows_and_unix_newline_modes_have_identical_content() {
    let diagnostic = snapshot_diagnostic();
    let sources = Sources::default().with("Chloë.poly", "let café = 1;\r\n");
    let lf = render_terminal_with_options(
        &diagnostic,
        &sources,
        RenderOptions {
            color: Color::Never,
            line_ending: LineEnding::Lf,
        },
    );
    let crlf = render_terminal_with_options(
        &diagnostic,
        &sources,
        RenderOptions {
            color: Color::Never,
            line_ending: LineEnding::CrLf,
        },
    );

    assert!(!lf.contains('\r'));
    assert!(!crlf.replace("\r\n", "").contains(['\r', '\n']));
    assert_eq!(crlf, lf.replace('\n', "\r\n"));
}

#[test]
fn unicode_and_zero_width_spans_are_rendered_safely() {
    let diagnostic = Diagnostic::error(
        DiagnosticCode::TypeMismatch,
        "zero-width Unicode location",
        file_source("λ.poly", 1, 1),
    );
    let sources = Sources::default().with("λ.poly", "λ🙂\n");
    let rendered = render_terminal(&diagnostic, &sources, Color::Never);

    assert!(rendered.contains("   1 | λ🙂"));
    assert!(rendered.contains("     | ^"));
}

#[test]
fn generated_arbitrary_spans_never_panic_or_slice_invalid_utf8() {
    let sources = Sources::default().with("unicode.poly", "λ🙂\r\nsecond café\n");
    let mut state = 0x9e37_79b9_7f4a_7c15_u64;

    for case in 0..2_048 {
        state = state
            .wrapping_mul(6_364_136_223_846_793_005)
            .wrapping_add(1);
        let start = if case % 127 == 0 {
            u64::MAX
        } else {
            state % 64
        };
        state = state
            .wrapping_mul(6_364_136_223_846_793_005)
            .wrapping_add(1);
        let end = if case % 131 == 0 {
            u64::MAX
        } else {
            state % 64
        };
        let diagnostic = Diagnostic::error(
            DiagnosticCode::TypeMismatch,
            "generated span",
            file_source("unicode.poly", start, end),
        );
        let rendered = render_terminal(&diagnostic, &sources, Color::Never);
        assert!(rendered.ends_with('\n'));
    }
}

#[test]
fn diagnostics_sort_by_source_then_code() {
    let mut diagnostics = vec![
        Diagnostic::error(
            DiagnosticCode::TypeMismatch,
            "b",
            file_source("b.poly", 0, 1),
        ),
        Diagnostic::error(
            DiagnosticCode::TypeMismatch,
            "a later code",
            file_source("a.poly", 0, 1),
        ),
        Diagnostic::error(
            DiagnosticCode::DuplicateDeclaration,
            "a earlier code",
            file_source("a.poly", 0, 1),
        ),
    ];
    sort_diagnostics(&mut diagnostics);

    assert_eq!(
        diagnostics
            .iter()
            .map(|diagnostic| diagnostic.code)
            .collect::<Vec<_>>(),
        vec![
            DiagnosticCode::DuplicateDeclaration,
            DiagnosticCode::TypeMismatch,
            DiagnosticCode::TypeMismatch,
        ]
    );
    assert_eq!(diagnostics[2].message, "b");
}
