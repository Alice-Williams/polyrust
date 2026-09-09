//! The storage proof must use the same authenticated C ABI types as the AST.
use super::{index_extent_fixture as arrays, numeric_fixture::Fixture, storage_fixture::*, *};

#[test]
fn compatible_scalar_spellings_work_through_storage_pointer_shapes() {
    for source in CScalarType::ALL {
        for destination in CScalarType::ALL {
            if !source.is_compatible_with(destination) {
                continue;
            }
            for shape in 0..3 {
                for qualified in [false, true] {
                    let mut f = Fixture::new(&[]);
                    let shape_type = |scalar| match shape {
                        0 => CObjectType::scalar(scalar),
                        1 => CObjectType::array(
                            CObjectType::scalar(scalar),
                            CArrayLength::new(2).unwrap(),
                        )
                        .unwrap(),
                        _ => pointer_type(CObjectType::scalar(scalar)),
                    };
                    let storage = local(&mut f, shape_type(source), "storage");
                    let mut target = shape_type(destination);
                    if qualified {
                        target = if shape == 1 {
                            CObjectType::array(
                                CObjectType::scalar(destination)
                                    .with_constness(CConstness::Const)
                                    .unwrap(),
                                CArrayLength::new(2).unwrap(),
                            )
                            .unwrap()
                        } else {
                            target.with_constness(CConstness::Const).unwrap()
                        };
                    }
                    let pointer = local(&mut f, pointer_type(target.clone()), "pointer");
                    let address = address(&f, f.values().local(storage.clone()).unwrap());
                    let address = if qualified {
                        f.values().add_const(pointer.ty().clone(), address).unwrap()
                    } else {
                        address
                    };
                    let place = f.values().dereference(f.read(&pointer)).unwrap();
                    let place = if shape == 1 {
                        f.values()
                            .index(CIndexBase::Array(Box::new(place)), f.size(1))
                            .unwrap()
                    } else {
                        place
                    };
                    let files = [f.source(vec![
                        arrays::declare(&f, &storage),
                        f.declare(&pointer, address),
                        f.discard(f.values().read(place).unwrap()),
                    ])];
                    f.registry.check_numeric_flow(&files).unwrap();
                    assert_eq!(
                        f.registry.check_storage_paths(&files),
                        Ok(()),
                        "{source:?} -> {destination:?}, shape={shape}, const={qualified}"
                    );
                }
            }
        }
    }
}
