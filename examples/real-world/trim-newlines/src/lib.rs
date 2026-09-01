#![forbid(unsafe_code)]

//! Complete runtime-behavior port of `sindresorhus/trim-newlines` 5.0.0.

use std::sync::Arc;

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

/// Builds and checks all three target-independent trim-newlines functions.
pub fn program() -> CheckedProgram {
    let mut module = ModuleBuilder::new("trim_newlines");
    let trim = module.function(
        "trim_newlines",
        Visibility::Public,
        vec!["Trim CR and LF scalars from both boundaries.".into()],
        |function| {
            function.parameter(Parameter::new("input", Type::string()));
            function.returns(Type::string());
            function.body(|body| {
                let input = body.local("input");
                let characters = body.literal(Value::string("\r\n"));
                let start = body.intrinsic(Operation::StringTrimStart, [input, characters]);
                let characters = body.literal(Value::string("\r\n"));
                let both = body.intrinsic(Operation::StringTrimEnd, [start, characters]);
                body.block([], Some(both))
            });
        },
    );
    let trim_start = module.function(
        "trim_newlines_start",
        Visibility::Public,
        vec!["Trim CR and LF scalars from the start boundary.".into()],
        |function| {
            function.parameter(Parameter::new("input", Type::string()));
            function.returns(Type::string());
            function.body(|body| {
                let input = body.local("input");
                let characters = body.literal(Value::string("\r\n"));
                let output = body.intrinsic(Operation::StringTrimStart, [input, characters]);
                body.block([], Some(output))
            });
        },
    );
    let trim_end = module.function(
        "trim_newlines_end",
        Visibility::Public,
        vec!["Trim CR and LF scalars from the end boundary.".into()],
        |function| {
            function.parameter(Parameter::new("input", Type::string()));
            function.returns(Type::string());
            function.body(|body| {
                let input = body.local("input");
                let characters = body.literal(Value::string("\r\n"));
                let output = body.intrinsic(Operation::StringTrimEnd, [input, characters]);
                body.block([], Some(output))
            });
        },
    );

    for (name, input, output) in both_vectors() {
        portable_test(&mut module, name, trim, input, output);
    }
    for (name, input, output) in start_vectors() {
        portable_test(&mut module, name, trim_start, input, output);
    }
    for (name, input, output) in end_vectors() {
        portable_test(&mut module, name, trim_end, input, output);
    }

    module
        .finish()
        .unwrap_or_else(|diagnostics| panic!("trim-newlines did not check: {diagnostics:#?}"))
}

fn portable_test(
    module: &mut ModuleBuilder,
    name: &str,
    function: portable_build::FunctionId,
    input: &str,
    output: &str,
) {
    module.portable_test(
        name,
        Visibility::Package,
        vec![],
        Invocation::function(
            function,
            [TypedValue::new(Type::string(), Value::string(input))],
        ),
        Expected::value(TypedValue::new(Type::string(), Value::string(output))),
    );
}

/// Generates all seven required target packages from one checked program.
pub fn manifests() -> Vec<(&'static str, OutputManifest)> {
    let program = program();
    let backends: [(&str, Arc<dyn Backend>); 7] = [
        ("rust", Arc::new(RustBackend)),
        ("typescript", Arc::new(TypeScriptBackend)),
        ("javascript", Arc::new(JavaScriptBackend)),
        ("python", Arc::new(PythonBackend)),
        ("go", Arc::new(GoV0Backend)),
        ("java", Arc::new(JavaBackend)),
        ("cpp", Arc::new(CppBackend)),
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

fn both_vectors() -> [(&'static str, &'static str, &'static str); 11] {
    [
        ("both_empty", "", ""),
        ("both_spaces", "  ", "  "),
        ("both_only_newlines", "\n\n\r", ""),
        ("both_one_each", "\nx\n", "x"),
        ("both_preserves_interior", "\nx\nx\n", "x\nx"),
        ("both_many_lf", "\n\n\nx\n\n\n", "x"),
        ("both_crlf", "\r\nx\r\n", "x"),
        ("both_mixed", "\n\r\n\nx\n\r\n\n", "x"),
        ("both_unicode", "\r\n🦄\n🦄\r\n", "🦄\n🦄"),
        (
            "both_unicode_separator",
            "\u{2028}x\u{2028}",
            "\u{2028}x\u{2028}",
        ),
        ("both_tab", "\tx\t", "\tx\t"),
    ]
}

fn start_vectors() -> [(&'static str, &'static str, &'static str); 10] {
    [
        ("start_empty", "", ""),
        ("start_spaces", "  ", "  "),
        ("start_only_newlines", "\n\n\r", ""),
        ("start_lf", "\nx", "x"),
        ("start_crlf", "\r\nx", "x"),
        ("start_many_lf", "\n\n\n\nx", "x"),
        ("start_mixed", "\n\n\r\n\nx", "x"),
        ("start_preserves_end", "x\n\n\r\n\n", "x\n\n\r\n\n"),
        ("start_unicode", "\r\n🦄\n", "🦄\n"),
        ("start_other_separator", "\u{2028}x", "\u{2028}x"),
    ]
}

fn end_vectors() -> [(&'static str, &'static str, &'static str); 10] {
    [
        ("end_empty", "", ""),
        ("end_spaces", "  ", "  "),
        ("end_only_newlines", "\n\n\r", ""),
        ("end_lf", "x\n", "x"),
        ("end_crlf", "x\r\n", "x"),
        ("end_many_lf", "x\n\n\n\n", "x"),
        ("end_mixed", "x\n\n\r\n\n", "x"),
        ("end_preserves_start", "\n\n\r\n\nx", "\n\n\r\n\nx"),
        ("end_unicode", "\n🦄\r\n", "\n🦄"),
        ("end_other_separator", "x\u{2028}", "x\u{2028}"),
    ]
}

#[cfg(test)]
mod tests {
    use portable_eval::Evaluator;

    use super::*;

    #[test]
    fn all_thirty_one_portable_vectors_pass_in_the_reference_evaluator() {
        let program = program();
        let results = Evaluator::new(&program).run_all_tests();
        assert_eq!(results.len(), 31);
        assert!(results.iter().all(|result| result.passed), "{results:#?}");
    }

    #[test]
    fn all_seven_manifests_are_nonempty_and_repeatable() {
        let first = manifests();
        let second = manifests();
        assert_eq!(first, second);
        assert_eq!(first.len(), 7);
        assert!(
            first
                .iter()
                .all(|(_, manifest)| !manifest.files().is_empty())
        );
    }
}
