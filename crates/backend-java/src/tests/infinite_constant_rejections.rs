//! Exact initializer syntax and source-shaped method bounds fail closed.
use super::*;

#[test]
fn infinity_initializers_reject_wrong_shapes_types_precedence_and_modifiers() {
    #[derive(Debug)]
    enum Fault {
        Boxed,
        WrongType,
        WrongField,
        Precedence,
        Cast,
        Negation,
        Local,
        Missing,
        Private,
        Mutable,
        Instance,
    }
    for fault in [
        Fault::Boxed,
        Fault::WrongType,
        Fault::WrongField,
        Fault::Precedence,
        Fault::Cast,
        Fault::Negation,
        Fault::Local,
        Fault::Missing,
        Fault::Private,
        Fault::Mutable,
        Fault::Instance,
    ] {
        let mut source = fixture(true);
        let field = source.field(0);
        match fault {
            Fault::Boxed => field.ty = field.ty.clone().boxed(),
            Fault::WrongType => {
                field.initializer.as_mut().unwrap().ty = JavaType::primitive(JavaPrimitive::Long)
            }
            Fault::WrongField => {
                field.initializer.as_mut().unwrap().kind =
                    JavaExprKind::Value(JavaValueRef::KnownField(JavaKnownField::LongMaxValue))
            }
            Fault::Precedence => {
                field.initializer.as_mut().unwrap().precedence = JavaPrecedence::Additive
            }
            Fault::Cast => {
                field.initializer = Some(JavaExpr {
                    ty: field.ty.clone(),
                    precedence: JavaPrecedence::Unary,
                    kind: JavaExprKind::Cast {
                        target: field.ty.clone(),
                        value: Box::new(expression(Binary64Sign::Positive)),
                    },
                })
            }
            Fault::Negation => {
                field.initializer = Some(JavaExpr {
                    ty: field.ty.clone(),
                    precedence: JavaPrecedence::Unary,
                    kind: JavaExprKind::Unary {
                        operator: JavaUnaryOperator::Negate,
                        operand: Box::new(expression(Binary64Sign::Negative)),
                    },
                })
            }
            Fault::Local => {
                field.initializer = Some(JavaExpr::local(field.ty.clone(), f::name("missing")))
            }
            Fault::Missing => field.initializer = None,
            Fault::Private => field.modifiers[0] = JavaModifier::Private,
            Fault::Mutable => field.modifiers.retain(|m| *m != JavaModifier::Final),
            Fault::Instance => field.modifiers.retain(|m| *m != JavaModifier::Static),
        }
        assert!(c::admit(source.finish()).is_err(), "{fault:?}");
    }
}

#[test]
fn direct_infinity_methods_reserve_repeated_owners_and_reject_value_shadowing() {
    let mut previous = 0;
    for count in [1, 128] {
        let mut functions = f::functions(0);
        functions.truncate(1);
        functions[0].result = JavaType::primitive(JavaPrimitive::Double);
        functions[0].body = JavaBlock::new(
            (0..count)
                .map(|index| JavaStmt::Local {
                    finality: JavaLocalFinality::Final,
                    ty: functions[0].result.clone(),
                    name: f::name(&format!("local{index}")),
                    value: Some(expression(Binary64Sign::Negative)),
                })
                .chain([JavaStmt::Return(Some(expression(Binary64Sign::Positive)))])
                .collect(),
        );
        let api = c::admit(f::package(0x927, functions)).unwrap();
        let output = text(&api);
        assert_eq!(output.matches("NEGATIVE_INFINITY").count(), count);
        let bound = api.source_byte_bound().unwrap();
        assert!(bound > previous);
        previous = bound;
    }
    for name in ["Double", "java"] {
        let mut functions = f::functions(0);
        functions.truncate(1);
        functions[0].result = JavaType::primitive(JavaPrimitive::Double);
        functions[0].body = JavaBlock::new(vec![
            JavaStmt::Local {
                finality: JavaLocalFinality::Final,
                ty: functions[0].result.clone(),
                name: f::name(name),
                value: Some(super::super::finite_constants::literal(0)).map(|value| {
                    JavaExpr::literal(JavaType::primitive(JavaPrimitive::Double), value)
                }),
            },
            JavaStmt::Return(Some(expression(Binary64Sign::Positive))),
        ]);
        assert!(c::admit(f::package(0x927, functions)).is_err(), "{name}");
    }
}
