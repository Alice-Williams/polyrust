#![forbid(unsafe_code)]

//! Complete typed-behavior port of `WebReflection/html-escaper` 3.0.3.

use std::sync::Arc;

use portable_backend_c::CBackend;
use portable_backend_cpp::CppBackend;
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

/// Builds and checks the target-independent html-escaper program.
pub fn program() -> CheckedProgram {
    let mut module = ModuleBuilder::new("html_escaper");
    let escape = module.function(
        "escape",
        Visibility::Public,
        vec!["Escape five HTML-significant characters.".into()],
        |function| {
            function.parameter(Parameter::new("string", Type::string()));
            function.returns(Type::string());
            function.body(|body| {
                let arguments = [
                    body.local("string"),
                    body.literal(Value::string("&")),
                    body.literal(Value::string("&amp;")),
                    body.literal(Value::string("<")),
                    body.literal(Value::string("&lt;")),
                    body.literal(Value::string(">")),
                    body.literal(Value::string("&gt;")),
                    body.literal(Value::string("'")),
                    body.literal(Value::string("&#39;")),
                    body.literal(Value::string("\"")),
                    body.literal(Value::string("&quot;")),
                ];
                let output = body.intrinsic(Operation::StringReplaceMany, arguments);
                body.block([], Some(output))
            });
        },
    );
    let unescape = module.function(
        "unescape",
        Visibility::Public,
        vec!["Decode the ten HTML entity spellings accepted upstream.".into()],
        |function| {
            function.parameter(Parameter::new("string", Type::string()));
            function.returns(Type::string());
            function.body(|body| {
                let arguments = [
                    body.local("string"),
                    body.literal(Value::string("&amp;")),
                    body.literal(Value::string("&")),
                    body.literal(Value::string("&#38;")),
                    body.literal(Value::string("&")),
                    body.literal(Value::string("&lt;")),
                    body.literal(Value::string("<")),
                    body.literal(Value::string("&#60;")),
                    body.literal(Value::string("<")),
                    body.literal(Value::string("&gt;")),
                    body.literal(Value::string(">")),
                    body.literal(Value::string("&#62;")),
                    body.literal(Value::string(">")),
                    body.literal(Value::string("&apos;")),
                    body.literal(Value::string("'")),
                    body.literal(Value::string("&#39;")),
                    body.literal(Value::string("'")),
                    body.literal(Value::string("&quot;")),
                    body.literal(Value::string("\"")),
                    body.literal(Value::string("&#34;")),
                    body.literal(Value::string("\"")),
                ];
                let output = body.intrinsic(Operation::StringReplaceMany, arguments);
                body.block([], Some(output))
            });
        },
    );

    for (name, input, output) in escape_vectors() {
        module.portable_test(
            format!("escape_{name}"),
            Visibility::Package,
            vec![],
            Invocation::function(
                escape,
                [TypedValue::new(Type::string(), Value::string(input))],
            ),
            Expected::value(TypedValue::new(Type::string(), Value::string(output))),
        );
    }
    for (name, input, output) in unescape_vectors() {
        module.portable_test(
            format!("unescape_{name}"),
            Visibility::Package,
            vec![],
            Invocation::function(
                unescape,
                [TypedValue::new(Type::string(), Value::string(input))],
            ),
            Expected::value(TypedValue::new(Type::string(), Value::string(output))),
        );
    }

    module
        .finish()
        .unwrap_or_else(|diagnostics| panic!("html-escaper did not check: {diagnostics:#?}"))
}

