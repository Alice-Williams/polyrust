#![forbid(unsafe_code)]

//! Complete typed-behavior port of stdlib `is-negative-zero` 0.2.3.

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

const NEGATIVE_ZERO_BITS: u64 = 0x8000_0000_0000_0000;

const VECTORS: &[(&str, u64)] = &[
    ("official_negative_zero", NEGATIVE_ZERO_BITS),
    ("official_positive_zero", 0x0000_0000_0000_0000),
    ("official_five", 0x4014_0000_0000_0000),
    ("official_negative_one", 0xbff0_0000_0000_0000),
    ("official_nan", 0x7ff8_0000_0000_0000),
    ("declaration_two", 0x4000_0000_0000_0000),
    ("positive_min_subnormal", 0x0000_0000_0000_0001),
    ("negative_min_subnormal", 0x8000_0000_0000_0001),
    ("positive_max_subnormal", 0x000f_ffff_ffff_ffff),
    ("negative_max_subnormal", 0x800f_ffff_ffff_ffff),
    ("positive_min_normal", 0x0010_0000_0000_0000),
    ("negative_min_normal", 0x8010_0000_0000_0000),
    ("positive_max_finite", 0x7fef_ffff_ffff_ffff),
    ("negative_max_finite", 0xffef_ffff_ffff_ffff),
    ("positive_infinity", 0x7ff0_0000_0000_0000),
    ("negative_infinity", 0xfff0_0000_0000_0000),
    ("positive_signaling_nan", 0x7ff0_0000_0000_0001),
    ("positive_quiet_nan", 0x7ff8_0000_0000_0001),
    ("positive_max_nan", 0x7fff_ffff_ffff_ffff),
    ("negative_signaling_nan", 0xfff0_0000_0000_0001),
    ("negative_quiet_nan", 0xfff8_0000_0000_0001),
    ("negative_max_nan", 0xffff_ffff_ffff_ffff),
];

/// Builds and checks the complete declared numeric API.
pub fn program() -> CheckedProgram {
    let mut module = ModuleBuilder::new("stdlib_is_negative_zero");
    let predicate = module.function(
        "is_negative_zero",
        Visibility::Public,
        vec![
            "Return true only for the IEEE-754 binary64 value whose raw bits are 0x8000000000000000."
                .into(),
        ],
        |function| {
            function.parameter(Parameter::new("value", Type::f64()));
            function.returns(Type::bool());
            function.body(|body| {
                let value = body.local("value");
                let result = body.intrinsic(Operation::FloatIsNegativeZero, [value]);
                body.block([], Some(result))
            });
        },
    );

    for &(name, bits) in VECTORS {
        module.portable_test(
            format!("classify_{name}"),
            Visibility::Package,
            vec![],
            Invocation::function(
                predicate,
                [TypedValue::new(Type::f64(), Value::f64_bits(bits))],
            ),
            Expected::value(TypedValue::new(
                Type::bool(),
                Value::bool(bits == NEGATIVE_ZERO_BITS),
            )),
        );
    }

    module.finish().unwrap_or_else(|diagnostics| {
        panic!("stdlib is-negative-zero did not check: {diagnostics:#?}")
    })
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

#[cfg(test)]
mod tests {
    use portable_eval::Evaluator;

    use super::*;

    #[test]
    fn all_twenty_two_exact_bit_vectors_pass_in_the_reference_evaluator() {
        let results = Evaluator::new(&program()).run_all_tests();
        assert_eq!(results.len(), 22);
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
