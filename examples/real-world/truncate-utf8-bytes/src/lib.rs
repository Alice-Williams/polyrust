#![forbid(unsafe_code)]

//! Complete representable typed-behavior port of `truncate-utf8-bytes` 1.0.2.

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

/// Builds and checks the target-independent truncate-utf8-bytes program.
pub fn program() -> CheckedProgram {
    let mut module = ModuleBuilder::new("truncate_utf8_bytes");
    let truncate = module.function(
        "truncate",
        Visibility::Public,
        vec!["Truncate a string to a UTF-8 byte budget without splitting a scalar.".into()],
        |function| {
            function.parameter(Parameter::new("string", Type::string()));
            function.parameter(Parameter::new("byte_length", Type::f64()));
            function.returns(Type::string());
            function.body(|body| {
                let string = body.local("string");
                let byte_length = body.local("byte_length");
                let output =
                    body.intrinsic(Operation::StringTruncateUtf8Bytes, [string, byte_length]);
                body.block([], Some(output))
            });
        },
    );

    for (name, input, budget, output) in vectors() {
        module.portable_test(
            format!("truncate_{name}"),
            Visibility::Package,
            vec![],
            Invocation::function(
                truncate,
                [
                    TypedValue::new(Type::string(), Value::string(input)),
                    TypedValue::new(Type::f64(), Value::f64(budget)),
                ],
            ),
            Expected::value(TypedValue::new(Type::string(), Value::string(output))),
        );
    }

    module
        .finish()
        .unwrap_or_else(|diagnostics| panic!("truncate-utf8-bytes did not check: {diagnostics:#?}"))
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

fn vectors() -> Vec<(&'static str, String, f64, String)> {
    vec![
        ("official_short", "a☃".into(), 2.0, "a".into()),
        ("empty_zero", String::new(), 0.0, String::new()),
        ("empty_negative", String::new(), -1.0, String::new()),
        ("empty_nan", String::new(), f64::NAN, String::new()),
        ("ascii_zero", "abc".into(), 0.0, String::new()),
        ("ascii_negative_zero", "abc".into(), -0.0, String::new()),
        ("ascii_half", "abc".into(), 0.5, String::new()),
        ("ascii_exact", "abc".into(), 2.0, "ab".into()),
        ("ascii_fractional", "abc".into(), 2.5, "ab".into()),
        ("ascii_full", "abc".into(), 3.0, "abc".into()),
        ("ascii_above", "abc".into(), 3.5, "abc".into()),
        ("two_byte_split", "éx".into(), 1.0, String::new()),
        ("two_byte_exact", "éx".into(), 2.0, "é".into()),
        ("three_byte_split", "a☃".into(), 3.0, "a".into()),
        ("three_byte_exact", "a☃".into(), 4.0, "a☃".into()),
        ("four_byte_split", "a🦀z".into(), 4.0, "a".into()),
        ("four_byte_exact", "a🦀z".into(), 5.0, "a🦀".into()),
        ("mixed_fractional", "a☃🦀".into(), 4.5, "a☃".into()),
        ("combining_split", "e\u{301}x".into(), 2.0, "e".into()),
        (
            "combining_exact",
            "e\u{301}x".into(),
            3.0,
            "e\u{301}".into(),
        ),
        ("nul", "a\0b".into(), 2.0, "a\0".into()),
        ("negative", "a☃".into(), -1.0, String::new()),
        (
            "negative_infinity",
            "a☃".into(),
            f64::NEG_INFINITY,
            String::new(),
        ),
        ("positive_infinity", "a☃".into(), f64::INFINITY, "a☃".into()),
        ("nan", "a☃".into(), f64::NAN, "a☃".into()),
        ("huge", "a☃".into(), f64::MAX, "a☃".into()),
        (
            "official_astral_250",
            format!("{}𐀀", "a".repeat(250)),
            255.0,
            format!("{}𐀀", "a".repeat(250)),
        ),
        (
            "official_astral_251",
            format!("{}𐀀", "a".repeat(251)),
            255.0,
            format!("{}𐀀", "a".repeat(251)),
        ),
        (
            "official_astral_252",
            format!("{}𐀀", "a".repeat(252)),
            255.0,
            "a".repeat(252),
        ),
        ("mixed_widths", "中🦀é".into(), 7.0, "中🦀".into()),
    ]
}

#[cfg(test)]
mod tests {
    use portable_eval::Evaluator;

    use super::*;

    #[test]
    fn all_thirty_vectors_pass_in_the_reference_evaluator() {
        let program = program();
        let results = Evaluator::new(&program).run_all_tests();
        assert_eq!(results.len(), 30);
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
