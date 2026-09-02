use std::{collections::BTreeMap, panic};

use serde::{Serialize, de::DeserializeOwned};

use super::*;

fn source(label: &str) -> SourceRef {
    SourceRef::logical([label])
}

fn node(id: u64) -> NodeMeta {
    NodeMeta::new(NodeId::new(id), source(&format!("node({id})")))
}

fn declaration_header(id: u64, name: &str) -> DeclarationHeader {
    DeclarationHeader {
        node: node(id),
        name: name.into(),
        visibility: Visibility::Public,
        documentation: vec![format!("Documentation for {name}.")],
    }
}

fn member_header(id: u64, name: &str) -> MemberHeader {
    MemberHeader {
        node: node(id),
        name: name.into(),
        documentation: vec![],
    }
}

fn literal(id: u64, value: Value) -> Expression {
    Expression::Literal {
        node: node(id),
        value,
    }
}

fn empty_block(id: u64, result: Option<Expression>) -> Block {
    Block {
        node: node(id),
        statements: vec![],
        result: result.map(Box::new),
    }
}

fn round_trip<T>(value: &T)
where
    T: Serialize + DeserializeOwned + std::fmt::Debug + Eq,
{
    let bytes = serde_json::to_vec(value).unwrap();
    assert_eq!(&serde_json::from_slice::<T>(&bytes).unwrap(), value);
}

fn exhaustive_document() -> Document {
    let record = RecordDeclaration {
        header: declaration_header(30, "Everything"),
        fields: vec![
            (301, "unit", TypeRef::Unit),
            (302, "flag", TypeRef::Bool),
            (303, "small", TypeRef::I32),
            (304, "large", TypeRef::I64),
            (305, "float", TypeRef::F64),
            (306, "character", TypeRef::Char),
            (307, "text", TypeRef::String),
            (308, "bytes", TypeRef::Bytes),
            (309, "items", TypeRef::List(Box::new(TypeRef::I64))),
            (310, "maybe", TypeRef::Option(Box::new(TypeRef::String))),
            (
                311,
                "outcome",
                TypeRef::Result {
                    ok: Box::new(TypeRef::I64),
                    error: Box::new(TypeRef::String),
                },
            ),
            (312, "alias", TypeRef::Named(NodeId(20))),
            (313, "validator", TypeRef::Contract(NodeId(50))),
        ]
        .into_iter()
        .map(|(id, name, ty)| FieldDeclaration {
            header: member_header(id, name),
            ty,
        })
        .collect(),
    };

    let enum_declaration = EnumDeclaration {
        header: declaration_header(40, "Choice"),
        variants: vec![
            EnumVariant {
                header: member_header(401, "Empty"),
                fields: vec![],
            },
            EnumVariant {
                header: member_header(402, "Named"),
                fields: vec![FieldDeclaration {
                    header: member_header(403, "name"),
                    ty: TypeRef::String,
                }],
            },
        ],
    };

    let contract = ContractDeclaration {
        header: declaration_header(50, "Validator"),
        methods: vec![MethodSignature {
            header: member_header(501, "accepts"),
            parameters: vec![Parameter {
                header: member_header(502, "value"),
                ty: TypeRef::Named(NodeId(30)),
            }],
            return_type: TypeRef::Bool,
        }],
    };

    let implementation = ImplementationDeclaration {
        header: declaration_header(60, "EverythingValidator"),
        contract: NodeId(50),
        record: NodeId(30),
        methods: vec![MethodImplementation {
            header: member_header(601, "accepts"),
            contract_method: NodeId(501),
            parameters: vec![Parameter {
                header: member_header(602, "value"),
                ty: TypeRef::Named(NodeId(30)),
            }],
            return_type: TypeRef::Bool,
            body: empty_block(603, Some(literal(604, Value::Bool(true)))),
        }],
    };

    let function = FunctionDeclaration {
        header: declaration_header(70, "run"),
        parameters: vec![Parameter {
            header: member_header(701, "validator"),
            ty: TypeRef::Contract(NodeId(50)),
        }],
        return_type: TypeRef::Bool,
        body: Block {
            node: node(702),
            statements: vec![Statement::Let {
                node: node(703),
                name: "answer".into(),
                annotation: Some(TypeRef::Bool),
                value: literal(704, Value::Bool(true)),
            }],
            result: Some(Box::new(Expression::Local {
                node: node(705),
                name: "answer".into(),
            })),
        },
    };

    let constant = ConstantDeclaration {
        header: declaration_header(10, "DEFAULT_CHOICE"),
        ty: TypeRef::Named(NodeId(40)),
        value: ConstantExpression::Enum {
            node: node(101),
            declaration: NodeId(40),
            variant: NodeId(402),
            fields: vec![ConstantField {
                field: NodeId(403),
                value: ConstantExpression::Literal {
                    node: node(102),
                    value: Value::String("Chloë 🦀".into()),
                },
            }],
        },
    };

    let test = TestDeclaration {
        header: declaration_header(80, "run returns true"),
        invocation: TestInvocation::Function {
            function: NodeId(70),
            arguments: vec![TypedValue {
                ty: TypeRef::Named(NodeId(30)),
                value: Value::Record {
                    declaration: NodeId(30),
                    fields: vec![],
                },
            }],
        },
        expected: ExpectedOutcome::Value(TypedValue {
            ty: TypeRef::Bool,
            value: Value::Bool(true),
        }),
    };

    let mut document = Document::new(
        IrVersion::CURRENT,
        Module {
            name: "every_node".into(),
            declarations: vec![
                Declaration::Test(test),
                Declaration::Function(function),
                Declaration::Implementation(implementation),
                Declaration::Contract(contract),
                Declaration::Enum(enum_declaration),
                Declaration::Record(record),
                Declaration::Alias(AliasDeclaration {
                    header: declaration_header(20, "Identifier"),
                    target: TypeRef::I64,
                }),
                Declaration::Constant(constant),
            ],
        },
    );
    document.metadata = BTreeMap::from([
        ("producer".into(), "polyrust-tests".into()),
        ("purpose".into(), "canonical-golden".into()),
    ]);
    document
}

