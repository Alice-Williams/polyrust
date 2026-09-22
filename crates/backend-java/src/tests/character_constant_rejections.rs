//! Primitive constant shape and resource accounting remain mandatory.
use super::*;

#[test]
fn int_constant_shapes_types_and_initializer_annotations_fail_closed() {
    #[derive(Debug)]
    enum Fault {
        MissingId,
        MissingValue,
        Boxed,
        WrongType,
        WrongLiteral,
        Precedence,
        Nonliteral,
        Private,
        Mutable,
        Instance,
        DuplicateId,
    }
    for fault in [
        Fault::MissingId,
        Fault::MissingValue,
        Fault::Boxed,
        Fault::WrongType,
        Fault::WrongLiteral,
        Fault::Precedence,
        Fault::Nonliteral,
        Fault::Private,
        Fault::Mutable,
        Fault::Instance,
        Fault::DuplicateId,
    ] {
        let mut source = fixture(&[0, 0x10ffff], true);
        match fault {
            Fault::MissingId => source.field(0).declared = None,
            Fault::MissingValue => source.field(0).initializer = None,
            Fault::Boxed => source.field(0).ty = f::int().boxed(),
            Fault::WrongType => {
                source.field(0).initializer.as_mut().unwrap().ty =
                    JavaType::primitive(JavaPrimitive::Long)
            }
            Fault::WrongLiteral => {
                source.field(0).initializer.as_mut().unwrap().kind =
                    JavaExprKind::Literal(JavaLiteral::I64(0))
            }
            Fault::Precedence => {
                source.field(0).initializer.as_mut().unwrap().precedence = JavaPrecedence::Additive
            }
            Fault::Nonliteral => {
                source.field(0).initializer = Some(JavaExpr::local(f::int(), f::name("missing")))
            }
            Fault::Private => source.field(0).modifiers[0] = JavaModifier::Private,
            Fault::Mutable => source
                .field(0)
                .modifiers
                .retain(|m| *m != JavaModifier::Final),
            Fault::Instance => source
                .field(0)
                .modifiers
                .retain(|m| *m != JavaModifier::Static),
            Fault::DuplicateId => source.field(0).declared = source.field(1).declared,
        }
        assert!(c::admit(source.finish()).is_err(), "{fault:?}");
    }
}

#[test]
fn int_constant_readers_account_for_long_names_and_repeated_accesses() {
    let mut previous = None;
    for width in [1, 2048] {
        let name = f::name(&format!("constant{}", "x".repeat(width)));
        let mut source = c::Fixture::configured_values(
            true,
            vec![JavaLiteral::I32(0x10ffff)],
            |_| {},
            |_, registration| registration.name = name.as_str().into(),
        );
        source.field(0).name = name;
        let method = source
            .facade
            .members
            .iter_mut()
            .find_map(|member| match member {
                JavaMember::Method(method) => Some(method),
                _ => None,
            })
            .unwrap();
        let JavaStmt::Return(Some(read)) = &method.body.as_ref().unwrap().statements[0] else {
            panic!("read")
        };
        let read = read.clone();
        method.body.as_mut().unwrap().statements = (0..32)
            .map(|index| JavaStmt::Local {
                finality: JavaLocalFinality::Final,
                ty: read.ty.clone(),
                name: f::name(&format!("local{index}")),
                value: Some(read.clone()),
            })
            .chain([JavaStmt::Return(Some(read.clone()))])
            .collect();
        let owner = c::admit(source.finish()).unwrap();
        text(&owner);
        let bound = owner.source_byte_bound().unwrap();
        if let Some(previous) = previous {
            assert!(bound - previous >= 2047 * 33);
        }
        previous = Some(bound);
    }
}

#[test]
fn int_constant_field_names_obey_jvm_utf8_capacity() {
    for width in [65_535, 65_536] {
        let name = f::name(&"c".repeat(width));
        let mut source = c::Fixture::configured_values(
            false,
            vec![JavaLiteral::I32(0x10ffff)],
            |_| {},
            |_, registration| registration.name = name.as_str().into(),
        );
        source.field(0).name = name;
        let result = c::admit(source.finish());
        if width == 65_535 {
            text(&result.unwrap());
        } else {
            let error = result.unwrap_err();
            assert!(
                error.contains("65536") && error.contains("65535"),
                "{error}"
            );
        }
    }
}
