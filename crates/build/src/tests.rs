use portable_check::v0::check_program;
use portable_diagnostics::DiagnosticCode;
use portable_eval::{EvaluationOutcome, Evaluator};
use portable_ir::v0::{NodeId, SourceRef, from_json, to_canonical_json};

use super::*;

struct Demo {
    document: portable_ir::v0::Document,
    test: TestId,
}

fn checked_demo() -> Demo {
    let mut module = ModuleBuilder::new("registration");
    let (label, text) = module.record(
        "Label",
        Visibility::Public,
        vec!["A display label.".to_owned()],
        |record| record.field("text", Type::string(), vec![]),
    );
    let (renderable, render) =
        module.contract("Renderable", Visibility::Public, vec![], |contract| {
            contract.method("render", vec![], vec![], Some(Type::string()))
        });
    let (_implementation, _method) = module.implementation(
        "LabelRenderable",
        Visibility::Package,
        vec![],
        renderable,
        label,
        |implementation| {
            implementation.method("render", render, vec![], |method| {
                method.returns(Type::string());
                method.body(|body| {
                    let receiver = body.self_value();
                    let value = body.field(receiver, text);
                    body.block([], Some(value))
                });
            })
        },
    );
    let call_render = module.function("call_render", Visibility::Public, vec![], |function| {
        function.parameter(Parameter::new("value", Type::contract(renderable)));
        function.returns(Type::string());
        function.body(|body| {
            let receiver = body.local("value");
            let value = body.contract_method(receiver, renderable, render, []);
            body.block([], Some(value))
        });
    });
    let test = module.portable_test(
        "call_render_returns_text",
        Visibility::Package,
        vec![],
        Invocation::function(
            call_render,
            [TypedValue::new(
                Type::named(label),
                Value::record(label, [(text, Value::string("hello"))]),
            )],
        ),
        Expected::value(TypedValue::new(Type::string(), Value::string("hello"))),
    );
    Demo {
        document: module.finish_unchecked().expect("demo builder completes"),
        test,
    }
}

#[test]
fn demonstration_round_trips_checks_and_evaluates() {
    let demo = checked_demo();
    let json = to_canonical_json(&demo.document).expect("builder output serializes");
    let parsed = from_json(&json).expect("builder output parses");
    assert_eq!(parsed, demo.document);
    let hand_authored = from_json(include_bytes!("../testdata/registration.poly.json"))
        .expect("hand-authored fixture parses");
    assert_eq!(demo.document, hand_authored);

    let checked = check_program(parsed).expect("demonstration checks");
    let result = Evaluator::new(&checked).run_test(demo.test.node_id());
    assert!(result.passed, "{result:#?}");
    assert_eq!(
        result.actual,
        EvaluationOutcome::Value(portable_ir::v0::Value::String("hello".to_owned()))
    );
}

#[test]
fn logical_sources_identify_module_and_builder_roles() {
    let demo = checked_demo();
    let json = serde_json::to_value(&demo.document).unwrap();
    let mut sources = Vec::new();
    collect_sources(&json, &mut sources);
    assert!(!sources.is_empty());
    assert!(sources.iter().all(|source| {
        matches!(
            source,
            SourceRef::Logical(path)
                if path.segments.first() == Some(&"module(registration)".to_owned())
                    && path.segments.len() >= 2
        )
    }));
}

fn collect_sources(value: &serde_json::Value, sources: &mut Vec<SourceRef>) {
    match value {
        serde_json::Value::Object(object) => {
            if let Some(source) = object.get("source") {
                sources.push(serde_json::from_value(source.clone()).unwrap());
            }
            for child in object.values() {
                collect_sources(child, sources);
            }
        }
        serde_json::Value::Array(values) => {
            for child in values {
                collect_sources(child, sources);
            }
        }
        _ => {}
    }
}

#[test]
fn incomplete_and_duplicate_builders_return_diagnostics_without_panicking() {
    let mut incomplete = ModuleBuilder::new("incomplete");
    incomplete.function("missing", Visibility::Public, vec![], |_function| {});
    let diagnostics = incomplete.finish_unchecked().unwrap_err();
    assert_eq!(diagnostics.len(), 2);
    assert!(diagnostics.iter().all(|diagnostic| {
        diagnostic.code == DiagnosticCode::InvalidStructure
            && diagnostic.message.contains("missing")
    }));

    let mut missing_contract_type = ModuleBuilder::new("contract_error");
    missing_contract_type.contract("C", Visibility::Public, vec![], |contract| {
        contract.method("missing", vec![], vec![], None);
    });
    assert_eq!(
        missing_contract_type.finish_unchecked().unwrap_err()[0].code,
        DiagnosticCode::InvalidStructure
    );

    let mut duplicate = ModuleBuilder::new("duplicate");
    duplicate.alias("Same", Visibility::Public, vec![], Type::i64());
    duplicate.alias("Same", Visibility::Public, vec![], Type::i64());
    assert!(
        duplicate
            .finish()
            .unwrap_err()
            .iter()
            .any(|diagnostic| diagnostic.code == DiagnosticCode::DuplicateDeclaration)
    );
}

