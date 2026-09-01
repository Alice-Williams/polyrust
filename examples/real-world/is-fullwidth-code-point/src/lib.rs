#![forbid(unsafe_code)]

//! Complete typed-behavior port of `is-fullwidth-code-point` 3.0.0.

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

const ACCEPTED_INTERVALS: [(u32, u32); 15] = [
    (0x1100, 0x115F),
    (0x2329, 0x232A),
    (0x2E80, 0x3247),
    (0x3250, 0x4DBF),
    (0x4E00, 0xA4C6),
    (0xA960, 0xA97C),
    (0xAC00, 0xD7A3),
    (0xF900, 0xFAFF),
    (0xFE10, 0xFE19),
    (0xFE30, 0xFE6B),
    (0xFF01, 0xFF60),
    (0xFFE0, 0xFFE6),
    (0x1B000, 0x1B001),
    (0x1F200, 0x1F251),
    (0x20000, 0x3FFFD),
];

/// Builds and checks the target-independent version-3 classifier.
pub fn program() -> CheckedProgram {
    let mut module = ModuleBuilder::new("is_fullwidth_code_point");
    let classifier = module.function(
        "is_fullwidth_code_point",
        Visibility::Public,
        vec![
            "Return whether a JavaScript-number code point is fullwidth under the pinned v3 table."
                .into(),
        ],
        |function| {
            function.parameter(Parameter::new("code_point", Type::f64()));
            function.returns(Type::bool());
            function.body(|body| {
                let is_nan = unary(body, Operation::FloatIsNaN);
                let not_nan = body.intrinsic(Operation::BoolNot, [is_nan]);
                let at_or_above_first = compare(body, Operation::GreaterEqual, 0x1100 as f64);

                let first_interval = between(body, 0x1100, 0x115F);
                let left_angle = compare(body, Operation::Equal, 0x2329 as f64);
                let right_angle = compare(body, Operation::Equal, 0x232A as f64);
                let cjk_with_hole = between(body, 0x2E80, 0x3247);
                let not_hole = compare(body, Operation::NotEqual, 0x303F as f64);
                let cjk_with_hole = and_all(body, [cjk_with_hole, not_hole]);
                let remaining = [
                    (0x3250, 0x4DBF),
                    (0x4E00, 0xA4C6),
                    (0xA960, 0xA97C),
                    (0xAC00, 0xD7A3),
                    (0xF900, 0xFAFF),
                    (0xFE10, 0xFE19),
                    (0xFE30, 0xFE6B),
                    (0xFF01, 0xFF60),
                    (0xFFE0, 0xFFE6),
                    (0x1B000, 0x1B001),
                    (0x1F200, 0x1F251),
                    (0x20000, 0x3FFFD),
                ]
                .map(|(start, end)| between(body, start, end));
                let accepted = or_all(
                    body,
                    [first_interval, left_angle, right_angle, cjk_with_hole]
                        .into_iter()
                        .chain(remaining),
                );
                let result = and_all(body, [not_nan, at_or_above_first, accepted]);
                body.block([], Some(result))
            });
        },
    );

    for (name, input) in vectors() {
        module.portable_test(
            format!("classify_{name}"),
            Visibility::Package,
            vec![],
            Invocation::function(
                classifier,
                [TypedValue::new(Type::f64(), Value::f64(input))],
            ),
            Expected::value(TypedValue::new(
                Type::bool(),
                Value::bool(is_fullwidth_reference(input)),
            )),
        );
    }

    module.finish().unwrap_or_else(|diagnostics| {
        panic!("is-fullwidth-code-point did not check: {diagnostics:#?}")
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

fn unary(body: &mut BodyBuilder<'_>, operation: Operation) -> Expr {
    let code_point = body.local("code_point");
    body.intrinsic(operation, [code_point])
}

fn compare(body: &mut BodyBuilder<'_>, operation: Operation, threshold: f64) -> Expr {
    let code_point = body.local("code_point");
    let threshold = body.literal(Value::f64(threshold));
    body.intrinsic(operation, [code_point, threshold])
}

fn between(body: &mut BodyBuilder<'_>, start: u32, end: u32) -> Expr {
    let lower = compare(body, Operation::GreaterEqual, start as f64);
    let upper = compare(body, Operation::LessEqual, end as f64);
    and_all(body, [lower, upper])
}

fn and_all(body: &mut BodyBuilder<'_>, expressions: impl IntoIterator<Item = Expr>) -> Expr {
    combine(body, Operation::BoolAnd, expressions)
}

fn or_all(body: &mut BodyBuilder<'_>, expressions: impl IntoIterator<Item = Expr>) -> Expr {
    combine(body, Operation::BoolOr, expressions)
}

fn combine(
    body: &mut BodyBuilder<'_>,
    operation: Operation,
    expressions: impl IntoIterator<Item = Expr>,
) -> Expr {
    let mut expressions = expressions.into_iter();
    let mut result = expressions.next().expect("range expression is nonempty");
    for expression in expressions {
        result = body.intrinsic(operation, [result, expression]);
    }
    result
}

fn is_fullwidth_reference(code_point: f64) -> bool {
    if code_point.is_nan() {
        return false;
    }
    code_point >= 0x1100 as f64
        && (code_point <= 0x115F as f64
            || code_point == 0x2329 as f64
            || code_point == 0x232A as f64
            || ((0x2E80 as f64..=0x3247 as f64).contains(&code_point)
                && code_point != 0x303F as f64)
            || (0x3250 as f64..=0x4DBF as f64).contains(&code_point)
            || (0x4E00 as f64..=0xA4C6 as f64).contains(&code_point)
            || (0xA960 as f64..=0xA97C as f64).contains(&code_point)
            || (0xAC00 as f64..=0xD7A3 as f64).contains(&code_point)
            || (0xF900 as f64..=0xFAFF as f64).contains(&code_point)
            || (0xFE10 as f64..=0xFE19 as f64).contains(&code_point)
            || (0xFE30 as f64..=0xFE6B as f64).contains(&code_point)
            || (0xFF01 as f64..=0xFF60 as f64).contains(&code_point)
            || (0xFFE0 as f64..=0xFFE6 as f64).contains(&code_point)
            || (0x1B000 as f64..=0x1B001 as f64).contains(&code_point)
            || (0x1F200 as f64..=0x1F251 as f64).contains(&code_point)
            || (0x20000 as f64..=0x3FFFD as f64).contains(&code_point))
}

fn vectors() -> Vec<(String, f64)> {
    let mut vectors = vec![
        ("official_hiragana".into(), 0x3042 as f64),
        ("official_cjk".into(), 0x8C22 as f64),
        ("official_hangul".into(), 0xACE0 as f64),
        ("official_nan".into(), f64::NAN),
        ("official_ascii".into(), 0x61 as f64),
        ("official_enclosed_ideograph".into(), 0x1F251 as f64),
    ];
    for (index, (start, end)) in ACCEPTED_INTERVALS.into_iter().enumerate() {
        vectors.extend([
            (
                format!("range_{index:02}_below"),
                start.saturating_sub(1) as f64,
            ),
            (format!("range_{index:02}_start"), start as f64),
            (format!("range_{index:02}_end"), end as f64),
            (
                format!("range_{index:02}_above"),
                end.saturating_add(1) as f64,
            ),
        ]);
    }
    vectors.extend([
        ("hole_before".into(), 0x303E as f64),
        ("hole_exact".into(), 0x303F as f64),
        ("hole_after".into(), 0x3040 as f64),
        ("negative_infinity".into(), f64::NEG_INFINITY),
        ("negative_maximum".into(), -f64::MAX),
        ("negative_one".into(), -1.0),
        ("negative_zero".into(), -0.0),
        ("positive_zero".into(), 0.0),
        ("smallest_positive_subnormal".into(), f64::from_bits(1)),
        ("smallest_negative_subnormal".into(), -f64::from_bits(1)),
        ("smallest_positive_normal".into(), f64::MIN_POSITIVE),
        ("below_first_fraction".into(), 0x10FF as f64 + 0.5),
        ("inside_first_fraction".into(), 0x1100 as f64 + 0.5),
        ("after_first_fraction".into(), 0x115F as f64 + 0.5),
        ("before_hole_fraction".into(), 0x303E as f64 + 0.5),
        ("hole_fraction".into(), 0x303F as f64 + 0.5),
        ("after_last_fraction".into(), 0x3FFFD as f64 + 0.5),
        ("unicode_maximum".into(), 0x10FFFF as f64),
        ("above_unicode_maximum".into(), 0x110000 as f64),
        ("maximum_finite".into(), f64::MAX),
        ("positive_infinity".into(), f64::INFINITY),
        (
            "positive_signaling_nan".into(),
            f64::from_bits(0x7ff0_0000_0000_0001),
        ),
        (
            "negative_quiet_nan".into(),
            f64::from_bits(0xfff8_0000_0000_0001),
        ),
    ]);
    vectors
}

#[cfg(test)]
mod tests {
    use portable_eval::Evaluator;

    use super::*;

    #[test]
    fn all_eighty_nine_vectors_pass_in_the_reference_evaluator() {
        let program = program();
        let results = Evaluator::new(&program).run_all_tests();
        assert_eq!(results.len(), 89);
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
