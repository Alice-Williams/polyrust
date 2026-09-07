use super::*;

#[test]
fn checked_remainder_minimum_by_minus_one_has_exact_overflow_payload() {
    for (callable, minimum) in [
        (
            JavaRuntimeCallable::CheckedRemI32,
            JavaKnownField::IntegerMinValue,
        ),
        (
            JavaRuntimeCallable::CheckedRemI64,
            JavaKnownField::LongMinValue,
        ),
    ] {
        let JavaMember::Method(method) = checked_integer_method(callable) else {
            panic!("checked remainder must lower to a method");
        };
        let body = method.body.expect("checked remainder method body");
        assert_eq!(
            body.statements.len(),
            3,
            "checked remainder must guard zero and signed overflow before evaluation"
        );
        assert_failure(
            if_block(&body.statements[0]),
            JavaRuntimeFailure::RemainderByZero,
        );

        let JavaStmt::If {
            condition,
            then_block,
            else_block: None,
        } = &body.statements[1]
        else {
            panic!("second checked remainder statement must be the signed overflow guard");
        };
        let JavaExprKind::Binary {
            operator: JavaBinaryOperator::LogicalAnd,
            left,
            right,
        } = &condition.kind
        else {
            panic!("signed overflow guard must test both operands");
        };
        assert_local_equals_known_field(left, "left", minimum);
        assert_local_equals_minus_one(right, callable == JavaRuntimeCallable::CheckedRemI64);
        assert_failure(then_block, JavaRuntimeFailure::CheckedOverflow);

        let JavaStmt::Return(Some(returned)) = &body.statements[2] else {
            panic!("checked remainder must return its successful result");
        };
        let JavaExprKind::Call {
            callable:
                JavaCallableRef::Runtime {
                    callable: JavaRuntimeCallable::Ok,
                    ..
                },
            arguments,
            ..
        } = &returned.kind
        else {
            panic!("checked remainder success must use Runtime.ok");
        };
        let [remainder] = arguments.as_slice() else {
            panic!("Runtime.ok must receive the remainder");
        };
        assert!(matches!(
            &remainder.kind,
            JavaExprKind::Binary {
                operator: JavaBinaryOperator::Remainder,
                ..
            }
        ));
    }
}

#[test]
fn raw_tagged_construction_is_private_and_factories_are_package_scoped() {
    for declaration in [
        validated_result_type(),
        validated_option_type(),
        validated_value_result_type(),
    ] {
        assert_eq!(declaration.kind, JavaDeclarationKind::FinalClass);
        assert!(declaration.record_components.is_empty());
        let constructor = declaration
            .members
            .iter()
            .find_map(|member| match member {
                JavaMember::Constructor(value) => Some(value),
                _ => None,
            })
            .expect("tagged class constructor");
        assert_eq!(constructor.modifiers, vec![JavaModifier::Private]);
        let fields = declaration
            .members
            .iter()
            .filter_map(|member| match member {
                JavaMember::Field(field) => Some(field),
                _ => None,
            })
            .collect::<Vec<_>>();
        assert!(!fields.is_empty());
        assert!(
            fields.iter().all(|field| {
                field.modifiers == vec![JavaModifier::Private, JavaModifier::Final]
            })
        );
    }

    let members = core_members()
        .into_iter()
        .chain(tagged_members())
        .collect::<Vec<_>>();
    for name in [
        "ok",
        "fail",
        "optionNone",
        "optionSome",
        "valueResultOk",
        "valueResultErr",
    ] {
        let method = members
            .iter()
            .find_map(|member| match member {
                JavaMember::Method(method) if method.name.as_str() == name => Some(method),
                _ => None,
            })
            .unwrap_or_else(|| panic!("missing raw tagged factory {name}"));
        assert_eq!(method.modifiers, vec![JavaModifier::Static]);
    }
}

#[test]
fn runtime_helper_methods_are_package_scoped() {
    for helper in JavaRuntimeHelper::ALL {
        let items = helper_items(helper);
        let [
            JavaFileItem::RuntimeMembers {
                helper: actual_helper,
                members,
            },
        ] = items.as_slice()
        else {
            panic!("runtime helper must produce exactly one typed member group");
        };
        assert_eq!(*actual_helper, helper);

        for method in members.iter().filter_map(|member| match member {
            JavaMember::Method(method) => Some(method),
            _ => None,
        }) {
            assert_eq!(
                method.modifiers,
                vec![JavaModifier::Static],
                "top-level runtime helper {} from {} must not be callable outside the generated package",
                method.name.as_str(),
                helper.name(),
            );
        }
    }
}

fn if_block(statement: &JavaStmt) -> &JavaBlock {
    let JavaStmt::If {
        then_block,
        else_block: None,
        ..
    } = statement
    else {
        panic!("expected an if statement without an else branch");
    };
    then_block
}

fn assert_failure(block: &JavaBlock, expected: JavaRuntimeFailure) {
    let [JavaStmt::Return(Some(returned))] = block.statements.as_slice() else {
        panic!("failure guard must return exactly one value");
    };
    let JavaExprKind::Call {
        callable:
            JavaCallableRef::Runtime {
                callable: JavaRuntimeCallable::Fail,
                ..
            },
        arguments,
        ..
    } = &returned.kind
    else {
        panic!("failure guard must return Runtime.fail");
    };
    let [code, message] = arguments.as_slice() else {
        panic!("Runtime.fail must receive code and message");
    };
    assert_eq!(string_value(code), expected.name());
    assert_eq!(string_value(message), expected.name());
}

fn assert_local_equals_known_field(value: &JavaExpr, local_name: &str, expected: JavaKnownField) {
    let JavaExprKind::Binary {
        operator: JavaBinaryOperator::Equal,
        left,
        right,
    } = &value.kind
    else {
        panic!("overflow operand must be an equality comparison");
    };
    assert!(matches!(
        &left.kind,
        JavaExprKind::Value(JavaValueRef::Local(name)) if name.as_str() == local_name
    ));
    assert!(matches!(
        &right.kind,
        JavaExprKind::Value(JavaValueRef::KnownField(field)) if *field == expected
    ));
}

fn assert_local_equals_minus_one(value: &JavaExpr, wide: bool) {
    let JavaExprKind::Binary {
        operator: JavaBinaryOperator::Equal,
        left,
        right,
    } = &value.kind
    else {
        panic!("overflow divisor must be an equality comparison");
    };
    assert!(matches!(
        &left.kind,
        JavaExprKind::Value(JavaValueRef::Local(name)) if name.as_str() == "right"
    ));
    assert!(matches!(
        (&right.kind, wide),
        (JavaExprKind::Literal(JavaLiteral::I32(-1)), false)
            | (JavaExprKind::Literal(JavaLiteral::I64(-1)), true)
    ));
}

fn string_value(value: &JavaExpr) -> &str {
    let JavaExprKind::Literal(JavaLiteral::String(value)) = &value.kind else {
        panic!("expected a string literal");
    };
    value
}
