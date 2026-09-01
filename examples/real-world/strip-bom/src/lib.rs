#![forbid(unsafe_code)]

//! Complete typed-behavior port of `sindresorhus/strip-bom` 5.0.0.

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

/// Builds and checks the target-independent strip-bom program.
pub fn program() -> CheckedProgram {
    let mut module = ModuleBuilder::new("strip_bom");
    let strip_bom = module.function(
        "strip_bom",
        Visibility::Public,
        vec!["Remove exactly one leading Unicode byte-order mark.".into()],
        |function| {
            function.parameter(Parameter::new("string", Type::string()));
            function.returns(Type::string());
            function.body(|body| {
                let string = body.local("string");
                let bom = body.literal(Value::string("\u{feff}"));
                let output = body.intrinsic(Operation::StringStripPrefix, [string, bom]);
                body.block([], Some(output))
            });
        },
    );

    for (name, input, output) in vectors() {
        module.portable_test(
            name,
            Visibility::Package,
            vec![],
            Invocation::function(
                strip_bom,
                [TypedValue::new(Type::string(), Value::string(input))],
            ),
            Expected::value(TypedValue::new(Type::string(), Value::string(output))),
        );
    }

    module
        .finish()
        .unwrap_or_else(|diagnostics| panic!("strip-bom did not check: {diagnostics:#?}"))
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

fn vectors() -> [(&'static str, &'static str, &'static str); 18] {
    [
        ("official_utf8_fixture", "\u{feff}Unicorn\n", "Unicorn\n"),
        (
            "official_middle_fixture",
            "Unicorn \u{feff}Unicorn\n",
            "Unicorn \u{feff}Unicorn\n",
        ),
        ("empty", "", ""),
        ("bom_only", "\u{feff}", ""),
        ("double_bom", "\u{feff}\u{feff}", "\u{feff}"),
        ("ordinary_ascii", "unicorn", "unicorn"),
        ("bom_in_middle", "uni\u{feff}corn", "uni\u{feff}corn"),
        ("bom_at_end", "unicorn\u{feff}", "unicorn\u{feff}"),
        ("astral_after_bom", "\u{feff}🦄", "🦄"),
        ("astral_before_bom", "🦄\u{feff}", "🦄\u{feff}"),
        ("combining_after_bom", "\u{feff}e\u{301}", "e\u{301}"),
        ("newline_after_bom", "\u{feff}\r\n", "\r\n"),
        ("nul_after_bom", "\u{feff}\0value", "\0value"),
        ("replacement_character", "\u{fffd}value", "\u{fffd}value"),
        ("reverse_bom", "\u{fffe}value", "\u{fffe}value"),
        ("word_joiner", "\u{2060}value", "\u{2060}value"),
        ("space_before_bom", " \u{feff}value", " \u{feff}value"),
        ("bom_then_slash", "\u{feff}\\/value", "\\/value"),
    ]
}

#[cfg(test)]
mod tests {
    use portable_eval::Evaluator;

    use super::*;

    #[test]
    fn all_eighteen_vectors_pass_in_the_reference_evaluator() {
        let program = program();
        let results = Evaluator::new(&program).run_all_tests();
        assert_eq!(results.len(), 18);
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
