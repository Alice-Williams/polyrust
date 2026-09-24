//! Certificate-owned type identity survives imports without copied definitions.
use super::nominal_fixture::{self as fixture, Operation};
use crate::{ast::*, dialect::*};
use portable_codegen::*;

#[test]
fn type_only_imports_copy_construct_and_read_original_fields() {
    let producer = fixture::producer();
    let proof = producer.structs().next().unwrap().clone();
    let input = fixture::fixture(
        80,
        std::slice::from_ref(&proof),
        &[
            Operation::Copy,
            Operation::Construct,
            Operation::Payload,
            Operation::Tag,
        ],
    );
    let expected = input.functions.clone();
    let package = fixture::certify(input).unwrap();
    let output = render_certified_package(&CStructuralRenderer, &package).unwrap();
    for file in output.files() {
        let OutputContents::Text(text) = file.contents() else {
            panic!("text")
        };
        assert!(!text.contains("struct poly_result {"));
    }
    let header = output
        .files()
        .iter()
        .find(|f| f.path().ends_with(".h"))
        .unwrap();
    let OutputContents::Text(text) = header.contents() else {
        panic!("text")
    };
    assert!(text.contains("#include \"polyrust_result.h\""), "{text}");
    let api = CDependencyApi::from_certificate(package).unwrap();
    assert_eq!(api.structure(proof.record()), Some(&proof));
    assert_eq!(api.functions().count(), expected.len());
    for (function, expected) in api.functions().zip(expected) {
        assert_eq!(function.signature(), expected.signature());
    }
}

#[test]
fn result_calls_relay_without_rebranding_nominal_types() {
    let producer = fixture::producer();
    let proof = producer.structs().next().unwrap().clone();
    let construct = producer
        .functions()
        .find(|f| f.function().key().name.as_str() == "construct")
        .unwrap();
    let relay = CDependencyApi::from_certificate(
        fixture::certify(fixture::fixture(
            81,
            std::slice::from_ref(&proof),
            &[Operation::Call(construct.clone())],
        ))
        .unwrap(),
    )
    .unwrap();
    assert_eq!(relay.structure(proof.record()), Some(&proof));
    let consumer = fixture::certify(fixture::fixture(
        82,
        std::slice::from_ref(&proof),
        &[
            Operation::Call(relay.functions().next().unwrap().clone()),
            Operation::Payload,
        ],
    ))
    .unwrap();
    render_certified_package(&CStructuralRenderer, &consumer).unwrap();
    let api = CDependencyApi::from_certificate(consumer).unwrap();
    assert_eq!(api.structure(proof.record()), Some(&proof));
}

#[test]
fn foreign_layout_membership_does_not_grant_mutation_or_declaration_rights() {
    let producer = fixture::producer();
    let proof = producer.structs().next().unwrap().clone();
    let owner = CAggregateRef::Struct(proof.record().clone());
    let mut r = CRegistry::new();
    assert!(r.members(&owner).is_err());
    let constructor = producer
        .functions()
        .find(|f| f.function().key().name.as_str() == "construct")
        .unwrap();
    assert!(r.import_function(constructor.clone()).is_err());
    r.import_struct(proof.clone()).unwrap();
    r.import_function(constructor.clone()).unwrap();
    assert_eq!(r.members(&owner).unwrap(), Some(proof.members()));
    let before = format!("{r:?}");
    assert!(r.import_struct(proof.clone()).is_err());
    assert!(
        r.define_aggregate(&owner, proof.members().to_vec())
            .is_err()
    );
    assert!(
        r.register_member(
            &owner,
            fixture::key("extra"),
            CObjectType::scalar(CScalarType::I32)
        )
        .is_err()
    );
    assert_eq!(format!("{r:?}"), before);
    let file = r
        .register_file(CFileKey {
            path: RelativeOutputPath::new("consumer.h").unwrap(),
            role: CFileRole::GeneratedPublicHeader,
        })
        .unwrap();
    let d = CDeclarations::new(&r, file).unwrap();
    assert!(d.aggregate(owner.clone()).is_err());
    assert!(d.forward_tag(owner).is_err());
}

