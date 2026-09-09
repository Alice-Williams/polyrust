//! A count's lexical lifetime is distinct from the allocation's original extent.
use super::{
    allocation_fixture::*, buffer_fixture::Buffer, contextual_reconstruction::key, heap_fixture::*,
    numeric_fixture::Fixture, storage_fixture::*, *,
};

#[test]
fn count_scope_exit_preserves_constant_storage_but_retires_current_index_observations() {
    for symbolic in [false, true] {
        let mut f = Fixture::new(&[CScalarType::Size]);
        let child = f
            .registry
            .register_scope(&f.function, Some(&f.scope), key("child"))
            .unwrap();
        let element = CObjectType::scalar(CScalarType::Size);
        let buffer = Buffer::scoped(&mut f, &child, element.clone());
        let saved = local(&mut f, pointer_type(element.clone()), "saved");
        let index = f
            .registry
            .register_local(&child, key("index"), element.clone())
            .unwrap();
        let mut body = vec![
            f.declare(&buffer.raw, null(&f, &buffer.raw)),
            f.declare(&buffer.data, null(&f, &buffer.data)),
            f.declare(&saved, null(&f, &saved)),
        ];
        let parent = f.scope.clone();
        f.scope = child.clone();
        let live = require_live(&mut f, &buffer.raw, vec![]);
        let condition = f.compare(
            CBinaryOperator::Less,
            f.read(&index),
            f.read(buffer.count.local()),
        );
        let guard = f.branch(
            condition,
            vec![],
            vec![buffer.release(&f), f.ast().return_statement(None).unwrap()],
        );
        let selected = if symbolic { f.read(&index) } else { f.size(0) };
        let mut inner = vec![
            f.declare(buffer.count.local(), f.size(2)),
            assign(
                &f,
                &buffer.raw,
                allocate(
                    &f,
                    f.binary(
                        CBinaryOperator::Multiply,
                        f.read(buffer.count.local()),
                        f.values().size_of(element).unwrap(),
                    ),
                ),
            ),
            live,
            assign(
                &f,
                &buffer.data,
                restore(&f, &buffer.descriptor, f.read(&buffer.raw)),
            ),
            f.declare(&index, f.input(0)),
            guard,
        ];
        inner.push(
            f.ast()
                .assign(buffer.place(&f, selected.clone()), f.size(7))
                .unwrap(),
        );
        inner.push(assign(&f, &saved, address(&f, buffer.place(&f, selected))));
        f.scope = parent;
        body.push(
            f.ast()
                .nested_block(f.ast().block(child, inner).unwrap())
                .unwrap(),
        );
        body.push(f.discard(pointed_read(&f, f.read(&saved))));
        body.push(buffer.release(&f));
        check(
            &f,
            body,
            if symbolic {
                Err(CSafetyError::UnprovedStorage)
            } else {
                Ok(())
            },
        );
    }
}
