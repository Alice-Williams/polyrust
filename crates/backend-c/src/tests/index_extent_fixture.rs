//! Small actual-AST fixtures shared by extent consumers, without proof inputs.
use super::{contextual_reconstruction::key, numeric_fixture::Fixture, *};

pub(super) fn array(f: &mut Fixture, length: u64, name: &str) -> CLocalRef {
    let ty = CObjectType::array(
        CObjectType::scalar(CScalarType::Int),
        CArrayLength::new(length).unwrap(),
    )
    .unwrap();
    f.registry.register_local(&f.scope, key(name), ty).unwrap()
}
pub(super) fn declare(f: &Fixture, local: &CLocalRef) -> CStatement {
    f.ast()
        .declare(
            local.clone(),
            Some(f.values().zero_initializer(local.ty().clone()).unwrap()),
        )
        .unwrap()
}
pub(super) fn place(f: &Fixture, local: &CLocalRef, index: CValue) -> CPlace {
    f.values()
        .index(
            CIndexBase::Array(Box::new(f.values().local(local.clone()).unwrap())),
            index,
        )
        .unwrap()
}
pub(super) fn read(f: &Fixture, local: &CLocalRef, index: CValue) -> CValue {
    f.values().read(place(f, local, index)).unwrap()
}
pub(super) fn check(f: &Fixture, body: Vec<CStatement>, expected: Result<(), CSafetyError>) {
    assert_eq!(f.registry.check_index_extents(&[f.source(body)]), expected);
}
