//! Actual graph transfers retire snapshot authority and its captured addresses.
use super::{audit, product::Product, solve};
use crate::ast::{
    buffer_fixture::Buffer,
    buffer_prefix_fixture::{Counted, immutable_size},
    contextual::flow_graph::Action,
    numeric_fixture::Fixture,
    *,
};
use crate::ownership::{
    context_facts::ContextFacts,
    loops,
    storage::{prefixes::Bound, state::State},
};

fn pointer_type(ty: CObjectType) -> CObjectType {
    CObjectType::pointer(CPointerTarget::Object(Box::new(ty)))
}
fn address(f: &Fixture, place: CPlace) -> CValue {
    f.values().address_of(place).unwrap()
}
fn pointed_read(f: &Fixture, pointer: CValue) -> CValue {
    f.values()
        .read(f.values().dereference(pointer).unwrap())
        .unwrap()
}

#[test]
fn replayed_snapshot_declaration_retires_captured_old_activation_addresses() {
    for points_at_snapshot in [false, true] {
        let mut f = Fixture::new(&[]);
        let construction = Counted::new(&mut f, "construct");
        let snapshot = immutable_size(&mut f, "snapshot");
        let stable = immutable_size(&mut f, "stable");
        let buffer = Buffer::new(&mut f, pointer_type(snapshot.ty().clone()));
        let mut body = vec![
            construction.declaration(&f),
            f.declare(&snapshot, f.read(&construction.counter)),
            f.declare(&stable, f.size(7)),
        ];
        let count = f.size(2);
        body.extend(buffer.prefix(&mut f, count));
        let target = if points_at_snapshot {
            &snapshot
        } else {
            &stable
        };
        let pointer = address(&f, f.values().local(target.clone()).unwrap());
        body.push(construction.finish(
            &f,
            buffer.count.local(),
            vec![
                f.ast().assign(buffer.place(&f, f.read(&construction.counter)), pointer).unwrap(),
                construction.step(&f),
            ],
        ));
        body.push(f.discard(pointed_read(&f, buffer.read(&f, 0))));
        body.push(buffer.release(&f));
        let files = [f.source(body)];
        assert_eq!(f.registry.check_storage_paths(&files), Ok(()));
        let context = ContextFacts::check(&f.registry, &files).unwrap();
        let graph = &context.functions()[0];
        let loops = loops::check(&context).unwrap();
        let incoming = solve::function(&context, graph, State::default(), &loops).unwrap();
        audit::check(&context, graph, &incoming, &loops).unwrap();
        let declaration = graph.points().find(|point| matches!(
            graph.node(*point).action(), Action::Declare(value) if value.local() == &snapshot
        )).unwrap();
        let read = graph
            .points()
            .find(|point| matches!(graph.node(*point).action(), Action::Discard(_)))
            .unwrap();
        let mut product: Product<'_> = incoming[read.index()].clone().unwrap();
        product
            .clone()
            .action(&context, graph, read, &loops, true)
            .unwrap();
        // Reapply the actual immutable declaration action through the paired
        // transfer. No graph, prefix state, pointer or authority is fabricated.
        product
            .action(&context, graph, declaration, &loops, true)
            .unwrap();
        let result = product.action(&context, graph, read, &loops, true);
        if points_at_snapshot {
            assert_eq!(result, Err(CSafetyError::UninitializedStorage));
        } else {
            assert_eq!(result, Ok(()));
        }
    }
}

#[test]
fn actual_scope_exit_removes_snapshot_authority_and_expires_prefix_addresses() {
    for points_at_snapshot in [false, true] {
        let mut f = Fixture::new(&[]);
        let stable = immutable_size(&mut f, "stable");
        let buffer = Buffer::new(&mut f, pointer_type(stable.ty().clone()));
        let mut body = vec![f.declare(&stable, f.size(7))];
        let count = f.size(2);
        body.extend(buffer.prefix(&mut f, count));
        let construction = Counted::new(&mut f, "construct");
        body.push(construction.declaration(&f));
        let child = construction.scope.clone();
        let parent = std::mem::replace(&mut f.scope, child.clone());
        let snapshot = immutable_size(&mut f, "snapshot");
        let frontier = immutable_size(&mut f, "frontier");
        let target = if points_at_snapshot {
            &snapshot
        } else {
            &stable
        };
        let pointer = address(&f, f.values().local(target.clone()).unwrap());
        let inner = vec![
            f.declare(&snapshot, f.read(&construction.counter)),
            f.ast()
                .assign(buffer.place(&f, f.read(&construction.counter)), pointer)
                .unwrap(),
            construction.step(&f),
            f.declare(&frontier, f.read(&construction.counter)),
            f.discard(pointed_read(&f, buffer.read(&f, 0))),
            construction.break_now(&f),
        ];
        f.scope = parent;
        body.push(construction.finish(&f, buffer.count.local(), inner));
        body.push(f.discard(pointed_read(&f, buffer.read(&f, 0))));
        body.push(buffer.release(&f));
        let files = [f.source(body)];
        let context = ContextFacts::check(&f.registry, &files).unwrap();
        let graph = &context.functions()[0];
        let loops = loops::check(&context).unwrap();
        let incoming = solve::function(&context, graph, State::default(), &loops).unwrap();
        let exit = graph
            .points()
            .find(|point| {
                matches!(
                    graph.node(*point).origin().map(CStatement::kind),
                    Some(CStatementKind::Break(_))
                )
            })
            .unwrap();
        let mut product = incoming[exit.index()].clone().unwrap();
        let bound = Bound::Snapshot(Box::new(frontier));
        assert!(
            product
                .memory
                .prefixes
                .values()
                .any(|prefixes| prefixes.get(&bound).is_some())
        );
        product.action(&context, graph, exit, &loops, true).unwrap();
        for edge in graph.node(exit).successors() {
            assert!(edge.exited_scopes().contains(&&child));
            let outgoing = product
                .edge(&context, graph, exit, edge, &loops, true)
                .unwrap()
                .unwrap();
            assert!(
                outgoing
                    .memory
                    .prefixes
                    .values()
                    .all(|prefixes| prefixes.get(&bound).is_none())
            );
        }
        let result = f.registry.check_storage_paths(&files);
        if points_at_snapshot {
            assert_eq!(result, Err(CSafetyError::UninitializedStorage));
        } else {
            assert_eq!(result, Ok(()));
        }
    }
}
