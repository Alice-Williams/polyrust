use super::{
    DiagnosticCode, JavaBlock, JavaConstructor, JavaExpr, JavaField, JavaIdentifier, JavaLiteral,
    JavaMember, JavaModifier, JavaPrimitive, JavaStmt, JavaType, fixture_declaration,
    ordinary_owner_fixture, parameter, this_field, verify_fixture,
};

#[test]
fn blank_final_fields_are_assigned_exactly_once_on_every_normal_constructor_exit() {
    let int = JavaType::primitive(JavaPrimitive::Int);
    let boolean = JavaType::primitive(JavaPrimitive::Boolean);
    let (_, owner_id) = ordinary_owner_fixture();
    let owner = JavaType::Reference(super::JavaTypeName::Generated(owner_id));
    let assignment = || JavaStmt::Assign {
        target: this_field(owner.clone(), int.clone(), "value"),
        value: JavaExpr::literal(int.clone(), JavaLiteral::I32(1)),
    };
    let constructor = |parameters, statements| {
        JavaMember::Constructor(JavaConstructor {
            modifiers: vec![],
            name: JavaIdentifier::from_portable("Fixture"),
            parameters,
            body: JavaBlock::new(statements),
        })
    };
    let declaration = |initializer: Option<JavaExpr>, constructor: Option<JavaMember>| {
        let mut members = vec![JavaMember::Field(JavaField {
            declared: None,
            modifiers: vec![JavaModifier::Private, JavaModifier::Final],
            ty: int.clone(),
            name: JavaIdentifier::from_portable("value"),
            initializer,
        })];
        members.extend(constructor);
        let mut declaration = fixture_declaration(members);
        declaration.declared = Some(owner_id);
        declaration
    };
    let verify = |declaration| {
        verify_fixture(
            ordinary_owner_fixture().0,
            vec![(vec![super::GeneratedSymbolId::Type(owner_id)], declaration)],
        )
    };

    let straight_line = declaration(None, Some(constructor(vec![], vec![assignment()])));
    let verification = verify(straight_line);
    assert!(verification.is_ok(), "{verification:?}");

    let both_branches = declaration(
        None,
        Some(constructor(
            vec![parameter(boolean.clone(), "condition")],
            vec![JavaStmt::If {
                condition: JavaExpr::local(
                    boolean.clone(),
                    JavaIdentifier::from_portable("condition"),
                ),
                then_block: JavaBlock::new(vec![assignment()]),
                else_block: Some(JavaBlock::new(vec![assignment()])),
            }],
        )),
    );
    let verification = verify(both_branches);
    assert!(verification.is_ok(), "{verification:?}");

    let conditional_missing = declaration(
        None,
        Some(constructor(
            vec![parameter(boolean.clone(), "condition")],
            vec![JavaStmt::If {
                condition: JavaExpr::local(
                    boolean.clone(),
                    JavaIdentifier::from_portable("condition"),
                ),
                then_block: JavaBlock::new(vec![assignment()]),
                else_block: None,
            }],
        )),
    );
    let diagnostics = verify(conditional_missing).unwrap_err();
    assert!(diagnostics.iter().any(|value| {
        value.code == DiagnosticCode::InvalidControlFlow
            && value
                .message
                .contains("without assigning blank final field")
    }));

    let duplicate = declaration(
        None,
        Some(constructor(vec![], vec![assignment(), assignment()])),
    );
    let diagnostics = verify(duplicate).unwrap_err();
    assert!(diagnostics.iter().any(|value| {
        value.code == DiagnosticCode::InvalidControlFlow && value.message.contains("more than once")
    }));

    let read_before_write = declaration(
        None,
        Some(constructor(
            vec![],
            vec![JavaStmt::Assign {
                target: this_field(owner.clone(), int.clone(), "value"),
                value: this_field(owner.clone(), int.clone(), "value"),
            }],
        )),
    );
    let diagnostics = verify(read_before_write).unwrap_err();
    assert!(diagnostics.iter().any(|value| {
        value.code == DiagnosticCode::InvalidControlFlow
            && value
                .message
                .contains("read before it is definitely assigned")
    }));

    let early_return = declaration(
        None,
        Some(constructor(
            vec![parameter(boolean.clone(), "condition")],
            vec![
                JavaStmt::If {
                    condition: JavaExpr::local(
                        boolean.clone(),
                        JavaIdentifier::from_portable("condition"),
                    ),
                    then_block: JavaBlock::new(vec![JavaStmt::Return(None)]),
                    else_block: None,
                },
                assignment(),
            ],
        )),
    );
    let diagnostics = verify(early_return).unwrap_err();
    assert!(diagnostics.iter().any(|value| {
        value.code == DiagnosticCode::InvalidControlFlow
            && value
                .message
                .contains("without assigning blank final field")
    }));

    let loop_assignment = declaration(
        None,
        Some(constructor(
            vec![parameter(boolean.clone(), "condition")],
            vec![JavaStmt::While {
                condition: JavaExpr::local(boolean, JavaIdentifier::from_portable("condition")),
                body: JavaBlock::new(vec![assignment()]),
            }],
        )),
    );
    let diagnostics = verify(loop_assignment).unwrap_err();
    assert!(diagnostics.iter().any(|value| {
        value.code == DiagnosticCode::InvalidControlFlow
            && value.message.contains("constructor loop")
    }));

    let initialized = declaration(
        Some(JavaExpr::literal(int.clone(), JavaLiteral::I32(0))),
        Some(constructor(vec![], vec![assignment()])),
    );
    let diagnostics = verify(initialized).unwrap_err();
    assert!(diagnostics.iter().any(|value| {
        value.code == DiagnosticCode::InvalidControlFlow
            && value.message.contains("initialized final")
    }));

    let implicit = declaration(None, None);
    let diagnostics = verify(implicit).unwrap_err();
    assert!(diagnostics.iter().any(|value| {
        value.code == DiagnosticCode::InvalidControlFlow
            && value.message.contains("implicit Java constructor")
    }));
}
