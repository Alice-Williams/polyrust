use super::{source_constant_fixture as c, source_dependency_fixture as f};
use crate::{ast::*, dialect::JavaStructuralRenderer};
use portable_codegen::{OutputContents, render_certified_package};

#[test]
fn source_reservation_charges_every_long_constant_read_and_expanded_docs() {
    let count = 32;
    let mut previous = None;
    for width in [1, 2048] {
        let name = f::name(&format!("constant{}", "x".repeat(width)));
        let mut fixture = c::Fixture::configured(
            true,
            |_| {},
            |i, registration| {
                if i == 0 {
                    registration.name = name.as_str().into();
                    let portable_codegen::GeneratedOrigin::RustSource(origin) =
                        &mut registration.origin
                    else {
                        panic!()
                    };
                    std::sync::Arc::make_mut(origin).documentation =
                        vec![crate::tests::source_documentation_fixture::HOSTILE.repeat(4)];
                }
            },
        );
        fixture.field(0).name = name;
        let method = fixture
            .facade
            .members
            .iter_mut()
            .find_map(|member| match member {
                JavaMember::Method(method) => Some(method),
                _ => None,
            })
            .unwrap();
        let body = method.body.as_mut().unwrap();
        let JavaStmt::Return(Some(read)) = &body.statements[0] else {
            panic!()
        };
        let read = read.clone();
        body.statements = (0..count)
            .map(|i| JavaStmt::Local {
                finality: JavaLocalFinality::Final,
                ty: read.ty.clone(),
                name: f::name(&format!("local{i}")),
                value: Some(read.clone()),
            })
            .chain([JavaStmt::Return(Some(read.clone()))])
            .collect();
        let api = c::admit(fixture.finish()).unwrap();
        let bound = api.source_byte_bound().unwrap();
        let output = render_certified_package(&JavaStructuralRenderer, api.package()).unwrap();
        let OutputContents::Text(text) = output.files()[0].contents() else {
            panic!()
        };
        assert!(bound >= text.len() as u64);
        if let Some(previous) = previous {
            // Each reference, not merely the once-per-symbol table, pays the name.
            assert!(bound - previous >= (2048 - 1) * (count + 1));
        }
        previous = Some(bound);
    }
}