#[test]
fn version_parsing_and_compatibility_are_explicit() {
    assert_eq!("0.1.0".parse(), Ok(IrVersion::CURRENT));
    assert!(
        "0.2.9"
            .parse::<IrVersion>()
            .unwrap()
            .is_compatible_with(IrVersion::CURRENT)
    );
    assert!(
        !"1.0.0"
            .parse::<IrVersion>()
            .unwrap()
            .is_compatible_with(IrVersion::CURRENT)
    );
    for invalid in ["", "1", "1.2", "1.2.3.4", "01.2.3", "a.2.3"] {
        assert!(
            invalid.parse::<IrVersion>().is_err(),
            "accepted {invalid:?}"
        );
    }
}

#[test]
fn exhaustive_fixture_round_trips_and_matches_golden() {
    let document = exhaustive_document();
    let bytes = to_canonical_json(&document).unwrap();
    if std::env::var_os("POLYRUST_UPDATE_GOLDEN").is_some() {
        std::fs::write(
            concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/src/v0/testdata/every-node.poly.json"
            ),
            &bytes,
        )
        .unwrap();
    }
    let reparsed = from_json(&bytes).unwrap();
    let mut expected = document.clone();
    expected
        .module
        .declarations
        .sort_by_key(|item| item.header().node.id);
    assert_eq!(reparsed, expected);
    assert_eq!(
        bytes.as_slice(),
        include_bytes!("testdata/every-node.poly.json")
    );
}

#[test]
fn declaration_insertion_order_cannot_change_canonical_bytes() {
    let original = exhaustive_document();
    let expected = to_canonical_json(&original).unwrap();
    let mut seed = 0x5eed_u64;
    for iteration in 0..64 {
        let mut shuffled = original.clone();
        for index in (1..shuffled.module.declarations.len()).rev() {
            seed = seed.wrapping_mul(6_364_136_223_846_793_005).wrapping_add(1);
            shuffled
                .module
                .declarations
                .swap(index, seed as usize % (index + 1));
        }
        shuffled.metadata.clear();
        let metadata = if iteration % 2 == 0 {
            [
                ("purpose", "canonical-golden"),
                ("producer", "polyrust-tests"),
            ]
        } else {
            [
                ("producer", "polyrust-tests"),
                ("purpose", "canonical-golden"),
            ]
        };
        for (key, value) in metadata {
            shuffled.metadata.insert(key.into(), value.into());
        }
        assert_eq!(to_canonical_json(&shuffled).unwrap(), expected);
    }
}