#[test]
fn foreign_read_membership_does_not_authorize_local_interface_witnesses() {
    let producer = fixture::producer();
    let proof = producer.structs().next().unwrap().clone();
    let checked = portable_check::v0::check_program(
        portable_ir::v0::from_json(include_bytes!(
            "../../../build/testdata/registration.poly.json"
        ))
        .unwrap(),
    )
    .unwrap();
    let core = portable_core_ir::lower_checked(&checked).unwrap();
    let implementation = core
        .module()
        .declarations
        .iter()
        .find_map(|item| match item {
            portable_core_ir::CoreDeclaration::Implementation(id) => Some(*id),
            _ => None,
        })
        .unwrap();
    let mut registry = CRegistry::new();
    registry.import_struct(proof.clone()).unwrap();
    let file = registry
        .register_file(CFileKey {
            path: RelativeOutputPath::new("local_interface.h").unwrap(),
            role: CFileRole::GeneratedPublicHeader,
        })
        .unwrap();
    let local = registry
        .declare_struct(&file, fixture::key("Local"))
        .unwrap();
    let before = format!("{registry:?}");
    for (interface, record) in [
        (proof.record(), &local),
        (&local, proof.record()),
        (proof.record(), proof.record()),
    ] {
        assert!(
            registry
                .register_interface_witness(
                    fixture::key("Witness"),
                    implementation,
                    interface,
                    record,
                    vec![],
                )
                .is_err()
        );
        assert_eq!(format!("{registry:?}"), before);
    }
    // Failed foreign registrations must not consume the key or implementation.
    registry
        .register_interface_witness(
            fixture::key("Witness"),
            implementation,
            &local,
            &local,
            vec![],
        )
        .unwrap();
}

#[test]
fn linked_imported_fields_and_types_cannot_be_renamed_deleted_or_reowned() {
    let producer = fixture::producer();
    let proof = producer.structs().next().unwrap().clone();
    let other = super::nominal_producer::producer(130, &["other"], "other_api");
    let other = other.structs().next().unwrap().clone();
    let package = fixture::certify(fixture::fixture(131, &[proof], &[Operation::Payload])).unwrap();
    let unit = &package
        .ast()
        .files()
        .iter()
        .find(|file| file.module().key().path.as_str().ends_with(".c"))
        .unwrap()
        .items()[0];
    assert!(CDialect.verify_resolved_file_item(unit).is_empty());
    let field = unit
        .names
        .keys()
        .find(|symbol| matches!(symbol, TargetSymbolRef::KnownField(_)))
        .unwrap()
        .clone();
    let ty = unit
        .names
        .keys()
        .find(|symbol| {
            matches!(
                symbol,
                TargetSymbolRef::KnownType(CReferencedType::Certified(_))
            )
        })
        .unwrap()
        .clone();
    for mode in 0..5 {
        let mut altered = unit.clone();
        match mode {
            0 => {
                altered.names.remove(&field);
            }
            1 => {
                altered.names.remove(&ty);
            }
            2 => {
                let ResolvedReference::Member { member, .. } =
                    altered.names.get_mut(&field).unwrap()
                else {
                    panic!("member")
                };
                *member = CIdentifier::new("renamed").unwrap();
            }
            3 => {
                let ResolvedReference::Member { owner, .. } =
                    altered.names.get_mut(&field).unwrap()
                else {
                    panic!("member")
                };
                *owner = other.clone();
            }
            4 => {
                let ResolvedReference::Imported { binding, .. } =
                    altered.names.get_mut(&ty).unwrap()
                else {
                    panic!("type")
                };
                *binding = CIdentifier::new("renamed").unwrap();
            }
            _ => unreachable!(),
        }
        assert!(
            !CDialect.verify_resolved_file_item(&altered).is_empty(),
            "mode {mode}"
        );
    }
}

