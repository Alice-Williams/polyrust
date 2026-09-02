#![forbid(unsafe_code)]

//! Complete explicit-argument port of `sindresorhus/has-flag` 5.0.1.

use std::sync::Arc;

use portable_backend_c::CBackend;
use portable_backend_cpp::CppBackend;
use portable_backend_go::GoV0Backend;
use portable_backend_java::JavaBackend;
use portable_backend_python::PythonBackend;
use portable_backend_rust::RustBackend;
use portable_backend_typescript::{JavaScriptBackend, TypeScriptBackend};
use portable_build::{
    BodyBuilder, Expected, Expr, Invocation, ModuleBuilder, Operation, Parameter, Type, TypedValue,
    Value, Visibility,
};
use portable_check::v0::CheckedProgram;
use portable_codegen::{Backend, BackendOptions, OutputManifest};

/// Builds `has_flag(flag, argv)` and the official plus portable boundary vectors.
pub fn program() -> CheckedProgram {
    let mut module = ModuleBuilder::new("has_flag");
    let has_flag = module.function(
        "has_flag",
        Visibility::Public,
        vec![
            "Return whether argv contains the normalized flag before the first -- terminator."
                .into(),
        ],
        |function| {
            function.parameter(Parameter::new("flag", Type::string()));
            function.parameter(Parameter::new("argv", Type::list(Type::string())));
            function.returns(Type::bool());
            function.body(|body| {
                let position_exists = {
                    let position = flag_position(body);
                    body.intrinsic(Operation::OptionIsSome, [position])
                };
                let terminator_absent = {
                    let terminator = terminator_position(body);
                    body.intrinsic(Operation::OptionIsNone, [terminator])
                };
                let position_value = {
                    let position = flag_position(body);
                    let fallback = body.literal(Value::i64(-1));
                    body.intrinsic(Operation::OptionUnwrapOr, [position, fallback])
                };
                let terminator_value = {
                    let terminator = terminator_position(body);
                    let fallback = body.literal(Value::i64(-1));
                    body.intrinsic(Operation::OptionUnwrapOr, [terminator, fallback])
                };
                let before_terminator =
                    body.intrinsic(Operation::Less, [position_value, terminator_value]);
                let admitted =
                    body.intrinsic(Operation::BoolOr, [terminator_absent, before_terminator]);
                let result = body.intrinsic(Operation::BoolAnd, [position_exists, admitted]);
                body.block([], Some(result))
            });
        },
    );

    for (name, flag, argv, expected) in vectors() {
        module.portable_test(
            name,
            Visibility::Package,
            vec![],
            Invocation::function(
                has_flag,
                [
                    TypedValue::new(Type::string(), Value::string(flag)),
                    TypedValue::new(
                        Type::list(Type::string()),
                        Value::list(argv.into_iter().map(Value::string)),
                    ),
                ],
            ),
            Expected::value(TypedValue::new(Type::bool(), Value::bool(expected))),
        );
    }

    module
        .finish()
        .unwrap_or_else(|diagnostics| panic!("has-flag did not check: {diagnostics:#?}"))
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

fn flag_prefix(body: &mut BodyBuilder<'_>) -> Expr {
    let flag = body.local("flag");
    let dash = body.literal(Value::string("-"));
    let already_prefixed = body.intrinsic(Operation::StringStartsWith, [flag, dash]);
    let empty = body.literal(Value::string(""));
    let flag = body.local("flag");
    let length = body.intrinsic(Operation::StringUtf16Length, [flag]);
    let one = body.literal(Value::i64(1));
    let is_single = body.intrinsic(Operation::Equal, [length, one]);
    let short = body.literal(Value::string("-"));
    let short_block = body.block([], Some(short));
    let long = body.literal(Value::string("--"));
    let long_block = body.block([], Some(long));
    let inferred = body.if_else(is_single, short_block, long_block);
    let empty_block = body.block([], Some(empty));
    let inferred_block = body.block([], Some(inferred));
    body.if_else(already_prefixed, empty_block, inferred_block)
}

fn flag_position(body: &mut BodyBuilder<'_>) -> Expr {
    let prefix = flag_prefix(body);
    let flag = body.local("flag");
    let candidate = body.intrinsic(Operation::StringConcat, [prefix, flag]);
    let argv = body.local("argv");
    body.intrinsic(Operation::ListIndexOf, [argv, candidate])
}

fn terminator_position(body: &mut BodyBuilder<'_>) -> Expr {
    let argv = body.local("argv");
    let terminator = body.literal(Value::string("--"));
    body.intrinsic(Operation::ListIndexOf, [argv, terminator])
}

fn vectors() -> Vec<(&'static str, &'static str, Vec<&'static str>, bool)> {
    vec![
        (
            "official_long",
            "unicorn",
            vec!["--foo", "--unicorn", "--bar"],
            true,
        ),
        (
            "official_optional_prefix",
            "--unicorn",
            vec!["--foo", "--unicorn", "--bar"],
            true,
        ),
        (
            "official_equals",
            "unicorn=rainbow",
            vec!["--foo", "--unicorn=rainbow", "--bar"],
            true,
        ),
        (
            "official_before_terminator",
            "unicorn",
            vec!["--unicorn", "--", "--foo"],
            true,
        ),
        (
            "official_after_terminator",
            "unicorn",
            vec!["--foo", "--", "--unicorn"],
            false,
        ),
        ("official_absent", "unicorn", vec!["--foo"], false),
        (
            "official_short_prefixed",
            "-u",
            vec!["-f", "-u", "-b"],
            true,
        ),
        (
            "official_short_prefixed_before",
            "-u",
            vec!["-u", "--", "-f"],
            true,
        ),
        ("official_short_inferred", "u", vec!["-f", "-u", "-b"], true),
        (
            "official_short_inferred_before",
            "u",
            vec!["-u", "--", "-f"],
            true,
        ),
        ("official_short_after", "f", vec!["-u", "--", "-f"], false),
        ("empty_argv", "x", vec![], false),
        ("empty_flag_is_terminator", "", vec!["--"], false),
        ("duplicate_first_before", "x", vec!["-x", "--", "-x"], true),
        ("duplicates_after_only", "x", vec!["--", "-x", "-x"], false),
        ("terminator_first", "x", vec!["--", "-x"], false),
        ("dash_flag", "-", vec!["-", "--"], true),
        ("terminator_as_flag", "--", vec!["--"], false),
        ("astral_uses_two_units", "🦀", vec!["--🦀"], true),
        ("astral_rejects_short_prefix", "🦀", vec!["-🦀"], false),
        ("bmp_uses_one_unit", "é", vec!["-é"], true),
        ("combining_uses_two_units", "é", vec!["--é"], true),
        ("embedded_equals", "x=y=z", vec!["--x=y=z"], true),
        ("nul_is_portable_text", "\0", vec!["-\0"], true),
        ("similar_prefix_not_equal", "foo", vec!["--foobar"], false),
    ]
}

#[cfg(test)]
mod tests {
    use portable_eval::Evaluator;

    use super::*;

    #[test]
    fn all_twenty_five_vectors_pass_in_the_reference_evaluator() {
        let program = program();
        let results = Evaluator::new(&program).run_all_tests();
        assert_eq!(results.len(), 25);
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
