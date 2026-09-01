#![forbid(unsafe_code)]

//! Complete typed-value port of `sindresorhus/normalize-newline` 5.0.0.

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

/// Builds both explicit overload-equivalent functions and all portable vectors.
pub fn program() -> CheckedProgram {
    let mut module = ModuleBuilder::new("normalize_newline");
    let normalize_text = module.function(
        "normalize_newline",
        Visibility::Public,
        vec!["Replace every CRLF sequence with LF in Unicode text.".into()],
        |function| {
            function.parameter(Parameter::new("input", Type::string()));
            function.returns(Type::string());
            function.body(|body| {
                let input = body.local("input");
                let crlf = body.literal(Value::string("\r\n"));
                let lf = body.literal(Value::string("\n"));
                let output = body.intrinsic(Operation::StringReplaceAll, [input, crlf, lf]);
                body.block([], Some(output))
            });
        },
    );
    let normalize_bytes = module.function(
        "normalize_newline_bytes",
        Visibility::Public,
        vec!["Replace every CRLF byte pair with LF in arbitrary bytes.".into()],
        |function| {
            function.parameter(Parameter::new("input", Type::bytes()));
            function.returns(Type::bytes());
            function.body(|body| {
                let input = body.local("input");
                let crlf = body.literal(Value::bytes([13, 10]));
                let lf = body.literal(Value::bytes([10]));
                let output = body.intrinsic(Operation::BytesReplaceAll, [input, crlf, lf]);
                body.block([], Some(output))
            });
        },
    );

    for (name, input, output) in string_vectors() {
        module.portable_test(
            name,
            Visibility::Package,
            vec![],
            Invocation::function(
                normalize_text,
                [TypedValue::new(Type::string(), Value::string(input))],
            ),
            Expected::value(TypedValue::new(Type::string(), Value::string(output))),
        );
    }
    for (name, input, output) in byte_vectors() {
        module.portable_test(
            name,
            Visibility::Package,
            vec![],
            Invocation::function(
                normalize_bytes,
                [TypedValue::new(Type::bytes(), Value::bytes(input))],
            ),
            Expected::value(TypedValue::new(Type::bytes(), Value::bytes(output))),
        );
    }

    module
        .finish()
        .unwrap_or_else(|diagnostics| panic!("normalize-newline did not check: {diagnostics:#?}"))
}

/// Generates all eight supported target packages from one checked program.
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

fn string_vectors() -> Vec<(&'static str, String, String)> {
    let mut vectors = vec![
        ("string_foo_multiple", "foo\r\nbar\r\nbaz", "foo\nbar\nbaz"),
        (
            "string_trailing_crlf",
            "foo\nbar\nbaz\r\n",
            "foo\nbar\nbaz\n",
        ),
        ("string_lf_only", "foo\nbar\n", "foo\nbar\n"),
        ("string_empty", "", ""),
        ("string_no_crlf", "no crlf here", "no crlf here"),
        ("string_only_crlf", "\r\n\r\n", "\n\n"),
        (
            "string_lone_carriage",
            "lone\rcarriage\nreturn",
            "lone\rcarriage\nreturn",
        ),
        ("string_leading", "\r\nvalue", "\nvalue"),
        ("string_unicode", "🦀\r\n🦄", "🦀\n🦄"),
        ("string_nul", "\0\r\n\0", "\0\n\0"),
        ("string_separated", "\rX\n", "\rX\n"),
        ("string_mixed", "\r\n\n\r\r\n", "\n\n\r\n"),
        ("string_astral_boundaries", "\r\n😀\r\n", "\n😀\n"),
        ("string_only_lone_cr", "\r\r\r", "\r\r\r"),
    ]
    .into_iter()
    .map(|(name, input, output)| (name, input.into(), output.into()))
    .collect::<Vec<_>>();
    vectors.push(("string_repeated", "\r\n".repeat(64), "\n".repeat(64)));
    vectors
}

fn byte_vectors() -> Vec<(&'static str, Vec<u8>, Vec<u8>)> {
    vec![
        (
            "bytes_foo",
            vec![102, 111, 111, 13, 10, 98, 97, 114],
            vec![102, 111, 111, 10, 98, 97, 114],
        ),
        ("bytes_multiple", vec![13, 10, 13, 10], vec![10, 10]),
        ("bytes_empty", vec![], vec![]),
        ("bytes_no_crlf", vec![102, 111, 111], vec![102, 111, 111]),
        (
            "bytes_lone_cr_end",
            vec![102, 111, 111, 13],
            vec![102, 111, 111, 13],
        ),
        (
            "bytes_lone_mixed",
            vec![13, 10, 13, 11, 10],
            vec![10, 13, 11, 10],
        ),
        (
            "bytes_invalid_utf8",
            vec![0, 255, 13, 10, 128],
            vec![0, 255, 10, 128],
        ),
        ("bytes_leading", vec![13, 10, 1], vec![10, 1]),
        ("bytes_trailing", vec![1, 13, 10], vec![1, 10]),
        (
            "bytes_zero_boundaries",
            vec![13, 10, 0, 13, 10],
            vec![10, 0, 10],
        ),
        ("bytes_non_cascading_left", vec![13, 13, 10], vec![13, 10]),
        ("bytes_non_cascading_right", vec![13, 10, 10], vec![10, 10]),
        ("bytes_lone_lf", vec![10], vec![10]),
        (
            "bytes_all_octets",
            (0_u8..=u8::MAX).collect(),
            (0_u8..=u8::MAX).collect(),
        ),
        ("bytes_repeated", [13_u8, 10].repeat(64), [10_u8].repeat(64)),
        (
            "bytes_high_no_match",
            vec![255, 254, 253],
            vec![255, 254, 253],
        ),
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
