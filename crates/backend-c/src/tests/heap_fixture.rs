//! Fixed heap fixtures build actual calls, null guards and root restores.
use super::{allocation_fixture::*, contextual_reconstruction::key, numeric_fixture::Fixture, *};

pub(super) fn descriptor(f: &mut Fixture, name: &str, ty: CObjectType) -> CAllocationRef {
    f.registry
        .register_allocation(&f.scope, key(name), ty, CAllocatorSource::Default)
        .unwrap()
}
pub(super) fn require_live(
    f: &mut Fixture,
    raw: &CLocalRef,
    mut cleanup: Vec<CStatement>,
) -> CStatement {
    cleanup.push(f.ast().return_statement(None).unwrap());
    f.branch(nonnull(f, f.read(raw)), vec![], cleanup)
}
pub(super) fn restore(f: &Fixture, descriptor: &CAllocationRef, pointer: CValue) -> CValue {
    f.values()
        .allocation_restore(descriptor.clone(), pointer)
        .unwrap()
}
pub(super) fn assign(f: &Fixture, local: &CLocalRef, value: CValue) -> CStatement {
    f.ast()
        .assign(f.values().local(local.clone()).unwrap(), value)
        .unwrap()
}
pub(super) fn pointee(f: &Fixture, local: &CLocalRef) -> CPlace {
    f.values().dereference(f.read(local)).unwrap()
}
pub(super) fn erase(f: &Fixture, pointer: CValue) -> CValue {
    f.values()
        .object_to_void(
            CObjectType::pointer(CPointerTarget::Void(CConstness::Unqualified)),
            pointer,
        )
        .unwrap()
}
pub(super) fn guarded(
    f: &mut Fixture,
    raw: &CLocalRef,
    typed: &CLocalRef,
    descriptor: &CAllocationRef,
    bytes: u64,
    actions: Vec<CStatement>,
) -> Vec<CStatement> {
    let mut success = vec![assign(f, typed, restore(f, descriptor, f.read(raw)))];
    success.extend(actions);
    success.push(release(f, f.read(raw)));
    let branch = f.branch(nonnull(f, f.read(raw)), success, vec![]);
    vec![
        f.declare(raw, allocate(f, f.size(bytes))),
        f.declare(typed, null(f, typed)),
        branch,
    ]
}
