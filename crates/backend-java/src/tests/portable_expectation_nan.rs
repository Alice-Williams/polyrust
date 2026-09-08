//! Evaluator/native parity for NaN-class expectations, not IEEE equality.

use super::totality_oracle::CompiledPackage;
use crate::JavaBackend;
use portable_build::{
    Expected, Invocation, ModuleBuilder, Operation, Parameter, Type, TypedValue, Value, Visibility,
};
use portable_check::v0::CheckedProgram;
use portable_codegen::{Backend, BackendOptions};
use portable_eval::Evaluator;

const ACTUAL_NAN: u64 = 0x7ff8_0000_0000_0001;
const EXPECTED_NAN: u64 = 0xfff8_0000_0000_0022;

fn add_case(module: &mut ModuleBuilder, name: &str, ty: Type, actual: Value, expected: Value) {
    let function = module.function(
        format!("read_{name}"),
        Visibility::Public,
        vec![],
        |function| {
            function.parameter(Parameter::new("value", ty.clone()));
            function.returns(ty.clone());
            function.body(|body| {
                let value = body.local("value");
                body.block([], Some(value))
            });
        },
    );
    module.portable_test(
        name,
        Visibility::Package,
        vec![],
        Invocation::function(function, [TypedValue::new(ty.clone(), actual)]),
        Expected::value(TypedValue::new(ty, expected)),
    );
}

fn fixture() -> CheckedProgram {
    let mut module = ModuleBuilder::new("nan_expectations");
    let actual = Value::f64_bits(ACTUAL_NAN);
    let expected = Value::f64_bits(EXPECTED_NAN);
    add_case(
        &mut module,
        "scalar",
        Type::f64(),
        actual.clone(),
        expected.clone(),
    );
    add_case(
        &mut module,
        "list",
        Type::list(Type::f64()),
        Value::list([actual.clone()]),
        Value::list([expected.clone()]),
    );
    add_case(
        &mut module,
        "option",
        Type::option(Type::f64()),
        Value::some(actual.clone()),
        Value::some(expected.clone()),
    );
    add_case(
        &mut module,
        "ok",
        Type::result(Type::f64(), Type::i32()),
        Value::ok(actual.clone()),
        Value::ok(expected.clone()),
    );
    add_case(
        &mut module,
        "err",
        Type::result(Type::i32(), Type::f64()),
        Value::err(actual.clone()),
        Value::err(expected.clone()),
    );
    let (record, field) = module.record("Holder", Visibility::Public, vec![], |record| {
        record.field("value", Type::option(Type::list(Type::f64())), vec![])
    });
    add_case(
        &mut module,
        "record",
        Type::named(record),
        Value::record(
            record,
            [(field, Value::some(Value::list([actual.clone()])))],
        ),
        Value::record(
            record,
            [(field, Value::some(Value::list([expected.clone()])))],
        ),
    );
    let (enumeration, (variant, field)) =
        module.enumeration("Choice", Visibility::Public, vec![], |enumeration| {
            enumeration.variant("Float", vec![], |variant| {
                variant.field("value", Type::f64(), vec![])
            })
        });
    add_case(
        &mut module,
        "payload_enum",
        Type::named(enumeration),
        Value::enumeration(enumeration, variant, [(field, actual)]),
        Value::enumeration(enumeration, variant, [(field, expected)]),
    );
    module.finish().expect("NaN expectation fixture checks")
}

