use std::collections::BTreeSet;

use portable_diagnostics::DiagnosticCode;
use portable_ir::v0::*;

use super::*;

const EVERY_NODE_PATH: &str = "crates/ir/src/v0/testdata/every-node.poly.json";

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

    fn parameter(&mut self, name: &str, ty: TypeRef) -> Parameter {
        Parameter {
            header: self.member(name),
            ty,
        }
    }
}

struct PositiveIds {
    parameter_x: NodeId,
    local_x: NodeId,
    local_item: NodeId,
    checked_add: NodeId,
    method_call: NodeId,
}

fn positive_document(order_reversed: bool) -> (Document, PositiveIds) {
    let mut factory = Factory::new();
    let data_header = factory.declaration("Data");
    let data_id = data_header.node.id;
    let mut fields = Vec::new();
    for (name, ty) in [
        ("unit", TypeRef::Unit),
        ("enabled", TypeRef::Bool),
        ("small", TypeRef::I32),
        ("large", TypeRef::I64),
        ("ratio", TypeRef::F64),
        ("scalar", TypeRef::Char),
        ("text", TypeRef::String),
        ("raw", TypeRef::Bytes),
        ("items", TypeRef::List(Box::new(TypeRef::I64))),
        ("maybe", TypeRef::Option(Box::new(TypeRef::String))),
        (
            "outcome",
            TypeRef::Result {
                ok: Box::new(TypeRef::I64),
                error: Box::new(TypeRef::String),
            },
        ),
    ] {
        fields.push(FieldDeclaration {
            header: factory.member(name),
            ty,
        });
    }
    let data = Declaration::Record(RecordDeclaration {
        header: data_header,
        fields,
    });

    let choice_header = factory.declaration("Choice");
    let choice_id = choice_header.node.id;
    let off = EnumVariant {
        header: factory.member("Off"),
        fields: vec![],
    };
    let on = EnumVariant {
        header: factory.member("On"),
        fields: vec![FieldDeclaration {
            header: factory.member("message"),
            ty: TypeRef::String,
        }],
    };
    let choice = Declaration::Enum(EnumDeclaration {
        header: choice_header,
        variants: vec![off, on],
    });

    let alias = Declaration::Alias(AliasDeclaration {
        header: factory.declaration("Count"),
        target: TypeRef::I64,
    });

    let contract_header = factory.declaration("Label");
    let contract_id = contract_header.node.id;
    let required_header = factory.member("label");
    let required_id = required_header.node.id;
    let clone_required_header = factory.member("clone_data");
    let clone_required_id = clone_required_header.node.id;
    let contract = Declaration::Contract(ContractDeclaration {
        header: contract_header,
        methods: vec![
            MethodSignature {
                header: required_header,
                parameters: vec![factory.parameter("prefix", TypeRef::String)],
                return_type: TypeRef::String,
            },
            MethodSignature {
                header: clone_required_header,
                parameters: vec![],
                return_type: TypeRef::Named(data_id),
            },
        ],
    });

    let implementation_header = factory.declaration("DataLabel");
    let implementation_id = implementation_header.node.id;
    let method_header = factory.member("label");
    let method_id = method_header.node.id;
    let method_parameter = factory.parameter("prefix", TypeRef::String);
    let method_local = factory.local("prefix");
    let method_body = factory.block(method_local);
    let clone_method_header = factory.member("clone_data");
    let self_value = Expression::SelfValue {
        node: factory.node(),
    };
    let clone_body = factory.block(self_value);
    let implementation = Declaration::Implementation(ImplementationDeclaration {
        header: implementation_header,
        contract: contract_id,
        record: data_id,
        methods: vec![
            MethodImplementation {
                header: method_header,
                contract_method: required_id,
                parameters: vec![method_parameter],
                return_type: TypeRef::String,
                body: method_body,
            },
            MethodImplementation {
                header: clone_method_header,
                contract_method: clone_required_id,
                parameters: vec![],
                return_type: TypeRef::Named(data_id),
                body: clone_body,
            },
        ],
    });

    let constant_header = factory.declaration("LIMIT");
    let constant_id = constant_header.node.id;
    let constant_node = factory.node();
    let constant = Declaration::Constant(ConstantDeclaration {
        header: constant_header,
        ty: TypeRef::I64,
        value: ConstantExpression::Literal {
            node: constant_node,
            value: Value::I64(2),
        },
    });

    let compute_header = factory.declaration("compute");
    let compute_id = compute_header.node.id;
    let x_parameter = factory.parameter("x", TypeRef::I64);
    let x_symbol = x_parameter.header.node.id;
    let flag_parameter = factory.parameter("flag", TypeRef::Bool);
    let items_parameter = factory.parameter("items", TypeRef::List(Box::new(TypeRef::I64)));
    let add_node = factory.node();
    let x_local = factory.local("x");
    let local_x = x_local.node().id;
    let constant_expression = Expression::Constant {
        node: factory.node(),
        declaration: constant_id,
    };
    let add = Expression::Intrinsic {
        node: add_node.clone(),
        operation: Intrinsic::IntAddChecked,
        arguments: vec![x_local, constant_expression],
    };
    let let_y = Statement::Let {
        node: factory.node(),
        name: "y".to_owned(),
        annotation: Some(TypeRef::I64),
        value: add,
    };
    let iterable = factory.local("items");
    let item_local = factory.local("item");
    let local_item = item_local.node().id;
    let loop_body = Block {
        node: factory.node(),
        statements: vec![Statement::Expression {
            node: factory.node(),
            value: item_local,
        }],
        result: None,
    };
    let for_each = Statement::ForEach {
        node: factory.node(),
        binding: "item".to_owned(),
        iterable,
        body: loop_body,
    };
    let condition = factory.local("flag");
    let then_local = factory.local("y");
    let then_block = factory.block(then_local);
    let else_literal = factory.literal(Value::I64(0));
    let else_block = factory.block(else_literal);
    let result_if = Expression::If {
        node: factory.node(),
        condition: Box::new(condition),
        then_block: Box::new(then_block),
        else_block: Box::new(else_block),
    };
    let compute = Declaration::Function(FunctionDeclaration {
        header: compute_header,
        parameters: vec![x_parameter, flag_parameter, items_parameter],
        return_type: TypeRef::I64,
        body: Block {
            node: factory.node(),
            statements: vec![let_y, for_each],
            result: Some(Box::new(result_if)),
        },
    });

    let option_header = factory.declaration("option_value");
    let option_parameter = factory.parameter("value", TypeRef::Option(Box::new(TypeRef::I64)));
    let matched = factory.local("value");
    let none_body_value = factory.literal(Value::I64(0));
    let none_body = factory.block(none_body_value);
    let some_body_value = factory.local("inner");
    let some_body = factory.block(some_body_value);
    let option_match = Expression::Match {
        node: factory.node(),
        value: Box::new(matched),
        arms: vec![
            MatchArm {
                node: factory.node(),
                pattern: Pattern::None {
                    node: factory.node(),
                },
                body: none_body,
            },
            MatchArm {
                node: factory.node(),
                pattern: Pattern::Some {
                    node: factory.node(),
                    binding: "inner".to_owned(),
                },
                body: some_body,
            },
        ],
    };
    let option_body = factory.block(option_match);
    let option_function = Declaration::Function(FunctionDeclaration {
        header: option_header,
        parameters: vec![option_parameter],
        return_type: TypeRef::I64,
        body: option_body,
    });

    let method_function_header = factory.declaration("call_label");
    let data_parameter = factory.parameter("data", TypeRef::Named(data_id));
    let receiver = factory.local("data");
    let prefix = factory.literal(Value::String("id=".to_owned()));
    let method_call_node = factory.node();
    let method_call_id = method_call_node.id;
    let call = Expression::MethodCall {
        node: method_call_node,
        receiver: Box::new(receiver),
        dispatch: MethodDispatch::Concrete {
            implementation: implementation_id,
            method: method_id,
        },
        arguments: vec![prefix],
    };
    let call_body = factory.block(call);
    let method_function = Declaration::Function(FunctionDeclaration {
        header: method_function_header,
        parameters: vec![data_parameter],
        return_type: TypeRef::String,
        body: call_body,
    });

    let contract_function_header = factory.declaration("call_contract");
    let contract_parameter = factory.parameter("value", TypeRef::Contract(contract_id));
    let contract_receiver = factory.local("value");
    let contract_prefix = factory.literal(Value::String("contract=".to_owned()));
    let contract_call = Expression::MethodCall {
        node: factory.node(),
        receiver: Box::new(contract_receiver),
        dispatch: MethodDispatch::Contract {
            contract: contract_id,
            method: required_id,
        },
        arguments: vec![contract_prefix],
    };
    let contract_call_body = factory.block(contract_call);
    let contract_function = Declaration::Function(FunctionDeclaration {
        header: contract_function_header,
        parameters: vec![contract_parameter],
        return_type: TypeRef::String,
        body: contract_call_body,
    });

    let enum_header = factory.declaration("default_choice");
    let off_id = match &choice {
        Declaration::Enum(enumeration) => enumeration.variants[0].header.node.id,
        _ => unreachable!(),
    };
    let enum_construct = Expression::ConstructEnum {
        node: factory.node(),
        declaration: choice_id,
        variant: off_id,
        fields: vec![],
    };
    let enum_body = factory.block(enum_construct);
    let enum_function = Declaration::Function(FunctionDeclaration {
        header: enum_header,
        parameters: vec![],
        return_type: TypeRef::Named(choice_id),
        body: enum_body,
    });

    let mut declarations = vec![
        data,
        choice,
        alias,
        contract,
        implementation,
        constant,
        compute,
        option_function,
        method_function,
        contract_function,
        enum_function,
    ];
    for index in 0..10 {
        let test_header = factory.declaration(&format!("compute_{index}"));
        declarations.push(Declaration::Test(TestDeclaration {
            header: test_header,
            invocation: TestInvocation::Function {
                function: compute_id,
                arguments: vec![
                    TypedValue {
                        ty: TypeRef::I64,
                        value: Value::I64(index),
                    },
                    TypedValue {
                        ty: TypeRef::Bool,
                        value: Value::Bool(index % 2 == 0),
                    },
                    TypedValue {
                        ty: TypeRef::List(Box::new(TypeRef::I64)),
                        value: Value::List(vec![Value::I64(index)]),
                    },
                ],
            },
            expected: ExpectedOutcome::Value(TypedValue {
                ty: TypeRef::I64,
                value: Value::I64(0),
            }),
        }));
    }
    if order_reversed {
        declarations.reverse();
    }
    (
        Document::new(
            IrVersion::CURRENT,
            Module {
                name: "positive".to_owned(),
                declarations,
            },
        ),
        PositiveIds {
            parameter_x: x_symbol,
            local_x,
            local_item,
            checked_add: add_node.id,
            method_call: method_call_id,
        },
    )
}

