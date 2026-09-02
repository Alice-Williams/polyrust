use portable_check::v0::check_program;
use portable_ir::v0::*;

use super::*;

const SEMANTIC_VECTORS: &str = include_str!("../../../conformance/v0/evaluator-vectors.json");

#[test]
fn canonical_values_round_trip_every_runtime_variant() {
    let values = [
        Value::Unit,
        Value::Bool(true),
        Value::I32(i32::MIN),
        Value::I64(i64::MAX),
        Value::F64(F64Bits(f64::NAN.to_bits())),
        Value::F64(F64Bits(f64::INFINITY.to_bits())),
        Value::F64(F64Bits(f64::NEG_INFINITY.to_bits())),
        Value::F64(F64Bits((-0.0_f64).to_bits())),
        Value::Char('🦀'),
        Value::String("e\u{301}🌍".to_owned()),
        Value::Bytes(vec![0, 1, 127, 128, 255]),
        Value::List(vec![Value::I32(1), Value::I32(2)]),
        Value::None,
        Value::Some(Box::new(Value::String("some".to_owned()))),
        Value::Ok(Box::new(Value::I64(42))),
        Value::Err(Box::new(Value::String("failure".to_owned()))),
        Value::Record {
            declaration: NodeId(100),
            fields: vec![ValueField {
                field: NodeId(101),
                value: Value::List(vec![Value::Some(Box::new(Value::Bool(true)))]),
            }],
        },
        Value::Enum {
            declaration: NodeId(200),
            variant: NodeId(201),
            fields: vec![ValueField {
                field: NodeId(202),
                value: Value::Bytes(vec![0xde, 0xad, 0xbe, 0xef]),
            }],
        },
    ];

    for value in values {
        let encoded = encode_canonical_value(&value);
        assert_eq!(decode_canonical_value(&encoded), Ok(value));
    }
}

#[test]
fn canonical_errors_and_outcomes_round_trip() {
    let errors = [
        EvaluationError::CheckedOverflow { operation: "add" },
        EvaluationError::DivisionByZero,
        EvaluationError::RemainderByZero,
        EvaluationError::InvalidShift {
            amount: -1,
            width: 64,
        },
        EvaluationError::NarrowingOutOfRange { value: i64::MAX },
        EvaluationError::IndexOutOfBounds {
            index: -1,
            length: 3,
        },
        EvaluationError::InvalidUtf8,
        EvaluationError::FuelExhausted { limit: 10 },
        EvaluationError::CallDepthExceeded { limit: 4 },
        EvaluationError::CollectionLimitExceeded {
            limit: 8,
            requested: 9,
        },
        EvaluationError::InvariantViolation {
            message: "test invariant".to_owned(),
        },
    ];

    for error in errors {
        let encoded = encode_canonical_error(&error);
        assert_eq!(decode_canonical_error(&encoded), Ok(error.clone()));
        let outcome = EvaluationOutcome::Error(error);
        let encoded = encode_canonical_outcome(&outcome);
        assert_eq!(decode_canonical_outcome(&encoded), Ok(outcome));
    }

    let outcome = EvaluationOutcome::Value(Value::F64(F64Bits((-0.0_f64).to_bits())));
    assert_eq!(
        decode_canonical_outcome(&encode_canonical_outcome(&outcome)),
        Ok(outcome)
    );
}

#[test]
fn canonical_encoding_is_stable_and_lossless_for_json_unsafe_integers() {
    let encoded = encode_canonical_outcome(&EvaluationOutcome::Value(Value::I64(i64::MAX)));
    assert_eq!(
        serde_json::to_string(&encoded).unwrap(),
        r#"{"outcome":"value","protocol":"polyrust.canonical.v0","value":{"type":"i64","value":"9223372036854775807"}}"#
    );
}

