//! A call result must be materialized before any non-local assignment.
use super::contextual_reconstruction::{fixture, key, package};
use super::sequencing_roots::{check, int, predicate_call};
use super::*;

#[derive(Clone, Copy)]
enum DestinationKind {
    Global,
    Member,
    Dereference,
    Index,
}
#[derive(Clone, Copy)]
enum AccessKind {
    Read,
    Address,
    Write,
}

#[test]
fn call_free_nonlocal_destinations_still_cannot_receive_a_call_root_rhs() {
    for kind in [
        DestinationKind::Global,
        DestinationKind::Member,
        DestinationKind::Dereference,
        DestinationKind::Index,
    ] {
        for hidden in [false, true] {
            let (mut registry, file, function, scope) = fixture();
            let scalar = CObjectType::scalar(CScalarType::Int);
            let record = matches!(kind, DestinationKind::Member)
                .then(|| registry.declare_struct(&file, key("Record")).unwrap());
            let member = record.as_ref().map(|record| {
                registry
                    .register_member(
                        &CAggregateRef::Struct(record.clone()),
                        key("value"),
                        scalar.clone(),
                    )
                    .unwrap()
            });
            if let Some(record) = &record {
                registry
                    .define_aggregate(
                        &CAggregateRef::Struct(record.clone()),
                        vec![member.clone().unwrap()],
                    )
                    .unwrap();
            }
            let ty = match kind {
                DestinationKind::Member => CObjectType::structure(record.clone().unwrap()),
                DestinationKind::Index => {
                    CObjectType::array(scalar.clone(), CArrayLength::new(2).unwrap()).unwrap()
                }
                _ => scalar,
            };
            let global = matches!(kind, DestinationKind::Global).then(|| {
                registry
                    .register_object(&file, key("global_value"), ty.clone())
                    .unwrap()
            });
            let local = (!matches!(kind, DestinationKind::Global)).then(|| {
                registry
                    .register_local(&scope, key("storage"), ty.clone())
                    .unwrap()
            });
            let values = CExpressions::new(&registry);
            let ast = CStatements::new(&registry, function.clone()).unwrap();
            let declarations = CDeclarations::new(&registry, file.clone()).unwrap();
            let mut body = Vec::new();
            if let Some(local) = &local {
                body.push(
                    ast.declare(
                        local.clone(),
                        Some(values.zero_initializer(ty.clone()).unwrap()),
                    )
                    .unwrap(),
                );
            }
            let place = match kind {
                DestinationKind::Global => values.global(global.clone().unwrap()).unwrap(),
                DestinationKind::Member => values
                    .member(values.local(local.unwrap()).unwrap(), member.unwrap())
                    .unwrap(),
                DestinationKind::Dereference => values
                    .dereference(
                        values
                            .address_of(values.local(local.unwrap()).unwrap())
                            .unwrap(),
                    )
                    .unwrap(),
                DestinationKind::Index => values
                    .index(
                        CIndexBase::Array(Box::new(values.local(local.unwrap()).unwrap())),
                        int(&values),
                    )
                    .unwrap(),
            };
            body.push(
                ast.assign(
                    place,
                    if hidden {
                        predicate_call(&values)
                    } else {
                        int(&values)
                    },
                )
                .unwrap(),
            );
            let mut source = package(&registry, file, function, scope, body);
            if let Some(record) = record {
                source.items.insert(
                    0,
                    CFileItem::Declaration(
                        declarations
                            .aggregate(CAggregateRef::Struct(record))
                            .unwrap(),
                    ),
                );
            }
            if let Some(global) = global {
                source.items.insert(
                    0,
                    CFileItem::Definition(
                        declarations
                            .object_definition(
                                global,
                                CLinkage::External,
                                values.zero_initializer(ty).unwrap(),
                            )
                            .unwrap(),
                    ),
                );
            }
            check(&registry, source, hidden);
        }
    }
}

#[test]
fn hidden_index_calls_are_rejected_for_reads_addresses_and_writes_including_member_bases() {
    for member_base in [false, true] {
        for access in [AccessKind::Read, AccessKind::Address, AccessKind::Write] {
            for hidden in [false, true] {
                let (mut registry, file, function, scope) = fixture();
                let scalar = CObjectType::scalar(CScalarType::Int);
                let record =
                    member_base.then(|| registry.declare_struct(&file, key("Record")).unwrap());
                let member = record.as_ref().map(|record| {
                    registry
                        .register_member(
                            &CAggregateRef::Struct(record.clone()),
                            key("value"),
                            scalar.clone(),
                        )
                        .unwrap()
                });
                if let Some(record) = &record {
                    registry
                        .define_aggregate(
                            &CAggregateRef::Struct(record.clone()),
                            vec![member.clone().unwrap()],
                        )
                        .unwrap();
                }
                let element = record
                    .as_ref()
                    .map_or(scalar, |record| CObjectType::structure(record.clone()));
                let ty = CObjectType::array(element, CArrayLength::new(2).unwrap()).unwrap();
                let local = registry
                    .register_local(&scope, key("storage"), ty.clone())
                    .unwrap();
                let values = CExpressions::new(&registry);
                let ast = CStatements::new(&registry, function.clone()).unwrap();
                let index = if hidden {
                    predicate_call(&values)
                } else {
                    int(&values)
                };
                let mut place = values
                    .index(
                        CIndexBase::Array(Box::new(values.local(local.clone()).unwrap())),
                        index,
                    )
                    .unwrap();
                if let Some(member) = member {
                    place = values.member(place, member).unwrap();
                }
                let operation = match access {
                    AccessKind::Read => ast.discard(values.read(place).unwrap()).unwrap(),
                    AccessKind::Address => ast.discard(values.address_of(place).unwrap()).unwrap(),
                    AccessKind::Write => ast.assign(place, int(&values)).unwrap(),
                };
                let mut source = package(
                    &registry,
                    file.clone(),
                    function,
                    scope,
                    vec![
                        ast.declare(local, Some(values.zero_initializer(ty).unwrap()))
                            .unwrap(),
                        operation,
                    ],
                );
                if let Some(record) = record {
                    source.items.insert(
                        0,
                        CFileItem::Declaration(
                            CDeclarations::new(&registry, file)
                                .unwrap()
                                .aggregate(CAggregateRef::Struct(record))
                                .unwrap(),
                        ),
                    );
                }
                check(&registry, source, hidden);
            }
        }
    }
}
