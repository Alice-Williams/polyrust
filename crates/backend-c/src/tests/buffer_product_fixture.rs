//! Product evidence is obtained from real checked graphs and their actual states.
use super::*;
use crate::ast::{numeric_fixture::Fixture, *};

pub(super) fn key(name: &str) -> CDeclarationKey {
    CDeclarationKey {
        name: CIdentifier::new(name).unwrap(),
        origin: CGeneratedOrigin::Synthesized(CSynthesisReason::TestHarness),
    }
}
pub(super) fn count(f: &mut Fixture, name: &str) -> CBufferCountRef {
    f.registry
        .register_buffer_count(&f.scope, key(name))
        .unwrap()
}
pub(super) fn buffer(
    f: &mut Fixture,
    name: &str,
    count: &CBufferCountRef,
    element: CObjectType,
) -> CAllocationRef {
    f.registry
        .register_buffer_allocation(
            &f.scope,
            key(name),
            element,
            count.clone(),
            CAllocatorSource::Default,
        )
        .unwrap()
}
pub(super) fn immutable(f: &mut Fixture, name: &str, ty: CScalarType) -> CLocalRef {
    f.registry
        .register_local(
            &f.scope,
            key(name),
            CObjectType::scalar(ty)
                .with_constness(CConstness::Const)
                .unwrap(),
        )
        .unwrap()
}
pub(super) fn allocate(f: &Fixture, bytes: CValue) -> CStatement {
    f.discard(
        f.values()
            .call_value(f.values().known(CKnownCall::Allocate), vec![bytes])
            .unwrap(),
    )
}
pub(super) fn multiply(f: &Fixture, left: CValue, right: CValue) -> CValue {
    f.binary(CBinaryOperator::Multiply, left, right)
}
pub(super) fn requests(f: &Fixture, body: Vec<CStatement>) -> Result<Vec<AllocationRequest>, E> {
    let files = [f.source(body)];
    let facts = NumericFacts::check(&f.registry, &files)?;
    facts
        .analysis
        .obligations
        .iter()
        .filter_map(|entry| {
            let Obligation::Call {
                call, arguments, ..
            } = &entry.kind
            else {
                return None;
            };
            if call.callable().kind() != &CCallableKind::Known(CKnownCall::Allocate) {
                return None;
            }
            let index = facts
                .context
                .functions()
                .iter()
                .position(|graph| graph.function() == entry.site.function)
                .unwrap();
            let state = facts.analysis.functions[index].incoming[entry.site.point.index()]
                .as_ref()
                .unwrap();
            Some(AllocationRequest::actual(
                &facts.context,
                &facts.context.functions()[index],
                entry.site.point,
                call,
                arguments[0].as_ref().unwrap(),
                state,
            ))
        })
        .collect()
}
pub(super) fn assume(f: &mut Fixture, condition: CValue) -> CStatement {
    f.branch(
        condition,
        vec![],
        vec![f.ast().return_statement(None).unwrap()],
    )
}
