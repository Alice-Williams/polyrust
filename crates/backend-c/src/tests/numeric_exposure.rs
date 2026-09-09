//! Structural base traversal is not actual address-taking/escape exposure.
use super::{contextual_reconstruction::key, numeric_fixture::Fixture, *};
use crate::dialect::CKnownCall;

fn allocate(f: &Fixture, value: CValue) -> CStatement {
    f.discard(
        f.values()
            .call_value(f.values().known(CKnownCall::Allocate), vec![value])
            .unwrap(),
    )
}

fn check_member(exposed: bool) -> Result<(), CSafetyError> {
    let mut f = Fixture::new(&[]);
    let record = f.registry.declare_struct(&f.file, key("Record")).unwrap();
    let owner = CAggregateRef::Struct(record.clone());
    let member = f
        .registry
        .register_member(&owner, key("size"), CObjectType::scalar(CScalarType::Size))
        .unwrap();
    f.registry
        .define_aggregate(&owner, vec![member.clone()])
        .unwrap();
    let local = f
        .registry
        .register_local(
            &f.scope,
            key("record"),
            CObjectType::structure(record.clone()),
        )
        .unwrap();
    let initializer = f
        .values()
        .struct_initializer(
            record,
            vec![(
                member.clone(),
                f.values().expression_initializer(f.size(1)).unwrap(),
            )],
        )
        .unwrap();
    let place = f
        .values()
        .member(f.values().local(local.clone()).unwrap(), member)
        .unwrap();
    let mut body = vec![f.ast().declare(local, Some(initializer)).unwrap()];
    if exposed {
        body.push(f.discard(f.values().address_of(place.clone()).unwrap()));
    }
    body.push(allocate(&f, f.size(1)));
    body.push(allocate(&f, f.values().read(place).unwrap()));
    let mut source = f.source(body);
    source.items.insert(
        0,
        CFileItem::Declaration(
            CDeclarations::new(&f.registry, f.file.clone())
                .unwrap()
                .aggregate(owner)
                .unwrap(),
        ),
    );
    f.registry.check_numeric_flow(&[source])
}

fn check_array(exposed: bool) -> Result<(), CSafetyError> {
    let mut f = Fixture::new(&[]);
    let ty = CObjectType::array(
        CObjectType::scalar(CScalarType::Size),
        CArrayLength::new(1).unwrap(),
    )
    .unwrap();
    let local = f
        .registry
        .register_local(&f.scope, key("array"), ty.clone())
        .unwrap();
    let initializer = f
        .values()
        .array_initializer(
            ty,
            vec![f.values().expression_initializer(f.size(1)).unwrap()],
        )
        .unwrap();
    let place = f
        .values()
        .index(
            CIndexBase::Array(Box::new(f.values().local(local.clone()).unwrap())),
            f.size(0),
        )
        .unwrap();
    let mut body = vec![f.ast().declare(local, Some(initializer)).unwrap()];
    if exposed {
        body.push(f.discard(f.values().address_of(place.clone()).unwrap()));
    }
    body.push(allocate(&f, f.size(1)));
    body.push(allocate(&f, f.values().read(place).unwrap()));
    f.check(body)
}

#[test]
fn member_reads_do_not_expose_local_storage_to_opaque_calls() {
    assert_eq!(check_member(false), Ok(()));
    assert_eq!(
        check_member(true),
        Err(CSafetyError::UnprovedSizeArithmetic)
    );
}

#[test]
fn array_reads_do_not_expose_local_storage_to_opaque_calls() {
    assert_eq!(check_array(false), Ok(()));
    assert_eq!(check_array(true), Err(CSafetyError::UnprovedSizeArithmetic));
}