fn codes(diagnostics: &[portable_diagnostics::Diagnostic]) -> BTreeSet<DiagnosticCode> {
    diagnostics
        .iter()
        .map(|diagnostic| diagnostic.code)
        .collect()
}

#[test]
fn positive_fixture_checks_ten_tests_and_exposes_typed_resolved_expressions() {
    let (document, ids) = positive_document(false);
    let checked = check_program(document).expect("positive fixture checks");

    assert_eq!(
        checked.resolved_local(ids.local_x).unwrap().node_id(),
        ids.parameter_x
    );
    assert!(checked.resolved_local(ids.local_item).unwrap().node_id().0 > 0);
    assert_eq!(
        checked.expression_type(ids.checked_add),
        Some(&TypeRef::I64)
    );
    assert_eq!(
        checked.expression_type(ids.method_call),
        Some(&TypeRef::String)
    );
    assert_eq!(
        checked
            .module()
            .declarations
            .iter()
            .filter(|declaration| matches!(declaration, Declaration::Test(_)))
            .count(),
        10
    );
}

#[test]
fn capability_sets_are_minimal_traceable_and_insertion_order_independent() {
    let (first, ids) = positive_document(false);
    let (reversed, _) = positive_document(true);
    let first = check_program(first).unwrap();
    let reversed = check_program(reversed).unwrap();

    assert_eq!(
        first.capabilities().program(),
        reversed.capabilities().program()
    );
    assert!(
        first
            .capabilities()
            .node(ids.checked_add)
            .unwrap()
            .contains(&Capability::CheckedIntegerArithmetic)
    );
    assert!(
        first
            .capabilities()
            .node(ids.method_call)
            .unwrap()
            .contains(&Capability::ContractDispatch)
    );
    for required in [
        Capability::Bytes,
        Capability::CheckedIntegerArithmetic,
        Capability::ContractDispatch,
        Capability::F64,
        Capability::ImmutableList,
        Capability::Option,
        Capability::Result,
        Capability::UnicodeScalar,
        Capability::BoundedIteration,
    ] {
        assert!(first.capabilities().program().contains(&required));
    }
    assert!(
        !first
            .capabilities()
            .program()
            .contains(&Capability::WrappingIntegerArithmetic)
    );
}

#[test]
fn exhaustive_schema_fixture_is_processed_without_panic() {
    let path = match (
        std::env::var_os("TEST_SRCDIR"),
        std::env::var_os("TEST_WORKSPACE"),
    ) {
        (Some(runfiles), Some(workspace)) => std::path::PathBuf::from(runfiles)
            .join(workspace)
            .join(EVERY_NODE_PATH),
        _ => std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../ir/src/v0/testdata/every-node.poly.json"),
    };
    let bytes = std::fs::read(path).expect("M02 exhaustive fixture is test data");
    let document = from_json(&bytes).expect("M02 exhaustive fixture parses");
    let diagnostics = check_program(document).unwrap_err();
    assert_eq!(
        codes(&diagnostics),
        BTreeSet::from([
            DiagnosticCode::InvalidContractPosition,
            DiagnosticCode::InvalidIdentifier,
            DiagnosticCode::InvalidInvocation,
        ])
    );
}

fn diagnostic_signatures(diagnostics: &[portable_diagnostics::Diagnostic]) -> Vec<String> {
    diagnostics
        .iter()
        .map(|diagnostic| {
            let source = diagnostic
                .labels
                .first()
                .map(|label| match &label.source {
                    SourceRef::Logical(path) => path.segments.join(" > "),
                    SourceRef::File(span) => {
                        format!("{}:{}..{}", span.file, span.start, span.end)
                    }
                })
                .unwrap_or_default();
            format!("{}@{source}", diagnostic.code)
        })
        .collect()
}

#[test]
fn structural_errors_are_rejected_before_semantic_checking() {
    let mut factory = Factory::new();
    let first = factory.declaration("One");
    let duplicate = first.clone();
    let document = Document::new(
        IrVersion::CURRENT,
        Module {
            name: "structure".to_owned(),
            declarations: vec![
                Declaration::Record(RecordDeclaration {
                    header: first,
                    fields: vec![],
                }),
                Declaration::Record(RecordDeclaration {
                    header: duplicate,
                    fields: vec![],
                }),
            ],
        },
    );
    let diagnostics = check_program(document).unwrap_err();
    assert_eq!(
        codes(&diagnostics),
        BTreeSet::from([DiagnosticCode::InvalidStructure])
    );
}

