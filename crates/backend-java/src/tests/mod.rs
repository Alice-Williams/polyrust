use super::*;
use std::collections::BTreeSet;

pub(crate) mod capability_fixtures;
mod mapping_coverage;
mod mapping_short_circuit;
pub(crate) mod match_dispatch;
mod name_identity_consumers;
mod nested_payload_matches;
mod resources_native;
mod totality_empty_interfaces;
mod totality_interfaces;
mod totality_names;
mod totality_nominal_factories;
mod totality_oracle;
mod totality_text;
mod typed_fixture;
use capability_fixtures::{capability_coverage_fixture, portable_method_invocation_fixture};
use portable_build::{
    Expected, I32, Invocation, ModuleBuilder, Operation, Parameter, Type, TypedValue, Value,
    Visibility, enum_arm, field, parameter, portable_name, typed_list, typed_program, variant,
};
use portable_codegen::OutputContents;
use typed_fixture::typed_fixture_manifests;

fn fixture() -> CheckedProgram {
    portable_check::v0::check_program(
        portable_ir::v0::from_json(include_bytes!(
            "../../../build/testdata/registration.poly.json"
        ))
        .expect("fixture parses"),
    )
    .expect("fixture checks")
}

pub(super) fn generated_text<'a>(manifest: &'a OutputManifest, path: &str) -> &'a str {
    match manifest.file(path).expect("generated file").contents() {
        OutputContents::Text(value) => value,
        OutputContents::Bytes(_) => panic!("Java source must be text"),
    }
}

#[test]
fn inferred_generation_is_total_deterministic_and_structural() {
    let [first, second, third] = typed_fixture_manifests();
    assert_eq!(first.canonical_json(), second.canonical_json());
    assert_eq!(second.canonical_json(), third.canonical_json());

    let generated = generated_text(
        &first,
        "src/main/java/org/polyrust/generated/Generated.java",
    );
    assert!(generated.contains("public static record Point3(int x, int y, int z)"));
    assert!(generated.contains("compute(final int left, final int right, final int scale)"));
    assert!(generated.contains("final int __polyrust_intrinsicOperand_0 = (left + right);"));
    assert!(generated.contains("final int __polyrust_intrinsicOperand_1 = (left - right);"));
    assert!(generated.contains(
        "final int __polyrust_intrinsicOperand_2 = (__polyrust_intrinsicOperand_0 * __polyrust_intrinsicOperand_1);"
    ));
    assert!(generated.contains("Runtime.ok((__polyrust_intrinsicOperand_2 + scale))"));
    assert!(generated.contains("Runtime.ok(new Point3(x, y, z))"));
    assert!(generated.contains("public enum TrafficLight"));
    assert!(generated.contains("RED,\n        AMBER,\n        GREEN;"));
    assert!(generated.contains("Runtime.PolyResult<TrafficLight> stop_light()"));
    assert!(generated.contains("Runtime.ok(TrafficLight.RED)"));
    assert!(generated.contains("Runtime.ok((value == TrafficLight.RED))"));
    assert!(generated.contains("Runtime.PolyResult<Integer> traffic_light_priority("));
    assert!(generated.contains("case RED ->"));
    assert!(generated.contains("case AMBER ->"));
    assert!(generated.contains("case GREEN ->"));
}

#[test]
fn generated_manifest_is_typed_deterministic_and_dependency_free() {
    let checked = fixture();
    let first = JavaBackend
        .generate(&checked, &BackendOptions::default())
        .unwrap();
    let second = JavaBackend
        .generate(&checked, &BackendOptions::default())
        .unwrap();
    let third = JavaBackend
        .generate(&checked, &BackendOptions::default())
        .unwrap();
    assert_eq!(first.canonical_json(), second.canonical_json());
    assert_eq!(second.canonical_json(), third.canonical_json());
    assert!(first.dependencies().is_empty());
    assert_eq!(first.files().len(), 6);
}

#[test]
fn generated_source_contains_direct_methods_and_no_legacy_interpreter() {
    let manifest = JavaBackend
        .generate(&fixture(), &BackendOptions::default())
        .unwrap();
    let generated = generated_text(
        &manifest,
        "src/main/java/org/polyrust/generated/Generated.java",
    );
    let runtime = generated_text(
        &manifest,
        "src/main/java/org/polyrust/generated/Runtime.java",
    );
    assert!(generated.contains("record Label"));
    assert!(generated.contains("interface Renderable"));
    for forbidden in [
        "serde_json",
        "jsonArray",
        "invokeTest",
        "invokeMethod",
        "readConstant",
        "POLYRUST-BEGIN",
        "POLYRUST-END",
        "Runtime.capture",
        "Runtime.unwrap",
        "class Halt",
        "extends RuntimeException",
    ] {
        assert!(!generated.contains(forbidden), "found {forbidden}");
        assert!(!runtime.contains(forbidden), "found {forbidden}");
    }
}

