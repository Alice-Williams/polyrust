//! Actual initialized allocations and annotated locals; no seeded owner state.
use super::{
    allocation_fixture::*, heap_fixture::*, numeric_fixture::Fixture, storage_fixture::*, *,
};

pub(super) struct Leaf {
    pub f: Fixture,
    pub raw: CLocalRef,
    pub temporary: CLocalRef,
    pub first: CLocalRef,
    pub second: CLocalRef,
    pub allocation: CAllocationRef,
}
impl Leaf {
    pub fn new() -> Self {
        Self::with_type(
            Fixture::new(&[CScalarType::Bool]),
            CObjectType::scalar(CScalarType::Int),
        )
    }
    pub fn with_type(mut f: Fixture, ty: CObjectType) -> Self {
        let raw = raw(&mut f, "raw");
        let allocation = descriptor(&mut f, "leaf", ty.clone());
        let temporary = local(&mut f, pointer_type(ty.clone()), "temporary");
        let first = local(&mut f, pointer_type(ty.clone()), "first");
        let second = local(&mut f, pointer_type(ty), "second");
        f.registry.register_local_owner(&first).unwrap();
        f.registry.register_local_owner(&second).unwrap();
        Self {
            f,
            raw,
            temporary,
            first,
            second,
            allocation,
        }
    }
    pub fn drop(&self, local: &CLocalRef) -> Vec<CStatement> {
        vec![
            release(&self.f, erase(&self.f, self.f.read(local))),
            assign(&self.f, local, null(&self.f, local)),
        ]
    }
    pub fn moved(&self, source: &CLocalRef, destination: &CLocalRef) -> Vec<CStatement> {
        vec![
            assign(&self.f, destination, self.f.read(source)),
            assign(&self.f, source, null(&self.f, source)),
        ]
    }
    pub fn body(&mut self, initialized: bool, actions: Vec<CStatement>) -> Vec<CStatement> {
        let construction = if initialized {
            vec![
                self.f
                    .ast()
                    .assign(pointee(&self.f, &self.temporary), self.f.int(7))
                    .unwrap(),
            ]
        } else {
            vec![]
        };
        self.body_with(4, construction, actions)
    }
    pub fn body_with(
        &mut self,
        bytes: u64,
        construction: Vec<CStatement>,
        actions: Vec<CStatement>,
    ) -> Vec<CStatement> {
        let mut success = vec![assign(
            &self.f,
            &self.temporary,
            restore(&self.f, &self.allocation, self.f.read(&self.raw)),
        )];
        success.extend(construction);
        success.push(assign(&self.f, &self.first, self.f.read(&self.temporary)));
        success.extend(actions);
        let guard = require_live(&mut self.f, &self.raw, vec![]);
        let mut body = vec![
            self.f
                .declare(&self.raw, allocate(&self.f, self.f.size(bytes))),
            self.f
                .declare(&self.temporary, null(&self.f, &self.temporary)),
            self.f.declare(&self.first, null(&self.f, &self.first)),
            self.f.declare(&self.second, null(&self.f, &self.second)),
            guard,
        ];
        body.extend(success);
        body
    }
    pub fn check(
        mut self,
        initialized: bool,
        actions: Vec<CStatement>,
        expected: Result<(), CSafetyError>,
    ) {
        let body = self.body(initialized, actions);
        let source = self.f.source(body);
        self.f
            .registry
            .check_package_structure(std::slice::from_ref(&source))
            .unwrap();
        let actual = self.f.registry.check_storage_paths(&[source]);
        // Strict replay may report the concrete expired read or its poisoned
        // owner join first; neither is an accepted lifecycle certificate.
        if expected == Err(CSafetyError::UnprovedOwnership)
            && actual == Err(CSafetyError::ExpiredStorage)
        {
            return;
        }
        assert_eq!(actual, expected);
    }
}
