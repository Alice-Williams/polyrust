#![forbid(unsafe_code)]

//! Complete string-separator port of `sindresorhus/split-on-first` 3.0.0.

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

/// Builds `split_on_first(input, separator)` and the official plus boundary vectors.
pub fn program() -> CheckedProgram {
    let mut module = ModuleBuilder::new("split_on_first");
    let split_on_first = module.function(
        "split_on_first",
        Visibility::Public,
        vec!["Split around the leftmost non-empty literal separator.".into()],
        |function| {
            function.parameter(Parameter::new("input", Type::string()));
            function.parameter(Parameter::new("separator", Type::string()));
            function.returns(Type::list(Type::string()));
            function.body(|body| {
                let input = body.local("input");
                let input_empty = body.intrinsic(Operation::StringIsEmpty, [input]);
                let separator = body.local("separator");
                let separator_empty = body.intrinsic(Operation::StringIsEmpty, [separator]);
                let empty_operand =
                    body.intrinsic(Operation::BoolOr, [input_empty, separator_empty]);

                let index = string_index(body);
                let found = body.intrinsic(Operation::OptionIsSome, [index]);
                let split = split_result(body);
                let split_block = body.block([], Some(split));
                let absent = empty_result(body);
                let absent_block = body.block([], Some(absent));
                let searched = body.if_else(found, split_block, absent_block);

                let empty = empty_result(body);
                let empty_block = body.block([], Some(empty));
                let searched_block = body.block([], Some(searched));
                let result = body.if_else(empty_operand, empty_block, searched_block);
                body.block([], Some(result))
            });
        },
    );

    for (name, input, separator, expected) in vectors() {
        module.portable_test(
            name,
            Visibility::Package,
            vec![],
            Invocation::function(
                split_on_first,
                [
                    TypedValue::new(Type::string(), Value::string(input)),
                    TypedValue::new(Type::string(), Value::string(separator)),
                ],
            ),
            Expected::value(TypedValue::new(
                Type::list(Type::string()),
                Value::list(expected.into_iter().map(Value::string)),
            )),
        );
    }

    module
        .finish()
        .unwrap_or_else(|diagnostics| panic!("split-on-first did not check: {diagnostics:#?}"))
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

fn string_index(body: &mut BodyBuilder<'_>) -> Expr {
    let input = body.local("input");
    let separator = body.local("separator");
    body.intrinsic(Operation::StringIndexOfLiteral, [input, separator])
}

fn split_result(body: &mut BodyBuilder<'_>) -> Expr {
    let input = body.local("input");
    let zero = body.literal(Value::i64(0));
    let prefix_end = unwrapped_index(body);
    let prefix = body.intrinsic(Operation::StringSliceScalars, [input, zero, prefix_end]);

    let index = unwrapped_index(body);
    let input = body.local("input");
    let end = body.literal(Value::i64(i64::MAX));
    let matched_tail = body.intrinsic(Operation::StringSliceScalars, [input, index, end]);
    let separator = body.local("separator");
    let suffix = body.intrinsic(Operation::StringStripPrefix, [matched_tail, separator]);
    body.list(Type::string(), [prefix, suffix])
}

fn unwrapped_index(body: &mut BodyBuilder<'_>) -> Expr {
    let index = string_index(body);
    let fallback = body.literal(Value::i64(0));
    body.intrinsic(Operation::OptionUnwrapOr, [index, fallback])
}

fn empty_result(body: &mut BodyBuilder<'_>) -> Expr {
    body.list(Type::string(), [])
}

type Vector = (&'static str, &'static str, &'static str, Vec<&'static str>);

fn vectors() -> Vec<Vector> {
    vec![
        ("official_hyphen", "a-b-c", "-", vec!["a", "b-c"]),
        (
            "official_colon",
            "key:value:value2",
            ":",
            vec!["key", "value:value2"],
        ),
        ("official_multi", "a---b---c", "---", vec!["a", "b---c"]),
        ("official_absent", "a-b-c", "+", vec![]),
        ("official_empty_separator", "abc", "", vec![]),
        ("official_both_empty", "", "", vec![]),
        ("empty_input", "", "-", vec![]),
        ("separator_at_start", "-abc", "-", vec!["", "abc"]),
        ("separator_at_end", "abc-", "-", vec!["abc", ""]),
        ("separator_is_input", "abc", "abc", vec!["", ""]),
        ("adjacent_separators", "a--b", "-", vec!["a", "-b"]),
        ("overlap_uses_first", "aaaa", "aa", vec!["", "aa"]),
        (
            "repeated_multi_separator",
            "a::b::c",
            "::",
            vec!["a", "b::c"],
        ),
        ("separator_longer", "ab", "abc", vec![]),
        ("case_sensitive", "Alpha", "a", vec!["Alph", ""]),
        (
            "space_separator",
            "hello world test",
            " ",
            vec!["hello", "world test"],
        ),
        ("tab_separator", "a\tb\tc", "\t", vec!["a", "b\tc"]),
        ("crlf_separator", "a\r\nb\r\nc", "\r\n", vec!["a", "b\r\nc"]),
        ("newline_first", "\na\n", "\n", vec!["", "a\n"]),
        ("nul_separator", "a\0b\0c", "\0", vec!["a", "b\0c"]),
        ("astral_before_match", "🦀a🦀b", "🦀b", vec!["🦀a", ""]),
        ("astral_separator", "a🦀b🦀c", "🦀", vec!["a", "b🦀c"]),
        ("astral_entire_input", "🦀", "🦀", vec!["", ""]),
        ("bmp_separator", "aébéc", "é", vec!["a", "béc"]),
        ("combining_scalar", "éx́y", "́", vec!["e", "x́y"]),
        ("canonical_forms_differ", "éx", "é", vec![]),
        ("adjacent_emoji", "🦀🚀🦀", "🚀", vec!["🦀", "🦀"]),
        (
            "multi_scalar_unicode",
            "a🦀🚀b🦀🚀c",
            "🦀🚀",
            vec!["a", "b🦀🚀c"],
        ),
        ("path_separator", "a/b/c", "/", vec!["a", "b/c"]),
        (
            "equals_separator",
            "key=value=tail",
            "=",
            vec!["key", "value=tail"],
        ),
        ("quote_separator", "a\"b\"c", "\"", vec!["a", "b\"c"]),
        ("astral_absent_from_empty", "", "🦀", vec![]),
    ]
}

#[cfg(test)]
mod tests {
    use portable_codegen::OutputContents;
    use portable_eval::Evaluator;

    use super::*;

    #[test]
    fn all_thirty_two_vectors_pass_in_the_reference_evaluator() {
        let program = program();
        let results = Evaluator::new(&program).run_all_tests();
        assert_eq!(results.len(), 32);
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

    #[test]
    fn c_list_construction_is_owned_and_failure_aware() {
        let manifests = manifests();
        let c = manifests
            .iter()
            .find(|(target, _)| *target == "c")
            .map(|(_, manifest)| manifest)
            .expect("C manifest");
        let source = match c
            .file("src/generated.c")
            .expect("generated C source")
            .contents()
        {
            OutputContents::Text(text) => text,
            OutputContents::Bytes(_) => panic!("generated C source must be text"),
        };
        assert!(source.contains(".data = allocator.allocate"));
        assert!(source.contains("poly_string_clone(allocator"));
        assert!(source.contains(".length = 2U"));
        assert!(source.contains("_list__string_drop(&temporary_"));
        assert!(source.contains("POLY_ALLOCATION_FAILED"));
    }
}
