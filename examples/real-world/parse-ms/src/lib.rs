#![forbid(unsafe_code)]

//! Complete representable typed-behavior port of `parse-ms` 3.0.0.

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

/// Builds and checks the target-independent parse-ms program.
pub fn program() -> CheckedProgram {
    let mut module = ModuleBuilder::new("parse_ms");
    let (time_components, fields) = module.record(
        "TimeComponents",
        Visibility::Public,
        vec!["A millisecond duration split into truncation-based components.".into()],
        |record| {
            [
                record.field("days", Type::f64(), vec![]),
                record.field("hours", Type::f64(), vec![]),
                record.field("minutes", Type::f64(), vec![]),
                record.field("seconds", Type::f64(), vec![]),
                record.field("milliseconds", Type::f64(), vec![]),
                record.field("microseconds", Type::f64(), vec![]),
                record.field("nanoseconds", Type::f64(), vec![]),
            ]
        },
    );
    let [
        days,
        hours,
        minutes,
        seconds,
        milliseconds,
        microseconds,
        nanoseconds,
    ] = fields;
    let parse_milliseconds = module.function(
        "parse_milliseconds",
        Visibility::Public,
        vec!["Split a floating-point millisecond duration exactly as parse-ms v3.".into()],
        |function| {
            function.parameter(Parameter::new("milliseconds", Type::f64()));
            function.returns(Type::named(time_components));
            function.body(|body| {
                let days_value = truncated_division(body, 86_400_000.0);
                let hours_value = truncated_division_remainder(body, 3_600_000.0, 24.0);
                let minutes_value = truncated_division_remainder(body, 60_000.0, 60.0);
                let seconds_value = truncated_division_remainder(body, 1_000.0, 60.0);
                let milliseconds_value = truncated_scale_remainder(body, 1.0);
                let microseconds_value = truncated_scale_remainder(body, 1_000.0);
                let nanoseconds_value = truncated_scale_remainder(body, 1_000_000.0);
                let result = body.record(
                    time_components,
                    [
                        (days, days_value),
                        (hours, hours_value),
                        (minutes, minutes_value),
                        (seconds, seconds_value),
                        (milliseconds, milliseconds_value),
                        (microseconds, microseconds_value),
                        (nanoseconds, nanoseconds_value),
                    ],
                );
                body.block([], Some(result))
            });
        },
    );

    for (name, input) in vectors() {
        let values = parse_reference(input);
        module.portable_test(
            format!("parse_{name}"),
            Visibility::Package,
            vec![],
            Invocation::function(
                parse_milliseconds,
                [TypedValue::new(Type::f64(), Value::f64(input))],
            ),
            Expected::value(TypedValue::new(
                Type::named(time_components),
                Value::record(
                    time_components,
                    [
                        (days, Value::f64(values[0])),
                        (hours, Value::f64(values[1])),
                        (minutes, Value::f64(values[2])),
                        (seconds, Value::f64(values[3])),
                        (milliseconds, Value::f64(values[4])),
                        (microseconds, Value::f64(values[5])),
                        (nanoseconds, Value::f64(values[6])),
                    ],
                ),
            )),
        );
    }

    module
        .finish()
        .unwrap_or_else(|diagnostics| panic!("parse-ms did not check: {diagnostics:#?}"))
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

fn truncated_division(body: &mut BodyBuilder<'_>, divisor: f64) -> Expr {
    let milliseconds = body.local("milliseconds");
    let divisor = body.literal(Value::f64(divisor));
    let divided = body.intrinsic(Operation::FloatDiv, [milliseconds, divisor]);
    body.intrinsic(Operation::FloatTrunc, [divided])
}

fn truncated_division_remainder(body: &mut BodyBuilder<'_>, divisor: f64, modulus: f64) -> Expr {
    let divided = truncated_division(body, divisor);
    let modulus = body.literal(Value::f64(modulus));
    body.intrinsic(Operation::FloatRemTrunc, [divided, modulus])
}

fn truncated_scale_remainder(body: &mut BodyBuilder<'_>, scale: f64) -> Expr {
    let milliseconds = body.local("milliseconds");
    let scale = body.literal(Value::f64(scale));
    let scaled = body.intrinsic(Operation::FloatMul, [milliseconds, scale]);
    let truncated = body.intrinsic(Operation::FloatTrunc, [scaled]);
    let modulus = body.literal(Value::f64(1_000.0));
    body.intrinsic(Operation::FloatRemTrunc, [truncated, modulus])
}

fn parse_reference(milliseconds: f64) -> [f64; 7] {
    [
        (milliseconds / 86_400_000.0).trunc(),
        (milliseconds / 3_600_000.0).trunc() % 24.0,
        (milliseconds / 60_000.0).trunc() % 60.0,
        (milliseconds / 1_000.0).trunc() % 60.0,
        milliseconds.trunc() % 1_000.0,
        (milliseconds * 1_000.0).trunc() % 1_000.0,
        (milliseconds * 1_000_000.0).trunc() % 1_000.0,
    ]
}

fn vectors() -> Vec<(&'static str, f64)> {
    vec![
        ("official_1400", 1_400.0),
        ("official_55_seconds", 55_000.0),
        ("official_67_seconds", 67_000.0),
        ("official_5_minutes", 300_000.0),
        ("official_67_minutes", 4_020_000.0),
        ("official_12_hours", 43_200_000.0),
        ("official_40_hours", 144_000_000.0),
        ("official_999_hours", 3_596_400_000.0),
        ("official_fractional", 60_500.345_678),
        ("official_nanoseconds", 0.000_543),
        ("negative_sub_microsecond", -0.000_5),
        ("negative_fraction", -0.3),
        ("negative_500", -500.0),
        ("negative_55_seconds", -55_000.0),
        ("negative_67_seconds", -67_000.0),
        ("negative_5_minutes", -300_000.0),
        ("negative_67_minutes", -4_020_000.0),
        ("negative_12_hours", -43_200_000.0),
        ("negative_40_hours", -144_000_000.0),
        ("negative_999_hours", -3_596_400_000.0),
        ("positive_zero", 0.0),
        ("negative_zero", -0.0),
        ("nan", f64::NAN),
        ("positive_infinity", f64::INFINITY),
        ("negative_infinity", f64::NEG_INFINITY),
        ("below_one", 0.999_999),
        ("negative_below_one", -0.999_999),
        ("below_second", 999.999_999),
        ("negative_below_second", -999.999_999),
        ("maximum", f64::MAX),
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