#[test]
fn malformed_canonical_values_are_rejected_without_panicking() {
    for malformed in [
        serde_json::json!(null),
        serde_json::json!({"type": "f64", "bits": "nan"}),
        serde_json::json!({"type": "char", "value": "ab"}),
        serde_json::json!({"type": "bytes", "hex": "F"}),
        serde_json::json!({"type": "unknown"}),
    ] {
        assert!(decode_canonical_value(&malformed).is_err());
    }
}

#[test]
fn shared_semantic_vector_fixture_is_unique_and_canonical() {
    let fixture: serde_json::Value =
        serde_json::from_str(SEMANTIC_VECTORS).expect("semantic vector JSON parses");
    assert_eq!(fixture["protocol"], "polyrust.evaluator-vectors.v0");
    let vectors = fixture["vectors"].as_array().expect("vectors is an array");
    assert!(vectors.len() >= 20);
    let mut ids = std::collections::BTreeSet::new();
    for vector in vectors {
        assert!(
            ids.insert(vector["id"].as_str().expect("vector id is text")),
            "duplicate vector id"
        );
        for argument in vector["arguments"]
            .as_array()
            .expect("arguments is an array")
        {
            decode_canonical_value(argument).expect("argument is canonical");
        }
        decode_canonical_outcome(&vector["expected"]).expect("outcome is canonical");
    }
}

struct Factory {
    next: u64,
}

impl Factory {
    fn new() -> Self {
        Self { next: 1 }
    }

    fn node(&mut self) -> NodeMeta {
        let id = self.next;
        self.next += 1;
        NodeMeta::new(NodeId(id), SourceRef::logical([format!("node({id})")]))
    }

    fn declaration(&mut self, name: &str) -> DeclarationHeader {
        DeclarationHeader {
            node: self.node(),
            name: name.to_owned(),
            visibility: Visibility::Public,
            documentation: vec![],
        }
    }

    fn member(&mut self, name: &str) -> MemberHeader {
        MemberHeader {
            node: self.node(),
            name: name.to_owned(),
            documentation: vec![],
        }
    }

    fn parameter(&mut self, name: &str, ty: TypeRef) -> Parameter {
        Parameter {
            header: self.member(name),
            ty,
        }
    }

    fn literal(&mut self, value: Value) -> Expression {
        Expression::Literal {
            node: self.node(),
            value,
        }
    }

    fn local(&mut self, name: &str) -> Expression {
        Expression::Local {
            node: self.node(),
            name: name.to_owned(),
        }
    }

    fn block(&mut self, result: Expression) -> Block {
        Block {
            node: self.node(),
            statements: vec![],
            result: Some(Box::new(result)),
        }
    }
}

struct FixtureIds {
    implementation: NodeId,
    method: NodeId,
    record: NodeId,
    field: NodeId,
    concrete_function: NodeId,
    interface_function: NodeId,
    short_function: NodeId,
    option_function: NodeId,
    first_function: NodeId,
}

