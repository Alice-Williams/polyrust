//! Public certificates retain strict primitive, declaration and initialization checks.
use super::*;

#[test]
fn finite_constant_public_shape_faults_cannot_enter_dependency_inventory() {
    #[derive(Debug)]
    enum Fault {
        MissingId,
        MissingValue,
        Boxed,
        WrongPrimitive,
        WrongLiteral,
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
        Fault::WrongPrimitive,
        Fault::WrongLiteral,
        Fault::Nonliteral,
        Fault::Private,
        Fault::Mutable,
        Fault::Instance,
        Fault::DuplicateId,
    ] {
        let mut source = fixture(&[0, 1 << 63], true);
        match fault {
            Fault::MissingId => source.field(0).declared = None,
            Fault::MissingValue => source.field(0).initializer = None,
            Fault::Boxed => source.field(0).ty = JavaType::primitive(JavaPrimitive::Double).boxed(),
            Fault::WrongPrimitive => source.field(0).ty = JavaType::primitive(JavaPrimitive::Long),
            Fault::WrongLiteral => {
                source.field(0).initializer.as_mut().unwrap().kind =
                    JavaExprKind::Literal(JavaLiteral::I64(0))
            }
            Fault::Nonliteral => {
                source.field(0).initializer = Some(JavaExpr::local(
                    JavaType::primitive(JavaPrimitive::Double),
                    f::name("missing"),
                ))
            }
            Fault::Private => source.field(0).modifiers[0] = JavaModifier::Private,
            Fault::Mutable => source
                .field(0)
                .modifiers
                .retain(|modifier| *modifier != JavaModifier::Final),
            Fault::Instance => source
                .field(0)
                .modifiers
                .retain(|modifier| *modifier != JavaModifier::Static),
            Fault::DuplicateId => source.field(0).declared = source.field(1).declared,
        }
        assert!(c::admit(source.finish()).is_err(), "{fault:?}");
    }
    for bits in [
        0x7ff0_0000_0000_0000,
        0xfff0_0000_0000_0000,
        0x7ff8_0000_0000_0000,
    ] {
        assert!(FiniteBinary64::from_bits(bits).is_err());
    }
}

#[test]
fn finite_constant_long_names_and_repeated_reads_are_in_source_bounds() {
    let mut previous = None;
    for width in [1, 2048] {
        let name = f::name(&format!("constant{}", "x".repeat(width)));
        let mut source = c::Fixture::configured_values(
            true,
            vec![literal(1 << 63)],
            |_| {},
            |_, registration| {
                registration.name = name.as_str().into();
            },
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
        check_bounds(&owner);
        let bound = owner.source_byte_bound().unwrap();
        if let Some(previous) = previous {
            assert!(bound - previous >= 2047 * 33);
        }
        previous = Some(bound);
    }
}