#[test]
fn all_intrinsics_values_and_constant_nodes_serialize() {
    let intrinsics = [
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
        Intrinsic::FloatIsNegativeZero,
        Intrinsic::FloatAbs,
        Intrinsic::FloatAdd,
        Intrinsic::FloatSub,
        Intrinsic::FloatMul,
        Intrinsic::FloatDiv,
        Intrinsic::FloatRemTrunc,
        Intrinsic::StringConcat,
        Intrinsic::StringScalarLength,
        Intrinsic::StringUtf16Length,
        Intrinsic::StringIndexOfLiteral,
        Intrinsic::StringSliceScalars,
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
        Intrinsic::BytesReplaceAll,
        Intrinsic::BytesLength,
        Intrinsic::BytesIsEmpty,
        Intrinsic::ListLength,
        Intrinsic::ListIsEmpty,
        Intrinsic::ListGetChecked,
        Intrinsic::ListAppend,
        Intrinsic::ListConcat,
        Intrinsic::ListContains,
        Intrinsic::ListIndexOf,
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
    for intrinsic in intrinsics {
        let encoded = serde_json::to_vec(&intrinsic).unwrap();
        assert_eq!(
            serde_json::from_slice::<Intrinsic>(&encoded).unwrap(),
            intrinsic
        );
    }
    assert_eq!(
        serde_json::to_string(&Intrinsic::FloatIsNaN).unwrap(),
        "\"float_is_nan\""
    );
    assert_eq!(
        serde_json::to_string(&Intrinsic::FloatIsNegativeZero).unwrap(),
        "\"float_is_negative_zero\""
    );
    assert_eq!(
        serde_json::to_string(&Intrinsic::FloatAbs).unwrap(),
        "\"float_abs\""
    );
    assert_eq!(
        serde_json::to_string(&Intrinsic::StringUtf16Length).unwrap(),
        "\"string_utf16_length\""
    );
    assert_eq!(
        serde_json::to_string(&Intrinsic::StringIndexOfLiteral).unwrap(),
        "\"string_index_of_literal\""
    );
    assert_eq!(
        serde_json::to_string(&Intrinsic::StringSliceScalars).unwrap(),
        "\"string_slice_scalars\""
    );
    assert_eq!(
        serde_json::to_string(&Intrinsic::ListIndexOf).unwrap(),
        "\"list_index_of\""
    );

    let values = vec![
        Value::Unit,
        Value::Bool(true),
        Value::I32(-1),
        Value::I64(2),
        Value::F64(F64Bits::from_f64(-0.0)),
        Value::Char('🦀'),
        Value::String("x".into()),
        Value::Bytes(vec![0, 255]),
        Value::List(vec![Value::I64(1)]),
        Value::None,
        Value::Some(Box::new(Value::Bool(true))),
        Value::Ok(Box::new(Value::Unit)),
        Value::Err(Box::new(Value::String("error".into()))),
        Value::Record {
            declaration: NodeId(1),
            fields: vec![],
        },
        Value::Enum {
            declaration: NodeId(2),
            variant: NodeId(3),
            fields: vec![],
        },
    ];
    for value in values {
        let encoded = serde_json::to_vec(&value).unwrap();
        assert_eq!(serde_json::from_slice::<Value>(&encoded).unwrap(), value);
    }
}

#[test]
fn every_remaining_syntax_variant_round_trips() {
    round_trip(&SourceRef::File(FileSpan {
        file: "fixtures/example.poly".into(),
        start: 4,
        end: 12,
    }));

    let base = literal(1_000, Value::I64(1));
    let expressions = vec![
        base.clone(),
        Expression::Local {
            node: node(1_001),
            name: "local".into(),
        },
        Expression::Constant {
            node: node(1_002),
            declaration: NodeId(10),
        },
        Expression::SelfValue { node: node(1_003) },
        Expression::ConstructRecord {
            node: node(1_004),
            declaration: NodeId(30),
            fields: vec![ExpressionField {
                field: NodeId(301),
                value: base.clone(),
            }],
        },
        Expression::ConstructEnum {
            node: node(1_005),
            declaration: NodeId(40),
            variant: NodeId(402),
            fields: vec![],
        },
        Expression::ConstructSome {
            node: node(1_006),
            value: Box::new(base.clone()),
        },
        Expression::ConstructNone {
            node: node(1_007),
            inner_type: TypeRef::I64,
        },
        Expression::ConstructOk {
            node: node(1_008),
            value: Box::new(base.clone()),
            error_type: TypeRef::String,
        },
        Expression::ConstructErr {
            node: node(1_009),
            value: Box::new(base.clone()),
            ok_type: TypeRef::I64,
        },
        Expression::ConstructList {
            node: node(1_010),
            element_type: TypeRef::I64,
            elements: vec![base.clone()],
        },
        Expression::Field {
            node: node(1_011),
            base: Box::new(base.clone()),
            field: NodeId(301),
        },
        Expression::Call {
            node: node(1_012),
            function: NodeId(70),
            arguments: vec![base.clone()],
        },
        Expression::MethodCall {
            node: node(1_013),
            receiver: Box::new(base.clone()),
            dispatch: MethodDispatch::Concrete {
                implementation: NodeId(60),
                method: NodeId(601),
            },
            arguments: vec![],
        },
        Expression::MethodCall {
            node: node(1_014),
            receiver: Box::new(base.clone()),
            dispatch: MethodDispatch::Contract {
                contract: NodeId(50),
                method: NodeId(501),
            },
            arguments: vec![],
        },
        Expression::Intrinsic {
            node: node(1_015),
            operation: Intrinsic::IntAddChecked,
            arguments: vec![base.clone(), base.clone()],
        },
        Expression::If {
            node: node(1_016),
            condition: Box::new(literal(1_017, Value::Bool(true))),
            then_block: Box::new(empty_block(1_018, Some(base.clone()))),
            else_block: Box::new(empty_block(1_019, Some(base.clone()))),
        },
        Expression::Match {
            node: node(1_020),
            value: Box::new(base.clone()),
            arms: vec![MatchArm {
                node: node(1_021),
                pattern: Pattern::Wildcard { node: node(1_022) },
                body: empty_block(1_023, Some(base.clone())),
            }],
        },
        Expression::Block(Box::new(empty_block(1_024, Some(base.clone())))),
    ];
    for expression in expressions {
        round_trip(&expression);
        assert!(expression.node().id.0 > 0);
    }

    let statements = vec![
        Statement::Let {
            node: node(1_100),
            name: "x".into(),
            annotation: None,
            value: base.clone(),
        },
        Statement::ForEach {
            node: node(1_101),
            binding: "item".into(),
            iterable: base.clone(),
            body: empty_block(1_102, None),
        },
        Statement::Return {
            node: node(1_103),
            value: Some(base.clone()),
        },
        Statement::Expression {
            node: node(1_104),
            value: base.clone(),
        },
    ];
    for statement in statements {
        round_trip(&statement);
    }

    let patterns = vec![
        Pattern::Wildcard { node: node(1_200) },
        Pattern::Bool {
            node: node(1_201),
            value: false,
        },
        Pattern::EnumVariant {
            node: node(1_202),
            declaration: NodeId(40),
            variant: NodeId(402),
            bindings: vec![FieldBinding {
                field: NodeId(403),
                binding: "name".into(),
            }],
        },
        Pattern::None { node: node(1_203) },
        Pattern::Some {
            node: node(1_204),
            binding: "some".into(),
        },
        Pattern::Ok {
            node: node(1_205),
            binding: "ok".into(),
        },
        Pattern::Err {
            node: node(1_206),
            binding: "error".into(),
        },
    ];
    for pattern in patterns {
        round_trip(&pattern);
    }

    let constant = ConstantExpression::Literal {
        node: node(1_300),
        value: Value::I64(1),
    };
    let constant_expressions = vec![
        constant.clone(),
        ConstantExpression::Reference {
            node: node(1_301),
            declaration: NodeId(10),
        },
        ConstantExpression::Record {
            node: node(1_302),
            declaration: NodeId(30),
            fields: vec![ConstantField {
                field: NodeId(301),
                value: constant.clone(),
            }],
        },
        ConstantExpression::Enum {
            node: node(1_303),
            declaration: NodeId(40),
            variant: NodeId(401),
            fields: vec![],
        },
        ConstantExpression::Some {
            node: node(1_304),
            value: Box::new(constant.clone()),
        },
        ConstantExpression::None {
            node: node(1_305),
            inner_type: TypeRef::I64,
        },
        ConstantExpression::Ok {
            node: node(1_306),
            value: Box::new(constant.clone()),
            error_type: TypeRef::String,
        },
        ConstantExpression::Err {
            node: node(1_307),
            value: Box::new(constant.clone()),
            ok_type: TypeRef::I64,
        },
        ConstantExpression::List {
            node: node(1_308),
            element_type: TypeRef::I64,
            elements: vec![constant.clone()],
        },
        ConstantExpression::Intrinsic {
            node: node(1_309),
            operation: Intrinsic::IntAddWrapping,
            arguments: vec![constant.clone(), constant],
        },
    ];
    for expression in constant_expressions {
        round_trip(&expression);
    }

    round_trip(&TestInvocation::Method {
        implementation: NodeId(60),
        method: NodeId(601),
        receiver: TypedValue {
            ty: TypeRef::Named(NodeId(30)),
            value: Value::Record {
                declaration: NodeId(30),
                fields: vec![],
            },
        },
        arguments: vec![],
    });
    round_trip(&ExpectedOutcome::Error(TypedValue {
        ty: TypeRef::String,
        value: Value::String("expected".into()),
    }));
    assert_eq!(
        F64Bits::from_f64(-0.0).to_f64().to_bits(),
        (-0.0_f64).to_bits()
    );
    assert!(F64Bits::from_f64(f64::NAN).to_f64().is_nan());
}

#[test]
fn v0_model_has_no_target_specific_type_variants() {
    let sources = concat!(
        include_str!("common.rs"),
        include_str!("declaration.rs"),
        include_str!("expression.rs")
    );
    for forbidden in ["RustType", "GoType", "PythonType", "TypeScriptType"] {
        assert!(!sources.contains(forbidden));
    }
}

#[test]
fn unknown_fields_and_major_versions_are_structured_errors() {
    let bytes = to_canonical_json(&exhaustive_document()).unwrap();
    let mut json: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    json.as_object_mut()
        .unwrap()
        .insert("surprise".into(), true.into());
    let error = from_json(&serde_json::to_vec(&json).unwrap()).unwrap_err();
    assert_eq!(error.kind, JsonErrorKind::UnknownField);

    json.as_object_mut().unwrap().remove("surprise");
    json["module"]
        .as_object_mut()
        .unwrap()
        .insert("nested_surprise".into(), true.into());
    let error = from_json(&serde_json::to_vec(&json).unwrap()).unwrap_err();
    assert_eq!(error.kind, JsonErrorKind::UnknownField);

    json["module"]
        .as_object_mut()
        .unwrap()
        .remove("nested_surprise");
    json.as_object_mut()
        .unwrap()
        .insert("ir_version".into(), "1.0.0".into());
    let error = from_json(&serde_json::to_vec(&json).unwrap()).unwrap_err();
    assert_eq!(error.kind, JsonErrorKind::UnsupportedVersion);
}

#[test]
fn structural_validation_detects_duplicate_and_zero_ids() {
    let mut duplicate = exhaustive_document();
    duplicate.module.declarations[1]
        .header_mut_for_test()
        .node
        .id = NodeId(80);
    assert_eq!(
        validate_structure(&duplicate),
        Err(StructuralError::DuplicateNodeId(NodeId(80)))
    );
    let mut zero = exhaustive_document();
    zero.module.declarations[0].header_mut_for_test().node.id = NodeId(0);
    assert_eq!(validate_structure(&zero), Err(StructuralError::ZeroNodeId));
}

#[test]
fn configured_limits_reject_each_resource_dimension() {
    let bytes = to_canonical_json(&exhaustive_document()).unwrap();
    let defaults = ReadLimits::default();
    let cases = [
        (
            ReadLimits {
                max_bytes: bytes.len() - 1,
                ..defaults
            },
            JsonErrorKind::TotalBytesLimit,
        ),
        (
            ReadLimits {
                max_depth: 2,
                ..defaults
            },
            JsonErrorKind::DepthLimit,
        ),
        (
            ReadLimits {
                max_nodes: 2,
                ..defaults
            },
            JsonErrorKind::NodeLimit,
        ),
        (
            ReadLimits {
                max_string_bytes: 12,
                ..defaults
            },
            JsonErrorKind::StringLimit,
        ),
    ];
    for (limits, expected) in cases {
        assert_eq!(
            from_json_with_limits(&bytes, limits).unwrap_err().kind,
            expected
        );
    }
}

#[test]
fn randomized_malformed_inputs_never_panic() {
    let mut seed = 0xdecafbad_u64;
    for length in 0..512 {
        let mut bytes = vec![0; length];
        for byte in &mut bytes {
            seed = seed
                .wrapping_mul(2_862_933_555_777_941_757)
                .wrapping_add(3_037_000_493);
            *byte = (seed >> 24) as u8;
        }
        assert!(panic::catch_unwind(|| from_json(&bytes)).is_ok());
    }
}

trait DeclarationTestExt {
    fn header_mut_for_test(&mut self) -> &mut DeclarationHeader;
}

impl DeclarationTestExt for Declaration {
    fn header_mut_for_test(&mut self) -> &mut DeclarationHeader {
        match self {
            Self::Constant(value) => &mut value.header,
            Self::Alias(value) => &mut value.header,
            Self::Record(value) => &mut value.header,
            Self::Enum(value) => &mut value.header,
            Self::Contract(value) => &mut value.header,
            Self::Implementation(value) => &mut value.header,
            Self::Function(value) => &mut value.header,
            Self::Test(value) => &mut value.header,
        }
    }
}