fn evaluator_fixture() -> (Document, FixtureIds) {
    let mut factory = Factory::new();

    let record_header = factory.declaration("Label");
    let record_id = record_header.node.id;
    let field_header = factory.member("text");
    let field_id = field_header.node.id;
    let record = Declaration::Record(RecordDeclaration {
        header: record_header,
        fields: vec![FieldDeclaration {
            header: field_header,
            ty: TypeRef::String,
        }],
    });

    let contract_header = factory.declaration("Renderable");
    let contract_id = contract_header.node.id;
    let required_header = factory.member("render");
    let required_id = required_header.node.id;
    let interface = Declaration::Interface(InterfaceDeclaration {
        header: contract_header,
        methods: vec![MethodSignature {
            header: required_header,
            parameters: vec![],
            return_type: TypeRef::String,
        }],
    });

    let implementation_header = factory.declaration("LabelRenderable");
    let implementation_id = implementation_header.node.id;
    let method_header = factory.member("render");
    let method_id = method_header.node.id;
    let self_node = factory.node();
    let field_node = factory.node();
    let method_body = factory.block(Expression::Field {
        node: field_node,
        base: Box::new(Expression::SelfValue { node: self_node }),
        field: field_id,
    });
    let implementation = Declaration::Implementation(ImplementationDeclaration {
        header: implementation_header,
        interface: contract_id,
        record: record_id,
        methods: vec![MethodImplementation {
            header: method_header,
            interface_method: required_id,
            parameters: vec![],
            return_type: TypeRef::String,
            body: method_body,
        }],
    });

    let concrete_header = factory.declaration("call_concrete");
    let concrete_function = concrete_header.node.id;
    let concrete_parameter = factory.parameter("label", TypeRef::Named(record_id));
    let concrete_receiver = factory.local("label");
    let concrete_call_node = factory.node();
    let concrete_call = Expression::MethodCall {
        node: concrete_call_node,
        receiver: Box::new(concrete_receiver),
        dispatch: MethodDispatch::Concrete {
            implementation: implementation_id,
            method: method_id,
        },
        arguments: vec![],
    };
    let concrete_body = factory.block(concrete_call);
    let concrete = Declaration::Function(FunctionDeclaration {
        header: concrete_header,
        parameters: vec![concrete_parameter],
        return_type: TypeRef::String,
        body: concrete_body,
    });

    let contract_function_header = factory.declaration("call_contract");
    let contract_function = contract_function_header.node.id;
    let contract_parameter = factory.parameter("value", TypeRef::Interface(contract_id));
    let contract_receiver = factory.local("value");
    let contract_call_node = factory.node();
    let contract_call = Expression::MethodCall {
        node: contract_call_node,
        receiver: Box::new(contract_receiver),
        dispatch: MethodDispatch::Interface {
            interface: contract_id,
            method: required_id,
        },
        arguments: vec![],
    };
    let contract_body = factory.block(contract_call);
    let contract_caller = Declaration::Function(FunctionDeclaration {
        header: contract_function_header,
        parameters: vec![contract_parameter],
        return_type: TypeRef::String,
        body: contract_body,
    });

    let short_header = factory.declaration("short_circuit");
    let short_function = short_header.node.id;
    let false_value = factory.literal(Value::Bool(false));
    let one = factory.literal(Value::I64(1));
    let zero_divisor = factory.literal(Value::I64(0));
    let division_node = factory.node();
    let division = Expression::Intrinsic {
        node: division_node,
        operation: Intrinsic::IntDivChecked,
        arguments: vec![one, zero_divisor],
    };
    let zero_compare = factory.literal(Value::I64(0));
    let equal_node = factory.node();
    let would_fail = Expression::Intrinsic {
        node: equal_node,
        operation: Intrinsic::Equal,
        arguments: vec![division, zero_compare],
    };
    let and_node = factory.node();
    let short_expression = Expression::Intrinsic {
        node: and_node,
        operation: Intrinsic::BoolAnd,
        arguments: vec![false_value, would_fail],
    };
    let short_body = factory.block(short_expression);
    let short = Declaration::Function(FunctionDeclaration {
        header: short_header,
        parameters: vec![],
        return_type: TypeRef::Bool,
        body: short_body,
    });

    let option_header = factory.declaration("option_or_zero");
    let option_function = option_header.node.id;
    let option_parameter = factory.parameter("value", TypeRef::Option(Box::new(TypeRef::I64)));
    let option_value = factory.local("value");
    let none_value = factory.literal(Value::I64(0));
    let none_body = factory.block(none_value);
    let some_value = factory.local("inner");
    let some_body = factory.block(some_value);
    let none_pattern_node = factory.node();
    let none_arm_node = factory.node();
    let some_pattern_node = factory.node();
    let some_arm_node = factory.node();
    let match_node = factory.node();
    let option_match = Expression::Match {
        node: match_node,
        value: Box::new(option_value),
        arms: vec![
            MatchArm {
                node: none_arm_node,
                pattern: Pattern::None {
                    node: none_pattern_node,
                },
                body: none_body,
            },
            MatchArm {
                node: some_arm_node,
                pattern: Pattern::Some {
                    node: some_pattern_node,
                    binding: "inner".to_owned(),
                },
                body: some_body,
            },
        ],
    };
    let option_body = factory.block(option_match);
    let option = Declaration::Function(FunctionDeclaration {
        header: option_header,
        parameters: vec![option_parameter],
        return_type: TypeRef::I64,
        body: option_body,
    });

    let first_header = factory.declaration("first_or_zero");
    let first_function = first_header.node.id;
    let items_parameter = factory.parameter("items", TypeRef::List(Box::new(TypeRef::I64)));
    let iterable = factory.local("items");
    let item = factory.local("item");
    let return_node = factory.node();
    let loop_body_node = factory.node();
    let loop_body = Block {
        node: loop_body_node,
        statements: vec![Statement::Return {
            node: return_node,
            value: Some(item),
        }],
        result: None,
    };
    let for_node = factory.node();
    let fallback = factory.literal(Value::I64(0));
    let first_body_node = factory.node();
    let first_body = Block {
        node: first_body_node,
        statements: vec![Statement::ForEach {
            node: for_node,
            binding: "item".to_owned(),
            iterable,
            body: loop_body,
        }],
        result: Some(Box::new(fallback)),
    };
    let first = Declaration::Function(FunctionDeclaration {
        header: first_header,
        parameters: vec![items_parameter],
        return_type: TypeRef::I64,
        body: first_body,
    });

    let ids = FixtureIds {
        implementation: implementation_id,
        method: method_id,
        record: record_id,
        field: field_id,
        concrete_function,
        interface_function: contract_function,
        short_function,
        option_function,
        first_function,
    };
    let declarations = vec![
        record,
        interface,
        implementation,
        concrete,
        contract_caller,
        short,
        option,
        first,
    ];
    (
        Document::new(
            IrVersion::CURRENT,
            Module {
                name: "evaluator_fixture".to_owned(),
                declarations,
            },
        ),
        ids,
    )
}

