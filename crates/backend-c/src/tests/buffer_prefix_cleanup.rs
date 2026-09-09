//! Cleanup jumps use only the initialized frontier that actually reached them.
use super::{
    buffer_prefix_fixture::{Counted, immutable_size},
    buffer_symbolic_fixture::*,
    contextual_reconstruction::key,
};

#[test]
fn cleanup_jumps_preserve_zero_or_advanced_frontiers_without_claiming_the_full_count() {
    for advance in [false, true] {
        for wrong_count in [false, true] {
            let (mut f, buffer, mut body) = setup();
            let construction = Counted::new(&mut f, "construct");
            let cleanup = Counted::new(&mut f, "cleanup");
            let limit = immutable_size(&mut f, "initialized_count");
            let label = f
                .registry
                .register_cleanup_exit(&f.scope, key("cleanup_start"))
                .unwrap();
            body.push(construction.declaration(&f));
            let mut actions = vec![write(&f, &buffer, f.read(&construction.counter), f.size(7))];
            if advance {
                actions.push(construction.step(&f));
            }
            actions.push(f.ast().cleanup_jump(label.clone()).unwrap());
            body.push(construction.finish(&f, buffer.count.local(), actions));
            body.push(
                f.ast()
                    .label(label, f.discard(f.read(&construction.counter)))
                    .unwrap(),
            );
            body.push(f.declare(
                &limit,
                if wrong_count {
                    f.read(buffer.count.local())
                } else {
                    f.read(&construction.counter)
                },
            ));
            body.push(cleanup.declaration(&f));
            body.push(cleanup.finish(
                &f,
                &limit,
                vec![
                    f.discard(selected(&f, &buffer, f.read(&cleanup.counter))),
                    cleanup.step(&f),
                ],
            ));
            body.push(buffer.release(&f));
            let result = f.registry.check_storage_paths(&[f.source(body)]);
            if wrong_count {
                assert!(result.is_err(), "{result:?}");
            } else {
                assert_eq!(result, Ok(()));
            }
        }
    }
}