#[test]
fn resolution_alias_and_naming_errors_accumulate_with_stable_sources() {
    let mut factory = Factory::new();
    let first_header = factory.declaration("Duplicate");
    let second_header = factory.declaration("Duplicate");
    let invalid_header = factory.declaration("not-valid");
    let first_alias_header = factory.declaration("FirstAlias");
    let first_alias_id = first_alias_header.node.id;
    let second_alias_header = factory.declaration("SecondAlias");
    let second_alias_id = second_alias_header.node.id;
    let bad_field = FieldDeclaration {
        header: factory.member("missing"),
        ty: TypeRef::Named(NodeId(99_999)),
    };
    let document = Document::new(
        IrVersion::CURRENT,
        Module {
            name: "resolution".to_owned(),
            declarations: vec![
                Declaration::Record(RecordDeclaration {
                    header: first_header,
                    fields: vec![],
                }),
                Declaration::Record(RecordDeclaration {
                    header: second_header,
                    fields: vec![],
                }),
                Declaration::Record(RecordDeclaration {
                    header: invalid_header,
                    fields: vec![bad_field],
                }),
                Declaration::Alias(AliasDeclaration {
                    header: first_alias_header,
                    target: TypeRef::Named(second_alias_id),
                }),
                Declaration::Alias(AliasDeclaration {
                    header: second_alias_header,
                    target: TypeRef::Named(first_alias_id),
                }),
            ],
        },
    );
    let diagnostics = check_program(document).unwrap_err();
    assert_eq!(
        codes(&diagnostics),
        BTreeSet::from([
            DiagnosticCode::AliasCycle,
            DiagnosticCode::DuplicateDeclaration,
            DiagnosticCode::InvalidIdentifier,
            DiagnosticCode::UnresolvedReference,
        ])
    );
    assert_eq!(
        diagnostic_signatures(&diagnostics),
        vec![
            "P0102@node(2)",
            "P0100@node(3)",
            "P0103@node(4)",
            "P0103@node(4)",
            "P0103@node(5)",
            "P0103@node(5)",
            "P0101@node(6)",
        ]
    );
}

#[test]
fn type_invocation_and_control_flow_errors_accumulate() {
    let mut factory = Factory::new();
    let callee_header = factory.declaration("callee");
    let callee_id = callee_header.node.id;
    let callee_parameter = factory.parameter("value", TypeRef::I64);
    let callee_local = factory.local("value");
    let callee_body = factory.block(callee_local);
    let callee = Declaration::Function(FunctionDeclaration {
        header: callee_header,
        parameters: vec![callee_parameter],
        return_type: TypeRef::I64,
        body: callee_body,
    });

    let broken_header = factory.declaration("broken");
    let wrong_argument = factory.literal(Value::Bool(true));
    let bad_call = Expression::Call {
        node: factory.node(),
        function: callee_id,
        arguments: vec![wrong_argument],
    };
    let wrong_arity_call = Expression::Call {
        node: factory.node(),
        function: callee_id,
        arguments: vec![],
    };
    let mixed_left = factory.literal(Value::I32(1));
    let mixed_right = factory.literal(Value::I64(2));
    let mixed_integer = Expression::Intrinsic {
        node: factory.node(),
        operation: Intrinsic::IntAddChecked,
        arguments: vec![mixed_left, mixed_right],
    };
    let unknown = factory.local("missing");
    let broken = Declaration::Function(FunctionDeclaration {
        header: broken_header,
        parameters: vec![],
        return_type: TypeRef::I64,
        body: Block {
            node: factory.node(),
            statements: vec![
                Statement::Expression {
                    node: factory.node(),
                    value: bad_call,
                },
                Statement::Expression {
                    node: factory.node(),
                    value: wrong_arity_call,
                },
                Statement::Expression {
                    node: factory.node(),
                    value: mixed_integer,
                },
                Statement::Expression {
                    node: factory.node(),
                    value: unknown,
                },
            ],
            result: None,
        },
    });
    let diagnostics = check_program(Document::new(
        IrVersion::CURRENT,
        Module {
            name: "typing".to_owned(),
            declarations: vec![callee, broken],
        },
    ))
    .unwrap_err();
    assert_eq!(
        codes(&diagnostics),
        BTreeSet::from([
            DiagnosticCode::InvalidControlFlow,
            DiagnosticCode::InvalidInvocation,
            DiagnosticCode::UnresolvedReference,
        ])
    );
}

#[test]
fn match_exhaustiveness_and_duplicate_patterns_are_checked() {
    let mut factory = Factory::new();
    let function_header = factory.declaration("match_bool");
    let parameter = factory.parameter("value", TypeRef::Bool);
    let value = factory.local("value");
    let first_value = factory.literal(Value::I64(1));
    let first_body = factory.block(first_value);
    let duplicate_value = factory.literal(Value::I64(2));
    let duplicate_body = factory.block(duplicate_value);
    let expression = Expression::Match {
        node: factory.node(),
        value: Box::new(value),
        arms: vec![
            MatchArm {
                node: factory.node(),
                pattern: Pattern::Bool {
                    node: factory.node(),
                    value: true,
                },
                body: first_body,
            },
            MatchArm {
                node: factory.node(),
                pattern: Pattern::Bool {
                    node: factory.node(),
                    value: true,
                },
                body: duplicate_body,
            },
        ],
    };
    let body = factory.block(expression);
    let diagnostics = check_program(Document::new(
        IrVersion::CURRENT,
        Module {
            name: "matching".to_owned(),
            declarations: vec![Declaration::Function(FunctionDeclaration {
                header: function_header,
                parameters: vec![parameter],
                return_type: TypeRef::I64,
                body,
            })],
        },
    ))
    .unwrap_err();
    assert_eq!(
        codes(&diagnostics),
        BTreeSet::from([
            DiagnosticCode::NonExhaustiveMatch,
            DiagnosticCode::UnreachablePattern,
        ])
    );
}

#[test]
fn missing_and_wrong_contract_methods_are_rejected() {
    let mut factory = Factory::new();
    let record_header = factory.declaration("Record");
    let record_id = record_header.node.id;
    let contract_header = factory.declaration("Contract");
    let contract_id = contract_header.node.id;
    let method_header = factory.member("required");
    let method_id = method_header.node.id;
    let contract = Declaration::Contract(ContractDeclaration {
        header: contract_header,
        methods: vec![MethodSignature {
            header: method_header,
            parameters: vec![factory.parameter("value", TypeRef::I64)],
            return_type: TypeRef::I64,
        }],
    });
    let wrong_header = factory.member("required");
    let wrong_parameter = factory.parameter("value", TypeRef::Bool);
    let wrong_value = factory.literal(Value::Bool(false));
    let wrong_body = factory.block(wrong_value);
    let implementation = Declaration::Implementation(ImplementationDeclaration {
        header: factory.declaration("Implementation"),
        contract: contract_id,
        record: record_id,
        methods: vec![
            MethodImplementation {
                header: wrong_header,
                contract_method: method_id,
                parameters: vec![wrong_parameter],
                return_type: TypeRef::Bool,
                body: wrong_body,
            },
            MethodImplementation {
                header: factory.member("extra"),
                contract_method: NodeId(99_999),
                parameters: vec![],
                return_type: TypeRef::Unit,
                body: Block {
                    node: factory.node(),
                    statements: vec![],
                    result: None,
                },
            },
        ],
    });
    let missing = Declaration::Implementation(ImplementationDeclaration {
        header: factory.declaration("MissingImplementation"),
        contract: contract_id,
        record: record_id,
        methods: vec![],
    });
    let diagnostics = check_program(Document::new(
        IrVersion::CURRENT,
        Module {
            name: "contracts".to_owned(),
            declarations: vec![
                Declaration::Record(RecordDeclaration {
                    header: record_header,
                    fields: vec![],
                }),
                contract,
                implementation,
                missing,
            ],
        },
    ))
    .unwrap_err();
    assert!(codes(&diagnostics).contains(&DiagnosticCode::ContractNonconformance));
    assert!(codes(&diagnostics).contains(&DiagnosticCode::DuplicateDeclaration));
}