#[test]
fn conformance_program_executes_the_exact_portable_test_inventory() {
    let checked = fixture();
    assert_eq!(
        checked
            .module()
            .declarations
            .iter()
            .filter(|declaration| matches!(declaration, portable_ir::v0::Declaration::Test(_)))
            .count(),
        1
    );
    let manifest = JavaBackend
        .generate(&checked, &BackendOptions::default())
        .unwrap();
    let conformance = generated_text(
        &manifest,
        "src/test/java/org/polyrust/generated/ConformanceTest.java",
    );
    assert!(conformance.contains("call_render_returns_text"));
    assert!(conformance.contains("int completed = 0;"));
    assert!(conformance.contains("completed = (completed + 1);"));
    assert!(conformance.contains("completed == 1"));
    assert!(conformance.contains("Runtime.deepEqual"));
    assert!(!conformance.contains("public static void main(final String[] arguments) {\n    }"));
}

#[test]
#[cfg(test)]
fn imports_are_derived_from_typed_references() {
    let manifest = JavaBackend
        .generate(&fixture(), &BackendOptions::default())
        .unwrap();
    let generated = generated_text(
        &manifest,
        "src/main/java/org/polyrust/generated/Generated.java",
    );
    assert!(!generated.contains("import java.math.BigInteger;"));
    assert!(!generated.contains("import java.nio.ByteBuffer;"));
    assert_eq!(generated.matches("import java.util.List;").count(), 0);
    assert_eq!(generated.matches("import java.util.Objects;").count(), 1);
    assert!(generated.contains("Runtime.requireScalarString(text)"));
}

#[test]
fn runtime_imports_are_exact_and_physically_deduplicated() {
    let manifest = JavaBackend
        .generate(&fixture(), &BackendOptions::default())
        .unwrap();
    let runtime = generated_text(
        &manifest,
        "src/main/java/org/polyrust/generated/Runtime.java",
    );
    let imports = runtime
        .lines()
        .filter_map(|line| line.strip_prefix("import ")?.strip_suffix(';'))
        .collect::<Vec<_>>();
    let unique = imports.iter().copied().collect::<BTreeSet<_>>();
    assert_eq!(imports.len(), unique.len(), "duplicate import: {imports:?}");
    assert_eq!(
        unique,
        BTreeSet::from(["java.util.List", "java.util.Objects"])
    );
}

#[test]
fn canonical_interface_and_composition_corpus_is_flat_and_deterministic() {
    let checked =
        portable_check::v0::check_program(portable_build::interface_composition_fixture().document)
            .expect("canonical interface corpus checks");
    let first = JavaBackend
        .generate(&checked, &BackendOptions::default())
        .unwrap();
    let second = JavaBackend
        .generate(&checked, &BackendOptions::default())
        .unwrap();
    let third = JavaBackend
        .generate(&checked, &BackendOptions::default())
        .unwrap();
    assert_eq!(first.canonical_json(), second.canonical_json());
    assert_eq!(second.canonical_json(), third.canonical_json());

    let generated = generated_text(
        &first,
        "src/main/java/org/polyrust/generated/Generated.java",
    );
    assert!(generated.contains("interface Labelled"));
    assert!(generated.contains("interface Measured"));
    assert!(generated.contains(
        "record Label(String text) implements org.polyrust.generated.Runtime.SemanticValue, Labelled, Measured"
    ));
    assert!(generated.contains("record Service(Labelled renderer)"));
    assert!(generated.contains("Objects.requireNonNull(renderer)"));
    assert!(!generated.contains("interface Labelled extends"));
    assert!(!generated.contains("interface Measured extends"));
}

#[test]
fn overlapping_erased_interface_methods_preserve_distinct_dispatch() {
    let mut module = ModuleBuilder::new("java_overlap");
    let (first, first_method) =
        module.interface("First", Visibility::Public, vec![], |interface| {
            interface.method("render", vec![], vec![], Some(Type::string()))
        });
    let (second, second_method) =
        module.interface("Second", Visibility::Public, vec![], |interface| {
            interface.method("render", vec![], vec![], Some(Type::string()))
        });
    let (record, ()) = module.record("Value", Visibility::Public, vec![], |_| {});
    module.implementation(
        "ValueFirst",
        Visibility::Package,
        vec![],
        first,
        record,
        |implementation| {
            implementation.method("render", first_method, vec![], |method| {
                method.returns(Type::string());
                method.body(|body| {
                    let value = body.literal(Value::string("first"));
                    body.block([], Some(value))
                });
            });
        },
    );
    module.implementation(
        "ValueSecond",
        Visibility::Package,
        vec![],
        second,
        record,
        |implementation| {
            implementation.method("render", second_method, vec![], |method| {
                method.returns(Type::string());
                method.body(|body| {
                    let value = body.literal(Value::string("second"));
                    body.block([], Some(value))
                });
            });
        },
    );
    let checked = module.finish().expect("portable overlap is valid");
    let manifest = JavaBackend
        .generate(&checked, &BackendOptions::default())
        .unwrap();
    totality_oracle::CompiledPackage::new(&manifest, "overlapping-methods").consumer(
        r#"
package org.polyrust.consumer;
import org.polyrust.generated.Generated;
public final class Consumer {
    private Consumer() {}
    public static void main(String[] args) {
        Generated.Value value = new Generated.Value();
        Generated.First first = value;
        Generated.Second second = value;
        if (!first.render().value().equals("first")
            || !second.render_1().value().equals("second")
            || !value.render().value().equals("first")
            || !value.render_1().value().equals("second")) {
            throw new AssertionError("interface identities merged");
        }
    }
}
"#,
    );
}

