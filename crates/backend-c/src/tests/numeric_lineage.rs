//! Regression controls for numeric history across storage and library transforms.
use super::{contextual_reconstruction::key, numeric_fixture::Fixture, *};
use crate::dialect::CKnownCall as K;
use CBinaryOperator as B;

fn wrapped(f: &Fixture) -> CValue {
    // The C result is exactly 2, safely representable as binary64. This avoids
    // accidentally testing the unrelated rounding of values near 2^64.
    f.binary(B::Multiply, f.size((1_u64 << 63) + 1), f.size(2))
}

#[test]
fn a_global_is_clean_after_a_join_only_if_every_predecessor_established_it() {
    for writes in 0..4 {
        let mut f = Fixture::new(&[CScalarType::Bool]);
        let global = f
            .registry
            .register_object(
                &f.file,
                key("global"),
                CObjectType::scalar(CScalarType::Size),
            )
            .unwrap();
        let place = f.values().global(global.clone()).unwrap();
        let yes = if writes & 1 != 0 {
            vec![f.ast().assign(place.clone(), f.size(2)).unwrap()]
        } else {
            vec![]
        };
        let no = if writes & 2 != 0 {
            vec![f.ast().assign(place.clone(), f.size(3)).unwrap()]
        } else {
            vec![]
        };
        let branch = f.branch(f.input(0), yes, no);
        let mut source = f.source(vec![branch, allocate(&f, f.values().read(place).unwrap())]);
        source.items.insert(
            0,
            CFileItem::Definition(
                CDeclarations::new(&f.registry, f.file.clone())
                    .unwrap()
                    .object_definition(
                        global,
                        CLinkage::External,
                        f.values().expression_initializer(wrapped(&f)).unwrap(),
                    )
                    .unwrap(),
            ),
        );
        let result = f.registry.check_numeric_flow(&[source]);
        if writes == 3 {
            result.unwrap();
        } else {
            assert_eq!(
                result,
                Err(CSafetyError::UnprovedSizeArithmetic),
                "write mask {writes}"
            );
        }
    }
}

#[test]
fn guards_cannot_turn_unproved_global_storage_into_clean_size_history() {
    for overwrite in [false, true] {
        let mut f = Fixture::new(&[]);
        let global = f
            .registry
            .register_object(
                &f.file,
                key("global"),
                CObjectType::scalar(CScalarType::Size),
            )
            .unwrap();
        let place = f.values().global(global.clone()).unwrap();
        let read = f.values().read(place.clone()).unwrap();
        let mut body = vec![];
        if overwrite {
            body.push(f.ast().assign(place, f.size(2)).unwrap());
        }
        body.push(f.branch(
            f.compare(B::Greater, read.clone(), f.size(0)),
            vec![allocate(&f, read)],
            vec![],
        ));
        let mut source = f.source(body);
        source.items.insert(
            0,
            CFileItem::Definition(
                CDeclarations::new(&f.registry, f.file.clone())
                    .unwrap()
                    .object_definition(
                        global,
                        CLinkage::External,
                        f.values().expression_initializer(wrapped(&f)).unwrap(),
                    )
                    .unwrap(),
            ),
        );
        let result = f.registry.check_numeric_flow(&[source]);
        if overwrite {
            result.unwrap();
        } else {
            assert_eq!(result, Err(CSafetyError::UnprovedSizeArithmetic));
        }
    }
}

#[test]
fn opaque_effects_cover_numeric_cells_that_were_not_previously_materialized() {
    let mut f = Fixture::new(&[]);
    let ty = CObjectType::array(
        CObjectType::scalar(CScalarType::Size),
        CArrayLength::new(1).unwrap(),
    )
    .unwrap();
    let array = f
        .registry
        .register_local(&f.scope, key("array"), ty.clone())
        .unwrap();
    let place = f.values().local(array.clone()).unwrap();
    let address = f.discard(f.values().address_of(place.clone()).unwrap());
    let read = f
        .values()
        .read(
            f.values()
                .index(CIndexBase::Array(Box::new(place)), f.size(0))
                .unwrap(),
        )
        .unwrap();
    let guarded = f.branch(
        f.compare(B::LessEqual, read.clone(), f.size(u64::MAX)),
        vec![allocate(&f, read)],
        vec![],
    );
    assert_eq!(
        f.check(vec![
            f.ast()
                .declare(array, Some(f.values().zero_initializer(ty).unwrap()))
                .unwrap(),
            address,
            allocate(&f, f.size(1)),
            guarded
        ]),
        Err(CSafetyError::UnprovedSizeArithmetic)
    );
}
fn allocate(f: &Fixture, value: CValue) -> CStatement {
    f.discard(
        f.values()
            .call_value(f.values().known(K::Allocate), vec![value])
            .unwrap(),
    )
}

