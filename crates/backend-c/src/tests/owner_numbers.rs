//! A move preserves the original numeric write, including wrap provenance.
use super::{
    heap_fixture::*,
    numeric_fixture::Fixture,
    numeric_memory_fixture::{consume, wrapped},
    owner_leaf_fixture::Leaf,
    storage_fixture::*,
    *,
};

#[test]
fn moved_payload_keeps_clean_and_wrapped_size_history() {
    for lossy in [false, true] {
        let mut leaf = Leaf::with_type(Fixture::new(&[]), CObjectType::scalar(CScalarType::Size));
        let value = if lossy {
            wrapped(&leaf.f)
        } else {
            leaf.f.size(2)
        };
        let construction = vec![
            leaf.f
                .ast()
                .assign(pointee(&leaf.f, &leaf.temporary), value)
                .unwrap(),
        ];
        let mut actions = leaf.moved(&leaf.first, &leaf.second);
        let bytes = pointed_read(&leaf.f, leaf.f.read(&leaf.second));
        actions.extend(consume(&mut leaf.f, bytes));
        actions.extend(leaf.drop(&leaf.second));
        let body = leaf.body_with(8, construction, actions);
        check(
            &leaf.f,
            body,
            if lossy {
                Err(CSafetyError::UnprovedSizeArithmetic)
            } else {
                Ok(())
            },
        );
    }
}