#[test]
fn every_declaration_expression_statement_pattern_type_and_value_builder_compiles() {
    let mut module = ModuleBuilder::new("all_families");
    let (record, field) = module.record("Record", Visibility::Public, vec![], |record| {
        record.field("field", Type::i64(), vec![])
    });
    let (enumeration, (unit_variant, (data_variant, enum_field))) =
        module.enumeration("Choice", Visibility::Public, vec![], |enumeration| {
            let (unit, ()) = enumeration.variant("Unit", vec![], |_| {});
            let (data, field) = enumeration.variant("Data", vec![], |variant| {
                variant.field("value", Type::string(), vec![])
            });
            (unit, (data, field))
        });
    let alias = module.alias("Count", Visibility::Public, vec![], Type::i64());
    let (contract, contract_method) =
        module.contract("Contract", Visibility::Public, vec![], |contract| {
            contract.method(
                "method",
                vec![],
                vec![Parameter::documented("input", Type::i64(), ["An input."])],
                Some(Type::bool()),
            )
        });
    let (implementation, (implementation_method, ())) = module.implementation(
        "Implementation",
        Visibility::Package,
        vec![],
        contract,
        record,
        |implementation| {
            implementation.method("method", contract_method, vec![], |method| {
                method.parameter(Parameter::new("input", Type::i64()));
                method.returns(Type::bool());
                method.body(|body| {
                    let value = body.literal(Value::bool(true));
                    body.block([], Some(value))
                });
            })
        },
    );

    let constant = module.constant(
        "CONSTANT",
        Visibility::Public,
        vec![],
        Type::i64(),
        |body| {
            let literal = body.constant_literal(Value::i64(1));
            let reference = body.constant_reference(ConstantId::new(NodeId(999_000)));
            let record_value = body.constant_literal(Value::i64(1));
            let _ = body.constant_record(record, [(field, record_value)]);
            let enum_value = body.constant_literal(Value::string("x"));
            let _ = body.constant_enum(enumeration, data_variant, [(enum_field, enum_value)]);
            let some_value = body.constant_literal(Value::i64(1));
            let _ = body.constant_some(some_value);
            let _ = body.constant_none(Type::i64());
            let ok_value = body.constant_literal(Value::i64(1));
            let _ = body.constant_ok(ok_value, Type::string());
            let err_value = body.constant_literal(Value::string("e"));
            let _ = body.constant_err(err_value, Type::i64());
            let list_value = body.constant_literal(Value::i64(1));
            let _ = body.constant_list(Type::i64(), [list_value]);
            let left = body.constant_literal(Value::i64(1));
            let right = body.constant_literal(Value::i64(2));
            let _ = body.constant_intrinsic(Operation::IntAddChecked, [left, right]);
            let _ = reference;
            literal
        },
    );

    let callee = module.function("callee", Visibility::Package, vec![], |function| {
        function.returns(Type::i64());
        function.body(|body| {
            let value = body.literal(Value::i64(1));
            body.block([], Some(value))
        });
    });
    let all = module.function("all", Visibility::Public, vec![], |function| {
        function.parameter(Parameter::new("contract_value", Type::contract(contract)));
        function.returns(Type::unit());
        function.body(|body| {
            let literal = body.literal(Value::i64(1));
            let local = body.local("contract_value");
            let _constant = body.constant(constant);
            let _self_value = body.self_value();
            let record_field_value = body.literal(Value::i64(1));
            let record_value = body.record(record, [(field, record_field_value)]);
            let enum_field_value = body.literal(Value::string("value"));
            let _enum_value =
                body.enumeration(enumeration, data_variant, [(enum_field, enum_field_value)]);
            let some_value = body.literal(Value::i64(1));
            let _some = body.some(some_value);
            let _none = body.none(Type::i64());
            let ok_value = body.literal(Value::i64(1));
            let _ok = body.ok(ok_value, Type::string());
            let err_value = body.literal(Value::string("error"));
            let _err = body.err(err_value, Type::i64());
            let list_element = body.literal(Value::i64(1));
            let list = body.list(Type::i64(), [list_element]);
            let _field = body.field(record_value, field);
            let _call = body.call(callee, []);
            let receiver_value = body.literal(Value::record(record, [(field, Value::i64(1))]));
            let _concrete = body.concrete_method(
                receiver_value,
                implementation,
                implementation_method,
                [literal],
            );
            let contract_argument = body.literal(Value::i64(1));
            let _contract =
                body.contract_method(local, contract, contract_method, [contract_argument]);
            let left = body.literal(Value::i64(1));
            let right = body.literal(Value::i64(2));
            let _intrinsic = body.intrinsic(Operation::IntAddChecked, [left, right]);
            let then_value = body.literal(Value::unit());
            let then_block = body.block([], Some(then_value));
            let else_value = body.literal(Value::unit());
            let else_block = body.block([], Some(else_value));
            let condition = body.literal(Value::bool(true));
            let _if_value = body.if_else(condition, then_block, else_block);

            let wildcard = body.wildcard_pattern();
            let wildcard_value = body.literal(Value::unit());
            let wildcard_body = body.block([], Some(wildcard_value));
            let _wildcard_arm = body.match_arm(wildcard, wildcard_body);
            let _bool_pattern = body.bool_pattern(true);
            let _enum_pattern = body.enum_pattern(
                enumeration,
                unit_variant,
                std::iter::empty::<(EnumFieldId, String)>(),
            );
            let _none_pattern = body.none_pattern();
            let _some_pattern = body.some_pattern("some");
            let _ok_pattern = body.ok_pattern("ok");
            let _err_pattern = body.err_pattern("err");
            let match_pattern = body.bool_pattern(true);
            let match_result = body.literal(Value::unit());
            let match_body = body.block([], Some(match_result));
            let match_arm = body.match_arm(match_pattern, match_body);
            let matched = body.literal(Value::bool(true));
            let match_value = body.match_value(matched, [match_arm]);
            let nested_result = body.literal(Value::unit());
            let nested_block = body.block([], Some(nested_result));
            let _block_expression = body.block_expression(nested_block);

            let let_value = body.literal(Value::i64(1));
            let let_statement = body.let_statement("x", Some(Type::i64()), let_value);
            let loop_body = body.block([], None);
            let for_each = body.for_each("item", list, loop_body);
            let expression_statement = body.expression_statement(match_value);
            let return_value = body.literal(Value::unit());
            let return_statement = body.return_statement(Some(return_value));
            body.block(
                [
                    let_statement,
                    for_each,
                    expression_statement,
                    return_statement,
                ],
                None,
            )
        });
    });

    module.portable_test(
        "typed_test",
        Visibility::Package,
        vec![],
        Invocation::method(
            implementation,
            implementation_method,
            TypedValue::new(
                Type::named(record),
                Value::record(record, [(field, Value::i64(1))]),
            ),
            [TypedValue::new(Type::i64(), Value::i64(1))],
        ),
        Expected::value(TypedValue::new(Type::bool(), Value::bool(true))),
    );
    let _ = all;

    let types = [
        Type::unit(),
        Type::bool(),
        Type::i32(),
        Type::i64(),
        Type::f64(),
        Type::char(),
        Type::string(),
        Type::bytes(),
        Type::list(Type::i64()),
        Type::option(Type::i64()),
        Type::result(Type::i64(), Type::string()),
        Type::named(alias),
        Type::named(record),
        Type::named(enumeration),
        Type::contract(contract),
    ];
    assert_eq!(types.len(), 15);
    let values = [
        Value::unit(),
        Value::bool(true),
        Value::i32(1),
        Value::i64(1),
        Value::f64(-0.0),
        Value::f64_bits(f64::NAN.to_bits()),
        Value::char('🦀'),
        Value::string("text"),
        Value::bytes([0_u8, 255]),
        Value::list([Value::i64(1)]),
        Value::none(),
        Value::some(Value::i64(1)),
        Value::ok(Value::i64(1)),
        Value::err(Value::string("e")),
        Value::record(record, [(field, Value::i64(1))]),
        Value::enumeration(enumeration, unit_variant, []),
    ];
    assert_eq!(values.len(), 16);

    let document = module.finish_unchecked().expect("all builders complete");
    to_canonical_json(&document).expect("all-builder document is structurally valid");
}