#[test]
fn contract_storage_and_return_positions_are_rejected() {
    let mut factory = Factory::new();
    let contract_header = factory.declaration("View");
    let contract_id = contract_header.node.id;
    let record_header = factory.declaration("Stored");
    let field = FieldDeclaration {
        header: factory.member("view"),
        ty: TypeRef::Contract(contract_id),
    };
    let function_header = factory.declaration("return_view");
    let equality_header = factory.declaration("compare_views");
    let left_parameter = factory.parameter("left", TypeRef::Contract(contract_id));
    let right_parameter = factory.parameter("right", TypeRef::Contract(contract_id));
    let left = factory.local("left");
    let right = factory.local("right");
    let equality = Expression::Intrinsic {
        node: factory.node(),
        operation: Intrinsic::Equal,
        arguments: vec![left, right],
    };
    let equality_body = factory.block(equality);
    let diagnostics = check_program(Document::new(
        IrVersion::CURRENT,
        Module {
            name: "positions".to_owned(),
            declarations: vec![
                Declaration::Contract(ContractDeclaration {
                    header: contract_header,
                    methods: vec![],
                }),
                Declaration::Record(RecordDeclaration {
                    header: record_header,
                    fields: vec![field],
                }),
                Declaration::Function(FunctionDeclaration {
                    header: function_header,
                    parameters: vec![],
                    return_type: TypeRef::Contract(contract_id),
                    body: Block {
                        node: factory.node(),
                        statements: vec![],
                        result: None,
                    },
                }),
                Declaration::Function(FunctionDeclaration {
                    header: equality_header,
                    parameters: vec![left_parameter, right_parameter],
                    return_type: TypeRef::Bool,
                    body: equality_body,
                }),
            ],
        },
    ))
    .unwrap_err();
    assert!(codes(&diagnostics).contains(&DiagnosticCode::InvalidContractPosition));
}

#[test]
fn plain_value_type_mismatches_use_the_stable_type_code() {
    let mut factory = Factory::new();
    let header = factory.declaration("WRONG");
    let literal_node = factory.node();
    let diagnostics = check_program(Document::new(
        IrVersion::CURRENT,
        Module {
            name: "mismatch".to_owned(),
            declarations: vec![Declaration::Constant(ConstantDeclaration {
                header,
                ty: TypeRef::I64,
                value: ConstantExpression::Literal {
                    node: literal_node,
                    value: Value::Bool(false),
                },
            })],
        },
    ))
    .unwrap_err();
    assert_eq!(
        codes(&diagnostics),
        BTreeSet::from([DiagnosticCode::TypeMismatch])
    );
}

#[test]
fn nested_branch_returns_satisfy_return_paths_and_make_following_code_unreachable() {
    let mut factory = Factory::new();
    let valid_header = factory.declaration("branch_return");
    let valid_parameter = factory.parameter("condition", TypeRef::Bool);
    let condition = factory.local("condition");
    let then_value = factory.literal(Value::I64(1));
    let else_value = factory.literal(Value::I64(2));
    let returning_if = Expression::If {
        node: factory.node(),
        condition: Box::new(condition),
        then_block: Box::new(Block {
            node: factory.node(),
            statements: vec![Statement::Return {
                node: factory.node(),
                value: Some(then_value),
            }],
            result: None,
        }),
        else_block: Box::new(Block {
            node: factory.node(),
            statements: vec![Statement::Return {
                node: factory.node(),
                value: Some(else_value),
            }],
            result: None,
        }),
    };
    let valid = Declaration::Function(FunctionDeclaration {
        header: valid_header,
        parameters: vec![valid_parameter],
        return_type: TypeRef::I64,
        body: Block {
            node: factory.node(),
            statements: vec![],
            result: Some(Box::new(returning_if)),
        },
    });
    check_program(Document::new(
        IrVersion::CURRENT,
        Module {
            name: "returns".to_owned(),
            declarations: vec![valid],
        },
    ))
    .expect("both returning branches satisfy the callable return path");

    let mut factory = Factory::new();
    let header = factory.declaration("unreachable_after_if");
    let parameter = factory.parameter("condition", TypeRef::Bool);
    let condition = factory.local("condition");
    let then_value = factory.literal(Value::Unit);
    let else_value = factory.literal(Value::Unit);
    let returning_if = Expression::If {
        node: factory.node(),
        condition: Box::new(condition),
        then_block: Box::new(Block {
            node: factory.node(),
            statements: vec![Statement::Return {
                node: factory.node(),
                value: Some(then_value),
            }],
            result: None,
        }),
        else_block: Box::new(Block {
            node: factory.node(),
            statements: vec![Statement::Return {
                node: factory.node(),
                value: Some(else_value),
            }],
            result: None,
        }),
    };
    let following = factory.literal(Value::Unit);
    let diagnostics = check_program(Document::new(
        IrVersion::CURRENT,
        Module {
            name: "unreachable".to_owned(),
            declarations: vec![Declaration::Function(FunctionDeclaration {
                header,
                parameters: vec![parameter],
                return_type: TypeRef::Unit,
                body: Block {
                    node: factory.node(),
                    statements: vec![
                        Statement::Expression {
                            node: factory.node(),
                            value: returning_if,
                        },
                        Statement::Expression {
                            node: factory.node(),
                            value: following,
                        },
                    ],
                    result: None,
                },
            })],
        },
    ))
    .unwrap_err();
    assert!(codes(&diagnostics).contains(&DiagnosticCode::InvalidControlFlow));
}

#[test]
fn invalid_portable_test_arity_and_expectation_are_rejected() {
    let mut factory = Factory::new();
    let function_header = factory.declaration("identity");
    let function_id = function_header.node.id;
    let parameter = factory.parameter("value", TypeRef::I64);
    let local = factory.local("value");
    let body = factory.block(local);
    let test_header = factory.declaration("invalid_test");
    let diagnostics = check_program(Document::new(
        IrVersion::CURRENT,
        Module {
            name: "tests".to_owned(),
            declarations: vec![
                Declaration::Function(FunctionDeclaration {
                    header: function_header,
                    parameters: vec![parameter],
                    return_type: TypeRef::I64,
                    body,
                }),
                Declaration::Test(TestDeclaration {
                    header: test_header,
                    invocation: TestInvocation::Function {
                        function: function_id,
                        arguments: vec![],
                    },
                    expected: ExpectedOutcome::Value(TypedValue {
                        ty: TypeRef::Bool,
                        value: Value::Bool(false),
                    }),
                }),
            ],
        },
    ))
    .unwrap_err();
    assert_eq!(
        codes(&diagnostics),
        BTreeSet::from([DiagnosticCode::InvalidPortableTest])
    );
}

#[test]
fn direct_and_indirect_recursion_are_rejected() {
    let mut factory = Factory::new();
    let first_header = factory.declaration("first");
    let first_id = first_header.node.id;
    let second_header = factory.declaration("second");
    let second_id = second_header.node.id;
    let first_call = Expression::Call {
        node: factory.node(),
        function: second_id,
        arguments: vec![],
    };
    let first_body = factory.block(first_call);
    let second_call = Expression::Call {
        node: factory.node(),
        function: first_id,
        arguments: vec![],
    };
    let second_body = factory.block(second_call);
    let diagnostics = check_program(Document::new(
        IrVersion::CURRENT,
        Module {
            name: "recursion".to_owned(),
            declarations: vec![
                Declaration::Function(FunctionDeclaration {
                    header: first_header,
                    parameters: vec![],
                    return_type: TypeRef::Bool,
                    body: first_body,
                }),
                Declaration::Function(FunctionDeclaration {
                    header: second_header,
                    parameters: vec![],
                    return_type: TypeRef::Bool,
                    body: second_body,
                }),
            ],
        },
    ))
    .unwrap_err();
    assert_eq!(
        codes(&diagnostics),
        BTreeSet::from([DiagnosticCode::RecursiveCall])
    );
    assert_eq!(diagnostics.len(), 2);
}

#[test]
fn bounded_hostile_expression_reports_complexity_instead_of_panicking() {
    let mut factory = Factory::new();
    let header = factory.declaration("deep");
    let mut expression = factory.literal(Value::I64(1));
    let mut ty = TypeRef::I64;
    for _ in 0..(super::checker::MAX_DEPTH + 8) {
        expression = Expression::ConstructSome {
            node: factory.node(),
            value: Box::new(expression),
        };
        ty = TypeRef::Option(Box::new(ty));
    }
    let body = factory.block(expression);
    let diagnostics = check_program(Document::new(
        IrVersion::CURRENT,
        Module {
            name: "hostile".to_owned(),
            declarations: vec![Declaration::Function(FunctionDeclaration {
                header,
                parameters: vec![],
                return_type: ty,
                body,
            })],
        },
    ))
    .unwrap_err();
    assert!(codes(&diagnostics).contains(&DiagnosticCode::ExcessiveComplexity));
}

