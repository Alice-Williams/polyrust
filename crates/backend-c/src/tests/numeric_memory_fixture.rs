//! Heap fixtures establish evidence only through actual statements and calls.
use super::{
    allocation_fixture::*, heap_fixture::*, numeric_fixture::Fixture, storage_fixture::*, *,
};

pub(super) struct Heap {
    pub raw: CLocalRef,
    pub typed: CLocalRef,
    allocation: CAllocationRef,
}
impl Heap {
    pub(super) fn new(f: &mut Fixture, name: &str, ty: CObjectType) -> Self {
        Self {
            raw: raw(f, &format!("{name}_raw")),
            typed: local(f, pointer_type(ty.clone()), &format!("{name}_typed")),
            allocation: descriptor(f, &format!("{name}_allocation"), ty),
        }
    }
    pub(super) fn prefix(
        &self,
        f: &mut Fixture,
        bytes: u64,
        cleanup: Vec<CStatement>,
    ) -> Vec<CStatement> {
        vec![
            f.declare(&self.raw, allocate(f, f.size(bytes))),
            require_live(f, &self.raw, cleanup),
            f.declare(&self.typed, restore(f, &self.allocation, f.read(&self.raw))),
        ]
    }
    pub(super) fn place(&self, f: &Fixture) -> CPlace {
        pointee(f, &self.typed)
    }
    pub(super) fn read(&self, f: &Fixture) -> CValue {
        f.values().read(self.place(f)).unwrap()
    }
    pub(super) fn write(&self, f: &Fixture, value: CValue) -> CStatement {
        f.ast().assign(self.place(f), value).unwrap()
    }
    pub(super) fn release(&self, f: &Fixture) -> CStatement {
        release(f, f.read(&self.raw))
    }
}
pub(super) fn consume(f: &mut Fixture, value: CValue) -> Vec<CStatement> {
    let next = raw(f, "next");
    vec![
        f.declare(&next, allocate(f, value)),
        release(f, f.read(&next)),
    ]
}
pub(super) fn wrapped(f: &Fixture) -> CValue {
    // Exactly two after unsigned wrap, not zero or an independently invalid size.
    f.binary(
        CBinaryOperator::Multiply,
        f.size((1_u64 << 63) + 1),
        f.size(2),
    )
}
pub(super) fn positive(f: &Fixture, value: CValue) -> CValue {
    f.compare(CBinaryOperator::Greater, value, f.size(0))
}
pub(super) fn assume(f: &mut Fixture, condition: CValue, cleanup: Vec<CStatement>) -> CStatement {
    let mut no = cleanup;
    no.push(f.ast().return_statement(None).unwrap());
    f.branch(condition, vec![], no)
}