/// Generates all eight required target packages from one checked program.
pub fn manifests() -> Vec<(&'static str, OutputManifest)> {
    let program = program();
    let backends: [(&str, Arc<dyn Backend>); 8] = [
        ("rust", Arc::new(RustBackend)),
        ("typescript", Arc::new(TypeScriptBackend)),
        ("javascript", Arc::new(JavaScriptBackend)),
        ("python", Arc::new(PythonBackend)),
        ("go", Arc::new(GoV0Backend)),
        ("java", Arc::new(JavaBackend)),
        ("cpp", Arc::new(CppBackend)),
        ("c", Arc::new(CBackend)),
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

fn escape_vectors() -> [(&'static str, &'static str, &'static str); 18] {
    [
        ("official_forward", "&<>'\"", "&amp;&lt;&gt;&#39;&quot;"),
        ("official_inverted", "<>'\"&", "&lt;&gt;&#39;&quot;&amp;"),
        ("empty", "", ""),
        ("plain_ascii", "plain text", "plain text"),
        ("ampersands", "&&", "&amp;&amp;"),
        ("angles", "<<>>", "&lt;&lt;&gt;&gt;"),
        ("single_quote", "'", "&#39;"),
        ("double_quote", "\"", "&quot;"),
        ("already_named", "&amp;", "&amp;amp;"),
        ("already_numeric", "&#38;", "&amp;#38;"),
        ("named_lt", "&lt;", "&amp;lt;"),
        ("unicode", "🦀 café e\u{301}", "🦀 café e\u{301}"),
        ("nul", "a\0b", "a\0b"),
        ("newlines", "<a>\r\n&", "&lt;a&gt;\r\n&amp;"),
        (
            "attribute",
            "<script data-x='\"'>&",
            "&lt;script data-x=&#39;&quot;&#39;&gt;&amp;",
        ),
        ("astral_boundaries", "🦀<&>🦀", "🦀&lt;&amp;&gt;🦀"),
        ("semicolon", ";&;", ";&amp;;"),
        ("slashes", "</a>\\", "&lt;/a&gt;\\"),
    ]
}

fn unescape_vectors() -> [(&'static str, &'static str, &'static str); 24] {
    [
        ("official_forward", "&amp;&lt;&gt;&#39;&quot;", "&<>'\""),
        ("official_inverted", "&lt;&gt;&#39;&quot;&amp;", "<>'\"&"),
        ("empty", "", ""),
        ("plain_ascii", "plain text", "plain text"),
        ("named_amp", "&amp;", "&"),
        ("numeric_amp", "&#38;", "&"),
        ("named_lt", "&lt;", "<"),
        ("numeric_lt", "&#60;", "<"),
        ("named_gt", "&gt;", ">"),
        ("numeric_gt", "&#62;", ">"),
        ("named_apos", "&apos;", "'"),
        ("numeric_apos", "&#39;", "'"),
        ("named_quot", "&quot;", "\""),
        ("numeric_quot", "&#34;", "\""),
        (
            "all_spellings",
            "&amp;&#38;&lt;&#60;&gt;&#62;&apos;&#39;&quot;&#34;",
            "&&<<>>''\"\"",
        ),
        ("nested_named", "&amp;lt;", "&lt;"),
        ("nested_numeric", "&#38;lt;", "&lt;"),
        ("double_amp", "&amp;amp;", "&amp;"),
        ("named_then_numeric", "&amp;#38;", "&#38;"),
        ("numeric_then_named", "&#38;amp;", "&amp;"),
        ("unknown", "&copy;&AMP;", "&copy;&AMP;"),
        ("incomplete", "&amp &lt", "&amp &lt"),
        ("unicode_and_nul", "🦀&lt;\0&gt;", "🦀<\0>"),
        ("adjacent_boundary", "&&amp;;&lt;", "&&;<"),
    ]
}

#[cfg(test)]
mod tests {
    use portable_eval::Evaluator;

    use super::*;

    #[test]
    fn all_forty_two_vectors_pass_in_the_reference_evaluator() {
        let program = program();
        let results = Evaluator::new(&program).run_all_tests();
        assert_eq!(results.len(), 42);
        assert!(results.iter().all(|result| result.passed), "{results:#?}");
    }

    #[test]
    fn all_eight_manifests_are_nonempty_and_repeatable() {
        let first = manifests();
        let second = manifests();
        assert_eq!(first, second);
        assert_eq!(first.len(), 8);
        assert!(
            first
                .iter()
                .all(|(_, manifest)| !manifest.files().is_empty())
        );
    }
}