#[test]
fn every_positive_expression_has_a_type_and_every_local_has_a_symbol() {
    let (document, _) = positive_document(false);
    let mut expression_ids = BTreeSet::new();
    for declaration in &document.module.declarations {
        match declaration {
            Declaration::Function(function) => {
                collect_block_expression_ids(&function.body, &mut expression_ids);
            }
            Declaration::Implementation(implementation) => {
                for method in &implementation.methods {
                    collect_block_expression_ids(&method.body, &mut expression_ids);
                }
            }
            _ => {}
        }
    }
    let checked = check_program(document).unwrap();
    let typed = checked
        .expression_types()
        .map(|(node, _)| node)
        .collect::<BTreeSet<_>>();
    assert_eq!(typed, expression_ids);
    assert!(
        checked
            .resolved_locals()
            .all(|(expression, symbol)| expression_ids.contains(&expression)
                && symbol.node_id().0 > 0)
    );
}

fn collect_block_expression_ids(block: &Block, ids: &mut BTreeSet<NodeId>) {
    for statement in &block.statements {
        match statement {
            Statement::Let { value, .. } | Statement::Expression { value, .. } => {
                collect_expression_ids(value, ids);
            }
            Statement::ForEach { iterable, body, .. } => {
                collect_expression_ids(iterable, ids);
                collect_block_expression_ids(body, ids);
            }
            Statement::Return { value, .. } => {
                if let Some(value) = value {
                    collect_expression_ids(value, ids);
                }
            }
        }
    }
    if let Some(result) = &block.result {
        collect_expression_ids(result, ids);
    }
}

fn collect_expression_ids(expression: &Expression, ids: &mut BTreeSet<NodeId>) {
    ids.insert(expression.node().id);
    match expression {
        Expression::ConstructRecord { fields, .. } | Expression::ConstructEnum { fields, .. } => {
            for field in fields {
                collect_expression_ids(&field.value, ids);
            }
        }
        Expression::ConstructSome { value, .. }
        | Expression::ConstructOk { value, .. }
        | Expression::ConstructErr { value, .. }
        | Expression::Field { base: value, .. } => collect_expression_ids(value, ids),
        Expression::ConstructList { elements, .. }
        | Expression::Call {
            arguments: elements,
            ..
        }
        | Expression::Intrinsic {
            arguments: elements,
            ..
        } => {
            for element in elements {
                collect_expression_ids(element, ids);
            }
        }
        Expression::MethodCall {
            receiver,
            arguments,
            ..
        } => {
            collect_expression_ids(receiver, ids);
            for argument in arguments {
                collect_expression_ids(argument, ids);
            }
        }
        Expression::If {
            condition,
            then_block,
            else_block,
            ..
        } => {
            collect_expression_ids(condition, ids);
            collect_block_expression_ids(then_block, ids);
            collect_block_expression_ids(else_block, ids);
        }
        Expression::Match { value, arms, .. } => {
            collect_expression_ids(value, ids);
            for arm in arms {
                collect_block_expression_ids(&arm.body, ids);
            }
        }
        Expression::Block(block) => collect_block_expression_ids(block, ids),
        Expression::Literal { .. }
        | Expression::Local { .. }
        | Expression::Constant { .. }
        | Expression::SelfValue { .. }
        | Expression::ConstructNone { .. } => {}
    }
}

#[test]
fn every_v0_intrinsic_has_a_checked_signature() {
    let mut factory = Factory::new();
    let mut declarations = Vec::new();
    for (index, (operation, arguments, result)) in intrinsic_cases().into_iter().enumerate() {
        let header = factory.declaration(&format!("intrinsic_{index}"));
        let mut parameters = Vec::new();
        let mut expressions = Vec::new();
        for (argument_index, ty) in arguments.into_iter().enumerate() {
            let name = format!("argument_{argument_index}");
            parameters.push(factory.parameter(&name, ty));
            expressions.push(factory.local(&name));
        }
        let expression = Expression::Intrinsic {
            node: factory.node(),
            operation,
            arguments: expressions,
        };
        let body = factory.block(expression);
        declarations.push(Declaration::Function(FunctionDeclaration {
            header,
            parameters,
            return_type: result,
            body,
        }));
    }
    let checked = check_program(Document::new(
        IrVersion::CURRENT,
        Module {
            name: "intrinsics".to_owned(),
            declarations,
        },
    ))
    .expect("every declared intrinsic signature checks");
    assert_eq!(checked.module().declarations.len(), ALL_INTRINSICS.len());
}

#[test]
fn string_replace_many_rejects_invalid_pair_shapes_and_operand_types() {
    let cases = [
        vec![],
        vec![Value::String("source".into())],
        vec![
            Value::String("source".into()),
            Value::String("needle".into()),
        ],
        vec![
            Value::String("source".into()),
            Value::String("needle".into()),
            Value::String("replacement".into()),
            Value::String("orphan".into()),
        ],
        vec![
            Value::String("source".into()),
            Value::Bool(true),
            Value::String("replacement".into()),
        ],
    ];
    for (index, values) in cases.into_iter().enumerate() {
        let mut factory = Factory::new();
        let arguments = values
            .into_iter()
            .map(|value| factory.literal(value))
            .collect();
        let expression = Expression::Intrinsic {
            node: factory.node(),
            operation: Intrinsic::StringReplaceMany,
            arguments,
        };
        let body = factory.block(expression);
        let declaration = Declaration::Function(FunctionDeclaration {
            header: factory.declaration(&format!("invalid_replace_many_{index}")),
            parameters: vec![],
            return_type: TypeRef::String,
            body,
        });
        let diagnostics = check_program(Document::new(
            IrVersion::CURRENT,
            Module {
                name: format!("invalid_replace_many_{index}"),
                declarations: vec![declaration],
            },
        ))
        .expect_err("invalid StringReplaceMany arguments must be diagnosed");
        assert!(
            codes(&diagnostics).contains(&DiagnosticCode::InvalidInvocation),
            "case {index}: {diagnostics:#?}"
        );
    }
}

#[test]
fn string_truncate_utf8_bytes_rejects_invalid_operand_shapes_and_types() {
    let cases = [
        vec![Value::String("source".into())],
        vec![Value::String("source".into()), Value::I64(2)],
        vec![
            Value::F64(F64Bits::from_f64(2.0)),
            Value::String("source".into()),
        ],
        vec![
            Value::String("source".into()),
            Value::F64(F64Bits::from_f64(2.0)),
            Value::Bool(true),
        ],
    ];
    for (index, values) in cases.into_iter().enumerate() {
        let mut factory = Factory::new();
        let arguments = values
            .into_iter()
            .map(|value| factory.literal(value))
            .collect();
        let expression = Expression::Intrinsic {
            node: factory.node(),
            operation: Intrinsic::StringTruncateUtf8Bytes,
            arguments,
        };
        let body = factory.block(expression);
        let declaration = Declaration::Function(FunctionDeclaration {
            header: factory.declaration(&format!("invalid_truncate_utf8_{index}")),
            parameters: vec![],
            return_type: TypeRef::String,
            body,
        });
        let diagnostics = check_program(Document::new(
            IrVersion::CURRENT,
            Module {
                name: format!("invalid_truncate_utf8_{index}"),
                declarations: vec![declaration],
            },
        ))
        .expect_err("invalid StringTruncateUtf8Bytes arguments must be diagnosed");
        assert!(
            codes(&diagnostics).contains(&DiagnosticCode::InvalidInvocation),
            "case {index}: {diagnostics:#?}"
        );
    }
}

