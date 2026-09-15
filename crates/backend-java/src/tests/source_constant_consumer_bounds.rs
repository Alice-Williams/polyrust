use super::{
    source_constant_consumer_fixture as f, source_constant_fixture as c,
    source_dependency_fixture as source,
};
use crate::{ast::*, dialect::*};
use portable_codegen::*;

#[test]
fn every_foreign_constant_read_reserves_its_qualified_name() {
    let mut previous = None;
    for width in [1, 2048] {
        let name = source::name(&format!("constant{}", "x".repeat(width)));
        let mut producer = c::Fixture::configured(
            false,
            |_| {},
            |i, registration| {
                if i == 0 {
                    registration.name = name.as_str().into();
                }
            },
        );
        producer.field(0).name = name;
        let producer = c::admit(producer.finish()).unwrap();
        let (scope, value) = JavaDependencyScope::new()
            .import_constant(producer.constants().next().unwrap().clone());
        let read = f::read(&value);
        let statements = (0..32)
            .map(|i| JavaStmt::Local {
                finality: JavaLocalFinality::Final,
                ty: read.ty.clone(),
                name: source::name(&format!("local{i}")),
                value: Some(read.clone()),
            })
            .chain([JavaStmt::Return(Some(read.clone()))])
            .collect();
        let function = source::Function {
            hash: 20,
            public: true,
            name: source::name("read"),
            parameters: vec![],
            result: read.ty,
            body: JavaBlock::new(statements),
        };
        let api = c::admit(source::package_with_dependencies(
            0x500,
            vec![function],
            scope.finish(),
        ))
        .unwrap();
        let bound = api.source_byte_bound().unwrap();
        let output = render_certified_package(&JavaStructuralRenderer, api.package()).unwrap();
        let OutputContents::Text(text) = output.files()[0].contents() else {
            panic!()
        };
        assert!(bound >= text.len() as u64);
        if let Some(previous) = previous {
            assert!(bound - previous >= 2047 * 33);
        }
        previous = Some(bound);
    }
}
#[test]
fn foreign_field_qualification_cannot_be_shadowed_by_a_local() {
    let fixture = f::Fixture::new(false);
    let read = f::read(&fixture.values[0]);
    let function = source::Function {
        hash: 20,
        public: true,
        name: source::name("bad"),
        parameters: vec![],
        result: read.ty.clone(),
        body: JavaBlock::new(vec![
            JavaStmt::Local {
                finality: JavaLocalFinality::Final,
                ty: source::int(),
                name: source::name("org"),
                value: Some(JavaExpr::literal(source::int(), JavaLiteral::I32(0))),
            },
            JavaStmt::Return(Some(read)),
        ]),
    };
    assert!(
        c::admit(source::package_with_dependencies(
            0x500,
            vec![function],
            fixture.bindings
        ))
        .is_err()
    );
}