#[test]
fn whole_record_assignment_and_initialization_preserve_field_history() {
    for initialization in [false, true] {
        for lossy in [false, true] {
            let mut f = Fixture::new(&[]);
            let record = f.registry.declare_struct(&f.file, key("Record")).unwrap();
            let owner = CAggregateRef::Struct(record.clone());
            let field = f
                .registry
                .register_member(&owner, key("bytes"), CObjectType::scalar(CScalarType::Size))
                .unwrap();
            f.registry
                .define_aggregate(&owner, vec![field.clone()])
                .unwrap();
            let ty = CObjectType::structure(record.clone());
            let source = f
                .registry
                .register_local(&f.scope, key("source"), ty.clone())
                .unwrap();
            let target = f
                .registry
                .register_local(&f.scope, key("target"), ty)
                .unwrap();
            let input = if lossy { wrapped(&f) } else { f.size(2) };
            let initializer = f
                .values()
                .struct_initializer(
                    record,
                    vec![(
                        field.clone(),
                        f.values().expression_initializer(input).unwrap(),
                    )],
                )
                .unwrap();
            let source_value = f
                .values()
                .read(f.values().local(source.clone()).unwrap())
                .unwrap();
            let mut body = vec![f.ast().declare(source, Some(initializer)).unwrap()];
            if initialization {
                body.push(f.declare(&target, source_value));
            } else {
                body.push(f.ast().declare(target.clone(), None).unwrap());
                body.push(
                    f.ast()
                        .assign(f.values().local(target.clone()).unwrap(), source_value)
                        .unwrap(),
                );
            }
            let read = f
                .values()
                .read(
                    f.values()
                        .member(f.values().local(target).unwrap(), field)
                        .unwrap(),
                )
                .unwrap();
            body.push(allocate(&f, read));
            let mut source = f.source(body);
            source.items.insert(
                0,
                CFileItem::Declaration(
                    CDeclarations::new(&f.registry, f.file.clone())
                        .unwrap()
                        .aggregate(owner)
                        .unwrap(),
                ),
            );
            let result = f.registry.check_numeric_flow(&[source]);
            if lossy {
                assert_eq!(result, Err(CSafetyError::UnprovedSizeArithmetic));
            } else {
                result.unwrap();
            }
        }
    }
}

#[test]
fn indirect_wrapped_assignment_cannot_be_read_back_as_clean_size() {
    let mut f = Fixture::new(&[]);
    let bytes = f.local(CScalarType::Size, "bytes");
    let place = f.values().local(bytes.clone()).unwrap();
    let pointer = f.values().address_of(place).unwrap();
    let write = f
        .ast()
        .assign(f.values().dereference(pointer).unwrap(), wrapped(&f))
        .unwrap();
    assert_eq!(
        f.check(vec![
            f.declare(&bytes, f.size(0)),
            write,
            allocate(&f, f.read(&bytes))
        ]),
        Err(CSafetyError::UnprovedSizeArithmetic)
    );
}

#[test]
fn variable_index_write_cannot_lose_the_new_numeric_history() {
    let mut f = Fixture::new(&[CScalarType::Size]);
    let ty = CObjectType::array(
        CObjectType::scalar(CScalarType::Size),
        CArrayLength::new(1).unwrap(),
    )
    .unwrap();
    let array = f
        .registry
        .register_local(&f.scope, key("array"), ty.clone())
        .unwrap();
    let base = || CIndexBase::Array(Box::new(f.values().local(array.clone()).unwrap()));
    let write = f
        .ast()
        .assign(f.values().index(base(), f.input(0)).unwrap(), wrapped(&f))
        .unwrap();
    let read = f
        .values()
        .read(f.values().index(base(), f.size(0)).unwrap())
        .unwrap();
    let init = f
        .values()
        .array_initializer(
            ty,
            vec![f.values().expression_initializer(f.size(0)).unwrap()],
        )
        .unwrap();
    let write = f.branch(
        f.compare(B::Equal, f.input(0), f.size(0)),
        vec![write, allocate(&f, read)],
        vec![],
    );
    assert_eq!(
        f.check(vec![f.ast().declare(array, Some(init)).unwrap(), write]),
        Err(CSafetyError::UnprovedSizeArithmetic)
    );
}

#[test]
fn opaque_write_to_a_previously_clean_exposed_scalar_is_not_clean_provenance() {
    let mut f = Fixture::new(&[]);
    let bytes = f.local(CScalarType::Size, "bytes");
    let address = f.discard(
        f.values()
            .address_of(f.values().local(bytes.clone()).unwrap())
            .unwrap(),
    );
    assert_eq!(
        f.check(vec![
            f.declare(&bytes, f.size(2)),
            address,
            allocate(&f, f.size(1)),
            allocate(&f, f.read(&bytes))
        ]),
        Err(CSafetyError::UnprovedSizeArithmetic)
    );
}

#[test]
fn truncation_and_remainder_are_numeric_transforms_not_history_reset_points() {
    for known in [K::FloatTruncate, K::FloatRemainder] {
        for lossy in [false, true] {
            let mut f = Fixture::new(&[]);
            let result = f.local(CScalarType::F64, "result");
            let input = if lossy { wrapped(&f) } else { f.size(2) };
            let mut arguments = vec![
                f.values()
                    .numeric_conversion(CScalarType::F64, input)
                    .unwrap(),
            ];
            if known == K::FloatRemainder {
                arguments.push(
                    f.values()
                        .numeric_conversion(CScalarType::F64, f.size(3))
                        .unwrap(),
                );
            }
            let call = f
                .values()
                .call_value(f.values().known(known), arguments)
                .unwrap();
            let double = |value| {
                f.values()
                    .numeric_conversion(CScalarType::F64, f.size(value))
                    .unwrap()
            };
            let condition = f.boolean(f.binary(
                B::LogicalAnd,
                f.compare(B::GreaterEqual, f.read(&result), double(0)),
                f.compare(B::Less, f.read(&result), double(1 << 63)),
            ));
            let allocation = allocate(
                &f,
                f.values()
                    .numeric_conversion(CScalarType::Size, f.read(&result))
                    .unwrap(),
            );
            let branch = f.branch(condition, vec![allocation], vec![]);
            let checked = f.check(vec![f.declare(&result, call), branch]);
            if lossy {
                assert_eq!(checked, Err(CSafetyError::UnprovedSizeArithmetic));
            } else {
                checked.unwrap();
            }
        }
    }
}