const ALL_INTRINSICS: [Intrinsic; 63] = [
    Intrinsic::BoolNot,
    Intrinsic::BoolAnd,
    Intrinsic::BoolOr,
    Intrinsic::Equal,
    Intrinsic::NotEqual,
    Intrinsic::Less,
    Intrinsic::LessEqual,
    Intrinsic::Greater,
    Intrinsic::GreaterEqual,
    Intrinsic::IntNegChecked,
    Intrinsic::IntAddChecked,
    Intrinsic::IntSubChecked,
    Intrinsic::IntMulChecked,
    Intrinsic::IntDivChecked,
    Intrinsic::IntRemChecked,
    Intrinsic::IntNegWrapping,
    Intrinsic::IntAddWrapping,
    Intrinsic::IntSubWrapping,
    Intrinsic::IntMulWrapping,
    Intrinsic::IntBitNot,
    Intrinsic::IntBitAnd,
    Intrinsic::IntBitOr,
    Intrinsic::IntBitXor,
    Intrinsic::IntShiftLeftChecked,
    Intrinsic::IntShiftRightChecked,
    Intrinsic::FloatNeg,
    Intrinsic::FloatTrunc,
    Intrinsic::FloatIsNaN,
    Intrinsic::FloatAdd,
    Intrinsic::FloatSub,
    Intrinsic::FloatMul,
    Intrinsic::FloatDiv,
    Intrinsic::FloatRemTrunc,
    Intrinsic::StringConcat,
    Intrinsic::StringScalarLength,
    Intrinsic::StringIsEmpty,
    Intrinsic::StringContains,
    Intrinsic::StringStartsWith,
    Intrinsic::StringStripPrefix,
    Intrinsic::StringEndsWith,
    Intrinsic::StringReplaceAll,
    Intrinsic::StringReplaceMany,
    Intrinsic::StringTruncateUtf8Bytes,
    Intrinsic::StringTrimStart,
    Intrinsic::StringTrimEnd,
    Intrinsic::BytesConcat,
    Intrinsic::BytesLength,
    Intrinsic::BytesIsEmpty,
    Intrinsic::ListLength,
    Intrinsic::ListIsEmpty,
    Intrinsic::ListGetChecked,
    Intrinsic::ListAppend,
    Intrinsic::ListConcat,
    Intrinsic::ListContains,
    Intrinsic::OptionIsSome,
    Intrinsic::OptionIsNone,
    Intrinsic::OptionUnwrapOr,
    Intrinsic::ResultIsOk,
    Intrinsic::ResultIsErr,
    Intrinsic::WidenI32ToI64,
    Intrinsic::NarrowI64ToI32Checked,
    Intrinsic::StringToUtf8,
    Intrinsic::StringFromUtf8Checked,
];

fn intrinsic_cases() -> Vec<(Intrinsic, Vec<TypeRef>, TypeRef)> {
    use Intrinsic::*;
    let list = TypeRef::List(Box::new(TypeRef::I64));
    let option = TypeRef::Option(Box::new(TypeRef::I64));
    let result = TypeRef::Result {
        ok: Box::new(TypeRef::I64),
        error: Box::new(TypeRef::String),
    };
    vec![
        (BoolNot, vec![TypeRef::Bool], TypeRef::Bool),
        (BoolAnd, vec![TypeRef::Bool, TypeRef::Bool], TypeRef::Bool),
        (BoolOr, vec![TypeRef::Bool, TypeRef::Bool], TypeRef::Bool),
        (Equal, vec![TypeRef::I64, TypeRef::I64], TypeRef::Bool),
        (
            NotEqual,
            vec![TypeRef::String, TypeRef::String],
            TypeRef::Bool,
        ),
        (Less, vec![TypeRef::I64, TypeRef::I64], TypeRef::Bool),
        (LessEqual, vec![TypeRef::I64, TypeRef::I64], TypeRef::Bool),
        (Greater, vec![TypeRef::I64, TypeRef::I64], TypeRef::Bool),
        (
            GreaterEqual,
            vec![TypeRef::I64, TypeRef::I64],
            TypeRef::Bool,
        ),
        (IntNegChecked, vec![TypeRef::I64], TypeRef::I64),
        (
            IntAddChecked,
            vec![TypeRef::I64, TypeRef::I64],
            TypeRef::I64,
        ),
        (
            IntSubChecked,
            vec![TypeRef::I64, TypeRef::I64],
            TypeRef::I64,
        ),
        (
            IntMulChecked,
            vec![TypeRef::I64, TypeRef::I64],
            TypeRef::I64,
        ),
        (
            IntDivChecked,
            vec![TypeRef::I64, TypeRef::I64],
            TypeRef::I64,
        ),
        (
            IntRemChecked,
            vec![TypeRef::I64, TypeRef::I64],
            TypeRef::I64,
        ),
        (IntNegWrapping, vec![TypeRef::I32], TypeRef::I32),
        (
            IntAddWrapping,
            vec![TypeRef::I32, TypeRef::I32],
            TypeRef::I32,
        ),
        (
            IntSubWrapping,
            vec![TypeRef::I32, TypeRef::I32],
            TypeRef::I32,
        ),
        (
            IntMulWrapping,
            vec![TypeRef::I32, TypeRef::I32],
            TypeRef::I32,
        ),
        (IntBitNot, vec![TypeRef::I32], TypeRef::I32),
        (IntBitAnd, vec![TypeRef::I32, TypeRef::I32], TypeRef::I32),
        (IntBitOr, vec![TypeRef::I32, TypeRef::I32], TypeRef::I32),
        (IntBitXor, vec![TypeRef::I32, TypeRef::I32], TypeRef::I32),
        (
            IntShiftLeftChecked,
            vec![TypeRef::I32, TypeRef::I32],
            TypeRef::I32,
        ),
        (
            IntShiftRightChecked,
            vec![TypeRef::I32, TypeRef::I32],
            TypeRef::I32,
        ),
        (FloatNeg, vec![TypeRef::F64], TypeRef::F64),
        (FloatTrunc, vec![TypeRef::F64], TypeRef::F64),
        (FloatIsNaN, vec![TypeRef::F64], TypeRef::Bool),
        (FloatAdd, vec![TypeRef::F64, TypeRef::F64], TypeRef::F64),
        (FloatSub, vec![TypeRef::F64, TypeRef::F64], TypeRef::F64),
        (FloatMul, vec![TypeRef::F64, TypeRef::F64], TypeRef::F64),
        (FloatDiv, vec![TypeRef::F64, TypeRef::F64], TypeRef::F64),
        (
            FloatRemTrunc,
            vec![TypeRef::F64, TypeRef::F64],
            TypeRef::F64,
        ),
        (
            StringConcat,
            vec![TypeRef::String, TypeRef::String],
            TypeRef::String,
        ),
        (StringScalarLength, vec![TypeRef::String], TypeRef::I64),
        (StringIsEmpty, vec![TypeRef::String], TypeRef::Bool),
        (
            StringContains,
            vec![TypeRef::String, TypeRef::String],
            TypeRef::Bool,
        ),
        (
            StringStartsWith,
            vec![TypeRef::String, TypeRef::String],
            TypeRef::Bool,
        ),
        (
            StringStripPrefix,
            vec![TypeRef::String, TypeRef::String],
            TypeRef::String,
        ),
        (
            StringEndsWith,
            vec![TypeRef::String, TypeRef::String],
            TypeRef::Bool,
        ),
        (
            StringReplaceAll,
            vec![TypeRef::String, TypeRef::String, TypeRef::String],
            TypeRef::String,
        ),
        (
            StringReplaceMany,
            vec![
                TypeRef::String,
                TypeRef::String,
                TypeRef::String,
                TypeRef::String,
                TypeRef::String,
            ],
            TypeRef::String,
        ),
        (
            StringTruncateUtf8Bytes,
            vec![TypeRef::String, TypeRef::F64],
            TypeRef::String,
        ),
        (
            StringTrimStart,
            vec![TypeRef::String, TypeRef::String],
            TypeRef::String,
        ),
        (
            StringTrimEnd,
            vec![TypeRef::String, TypeRef::String],
            TypeRef::String,
        ),
        (
            BytesConcat,
            vec![TypeRef::Bytes, TypeRef::Bytes],
            TypeRef::Bytes,
        ),
        (BytesLength, vec![TypeRef::Bytes], TypeRef::I64),
        (BytesIsEmpty, vec![TypeRef::Bytes], TypeRef::Bool),
        (ListLength, vec![list.clone()], TypeRef::I64),
        (ListIsEmpty, vec![list.clone()], TypeRef::Bool),
        (
            ListGetChecked,
            vec![list.clone(), TypeRef::I64],
            TypeRef::I64,
        ),
        (ListAppend, vec![list.clone(), TypeRef::I64], list.clone()),
        (ListConcat, vec![list.clone(), list.clone()], list.clone()),
        (
            ListContains,
            vec![list.clone(), TypeRef::I64],
            TypeRef::Bool,
        ),
        (OptionIsSome, vec![option.clone()], TypeRef::Bool),
        (OptionIsNone, vec![option.clone()], TypeRef::Bool),
        (OptionUnwrapOr, vec![option, TypeRef::I64], TypeRef::I64),
        (ResultIsOk, vec![result.clone()], TypeRef::Bool),
        (ResultIsErr, vec![result], TypeRef::Bool),
        (WidenI32ToI64, vec![TypeRef::I32], TypeRef::I64),
        (NarrowI64ToI32Checked, vec![TypeRef::I64], TypeRef::I32),
        (StringToUtf8, vec![TypeRef::String], TypeRef::Bytes),
        (StringFromUtf8Checked, vec![TypeRef::Bytes], TypeRef::String),
    ]
}

