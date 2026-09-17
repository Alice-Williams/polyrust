//! Constant-only imports still require their original producer's math library.
use super::super::super::{
    constant_consumer_fixture as consumers, owned_constant_fixture as constants,
    owned_constant_tests,
};
use super::*;

#[test]
fn constant_only_consumer_keeps_original_math_linkage_without_local_header() {
    let mut producer = constants::fixture(constants::Shape::Mixed);
    let e = CExpressions::new(producer.registry.registrations());
    let declarations = CDeclarations::new(
        producer.registry.registrations(),
        producer.files[1].identity().clone(),
    )
    .unwrap();
    let items = producer.files[1]
        .items()
        .iter()
        .map(|item| {
            let CFileItem::Definition(definition) = item else {
                return item.clone();
            };
            let CDefinitionKind::Function {
                function,
                linkage,
                parameters,
                body,
            } = definition.kind()
            else {
                return item.clone();
            };
            let statements =
                CStatements::new(producer.registry.registrations(), function.clone()).unwrap();
            let zero = e
                .literal(CLiteral::F64(
                    portable_binary64::FiniteBinary64::from_bits(0).unwrap(),
                ))
                .unwrap();
            let call = e
                .call_value(e.known(CKnownCall::FloatTruncate), vec![zero])
                .unwrap();
            let mut contents = vec![statements.discard(call).unwrap()];
            contents.extend_from_slice(body.statements());
            let body = statements.block(body.scope().clone(), contents).unwrap();
            CFileItem::Definition(
                declarations
                    .function_definition(function.clone(), *linkage, parameters.clone(), body)
                    .unwrap(),
            )
        })
        .collect();
    producer.files[1] = declarations.source_file(items).unwrap();
    let producer = CDependencyApi::from_certificate(
        certify_resolved_package(&CDialect, owned_constant_tests::linked(&producer)).unwrap(),
    )
    .unwrap();
    assert_eq!(
        producer.system_libraries(),
        &BTreeSet::from([CSystemLibrary::Math])
    );
    let imported: Vec<_> = producer.constants().take(1).cloned().collect();
    let consumer = consumers::fixture(107, &imported, None, consumers::Usage::Read);
    let consumer = CDependencyApi::from_certificate(
        certify_resolved_package(&CDialect, owned_constant_tests::linked(&consumer)).unwrap(),
    )
    .unwrap();
    assert_eq!(consumer.system_libraries(), producer.system_libraries());
    assert_eq!(
        crate::dialect::c_imported_functions(consumer.package()).count(),
        0
    );
    assert_eq!(
        crate::dialect::c_imported_constants(consumer.package()).count(),
        1
    );
    let rendered = render_certified_package(&CStructuralRenderer, consumer.package()).unwrap();
    for file in rendered.files() {
        let OutputContents::Text(text) = file.contents() else {
            panic!("text")
        };
        assert!(!text.contains("math.h"));
    }
}