#[test]
fn portable_expectation_nan_class_matches_in_both_native_entry_points() {
    let checked = fixture();
    let outcomes = Evaluator::new(&checked).run_all_tests();
    assert_eq!(outcomes.len(), 7);
    assert!(outcomes.iter().all(|outcome| outcome.passed));
    let manifest = JavaBackend
        .generate(&checked, &BackendOptions::default())
        .unwrap();
    let compiled = CompiledPackage::new(&manifest, "portable-nan-class");
    compiled.compile_harnesses(&manifest);
    let mut failures = Vec::new();
    for name in ["GeneratedTest", "ConformanceTest"] {
        let source = super::generated_text(
            &manifest,
            &format!("src/test/java/org/polyrust/generated/{name}.java"),
        );
        assert!(source.contains("portable conformance inventory mismatch: expected 7 tests"));
        let output = compiled.run_harness(name);
        if !output.status.success() {
            failures.push(format!(
                "{name}: {}",
                String::from_utf8_lossy(&output.stderr)
            ));
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

#[test]
fn portable_expectation_nan_negative_controls_fail_in_both_entry_points() {
    for (name, actual, expected) in [
        ("finite", 1.0_f64.to_bits(), 2.0_f64.to_bits()),
        ("nan_finite", ACTUAL_NAN, 1.0_f64.to_bits()),
        ("finite_nan", 1.0_f64.to_bits(), EXPECTED_NAN),
        ("signed_zero", 0.0_f64.to_bits(), (-0.0_f64).to_bits()),
    ] {
        let mut module = ModuleBuilder::new("negative_expectations");
        add_case(
            &mut module,
            name,
            Type::f64(),
            Value::f64_bits(actual),
            Value::f64_bits(expected),
        );
        let checked = module.finish().unwrap();
        let outcomes = Evaluator::new(&checked).run_all_tests();
        assert_eq!(outcomes.len(), 1);
        assert!(!outcomes[0].passed, "{name}");
        let manifest = JavaBackend
            .generate(&checked, &BackendOptions::default())
            .unwrap();
        let compiled = CompiledPackage::new(&manifest, name);
        compiled.compile_harnesses(&manifest);
        for entry in ["GeneratedTest", "ConformanceTest"] {
            let output = compiled.run_harness(entry);
            let stderr = String::from_utf8_lossy(&output.stderr);
            assert!(
                !output.status.success(),
                "{name}/{entry} unexpectedly passed"
            );
            assert!(
                stderr.contains(&format!("portable test 0 ({name}) value mismatch")),
                "{name}/{entry}: {stderr}"
            );
        }
    }
}

#[test]
fn portable_expectation_nan_does_not_weaken_semantics_or_raw_bit_audits() {
    let mut module = ModuleBuilder::new("float_representation");
    module.function("equal", Visibility::Public, vec![], |function| {
        function.parameter(Parameter::new("left", Type::f64()));
        function.parameter(Parameter::new("right", Type::f64()));
        function.returns(Type::bool());
        function.body(|body| {
            let left = body.local("left");
            let right = body.local("right");
            let equal = body.intrinsic(Operation::Equal, [left, right]);
            body.block([], Some(equal))
        });
    });
    module.function("literal", Visibility::Public, vec![], |function| {
        function.returns(Type::f64());
        function.body(|body| {
            let value = body.literal(Value::f64_bits(EXPECTED_NAN));
            body.block([], Some(value))
        });
    });
    module.function("absolute", Visibility::Public, vec![], |function| {
        function.parameter(Parameter::new("value", Type::f64()));
        function.returns(Type::f64());
        function.body(|body| {
            let value = body.local("value");
            let absolute = body.intrinsic(Operation::FloatAbs, [value]);
            body.block([], Some(absolute))
        });
    });
    let checked = module.finish().unwrap();
    let manifest = JavaBackend
        .generate(&checked, &BackendOptions::default())
        .unwrap();
    CompiledPackage::new(&manifest, "float-representation").consumer(r#"
package org.polyrust.consumer;
import org.polyrust.generated.Generated;
public final class Consumer {
    private Consumer() {}
    public static void main(String[] args) {
        double left = Double.longBitsToDouble(0x7ff8000000000001L);
        double right = Double.longBitsToDouble(0xfff8000000000022L);
        if (Generated.equal(left, left).value() || Generated.equal(left, right).value()
            || !Generated.equal(0.0, -0.0).value()) {
            throw new AssertionError("IEEE equality changed");
        }
        if (Double.doubleToRawLongBits(left) == Double.doubleToRawLongBits(right)) {
            throw new AssertionError("raw-bit negative control accepted distinct NaNs");
        }
        if (Double.doubleToRawLongBits(Generated.literal().value()) != 0xfff8000000000022L) {
            throw new AssertionError("literal payload/sign changed");
        }
        if (Double.doubleToRawLongBits(Generated.absolute(right).value()) != 0x7ff8000000000022L
            || Double.doubleToRawLongBits(Generated.absolute(left).value()) != 0x7ff8000000000001L) {
            throw new AssertionError("FloatAbs did not preserve payload while clearing sign");
        }
    }
}
"#);
}
