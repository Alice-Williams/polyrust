use std::collections::BTreeSet;

use portable_build::{
    Expected, Invocation, ModuleBuilder, Operation, Parameter, Type, TypedValue, Value, Visibility,
};
use portable_check::v0::CheckedProgram;
use portable_diagnostics::DiagnosticCode;
use portable_eval::{EvaluationOutcome, Evaluator};
use portable_ir::v0::Value as IrValue;

use super::*;

fn checked_fixture() -> CheckedProgram {
    let mut module = ModuleBuilder::new("core_ir_fixture");
    let greeting = module.constant(
        "GREETING",
        Visibility::Public,
        vec![],
        Type::string(),
        |body| body.constant_literal(Value::string("hello")),
    );
    let (counter, base) = module.record("Counter", Visibility::Public, vec![], |record| {
        record.field("base", Type::i64(), vec![])
    });
    let (adder, add) = module.contract("Adder", Visibility::Public, vec![], |contract| {
        contract.method(
            "add",
            vec![],
            vec![Parameter::new("amount", Type::i64())],
            Some(Type::i64()),
        )
    });
    let (counter_adder, (add_impl, ())) = module.implementation(
        "CounterAdder",
        Visibility::Public,
        vec![],
        adder,
        counter,
        |implementation| {
            implementation.method("add", add, vec![], |method| {
                method.parameter(Parameter::new("amount", Type::i64()));
                method.returns(Type::i64());
                method.body(|body| {
                    let receiver = body.self_value();
                    let left = body.field(receiver, base);
                    let right = body.local("amount");
                    let result = body.intrinsic(Operation::IntAddChecked, [left, right]);
                    body.block([], Some(result))
                });
            })
        },
    );
    let static_add = module.function("static_add", Visibility::Public, vec![], |function| {
        function.parameter(Parameter::new("counter", Type::named(counter)));
        function.parameter(Parameter::new("amount", Type::i64()));
        function.returns(Type::i64());
        function.body(|body| {
            let receiver = body.local("counter");
            let amount = body.local("amount");
            let result = body.concrete_method(receiver, counter_adder, add_impl, [amount]);
            body.block([], Some(result))
        });
    });
    let dynamic_add = module.function("dynamic_add", Visibility::Public, vec![], |function| {
        function.parameter(Parameter::new("adder", Type::contract(adder)));
        function.parameter(Parameter::new("amount", Type::i64()));
        function.returns(Type::i64());
        function.body(|body| {
            let receiver = body.local("adder");
            let amount = body.local("amount");
            let result = body.contract_method(receiver, adder, add, [amount]);
            body.block([], Some(result))
        });
    });
    let greeting_function = module.function("greeting", Visibility::Public, vec![], |function| {
        function.returns(Type::string());
        function.body(|body| {
            let value = body.constant(greeting);
            body.block([], Some(value))
        });
    });
    let option_text = module.function("option_text", Visibility::Public, vec![], |function| {
        function.parameter(Parameter::new("value", Type::option(Type::string())));
        function.returns(Type::string());
        function.body(|body| {
            let value = body.local("value");
            let some_pattern = body.some_pattern("text");
            let some_value = body.local("text");
            let some_body = body.block([], Some(some_value));
            let some_arm = body.match_arm(some_pattern, some_body);
            let none_pattern = body.none_pattern();
            let fallback = body.literal(Value::string("none"));
            let none_body = body.block([], Some(fallback));
            let none_arm = body.match_arm(none_pattern, none_body);
            let result = body.match_value(value, [some_arm, none_arm]);
            body.block([], Some(result))
        });
    });
    let choose = module.function("choose", Visibility::Public, vec![], |function| {
        function.parameter(Parameter::new("flag", Type::bool()));
        function.returns(Type::string());
        function.body(|body| {
            let condition = body.local("flag");
            let yes = body.literal(Value::string("yes"));
            let yes = body.block([], Some(yes));
            let no = body.literal(Value::string("no"));
            let no = body.block([], Some(no));
            let result = body.if_else(condition, yes, no);
            body.block([], Some(result))
        });
    });
    let visit = module.function("visit", Visibility::Public, vec![], |function| {
        function.parameter(Parameter::new("values", Type::list(Type::i64())));
        function.returns(Type::unit());
        function.body(|body| {
            let values = body.local("values");
            let item = body.local("item");
            let evaluate = body.expression_statement(item);
            let loop_body = body.block([evaluate], None);
            let loop_statement = body.for_each("item", values, loop_body);
            let unit = body.literal(Value::unit());
            body.block([loop_statement], Some(unit))
        });
    });

    let counter_value = || {
        TypedValue::new(
            Type::named(counter),
            Value::record(counter, [(base, Value::i64(40))]),
        )
    };
    for (name, function) in [
        ("static_dispatch", static_add),
        ("dynamic_dispatch", dynamic_add),
    ] {
        module.portable_test(
            name,
            Visibility::Package,
            vec![],
            Invocation::function(
                function,
                [counter_value(), TypedValue::new(Type::i64(), Value::i64(2))],
            ),
            Expected::value(TypedValue::new(Type::i64(), Value::i64(42))),
        );
    }
    module.portable_test(
        "constant_reference",
        Visibility::Package,
        vec![],
        Invocation::function(greeting_function, []),
        Expected::value(TypedValue::new(Type::string(), Value::string("hello"))),
    );
    module.portable_test(
        "option_some",
        Visibility::Package,
        vec![],
        Invocation::function(
            option_text,
            [TypedValue::new(
                Type::option(Type::string()),
                Value::some(Value::string("value")),
            )],
        ),
        Expected::value(TypedValue::new(Type::string(), Value::string("value"))),
    );
    module.portable_test(
        "if_branch",
        Visibility::Package,
        vec![],
        Invocation::function(choose, [TypedValue::new(Type::bool(), Value::bool(false))]),
        Expected::value(TypedValue::new(Type::string(), Value::string("no"))),
    );
    module.portable_test(
        "bounded_iteration",
        Visibility::Package,
        vec![],
        Invocation::function(
            visit,
            [TypedValue::new(
                Type::list(Type::i64()),
                Value::list([Value::i64(1), Value::i64(2)]),
            )],
        ),
        Expected::value(TypedValue::new(Type::unit(), Value::unit())),
    );
    module.finish().expect("CoreIR fixture checks")
}