fn label_value(ids: &FixtureIds, text: &str) -> Value {
    Value::Record {
        declaration: ids.record,
        fields: vec![ValueField {
            field: ids.field,
            value: Value::String(text.to_owned()),
        }],
    }
}

fn push_test(
    document: &mut Document,
    id: u64,
    name: &str,
    invocation: TestInvocation,
    expected: TypedValue,
) {
    document
        .module
        .declarations
        .push(Declaration::Test(TestDeclaration {
            header: DeclarationHeader {
                node: NodeMeta::new(
                    NodeId(10_000 + id),
                    SourceRef::logical([format!("test({name})")]),
                ),
                name: name.to_owned(),
                visibility: Visibility::Package,
                documentation: vec![],
            },
            invocation,
            expected: ExpectedOutcome::Value(expected),
        }));
}

#[test]
fn checked_fixture_runs_eleven_declared_tests_and_both_dispatch_modes() {
    let (mut document, ids) = evaluator_fixture();
    let mut next = 1;
    for (name, text) in [
        ("method_ascii", "alpha"),
        ("method_astral", "🦀"),
        ("method_combining", "e\u{301}"),
    ] {
        push_test(
            &mut document,
            next,
            name,
            TestInvocation::Method {
                implementation: ids.implementation,
                method: ids.method,
                receiver: TypedValue {
                    ty: TypeRef::Named(ids.record),
                    value: label_value(&ids, text),
                },
                arguments: vec![],
            },
            TypedValue {
                ty: TypeRef::String,
                value: Value::String(text.to_owned()),
            },
        );
        next += 1;
    }

    for (name, function, text) in [
        ("concrete_one", ids.concrete_function, "concrete"),
        ("concrete_two", ids.concrete_function, "second"),
        ("contract_one", ids.interface_function, "interface"),
        ("contract_two", ids.interface_function, "dynamic"),
    ] {
        push_test(
            &mut document,
            next,
            name,
            TestInvocation::Function {
                function,
                arguments: vec![TypedValue {
                    ty: TypeRef::Named(ids.record),
                    value: label_value(&ids, text),
                }],
            },
            TypedValue {
                ty: TypeRef::String,
                value: Value::String(text.to_owned()),
            },
        );
        next += 1;
    }

    push_test(
        &mut document,
        next,
        "short_circuit_skips_error",
        TestInvocation::Function {
            function: ids.short_function,
            arguments: vec![],
        },
        TypedValue {
            ty: TypeRef::Bool,
            value: Value::Bool(false),
        },
    );
    next += 1;
    for (name, input, expected) in [
        (
            "option_some",
            Value::Some(Box::new(Value::I64(42))),
            Value::I64(42),
        ),
        ("option_none", Value::None, Value::I64(0)),
    ] {
        push_test(
            &mut document,
            next,
            name,
            TestInvocation::Function {
                function: ids.option_function,
                arguments: vec![TypedValue {
                    ty: TypeRef::Option(Box::new(TypeRef::I64)),
                    value: input,
                }],
            },
            TypedValue {
                ty: TypeRef::I64,
                value: expected,
            },
        );
        next += 1;
    }
    push_test(
        &mut document,
        next,
        "bounded_iteration_returns_first",
        TestInvocation::Function {
            function: ids.first_function,
            arguments: vec![TypedValue {
                ty: TypeRef::List(Box::new(TypeRef::I64)),
                value: Value::List(vec![Value::I64(7), Value::I64(8)]),
            }],
        },
        TypedValue {
            ty: TypeRef::I64,
            value: Value::I64(7),
        },
    );

    let checked = check_program(document).expect("evaluator fixture checks");
    let evaluator = Evaluator::new(&checked);
    let results = evaluator.run_all_tests();
    assert_eq!(results.len(), 11);
    assert!(
        results.iter().all(|result| result.passed),
        "portable test failures: {results:#?}"
    );
    assert_eq!(
        evaluator.invoke_function(
            ids.interface_function,
            &[label_value(&ids, "interface direct")]
        ),
        EvaluationOutcome::Value(Value::String("interface direct".to_owned()))
    );
    assert_eq!(
        evaluator.invoke_function(
            ids.concrete_function,
            &[label_value(&ids, "concrete direct")]
        ),
        EvaluationOutcome::Value(Value::String("concrete direct".to_owned()))
    );
}

