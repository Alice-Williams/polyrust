#![forbid(unsafe_code)]

//! Complete typed-behavior port of \`sindresorhus/escape-string-regexp\` 5.0.0.
//!
//! This Rust code authors one checked portable program. It does not implement
//! the target function itself; six outputs generate independently tested Rust,
//! TypeScript, JavaScript, Python, Go, and Java packages from that program.

use std::sync::Arc;

use portable_backend_go::GoV0Backend;
use portable_backend_java::JavaBackend;
use portable_backend_python::PythonBackend;
use portable_backend_rust::RustBackend;
use portable_backend_typescript::{JavaScriptBackend, TypeScriptBackend};
use portable_build::{
    Expected, Invocation, ModuleBuilder, Operation, Parameter, Type, TypedValue, Value, Visibility,
};
use portable_check::v0::CheckedProgram;
use portable_codegen::{Backend, BackendOptions, OutputManifest};

/// Builds and checks the target-independent escape-string-regexp program.
pub fn program() -> CheckedProgram {
    let mut module = ModuleBuilder::new("escape_string_regexp");
    let escape = module.function(
        "escape_string_regexp",
        Visibility::Public,
        vec!["Escape RegExp syntax characters in a literal string.".into()],
        |function| {
            function.parameter(Parameter::new("input", Type::string()));
            function.returns(Type::string());
            function.body(|body| {
                let mut escaped = body.local("input");

                // Upstream performs a character-class replacement followed by
                // a hyphen replacement. Replacing these distinct one-character
                // needles sequentially is equivalent. Backslash must be first
                // so escape prefixes inserted by later passes remain untouched.
                for (needle, replacement) in [
                    ("\\", "\\\\"),
                    ("|", "\\|"),
                    ("{", "\\{"),
                    ("}", "\\}"),
                    ("(", "\\("),
                    (")", "\\)"),
                    ("[", "\\["),
                    ("]", "\\]"),
                    ("^", "\\^"),
                    ("$", "\\$"),
                    ("+", "\\+"),
                    ("*", "\\*"),
                    ("?", "\\?"),
                    (".", "\\."),
                    ("-", "\\x2d"),
                ] {
                    let needle = body.literal(Value::string(needle));
                    let replacement = body.literal(Value::string(replacement));
                    escaped =
                        body.intrinsic(Operation::StringReplaceAll, [escaped, needle, replacement]);
                }
                body.block([], Some(escaped))
            });
        },
    );

    for (name, input, output) in portable_vectors() {
        module.portable_test(
            name,
            Visibility::Package,
            vec![],
            Invocation::function(
                escape,
                [TypedValue::new(Type::string(), Value::string(input))],
            ),
            Expected::value(TypedValue::new(Type::string(), Value::string(output))),
        );
    }

    module.finish().unwrap_or_else(|diagnostics| {
        panic!("escape-string-regexp did not check: {diagnostics:#?}")
    })
}

/// Generates all six required target packages from the same checked program.
pub fn manifests() -> Vec<(&'static str, OutputManifest)> {
    let program = program();
    let backends: [(&str, Arc<dyn Backend>); 6] = [
        ("rust", Arc::new(RustBackend)),
        ("typescript", Arc::new(TypeScriptBackend)),
        ("javascript", Arc::new(JavaScriptBackend)),
        ("python", Arc::new(PythonBackend)),
        ("go", Arc::new(GoV0Backend)),
        ("java", Arc::new(JavaBackend)),
    ];
    backends
        .into_iter()
        .map(|(directory, backend)| {
            let manifest = backend
                .generate(&program, &BackendOptions::default())
                .unwrap_or_else(|error| panic!("{directory} generation failed: {error:?}"));
            (directory, manifest)
        })
        .collect()
}

fn portable_vectors() -> [(&'static str, &'static str, &'static str); 18] {
    [
        ("empty", "", ""),
        ("ordinary_ascii", "hello world", "hello world"),
        (
            "official_special_characters",
            "\\ ^ $ * + ? . ( ) | { } [ ]",
            "\\\\ \\^ \\$ \\* \\+ \\? \\. \\( \\) \\| \\{ \\} \\[ \\]",
        ),
        ("official_hyphen", "foo - bar", "foo \\x2d bar"),
        ("official_unicode_hyphen", "-", "\\x2d"),
        ("repeated_dot", "...", "\\.\\.\\."),
        ("repeated_backslash", "\\\\", "\\\\\\\\"),
        ("mixed_adjacent", "[a-z]+", "\\[a\\x2dz\\]\\+"),
        (
            "all_compact",
            "|\\{}()[\\]^$+*?.-",
            "\\|\\\\\\{\\}\\(\\)\\[\\\\\\]\\^\\$\\+\\*\\?\\.\\x2d",
        ),
        ("dollar_replacement_is_literal", "$&", "\\$&"),
        ("newline", "a\nb", "a\nb"),
        ("tab", "a\tb", "a\tb"),
        ("nul", "a\0b", "a\0b"),
        ("combining_text", "e\u{301}", "e\u{301}"),
        ("non_bmp", "🦄", "🦄"),
        (
            "unicode_with_special",
            "How much $ for a 🦄?",
            "How much \\$ for a 🦄\\?",
        ),
        ("hyphen_and_backslash", "\\-", "\\\\\\x2d"),
        ("slashes_and_brackets", "\\[\\]", "\\\\\\[\\\\\\]"),
    ]
}

#[cfg(test)]
mod tests {
    use portable_eval::Evaluator;

    use super::*;

    #[test]
    fn all_portable_vectors_pass_in_the_reference_evaluator() {
        let program = program();
        let results = Evaluator::new(&program).run_all_tests();
        assert_eq!(results.len(), portable_vectors().len());
        assert!(results.iter().all(|result| result.passed), "{results:#?}");
    }

    #[test]
    fn all_six_manifests_are_nonempty_and_repeatable() {
        let first = manifests();
        let second = manifests();
        assert_eq!(first, second);
        assert_eq!(first.len(), 6);
        assert!(
            first
                .iter()
                .all(|(_, manifest)| !manifest.files().is_empty())
        );
    }
}
