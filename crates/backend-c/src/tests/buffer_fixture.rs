//! Dynamic buffer tests construct actual counts, calls, guards and element places.
use super::contextual_reconstruction::key;
use super::{
    allocation_fixture::*, heap_fixture::*, numeric_fixture::Fixture, storage_fixture::*, *,
};

pub(crate) struct Buffer {
    pub count: CBufferCountRef,
    pub descriptor: CAllocationRef,
    pub raw: CLocalRef,
    pub data: CLocalRef,
    element: CObjectType,
}
impl Buffer {
    pub fn new(f: &mut Fixture, element: CObjectType) -> Self {
        Self::scoped(f, &f.scope.clone(), element)
    }
    pub fn scoped(f: &mut Fixture, scope: &CScopeRef, element: CObjectType) -> Self {
        Self::scoped_named(f, scope, element, "buffer")
    }
    pub fn named(f: &mut Fixture, element: CObjectType, name: &str) -> Self {
        Self::scoped_named(f, &f.scope.clone(), element, name)
    }
    fn scoped_named(f: &mut Fixture, scope: &CScopeRef, element: CObjectType, name: &str) -> Self {
        let count = f
            .registry
            .register_buffer_count(scope, key(&format!("{name}_count")))
            .unwrap();
        let descriptor = f
            .registry
            .register_buffer_allocation(
                scope,
                key(&format!("{name}_allocation")),
                element.clone(),
                count.clone(),
                CAllocatorSource::Default,
            )
            .unwrap();
        let raw = raw(f, &format!("{name}_raw"));
        let data = local(f, pointer_type(element.clone()), &format!("{name}_data"));
        Self {
            count,
            descriptor,
            raw,
            data,
            element,
        }
    }
    pub fn prefix(&self, f: &mut Fixture, count: CValue) -> Vec<CStatement> {
        let product = f.binary(
            CBinaryOperator::Multiply,
            f.read(self.count.local()),
            f.values().size_of(self.element.clone()).unwrap(),
        );
        let live = require_live(f, &self.raw, vec![]);
        vec![
            f.declare(self.count.local(), count),
            f.declare(&self.raw, allocate(f, product)),
            live,
            f.declare(&self.data, restore(f, &self.descriptor, f.read(&self.raw))),
        ]
    }
    pub fn place(&self, f: &Fixture, index: CValue) -> CPlace {
        f.values()
            .index(CIndexBase::Pointer(Box::new(f.read(&self.data))), index)
            .unwrap()
    }
    pub fn write(&self, f: &Fixture, index: u64, value: CValue) -> CStatement {
        f.ast().assign(self.place(f, f.size(index)), value).unwrap()
    }
    pub fn read(&self, f: &Fixture, index: u64) -> CValue {
        f.values().read(self.place(f, f.size(index))).unwrap()
    }
    pub fn release(&self, f: &Fixture) -> CStatement {
        release(f, f.read(&self.raw))
    }
}