#[test]
fn foreign_registration_order_and_unused_types_do_not_change_output() {
    let producer = fixture::producer();
    let first = producer.structs().next().unwrap().clone();
    let other = super::nominal_producer::producer(132, &["other"], "other_api");
    let second = other.structs().next().unwrap().clone();
    let constructor = producer
        .functions()
        .find(|f| f.function().key().name.as_str() == "construct")
        .unwrap();
    let outputs: Vec<_> = [
        vec![first.clone()],
        vec![first.clone(), second.clone()],
        vec![second, first],
    ]
    .into_iter()
    .map(|proofs| {
        let package = fixture::certify(fixture::fixture(
            133,
            &proofs,
            &[Operation::Call(constructor.clone())],
        ))
        .unwrap();
        render_certified_package(&CStructuralRenderer, &package).unwrap()
    })
    .collect();
    assert_eq!(outputs[0], outputs[1]);
    assert_eq!(outputs[1], outputs[2]);
}

#[test]
fn a_registered_same_shape_type_cannot_authorize_another_signature_or_argument() {
    let producer = fixture::producer();
    let expected = producer.structs().next().unwrap().clone();
    let other = super::nominal_producer::producer(160, &["other_result"], "other_api");
    let other = other.structs().next().unwrap().clone();
    let inspect = producer
        .functions()
        .find(|f| f.function().key().name.as_str() == "inspect")
        .unwrap();
    let mut registry = CRegistry::new();
    registry.import_struct(other.clone()).unwrap();
    let before = format!("{registry:?}");
    assert!(registry.import_function(inspect.clone()).is_err());
    assert_eq!(format!("{registry:?}"), before);
    registry.import_struct(expected.clone()).unwrap();
    let imported = registry.import_function(inspect.clone()).unwrap();
    let source = registry
        .register_file(CFileKey {
            path: RelativeOutputPath::new("wrong_owner.c").unwrap(),
            role: CFileRole::TestSource,
        })
        .unwrap();
    let function = registry
        .register_function(
            &source,
            fixture::key("consumer"),
            CFunctionType::new(
                CReturnType::Value(
                    CReturnValue::new(CObjectType::scalar(CScalarType::I32)).unwrap(),
                ),
                vec![
                    CParameterType::new(CObjectType::structure(other.record().clone())).unwrap(),
                    CParameterType::new(CObjectType::structure(expected.record().clone())).unwrap(),
                ],
            ),
        )
        .unwrap();
    let wrong = registry
        .register_parameter(&function, 0, fixture::key("wrong"), CConstness::Unqualified)
        .unwrap();
    let correct = registry
        .register_parameter(
            &function,
            1,
            fixture::key("correct"),
            CConstness::Unqualified,
        )
        .unwrap();
    let e = CExpressions::new(&registry);
    for (parameter, valid) in [(wrong, false), (correct, true)] {
        let result = e.call_value(
            e.direct(imported.clone()).unwrap(),
            vec![
                e.read(e.parameter(parameter).unwrap()).unwrap(),
                e.literal(CLiteral::Bool(true)).unwrap(),
            ],
        );
        assert_eq!(result.is_ok(), valid);
    }
}

#[test]
fn private_and_unused_nominal_imports_do_not_widen_the_published_type_api() {
    let producer = fixture::producer();
    let proof = producer.structs().next().unwrap().clone();
    let scalar = producer
        .functions()
        .find(|f| f.function().key().name.as_str() == "entry")
        .unwrap();
    for operation in [Operation::Call(scalar.clone()), Operation::ScalarLocal] {
        let input = fixture::fixture(163, std::slice::from_ref(&proof), &[operation]);
        assert_eq!(input.registry.registrations().imported_structs().count(), 1);
        let package = fixture::certify(input).unwrap();
        let output = render_certified_package(&CStructuralRenderer, &package).unwrap();
        let header = output
            .files()
            .iter()
            .find(|f| f.path().ends_with(".h"))
            .unwrap();
        let OutputContents::Text(text) = header.contents() else {
            panic!("text")
        };
        assert!(!text.contains("struct poly_result"));
        assert!(!text.contains("polyrust_result.h"));
        let api = CDependencyApi::from_certificate(package).unwrap();
        assert_eq!(api.structs().count(), 0);
        assert!(api.structure(proof.record()).is_none());
        // Publication filtering must not discard the original registry authority.
        let retained = api.package().ast().files()[0].items()[0]
            .unit
            .projection
            .registry
            .registrations();
        assert_eq!(retained.imported_struct(proof.record()).unwrap(), &proof);
    }
}
