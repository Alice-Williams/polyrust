//! Actual C builders for raw allocation tests; no caller-authored evidence.
use super::{numeric_fixture::Fixture, storage_fixture::local, *};
use crate::dialect::CKnownCall;

pub(super) fn raw(f: &mut Fixture, name: &str) -> CLocalRef {
    local(
        f,
        CObjectType::pointer(CPointerTarget::Void(CConstness::Unqualified)),
        name,
    )
}
pub(super) fn allocate(f: &Fixture, bytes: CValue) -> CValue {
    f.values()
        .call_value(f.values().known(CKnownCall::Allocate), vec![bytes])
        .unwrap()
}
pub(super) fn release(f: &Fixture, pointer: CValue) -> CStatement {
    f.ast()
        .evaluate(
            f.values()
                .call_effect(f.values().known(CKnownCall::Release), vec![pointer])
                .unwrap(),
        )
        .unwrap()
}
pub(super) fn null(f: &Fixture, pointer: &CLocalRef) -> CValue {
    f.values()
        .literal(CLiteral::NullPointer(
            CNullPointer::new(pointer.ty().clone()).unwrap(),
        ))
        .unwrap()
}
pub(super) fn nonnull(f: &Fixture, pointer: CValue) -> CValue {
    f.boolean(
        f.values()
            .pointer_test(CPointerTest::IsNonNull(Box::new(pointer)))
            .unwrap(),
    )
}
