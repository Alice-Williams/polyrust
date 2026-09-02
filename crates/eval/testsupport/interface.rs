use portable_build::{Type, Value, interface_composition_fixture};
use portable_check::v0::check_program;
use portable_eval::{EvaluationOutcome, Evaluator};

fn label_value(fixture: &portable_build::InterfaceFixture) -> Value {
    Value::record(
        fixture.label,
        [(fixture.label_text, Value::string("value"))],
    )
}

#[test]
fn canonical_interface_and_composition_corpus_is_deterministic() {
    let fixture = interface_composition_fixture();
    let checked = check_program(fixture.document.clone()).expect("canonical corpus checks");
    let evaluator = Evaluator::new(&checked);
    let results = evaluator.run_all_tests();
    assert_eq!(results.len(), 9);
    assert!(results.iter().all(|result| result.passed), "{results:#?}");

    let value = label_value(&fixture);
    for (function, expected) in [
        (fixture.local_dispatch, "local:value"),
        (fixture.list_dispatch, "list:value"),
        (fixture.option_dispatch, "option:value"),
        (fixture.result_dispatch, "result:value"),
        (fixture.composition_dispatch, "composition:value"),
        (fixture.enum_dispatch, "enum:value"),
    ] {
        assert_eq!(
            evaluator.invoke_function(function.node_id(), &[value.as_ir().clone()]),
            EvaluationOutcome::Value(Value::string(expected).as_ir().clone()),
        );
    }
    assert_eq!(
        evaluator.invoke_function(
            fixture.measured_dispatch.node_id(),
            &[value.as_ir().clone()],
        ),
        EvaluationOutcome::Value(Value::i64(5).as_ir().clone()),
    );

    // The public canonical value algebra deliberately has no interface value
    // constructor; the only authoring path is an expression-level witness.
    let _interface_type = Type::interface(fixture.labelled);
}
