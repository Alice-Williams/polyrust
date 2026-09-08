use super::*;

#[derive(Clone, Copy, Debug)]
enum Shape {
    Record,
    PayloadEnum,
    AliasRecord,
}

fn recursive_document(shape: Shape, interface: bool, operation: Intrinsic) -> Document {
    let mut factory = Factory::new();
    let header = factory.declaration("Node");
    let node = header.node.id;
    let view_header = factory.declaration("View");
    let view = view_header.node.id;
    let alias_header = factory.declaration("Nodes");
    let alias = alias_header.node.id;
    let children = TypeRef::List(Box::new(TypeRef::Named(node)));
    let mut fields = vec![FieldDeclaration {
        header: factory.member("children"),
        ty: if matches!(shape, Shape::AliasRecord) {
            TypeRef::Named(alias)
        } else {
            children.clone()
        },
    }];
    // The cycle deliberately precedes the interface-bearing sibling.
    if interface {
        fields.push(FieldDeclaration {
            header: factory.member("view"),
            ty: TypeRef::Option(Box::new(TypeRef::Interface(view))),
        });
    }
    let aggregate = match shape {
        Shape::Record | Shape::AliasRecord => {
            Declaration::Record(RecordDeclaration { header, fields })
        }
        Shape::PayloadEnum => Declaration::Enum(EnumDeclaration {
            header,
            variants: vec![EnumVariant {
                header: factory.member("Branch"),
                fields,
            }],
        }),
    };
    let search = matches!(operation, Intrinsic::ListContains | Intrinsic::ListIndexOf);
    let result_type = if operation == Intrinsic::ListIndexOf {
        TypeRef::Option(Box::new(TypeRef::I64))
    } else {
        TypeRef::Bool
    };
    let parameters = vec![
        factory.parameter(
            "left",
            if search {
                children.clone()
            } else {
                TypeRef::Named(node)
            },
        ),
        factory.parameter("right", TypeRef::Named(node)),
    ];
    let expression = Expression::Intrinsic {
        node: factory.node(),
        operation,
        arguments: vec![factory.local("left"), factory.local("right")],
    };
    Document::new(
        IrVersion::CURRENT,
        Module {
            name: "recursive_equality".to_owned(),
            declarations: vec![
                aggregate,
                Declaration::Interface(InterfaceDeclaration {
                    header: view_header,
                    methods: vec![],
                }),
                Declaration::Alias(AliasDeclaration {
                    header: alias_header,
                    target: children,
                }),
                Declaration::Function(FunctionDeclaration {
                    header: factory.declaration("compare"),
                    parameters,
                    return_type: result_type,
                    body: factory.block(expression),
                }),
            ],
        },
    )
}

#[test]
fn recursive_aggregates_terminate_and_find_interface_siblings() {
    for shape in [Shape::Record, Shape::PayloadEnum, Shape::AliasRecord] {
        for operation in [
            Intrinsic::Equal,
            Intrinsic::NotEqual,
            Intrinsic::ListContains,
            Intrinsic::ListIndexOf,
        ] {
            check_program(recursive_document(shape, false, operation))
                .unwrap_or_else(|errors| panic!("{shape:?}/{operation:?}: {errors:?}"));
            let errors = check_program(recursive_document(shape, true, operation)).unwrap_err();
            assert!(
                codes(&errors).contains(&DiagnosticCode::InvalidInterfacePosition),
                "{shape:?}/{operation:?}: {errors:?}"
            );
        }
    }
}

#[test]
fn pure_alias_cycles_keep_the_existing_diagnostic() {
    let mut document = recursive_document(Shape::AliasRecord, false, Intrinsic::ListContains);
    for declaration in &mut document.module.declarations {
        if let Declaration::Alias(alias) = declaration {
            alias.target = TypeRef::List(Box::new(TypeRef::Named(alias.header.node.id)));
        }
    }
    let errors = check_program(document).unwrap_err();
    assert!(codes(&errors).contains(&DiagnosticCode::AliasCycle));
}
