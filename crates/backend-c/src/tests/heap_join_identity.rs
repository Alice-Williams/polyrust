//! Branch aliases must preserve both the target and its initialized contents.
use super::{
    allocation_fixture::*, heap_fixture::*, numeric_fixture::Fixture, storage_fixture::*, *,
};

#[test]
fn compatible_spellings_and_immediate_qualifiers_share_heap_identity_at_joins() {
    for (left, right) in [
        (CScalarType::Int, CScalarType::I32),
        (CScalarType::Size, CScalarType::U64),
    ] {
        for array in [false, true] {
            for qualified in [false, true] {
                for complete in [true, false] {
                    let mut f = Fixture::new(&[CScalarType::Bool]);
                    let shape = |scalar, qualifier| {
                        let scalar = CObjectType::scalar(scalar)
                            .with_constness(qualifier)
                            .unwrap();
                        if array {
                            CObjectType::array(scalar, CArrayLength::new(2).unwrap()).unwrap()
                        } else {
                            scalar
                        }
                    };
                    let a = descriptor(&mut f, "a", shape(left, CConstness::Unqualified));
                    let b = descriptor(
                        &mut f,
                        "b",
                        shape(
                            right,
                            if qualified {
                                CConstness::Const
                            } else {
                                CConstness::Unqualified
                            },
                        ),
                    );
                    let raw = raw(&mut f, "raw");
                    let typed = local(&mut f, pointer_type(a.object_type().clone()), "typed");
                    let live = require_live(&mut f, &raw, vec![]);
                    let selected = if array {
                        f.values()
                            .index(CIndexBase::Array(Box::new(pointee(&f, &typed))), f.size(0))
                            .unwrap()
                    } else {
                        pointee(&f, &typed)
                    };
                    let value = if left == CScalarType::Int {
                        f.int(7)
                    } else {
                        f.size(7)
                    };
                    let initialize = f.ast().assign(selected.clone(), value).unwrap();
                    let branch_body = |first: &CAllocationRef, initialize_it: bool| {
                        let mut body = vec![
                            f.discard(restore(&f, first, f.read(&raw))),
                            assign(&f, &typed, restore(&f, &a, f.read(&raw))),
                        ];
                        if initialize_it {
                            body.push(initialize.clone());
                        }
                        body
                    };
                    let yes = branch_body(&a, true);
                    let no = branch_body(&b, complete);
                    let branch = f.branch(f.input(0), yes, no);
                    let source = f.source(vec![
                        f.declare(&raw, allocate(&f, f.size(16))),
                        live,
                        f.declare(&typed, null(&f, &typed)),
                        branch,
                        f.discard(f.values().read(selected).unwrap()),
                        f.discard(restore(&f, &a, f.read(&raw))),
                        release(&f, f.read(&raw)),
                    ]);
                    assert_eq!(
                        f.registry.check_storage_paths(&[source]),
                        if complete {
                            Ok(())
                        } else {
                            Err(CSafetyError::UninitializedStorage)
                        },
                        "left={left:?}, array={array}, qualified={qualified}, complete={complete}"
                    );
                }
            }
        }
    }
}