#[test]
fn generated_expressions_are_evaluated_once_in_source_order() {
    let mut module = ModuleBuilder::new("java_evaluation_order");
    let (boxed, field) = module.record("Boxed", Visibility::Public, vec![], |record| {
        record.field("value", Type::i32(), vec![])
    });
    let later = module.function("later", Visibility::Package, vec![], |function| {
        function.returns(Type::i32());
        function.body(|body| {
            let value = body.literal(Value::i32(2));
            body.block([], Some(value))
        });
    });
    let combine = module.function("combine", Visibility::Package, vec![], |function| {
        function.parameter(Parameter::new("left", Type::named(boxed)));
        function.parameter(Parameter::new("right", Type::i32()));
        function.returns(Type::i32());
        function.body(|body| {
            let right = body.local("right");
            body.block([], Some(right))
        });
    });
    module.function("entry", Visibility::Public, vec![], |function| {
        function.returns(Type::i32());
        function.body(|body| {
            let one = body.literal(Value::i32(1));
            let allocated = body.record(boxed, [(field, one)]);
            let later_value = body.call(later, []);
            let result = body.call(combine, [allocated, later_value]);
            body.block([], Some(result))
        });
    });
    module.function("unwrap_once", Visibility::Public, vec![], |function| {
        function.returns(Type::string());
        function.body(|body| {
            let a = body.literal(Value::string("a"));
            let b = body.literal(Value::string("b"));
            let concatenated = body.intrinsic(Operation::StringConcat, [a, b]);
            let present = body.some(concatenated);
            let fallback = body.literal(Value::string("fallback"));
            let result = body.intrinsic(Operation::OptionUnwrapOr, [present, fallback]);
            body.block([], Some(result))
        });
    });

    let checked = module.finish().expect("evaluation-order fixture checks");
    let manifest = JavaBackend
        .generate(&checked, &BackendOptions::default())
        .expect("evaluation-order fixture generates");
    let generated = generated_text(
        &manifest,
        "src/main/java/org/polyrust/generated/Generated.java",
    );

    let entry = generated
        .split(" entry()")
        .nth(1)
        .expect("entry method is generated")
        .split("\n    }")
        .next()
        .expect("entry method body");
    let allocation = entry.find("new Boxed(").expect("record allocation");
    let later_call = entry.find("later()").expect("later operand call");
    let combine_call = entry.find("combine(").expect("outer call");
    assert!(
        allocation < later_call,
        "allocation must precede later operand"
    );
    assert!(
        later_call < combine_call,
        "operands must precede outer call"
    );
    assert_eq!(entry.matches("new Boxed(").count(), 1);

    let unwrap = generated
        .split(" unwrap_once()")
        .nth(1)
        .expect("unwrap_once method is generated")
        .split("\n    }")
        .next()
        .expect("unwrap_once method body");
    assert_eq!(
        unwrap.matches("\"a\".concat(\"b\")").count(),
        1,
        "the nontrivial left operand must be evaluated exactly once"
    );
}

#[test]
fn portable_evaluate_lowers_to_a_valid_java_local() {
    let mut module = ModuleBuilder::new("java_evaluate");
    module.function("visit", Visibility::Public, vec![], |function| {
        function.parameter(Parameter::new("value", Type::i64()));
        function.returns(Type::unit());
        function.body(|body| {
            let value = body.local("value");
            let evaluate = body.expression_statement(value);
            let unit = body.literal(Value::unit());
            body.block([evaluate], Some(unit))
        });
    });

    let checked = module.finish().expect("Evaluate fixture checks");
    let manifest = JavaBackend
        .generate(&checked, &BackendOptions::default())
        .expect("valid Evaluate fixture generates");
    let generated = generated_text(
        &manifest,
        "src/main/java/org/polyrust/generated/Generated.java",
    );
    assert_eq!(
        generated.matches("final long __polyrust_evaluate_").count(),
        1,
        "portable Evaluate is materialized as a legal Java local initializer"
    );
    assert!(!generated.lines().any(|line| line.trim() == "value;"));
}