#[test]
fn public_evaluator_enforces_fuel_size_and_call_depth_limits() {
    let (document, ids) = evaluator_fixture();
    let checked = check_program(document).expect("evaluator fixture checks");

    let no_fuel = Evaluator::with_limits(
        &checked,
        EvaluationLimits {
            fuel: 0,
            ..EvaluationLimits::default()
        },
    );
    assert_eq!(
        no_fuel.invoke_function(ids.short_function, &[]),
        EvaluationOutcome::Error(EvaluationError::FuelExhausted { limit: 0 })
    );

    let small_collections = Evaluator::with_limits(
        &checked,
        EvaluationLimits {
            collection_size: 1,
            ..EvaluationLimits::default()
        },
    );
    assert_eq!(
        small_collections.invoke_function(
            ids.first_function,
            &[Value::List(vec![Value::I64(1), Value::I64(2)])]
        ),
        EvaluationOutcome::Error(EvaluationError::CollectionLimitExceeded {
            limit: 1,
            requested: 2,
        })
    );

    let no_calls = Evaluator::with_limits(
        &checked,
        EvaluationLimits {
            call_depth: 0,
            ..EvaluationLimits::default()
        },
    );
    assert_eq!(
        no_calls.invoke_function(ids.short_function, &[]),
        EvaluationOutcome::Error(EvaluationError::CallDepthExceeded { limit: 0 })
    );
}
