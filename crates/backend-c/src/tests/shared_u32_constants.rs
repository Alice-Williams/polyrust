//! Target U32 constant facts remain distinct from original Rust Char authority.
use super::{
    CDependencyApi, CDialect, CStructuralRenderer, constant_consumer_fixture,
    constant_export_fixture,
    constant_producer_tests::certify,
    owned_constant_fixture::{self, Fixture},
    owned_constant_tests::linked,
    project_c_package,
};
use crate::ast::*;
use portable_codegen::*;

#[path = "shared_u32_constant_native.rs"]
mod native;
#[path = "shared_u32_constant_rejections.rs"]
mod rejections;

fn literal(value: u32) -> CLiteral {
    CLiteral::Unsigned(CUnsignedLiteral::U32(value))
}

fn fixture(values: &[u32]) -> Fixture {
    let names: Vec<_> = (0..values.len())
        .map(|index| format!("scalar_{index}"))
        .collect();
    let literals: Vec<_> = names
        .iter()
        .zip(values)
        .map(|(name, value)| (name.as_str(), literal(*value)))
        .collect();
    owned_constant_fixture::literal_fixture(&literals)
}

fn api(source: &Fixture) -> CDependencyApi {
    CDependencyApi::from_certificate(certify(source)).unwrap()
}

fn check_resources(owner: &CDependencyApi) {
    let output = render_certified_package(&CStructuralRenderer, owner.package()).unwrap();
    assert_eq!(output.files().len(), 2);
    let mut bytes = 0;
    for file in output.files() {
        let OutputContents::Text(text) = file.contents() else {
            panic!("C text")
        };
        bytes += text.len() as u64;
        assert!(!text.contains("runtime") && !text.contains("goto "));
    }
    assert!(bytes <= crate::dialect::c_output_byte_bound(owner.package()).unwrap());
    assert!(owner.system_libraries().is_empty());
}

#[test]
fn u32_constants_preserve_full_target_domain_and_original_imports() {
    let values = [
        0,
        0xd800,
        0xdfff,
        0xffff,
        0x10000,
        0x10ffff,
        0x110000,
        u32::MAX,
    ];
    let source = fixture(&values);
    let original = api(&source);
    assert_eq!(original.constants().count(), values.len());
    assert_eq!(original.functions().count(), 0);
    for (constant, value) in original.constants().zip(values) {
        assert_eq!(constant.value(), &CScalarConstantValue::U32(value));
        assert_eq!(constant.value().literal(), Some(literal(value)));
        assert_eq!(constant.read_type(), &CObjectType::scalar(CScalarType::U32));
        assert_eq!(constant.object().ty().constness(), CConstness::Const);
        assert_eq!(constant.package_identity().stack_bound_bytes(), 0);
    }
    check_resources(&original);
    let constants: Vec<_> = original.constants().cloned().collect();
    let facade = api(&constant_export_fixture::facade(921, &constants));
    let aliases: Vec<_> = facade
        .foreign_constants()
        .map(|binding| binding.dependency().clone())
        .collect();
    assert_eq!(constants, aliases);
    check_resources(&facade);
    let reader = api(&constant_consumer_fixture::fixture(
        922,
        &aliases,
        None,
        constant_consumer_fixture::Usage::Read,
    ));
    assert_eq!(reader.functions().count(), values.len());
    check_resources(&reader);
    let measured = super::resources::measure_package(&linked(&source)).unwrap();
    assert_eq!(measured.total.frame_bound, 0);
    assert!(measured.total.value_bytes >= 4 * values.len() as u64);
}

#[test]
fn equal_identity_different_values_and_lookalike_certificates_are_not_interchangeable() {
    let mut source = fixture(&[0x10000]);
    let original = api(&source);
    let object = source.objects[0].clone();
    let ast = CExpressions::new(source.registry.registrations());
    let definitions = CDeclarations::new(
        source.registry.registrations(),
        source.files[1].identity().clone(),
    )
    .unwrap();
    let initializer = ast
        .expression_initializer(ast.literal(literal(0x10001)).unwrap())
        .unwrap();
    source.files[1] = definitions
        .source_file(vec![CFileItem::Definition(
            definitions
                .object_definition(object, CLinkage::External, initializer)
                .unwrap(),
        )])
        .unwrap();
    let changed = api(&source);
    let before = original.constants().next().unwrap();
    let after = changed.constants().next().unwrap();
    assert_eq!(before.object(), after.object());
    assert_eq!(before.declaration(), after.declaration());
    assert_ne!(before.value(), after.value());
    assert_ne!(before, after);
    let lookalike = api(&fixture(&[0x10000]));
    for replacement in [after, lookalike.constants().next().unwrap()] {
        assert_ne!(before.package_identity(), replacement.package_identity());
        let mut registry = CRegistry::new();
        registry.import_constant(before.clone()).unwrap();
        assert!(registry.import_constant(replacement.clone()).is_err());
    }
}