#[test]
fn checked_fixture_lowers_verifies_and_is_byte_deterministic_three_times() {
    let dumps = (0..3)
        .map(|_| lower_checked(&checked_fixture()).unwrap().canonical_json())
        .collect::<Vec<_>>();
    assert_eq!(dumps[0], dumps[1]);
    assert_eq!(dumps[1], dumps[2]);
}

#[test]
fn evaluator_outcomes_match_lowered_portable_test_values() {
    let checked = checked_fixture();
    let results = Evaluator::new(&checked).run_all_tests();
    assert_eq!(results.len(), 6);
    assert!(results.iter().all(|result| result.passed), "{results:#?}");
    let core = lower_checked(&checked).unwrap();
    for result in results {
        let test = core
            .tests()
            .iter()
            .find(|test| test.header.name == result.name)
            .expect("lowered test retained");
        let CoreExpectedOutcome::Value(expected) = &test.expected else {
            panic!("fixture uses normal values")
        };
        assert_eq!(
            result.actual,
            EvaluationOutcome::Value(to_ir_value(&expected.value))
        );
    }
}

#[test]
fn canonical_core_contains_no_target_or_inheritance_escape_hatches() {
    let dump = lower_checked(&checked_fixture()).unwrap().canonical_json();
    for forbidden in [
        "raw_text",
        "verbatim",
        "snippet",
        "template",
        "import",
        "include",
        "extends",
        "inherits",
        "rust",
        "typescript",
        "javascript",
        "python",
        "java",
        "cpp",
    ] {
        assert!(
            !dump.contains(forbidden),
            "found target concept {forbidden:?}"
        );
    }
}

#[test]
fn fabricated_reference_type_and_evaluation_order_fail_deterministically() {
    let valid = lower_checked(&checked_fixture()).unwrap();

    let mut invalid_reference = valid.clone();
    let missing = CoreFunctionId::from_index(invalid_reference.functions().len());
    invalid_reference
        .module_mut()
        .declarations
        .push(CoreDeclaration::Function(missing));
    assert!(
        codes(verify_core(&invalid_reference).unwrap_err())
            .contains(&DiagnosticCode::UnresolvedReference)
    );

    let mut invalid_type = valid.clone();
    let string = valid
        .types()
        .iter()
        .find_map(|(id, ty)| (*ty == CoreType::String).then_some(id))
        .unwrap();
    let (bool_expression, _) = valid
        .expressions()
        .iter()
        .find(|(_, expression)| valid.types().get(expression.ty) == Some(&CoreType::Bool))
        .unwrap();
    invalid_type
        .expressions_mut()
        .get_mut(bool_expression)
        .unwrap()
        .ty = string;
    assert!(codes(verify_core(&invalid_type).unwrap_err()).contains(&DiagnosticCode::TypeMismatch));

    let mut invalid_order = valid.clone();
    let (call_id, _) = valid
        .expressions()
        .iter()
        .find(|(_, expression)| matches!(expression.kind, CoreExprKind::InterfaceCall { .. }))
        .unwrap();
    let CoreExprKind::InterfaceCall { receiver, .. } = &mut invalid_order
        .expressions_mut()
        .get_mut(call_id)
        .unwrap()
        .kind
    else {
        unreachable!()
    };
    *receiver = call_id;
    assert!(
        codes(verify_core(&invalid_order).unwrap_err())
            .contains(&DiagnosticCode::InvalidControlFlow)
    );
}

fn codes(diagnostics: Vec<portable_diagnostics::Diagnostic>) -> BTreeSet<DiagnosticCode> {
    diagnostics
        .into_iter()
        .map(|diagnostic| diagnostic.code)
        .collect()
}

fn to_ir_value(value: &CoreValue) -> IrValue {
    match value {
        CoreValue::Unit => IrValue::Unit,
        CoreValue::Bool(value) => IrValue::Bool(*value),
        CoreValue::I32(value) => IrValue::I32(*value),
        CoreValue::I64(value) => IrValue::I64(*value),
        CoreValue::F64(value) => IrValue::F64(*value),
        CoreValue::Char(value) => IrValue::Char(*value),
        CoreValue::String(value) => IrValue::String(value.clone()),
        CoreValue::Bytes(value) => IrValue::Bytes(value.clone()),
        CoreValue::List(values) => IrValue::List(values.iter().map(to_ir_value).collect()),
        CoreValue::None => IrValue::None,
        CoreValue::Some(value) => IrValue::Some(Box::new(to_ir_value(value))),
        CoreValue::Ok(value) => IrValue::Ok(Box::new(to_ir_value(value))),
        CoreValue::Err(value) => IrValue::Err(Box::new(to_ir_value(value))),
        CoreValue::Record { .. } | CoreValue::Enum { .. } => {
            panic!("fixture outcomes do not contain nominal values")
        }
    }
}