#[test]
fn all_expression_constructors_patterns_and_statement_forms_check() {
    let mut factory = Factory::new();
    let record_header = factory.declaration("Pair");
    let record_id = record_header.node.id;
    let field = FieldDeclaration {
        header: factory.member("value"),
        ty: TypeRef::I64,
    };
    let field_id = field.header.node.id;
    let enumeration_header = factory.declaration("Only");
    let enumeration_id = enumeration_header.node.id;
    let variant = EnumVariant {
        header: factory.member("Item"),
        fields: vec![FieldDeclaration {
            header: factory.member("value"),
            ty: TypeRef::I64,
        }],
    };
    let variant_id = variant.header.node.id;
    let variant_field_id = variant.fields[0].header.node.id;
    let mut declarations = vec![
        Declaration::Record(RecordDeclaration {
            header: record_header,
            fields: vec![field],
        }),
        Declaration::Enum(EnumDeclaration {
            header: enumeration_header,
            variants: vec![variant],
        }),
    ];

    let identity_header = factory.declaration("identity");
    let identity_id = identity_header.node.id;
    let identity_parameter = factory.parameter("value", TypeRef::I64);
    let identity_local = factory.local("value");
    let identity_body = factory.block(identity_local);
    declarations.push(Declaration::Function(FunctionDeclaration {
        header: identity_header,
        parameters: vec![identity_parameter],
        return_type: TypeRef::I64,
        body: identity_body,
    }));

    let construct_record = Expression::ConstructRecord {
        node: factory.node(),
        declaration: record_id,
        fields: vec![ExpressionField {
            field: field_id,
            value: factory.literal(Value::I64(7)),
        }],
    };
    let record_body = factory.block(construct_record);
    declarations.push(Declaration::Function(FunctionDeclaration {
        header: factory.declaration("make_record"),
        parameters: vec![],
        return_type: TypeRef::Named(record_id),
        body: record_body,
    }));

    let field_base = Expression::ConstructRecord {
        node: factory.node(),
        declaration: record_id,
        fields: vec![ExpressionField {
            field: field_id,
            value: factory.literal(Value::I64(8)),
        }],
    };
    let field_expression = Expression::Field {
        node: factory.node(),
        base: Box::new(field_base),
        field: field_id,
    };
    let field_body = factory.block(field_expression);
    declarations.push(Declaration::Function(FunctionDeclaration {
        header: factory.declaration("read_field"),
        parameters: vec![],
        return_type: TypeRef::I64,
        body: field_body,
    }));

    let constructors = vec![
        (
            "make_list",
            TypeRef::List(Box::new(TypeRef::I64)),
            Expression::ConstructList {
                node: factory.node(),
                element_type: TypeRef::I64,
                elements: vec![factory.literal(Value::I64(1))],
            },
        ),
        (
            "make_none",
            TypeRef::Option(Box::new(TypeRef::I64)),
            Expression::ConstructNone {
                node: factory.node(),
                inner_type: TypeRef::I64,
            },
        ),
        (
            "make_some",
            TypeRef::Option(Box::new(TypeRef::I64)),
            Expression::ConstructSome {
                node: factory.node(),
                value: Box::new(factory.literal(Value::I64(1))),
            },
        ),
        (
            "make_ok",
            TypeRef::Result {
                ok: Box::new(TypeRef::I64),
                error: Box::new(TypeRef::String),
            },
            Expression::ConstructOk {
                node: factory.node(),
                value: Box::new(factory.literal(Value::I64(1))),
                error_type: TypeRef::String,
            },
        ),
        (
            "make_err",
            TypeRef::Result {
                ok: Box::new(TypeRef::I64),
                error: Box::new(TypeRef::String),
            },
            Expression::ConstructErr {
                node: factory.node(),
                value: Box::new(factory.literal(Value::String("error".to_owned()))),
                ok_type: TypeRef::I64,
            },
        ),
        (
            "make_enum",
            TypeRef::Named(enumeration_id),
            Expression::ConstructEnum {
                node: factory.node(),
                declaration: enumeration_id,
                variant: variant_id,
                fields: vec![ExpressionField {
                    field: variant_field_id,
                    value: factory.literal(Value::I64(1)),
                }],
            },
        ),
    ];
    for (name, return_type, expression) in constructors {
        let body = factory.block(expression);
        declarations.push(Declaration::Function(FunctionDeclaration {
            header: factory.declaration(name),
            parameters: vec![],
            return_type,
            body,
        }));
    }

    let call_argument = factory.literal(Value::I64(3));
    let call = Expression::Call {
        node: factory.node(),
        function: identity_id,
        arguments: vec![call_argument],
    };
    let call_body = factory.block(call);
    declarations.push(Declaration::Function(FunctionDeclaration {
        header: factory.declaration("call_identity"),
        parameters: vec![],
        return_type: TypeRef::I64,
        body: call_body,
    }));

    let nested_value = factory.literal(Value::I64(4));
    let nested_block = factory.block(nested_value);
    let nested = Expression::Block(Box::new(nested_block));
    let nested_body = factory.block(nested);
    declarations.push(Declaration::Function(FunctionDeclaration {
        header: factory.declaration("nested_block"),
        parameters: vec![],
        return_type: TypeRef::I64,
        body: nested_body,
    }));

    let return_value = factory.literal(Value::I64(5));
    let return_statement = Statement::Return {
        node: factory.node(),
        value: Some(return_value),
    };
    declarations.push(Declaration::Function(FunctionDeclaration {
        header: factory.declaration("explicit_return"),
        parameters: vec![],
        return_type: TypeRef::I64,
        body: Block {
            node: factory.node(),
            statements: vec![return_statement],
            result: None,
        },
    }));

    let enum_parameter = factory.parameter("input", TypeRef::Named(enumeration_id));
    let enum_value = factory.local("input");
    let enum_binding_value = factory.local("bound");
    let enum_arm_body = factory.block(enum_binding_value);
    let enum_match = Expression::Match {
        node: factory.node(),
        value: Box::new(enum_value),
        arms: vec![MatchArm {
            node: factory.node(),
            pattern: Pattern::EnumVariant {
                node: factory.node(),
                declaration: enumeration_id,
                variant: variant_id,
                bindings: vec![FieldBinding {
                    field: variant_field_id,
                    binding: "bound".to_owned(),
                }],
            },
            body: enum_arm_body,
        }],
    };
    let enum_match_body = factory.block(enum_match);
    declarations.push(Declaration::Function(FunctionDeclaration {
        header: factory.declaration("match_enum"),
        parameters: vec![enum_parameter],
        return_type: TypeRef::I64,
        body: enum_match_body,
    }));

    let result_type = TypeRef::Result {
        ok: Box::new(TypeRef::I64),
        error: Box::new(TypeRef::I64),
    };
    let result_parameter = factory.parameter("input", result_type.clone());
    let result_value = factory.local("input");
    let ok_value = factory.local("ok_value");
    let ok_body = factory.block(ok_value);
    let error_value = factory.local("error_value");
    let error_body = factory.block(error_value);
    let result_match = Expression::Match {
        node: factory.node(),
        value: Box::new(result_value),
        arms: vec![
            MatchArm {
                node: factory.node(),
                pattern: Pattern::Ok {
                    node: factory.node(),
                    binding: "ok_value".to_owned(),
                },
                body: ok_body,
            },
            MatchArm {
                node: factory.node(),
                pattern: Pattern::Err {
                    node: factory.node(),
                    binding: "error_value".to_owned(),
                },
                body: error_body,
            },
        ],
    };
    let result_match_body = factory.block(result_match);
    declarations.push(Declaration::Function(FunctionDeclaration {
        header: factory.declaration("match_result"),
        parameters: vec![result_parameter],
        return_type: TypeRef::I64,
        body: result_match_body,
    }));

    check_program(Document::new(
        IrVersion::CURRENT,
        Module {
            name: "constructors".to_owned(),
            declarations,
        },
    ))
    .expect("all v0 constructors, patterns, calls, blocks, fields, and returns check");
}

