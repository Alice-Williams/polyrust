//! Allocation request identity/extent controls inside its private proof boundary.
use super::*;
use crate::ast::{CValueKind, numeric_fixture::Fixture};

#[test]
fn object_layout_requires_original_bytes_and_measured_alignment() {
    use crate::ast::{
        CAllocatorSource, CDeclarationKey, CGeneratedOrigin, CIdentifier, CScalarType,
        CSynthesisReason,
    };
    let mut f = Fixture::new(&[]);
    let object = f
        .registry
        .register_allocation(
            &f.scope,
            CDeclarationKey {
                name: CIdentifier::new("object").unwrap(),
                origin: CGeneratedOrigin::Synthesized(CSynthesisReason::TestHarness),
            },
            CObjectType::scalar(CScalarType::I64),
            CAllocatorSource::Default,
        )
        .unwrap();
    let value = f
        .values()
        .call_value(f.values().known(CKnownCall::Allocate), vec![f.size(8)])
        .unwrap();
    let files = [f.source(vec![f.discard(value)])];
    let facts = NumericFacts::check(&f.registry, &files).unwrap();
    let actual = facts
        .analysis
        .obligations
        .iter()
        .find_map(|entry| match &entry.kind {
            Obligation::Call { call, .. } => Some(*call),
            _ => None,
        })
        .unwrap();
    let request = facts.allocation_request(actual).unwrap();
    assert_eq!(request.admits_object(&object, &f.registry), Ok(()));
    for (bytes, alignment) in [((7, 64), 16), ((8, 8), 4)] {
        let mut insufficient = request.clone();
        insufficient.bytes = bytes;
        insufficient.alignment = alignment;
        assert_eq!(
            insufficient.admits_object(&object, &f.registry),
            Err(E::UnprovedAllocationSize)
        );
    }
}

#[test]
fn request_retains_actual_call_site_bytes_and_measured_alignment() {
    let f = Fixture::new(&[]);
    let value = f
        .values()
        .call_value(f.values().known(CKnownCall::Allocate), vec![f.size(19)])
        .unwrap();
    let files = [f.source(vec![f.discard(value)])];
    let facts = NumericFacts::check(&f.registry, &files).unwrap();
    let entry = facts
        .analysis
        .obligations
        .iter()
        .find(|entry| matches!(entry.kind, Obligation::Call { .. }))
        .unwrap();
    let Obligation::Call { call, .. } = &entry.kind else {
        panic!("call")
    };
    let request = facts.allocation_request(call).unwrap();
    assert_eq!(request.bytes, (19, 19));
    assert_eq!(request.alignment, 16);
    assert_eq!(&request.origin.function, entry.site.function);
    assert_eq!(request.origin.point, entry.site.point);
    assert_eq!(request.scope, f.scope);
    let cloned = (*call).clone();
    assert_eq!(
        facts.allocation_request(&cloned),
        Err(E::InvalidNumericSite)
    );
    let graph = &facts.context.functions()[0];
    let crate::ast::contextual::flow_graph::Action::Discard(value) =
        graph.node(entry.site.point).action()
    else {
        panic!("discard")
    };
    let CValueKind::Call(actual) = value.kind() else {
        panic!("call")
    };
    assert!(std::ptr::eq(*call, actual));
}

#[test]
fn allocation_request_rejects_a_reassigned_graph_site() {
    let f = Fixture::new(&[]);
    let call = f
        .values()
        .call_value(f.values().known(CKnownCall::Allocate), vec![f.size(8)])
        .unwrap();
    let files = [f.source(vec![f.discard(call), f.discard(f.int(1))])];
    let mut facts = NumericFacts::check(&f.registry, &files).unwrap();
    let graph = &facts.context.functions()[0];
    let wrong = graph
        .points()
        .find(|point| {
            matches!(
                graph.node(*point).action(),
                crate::ast::contextual::flow_graph::Action::FunctionEnd
            )
        })
        .unwrap();
    let entry = facts
        .analysis
        .obligations
        .iter_mut()
        .find(|entry| matches!(entry.kind, Obligation::Call { .. }))
        .unwrap();
    entry.site.point = wrong;
    assert_eq!(facts.validate_sites(), Err(E::InvalidNumericSite));
}

#[test]
fn byte_extent_is_captured_at_the_producing_call_not_a_later_write() {
    let mut f = Fixture::new(&[]);
    let size = f.local(crate::ast::CScalarType::Size, "size");
    let call = f
        .values()
        .call_value(f.values().known(CKnownCall::Allocate), vec![f.read(&size)])
        .unwrap();
    let files = [f.source(vec![
        f.declare(&size, f.size(32)),
        f.discard(call),
        f.ast()
            .assign(f.values().local(size).unwrap(), f.size(1))
            .unwrap(),
    ])];
    let facts = NumericFacts::check(&f.registry, &files).unwrap();
    let actual = facts
        .analysis
        .obligations
        .iter()
        .find_map(|entry| match &entry.kind {
            Obligation::Call { call, .. } => Some(*call),
            _ => None,
        })
        .unwrap();
    assert_eq!(facts.allocation_request(actual).unwrap().bytes, (32, 32));
}

#[test]
fn composed_requests_reject_cloned_calls_wrong_points_and_foreign_graphs() {
    use crate::ast::contextual::flow_graph::Action;
    let f = Fixture::new(&[]);
    let call = f
        .values()
        .call_value(f.values().known(CKnownCall::Allocate), vec![f.size(8)])
        .unwrap();
    let files = [f.source(vec![f.discard(call)])];
    let facts = NumericFacts::check(&f.registry, &files).unwrap();
    let entry = facts
        .analysis
        .obligations
        .iter()
        .find(|e| matches!(e.kind, Obligation::Call { .. }))
        .unwrap();
    let Obligation::Call {
        call, arguments, ..
    } = &entry.kind
    else {
        panic!("call");
    };
    let bytes = arguments[0].as_ref().unwrap();
    let graph = &facts.context.functions()[0];
    let expected = facts.allocation_request(call).unwrap();
    let state = crate::ownership::numeric_flow::State::default();
    assert_eq!(
        AllocationRequest::actual(&facts.context, graph, entry.site.point, call, bytes, &state),
        Ok(expected.clone())
    );
    assert_eq!(
        AllocationRequest::actual(
            &facts.context,
            graph,
            entry.site.point,
            &(*call).clone(),
            bytes,
            &state
        ),
        Err(E::InvalidNumericSite)
    );
    let wrong = graph
        .points()
        .find(|p| matches!(graph.node(*p).action(), Action::FunctionEnd))
        .unwrap();
    assert_eq!(
        AllocationRequest::actual(&facts.context, graph, wrong, call, bytes, &state),
        Err(E::InvalidNumericSite)
    );
    let second = crate::ownership::context_facts::ContextFacts::check(&f.registry, &files).unwrap();
    assert_eq!(
        AllocationRequest::actual(
            &facts.context,
            &second.functions()[0],
            entry.site.point,
            call,
            bytes,
            &state
        ),
        Err(E::InvalidNumericSite)
    );
    let mut other = expected.clone();
    other.bytes = (4, 16);
    assert_eq!(expected.join(&other).unwrap().bytes, (4, 16));
    other.alignment = 8;
    assert_eq!(expected.join(&other), None);
}
