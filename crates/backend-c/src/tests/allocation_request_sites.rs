//! Allocation request identity/extent controls inside its private proof boundary.
use super::*;
use crate::ast::{CValueKind, numeric_fixture::Fixture};

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