#[test]
fn every_constant_expression_form_checks_and_constant_cycles_fail() {
    let mut factory = Factory::new();
    let record_header = factory.declaration("Boxed");
    let record_id = record_header.node.id;
    let record_field = FieldDeclaration {
        header: factory.member("value"),
        ty: TypeRef::I64,
    };
    let record_field_id = record_field.header.node.id;
    let enum_header = factory.declaration("Tagged");
    let enum_id = enum_header.node.id;
    let variant = EnumVariant {
        header: factory.member("Value"),
        fields: vec![FieldDeclaration {
            header: factory.member("value"),
            ty: TypeRef::I64,
        }],
    };
    let variant_id = variant.header.node.id;
    let variant_field_id = variant.fields[0].header.node.id;
    let literal_header = factory.declaration("LITERAL");
    let literal_id = literal_header.node.id;
    let declarations = vec![
        Declaration::Record(RecordDeclaration {
            header: record_header,
            fields: vec![record_field],
        }),
        Declaration::Enum(EnumDeclaration {
            header: enum_header,
            variants: vec![variant],
        }),
        Declaration::Constant(ConstantDeclaration {
            header: literal_header,
            ty: TypeRef::I64,
            value: ConstantExpression::Literal {
                node: factory.node(),
                value: Value::I64(1),
            },
        }),
        Declaration::Constant(ConstantDeclaration {
            header: factory.declaration("REFERENCE"),
            ty: TypeRef::I64,
            value: ConstantExpression::Reference {
                node: factory.node(),
                declaration: literal_id,
            },
        }),
        Declaration::Constant(ConstantDeclaration {
            header: factory.declaration("RECORD"),
            ty: TypeRef::Named(record_id),
            value: ConstantExpression::Record {
                node: factory.node(),
                declaration: record_id,
                fields: vec![ConstantField {
                    field: record_field_id,
                    value: ConstantExpression::Literal {
                        node: factory.node(),
                        value: Value::I64(1),
                    },
                }],
            },
        }),
        Declaration::Constant(ConstantDeclaration {
            header: factory.declaration("ENUM"),
            ty: TypeRef::Named(enum_id),
            value: ConstantExpression::Enum {
                node: factory.node(),
                declaration: enum_id,
                variant: variant_id,
                fields: vec![ConstantField {
                    field: variant_field_id,
                    value: ConstantExpression::Literal {
                        node: factory.node(),
                        value: Value::I64(1),
                    },
                }],
            },
        }),
        Declaration::Constant(ConstantDeclaration {
            header: factory.declaration("SOME"),
            ty: TypeRef::Option(Box::new(TypeRef::I64)),
            value: ConstantExpression::Some {
                node: factory.node(),
                value: Box::new(ConstantExpression::Literal {
                    node: factory.node(),
                    value: Value::I64(1),
                }),
            },
        }),
        Declaration::Constant(ConstantDeclaration {
            header: factory.declaration("NONE"),
            ty: TypeRef::Option(Box::new(TypeRef::I64)),
            value: ConstantExpression::None {
                node: factory.node(),
                inner_type: TypeRef::I64,
            },
        }),
        Declaration::Constant(ConstantDeclaration {
            header: factory.declaration("OK"),
            ty: TypeRef::Result {
                ok: Box::new(TypeRef::I64),
                error: Box::new(TypeRef::String),
            },
            value: ConstantExpression::Ok {
                node: factory.node(),
                value: Box::new(ConstantExpression::Literal {
                    node: factory.node(),
                    value: Value::I64(1),
                }),
                error_type: TypeRef::String,
            },
        }),
        Declaration::Constant(ConstantDeclaration {
            header: factory.declaration("ERR"),
            ty: TypeRef::Result {
                ok: Box::new(TypeRef::I64),
                error: Box::new(TypeRef::String),
            },
            value: ConstantExpression::Err {
                node: factory.node(),
                value: Box::new(ConstantExpression::Literal {
                    node: factory.node(),
                    value: Value::String("error".to_owned()),
                }),
                ok_type: TypeRef::I64,
            },
        }),
        Declaration::Constant(ConstantDeclaration {
            header: factory.declaration("LIST"),
            ty: TypeRef::List(Box::new(TypeRef::I64)),
            value: ConstantExpression::List {
                node: factory.node(),
                element_type: TypeRef::I64,
                elements: vec![ConstantExpression::Literal {
                    node: factory.node(),
                    value: Value::I64(1),
                }],
            },
        }),
        Declaration::Constant(ConstantDeclaration {
            header: factory.declaration("INTRINSIC"),
            ty: TypeRef::I64,
            value: ConstantExpression::Intrinsic {
                node: factory.node(),
                operation: Intrinsic::IntAddChecked,
                arguments: vec![
                    ConstantExpression::Literal {
                        node: factory.node(),
                        value: Value::I64(1),
                    },
                    ConstantExpression::Literal {
                        node: factory.node(),
                        value: Value::I64(2),
                    },
                ],
            },
        }),
    ];
    check_program(Document::new(
        IrVersion::CURRENT,
        Module {
            name: "constants".to_owned(),
            declarations,
        },
    ))
    .expect("every constant expression form checks");

    let mut factory = Factory::new();
    let first_header = factory.declaration("FIRST");
    let first_id = first_header.node.id;
    let second_header = factory.declaration("SECOND");
    let second_id = second_header.node.id;
    let cycle = Document::new(
        IrVersion::CURRENT,
        Module {
            name: "constant_cycle".to_owned(),
            declarations: vec![
                Declaration::Constant(ConstantDeclaration {
                    header: first_header,
                    ty: TypeRef::I64,
                    value: ConstantExpression::Reference {
                        node: factory.node(),
                        declaration: second_id,
                    },
                }),
                Declaration::Constant(ConstantDeclaration {
                    header: second_header,
                    ty: TypeRef::I64,
                    value: ConstantExpression::Reference {
                        node: factory.node(),
                        declaration: first_id,
                    },
                }),
            ],
        },
    );
    let diagnostics = check_program(cycle).unwrap_err();
    assert_eq!(
        codes(&diagnostics),
        BTreeSet::from([DiagnosticCode::RecursiveCall])
    );
}
