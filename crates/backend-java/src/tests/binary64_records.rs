//! A double field/constructor/local stays primitive through the owner certificate.
use super::*;

pub(super) fn record() -> JavaDependencyApi {
    let mut fixture = crate::tests::source_documentation_fixture::binary64_fixture(
        crate::tests::source_record_fixture::Facade::EntryPoint,
    );
    fixture
        .facade
        .members
        .push(JavaMember::Constructor(JavaConstructor {
            modifiers: vec![JavaModifier::Private],
            name: f::name("Generated"),
            parameters: vec![],
            body: JavaBlock::new(vec![]),
        }));
    JavaDependencyApi::from_certificate(f::certify(fixture.finish())).unwrap()
}

#[test]
fn primitive_double_record_and_local_transport_certifies() {
    let owner = record();
    let output = render_certified_package(&JavaStructuralRenderer, owner.package()).unwrap();
    let OutputContents::Text(text) = output.files()[0].contents() else {
        panic!("source")
    };
    for fragment in ["private record Cell(", "double value", "boolean flag"] {
        assert!(text.contains(fragment), "missing {fragment}: {text}");
    }
    assert!(text.contains("double inspect"));
    assert!(text.len() as u64 <= owner.source_byte_bound().unwrap());
}

#[test]
fn double_signatures_use_two_slots_at_the_certified_boundary() {
    for (count, accepted) in [(127, true), (128, false)] {
        let mut declarations = functions(&[0]);
        declarations[0].parameters = (0..count)
            .map(|index| JavaParameter {
                ty: double(),
                name: f::name(&format!("arg{index}")),
                final_parameter: true,
            })
            .collect();
        let draft = f::package(91, declarations);
        let verified = verify_unresolved_package(&JavaDialect, draft);
        let result = verified
            .and_then(|verified| TargetLinker::new(JavaDialect).link_ast(&verified))
            .and_then(|linked| certify_resolved_package(&JavaDialect, linked));
        assert_eq!(result.is_ok(), accepted, "{count}: {result:?}");
        if let Ok(package) = result {
            JavaDependencyApi::from_certificate(package).unwrap();
        }
    }
}
