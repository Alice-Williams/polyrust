//! Actual solved requests keep original extent after count authority is retired.
use super::*;
use crate::ast::contextual::flow_graph::Action;
use crate::ast::{buffer_fixture::Buffer, numeric_fixture::Fixture, *};
use crate::ownership::paths::{Key, Root};

#[test]
fn retired_count_activations_cannot_restore_or_supply_a_new_relational_bound() {
    let mut f = Fixture::new(&[]);
    let buffer = Buffer::new(&mut f, CObjectType::scalar(CScalarType::Size));
    let count = f.size(2);
    let mut body = buffer.prefix(&mut f, count);
    body.push(buffer.write(&f, 0, f.size(7)));
    body.push(buffer.release(&f));
    let files = [f.source(body)];
    let context = ContextFacts::check(&f.registry, &files).unwrap();
    let graph = &context.functions()[0];
    let loops = loops::check(&context).unwrap();
    let incoming = solve::function(&context, graph, State::default(), &loops).unwrap();
    let memory = incoming
        .into_iter()
        .flatten()
        .find_map(|product| {
            product
                .memory
                .roots
                .keys()
                .any(|root| matches!(root, Root::Allocation(..)))
                .then_some(product.memory)
        })
        .unwrap();
    let root = memory
        .roots
        .keys()
        .find(|root| matches!(root, Root::Allocation(..)))
        .unwrap()
        .clone();
    let Root::Allocation(origin, _) = &root else {
        unreachable!()
    };
    let base = Key::from_root(root.clone());
    assert!(!base.whole_root());
    assert!(base.parent().is_none());
    assert!(base.allocation_base());
    // The frozen LP64 storage identity normalizes size_t to the ABI U64 type.
    assert_eq!(base.ty(), CObjectType::scalar(CScalarType::U64));
    assert_eq!(
        memory.allocations.current_count(&root),
        Some(buffer.count.clone())
    );
    let declaration = graph
        .nodes()
        .iter()
        .find_map(|node| match node.action() {
            Action::Declare(value) if value.local() == buffer.count.local() => Some(node.action()),
            _ => None,
        })
        .unwrap();
    for scope_exit in [false, true] {
        let mut retired = memory.clone();
        if scope_exit {
            retired.leave(buffer.count.local().scope());
        } else {
            // Re-execute the actual declaration transfer, not a caller-provided
            // proof flag. A fresh local value cannot refresh the old request.
            Engine {
                context: &context,
                site: None,
                numeric: None,
            }
            .action(declaration, &mut retired)
            .unwrap();
        }
        assert_eq!(retired.allocations.current_count(&root), None);
        assert_eq!(
            retired.allocations.buffer_bounds(&root, &f.registry),
            Ok((2, 2))
        );
        assert_eq!(
            retired
                .allocations
                .restored_root(origin, &buffer.descriptor, &f.registry, false),
            Err(E::UnprovedAllocation)
        );
        for joined in [
            memory.join(&retired, &f.registry).unwrap(),
            retired.join(&memory, &f.registry).unwrap(),
        ] {
            assert_eq!(joined.allocations.current_count(&root), None);
        }
    }
}
