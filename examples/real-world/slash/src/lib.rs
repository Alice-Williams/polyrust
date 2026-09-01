#![forbid(unsafe_code)]

//! Complete typed-behavior port of `sindresorhus/slash` 5.1.0.

use std::sync::Arc;

use portable_backend_go::GoV0Backend;
use portable_backend_python::PythonBackend;
use portable_backend_rust::RustBackend;
use portable_backend_typescript::TypeScriptBackend;
use portable_build::{
    Expected, Invocation, ModuleBuilder, Operation, Parameter, Type, TypedValue, Value, Visibility,
};
use portable_check::v0::CheckedProgram;
use portable_codegen::{Backend, BackendOptions, OutputManifest};

/// Builds and checks the target-independent slash program.
pub fn program() -> CheckedProgram {
    let mut module = ModuleBuilder::new("slash");
    let slash = module.function(
        "slash",
        Visibility::Public,
        vec!["Convert non-extended Windows backslash paths to slash paths.".into()],
        |function| {
            function.parameter(Parameter::new("path", Type::string()));
            function.returns(Type::string());
            function.body(|body| {
                let path = body.local("path");
                let extended_prefix = body.literal(Value::string("\\\\?\\"));
                let is_extended =
                    body.intrinsic(Operation::StringStartsWith, [path, extended_prefix]);

                let unchanged = body.local("path");
                let unchanged = body.block([], Some(unchanged));

                let path = body.local("path");
                let backslash = body.literal(Value::string("\\"));
                let forward_slash = body.literal(Value::string("/"));
                let converted = body.intrinsic(
                    Operation::StringReplaceAll,
                    [path, backslash, forward_slash],
                );
                let converted = body.block([], Some(converted));

                let result = body.if_else(is_extended, unchanged, converted);
                body.block([], Some(result))
            });
        },
    );

    for (name, input, output) in vectors() {
        module.portable_test(
            name,
            Visibility::Package,
            vec![],
            Invocation::function(
                slash,
                [TypedValue::new(Type::string(), Value::string(input))],
            ),
            Expected::value(TypedValue::new(Type::string(), Value::string(output))),
        );
    }

    module
        .finish()
        .unwrap_or_else(|diagnostics| panic!("slash did not check: {diagnostics:#?}"))
}

/// Generates all four required target packages from one checked program.
pub fn manifests() -> Vec<(&'static str, OutputManifest)> {
    let program = program();
    let backends: [(&str, Arc<dyn Backend>); 4] = [
        ("rust", Arc::new(RustBackend)),
        ("typescript", Arc::new(TypeScriptBackend)),
        ("python", Arc::new(PythonBackend)),
        ("go", Arc::new(GoV0Backend)),
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

fn vectors() -> [(&'static str, &'static str, &'static str); 15] {
    [
        ("official_mixed", "c:/aaaa\\bbbb", "c:/aaaa/bbbb"),
        ("official_windows", "c:\\aaaa\\bbbb", "c:/aaaa/bbbb"),
        ("official_unicode", "c:\\aaaa\\bbbb\\★", "c:/aaaa/bbbb/★"),
        (
            "official_extended",
            "\\\\?\\c:\\aaaa\\bbbb",
            "\\\\?\\c:\\aaaa\\bbbb",
        ),
        ("empty", "", ""),
        ("forward_only", "c:/a/b", "c:/a/b"),
        ("single_backslash", "\\", "/"),
        ("repeated_backslashes", "a\\\\\\b", "a///b"),
        ("unc_not_extended", "\\\\server\\share", "//server/share"),
        ("near_miss_one_slash", "\\?\\c:\\x", "/?/c:/x"),
        ("near_miss_question", "\\\\x\\c:\\y", "//x/c:/y"),
        ("prefix_later", "x\\\\?\\c:\\y", "x//?/c:/y"),
        ("unicode_directory", "🦄\\🐐", "🦄/🐐"),
        ("newline_is_preserved", "a\\b\nc\\d", "a/b\nc/d"),
        (
            "extended_unicode_unchanged",
            "\\\\?\\🦄\\🐐",
            "\\\\?\\🦄\\🐐",
        ),
    ]
}

#[cfg(test)]
mod tests {
    use portable_eval::Evaluator;

    use super::*;

    #[test]
    fn all_fifteen_vectors_pass_in_the_reference_evaluator() {
        let program = program();
        let results = Evaluator::new(&program).run_all_tests();
        assert_eq!(results.len(), 15);
        assert!(results.iter().all(|result| result.passed), "{results:#?}");
    }

    #[test]
    fn all_four_manifests_are_nonempty_and_repeatable() {
        let first = manifests();
        let second = manifests();
        assert_eq!(first, second);
        assert_eq!(first.len(), 4);
        assert!(
            first
                .iter()
                .all(|(_, manifest)| !manifest.files().is_empty())
        );
    }
}
