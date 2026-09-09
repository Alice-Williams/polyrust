//! Actual AST helpers; tests do not seed storage evidence.
use super::{contextual_reconstruction::key, numeric_fixture::Fixture, *};

pub(super) fn local(f: &mut Fixture, ty: CObjectType, name: &str) -> CLocalRef {
    f.registry.register_local(&f.scope, key(name), ty).unwrap()
}
pub(super) fn address(f: &Fixture, place: CPlace) -> CValue {
    f.values().address_of(place).unwrap()
}
pub(super) fn pointer_type(ty: CObjectType) -> CObjectType {
    CObjectType::pointer(CPointerTarget::Object(Box::new(ty)))
}
pub(super) fn pointer_index(f: &Fixture, pointer: CValue, index: u64) -> CPlace {
    f.values()
        .index(CIndexBase::Pointer(Box::new(pointer)), f.size(index))
        .unwrap()
}
pub(super) fn pointed_read(f: &Fixture, pointer: CValue) -> CValue {
    f.values()
        .read(f.values().dereference(pointer).unwrap())
        .unwrap()
}
pub(super) fn check(f: &Fixture, body: Vec<CStatement>, expected: Result<(), CSafetyError>) {
    assert_eq!(f.registry.check_storage_paths(&[f.source(body)]), expected);
}
pub(super) fn aggregate_source(
    f: &Fixture,
    owner: CAggregateRef,
    body: Vec<CStatement>,
) -> CSourceFile {
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
    source
}
