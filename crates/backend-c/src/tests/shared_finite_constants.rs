//! Finite bits, original owner authority and resource proofs for const objects.
use super::{
    CDependencyApi, CDialect, CStructuralRenderer, constant_consumer_fixture,
    constant_export_fixture,
    constant_producer_tests::certify,
    owned_constant_fixture::{self, Fixture},
    owned_constant_tests::linked,
    project_c_package,
};
use crate::ast::*;
use portable_binary64::FiniteBinary64;
use portable_codegen::*;

#[path = "shared_finite_constant_native.rs"]
mod native;
#[path = "shared_finite_constant_rejections.rs"]
mod rejections;

fn literal(bits: u64) -> CLiteral {
    CLiteral::F64(FiniteBinary64::from_bits(bits).unwrap())
}

fn fixture(bits: &[u64]) -> Fixture {
    let names: Vec<_> = (0..bits.len())
        .map(|index| format!("finite_{index}"))
        .collect();
    let literals: Vec<_> = names
        .iter()
        .zip(bits)
        .map(|(name, bits)| (name.as_str(), literal(*bits)))
        .collect();
    owned_constant_fixture::literal_fixture(&literals)
}

fn api(fixture: &Fixture) -> CDependencyApi {
    CDependencyApi::from_certificate(certify(fixture)).unwrap()
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
fn finite_constants_keep_bits_types_original_alias_authority_and_bounds() {
    let bits = [0, 1 << 63, 1, 0x0010_0000_0000_0000, 0x7fef_ffff_ffff_ffff];
    let source = fixture(&bits);
    let original = api(&source);
    assert_eq!(original.constants().count(), bits.len());
    assert_eq!(original.functions().count(), 0);
    for (value, bits) in original.constants().zip(bits) {
        assert_eq!(value.value(), &literal(bits));
        assert_eq!(value.read_type(), &CObjectType::scalar(CScalarType::F64));
        assert_eq!(value.object().ty().constness(), CConstness::Const);
        assert_eq!(value.package_identity().stack_bound_bytes(), 0);
    }
    check_resources(&original);
    let values: Vec<_> = original.constants().cloned().collect();
    let facade = api(&constant_export_fixture::facade(921, &values));
    assert_eq!(facade.constants().count(), 0);
    let aliases: Vec<_> = facade
        .foreign_constants()
        .map(|binding| binding.dependency().clone())
        .collect();
    assert_eq!(aliases, values);
    check_resources(&facade);
    let reader = constant_consumer_fixture::fixture(
        922,
        &aliases,
        None,
        constant_consumer_fixture::Usage::Read,
    );
    let reader = api(&reader);
    assert_eq!(reader.functions().count(), bits.len());
    check_resources(&reader);
    let source = linked(&source);
    let measured = super::resources::measure_package(&source).unwrap();
    assert_eq!(measured.total.frame_bound, 0);
    assert!(measured.total.value_bytes >= 8 * bits.len() as u64);
}

#[test]
fn zero_sign_and_identical_lookalikes_cannot_replace_original_certificates() {
    let mut source = fixture(&[0]);
    let original = api(&source);
    let object = source.objects[0].clone();
    let expressions = CExpressions::new(source.registry.registrations());
    let declarations = CDeclarations::new(
        source.registry.registrations(),
        source.files[1].identity().clone(),
    )
    .unwrap();
    let initializer = expressions
        .expression_initializer(expressions.literal(literal(1 << 63)).unwrap())
        .unwrap();
    source.files[1] = declarations
        .source_file(vec![CFileItem::Definition(
            declarations
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
    assert_ne!(before.cmp(after), std::cmp::Ordering::Equal);
    let lookalike = api(&fixture(&[0]));
    assert_eq!(
        before.value(),
        lookalike.constants().next().unwrap().value()
    );
    for replacement in [after, lookalike.constants().next().unwrap()] {
        assert_ne!(before.package_identity(), replacement.package_identity());
        let mut registry = CRegistry::new();
        registry.import_constant(before.clone()).unwrap();
        assert!(registry.import_constant(replacement.clone()).is_err());
    }
}
