//! Unresolved facade metadata cannot replace exact imported constant evidence.
use super::{
    constant_consumer_fixture::producer, constant_export_fixture::configured,
    owned_constant_fixture::Shape, project_c_package,
};
use portable_codegen::*;

#[derive(Clone, Copy, Debug)]
enum Mutation {
    MissingWitness,
    WrongNamespace,
    ForeignModule,
    ForeignFunction,
    Empty,
    UnknownModule,
}

#[test]
fn unsupported_or_unwitnessed_export_graphs_reject_before_shared_linking() {
    let producer = producer(Shape::Mixed);
    let values: Vec<_> = producer.constants().cloned().collect();
    let foreign_function = producer.functions().next().unwrap().declaration();
    for mutation in [
        Mutation::MissingWitness,
        Mutation::WrongNamespace,
        Mutation::ForeignModule,
        Mutation::ForeignFunction,
        Mutation::Empty,
        Mutation::UnknownModule,
    ] {
        let fixture = configured(95, &values, |_, graph| {
            let root = graph.root;
            match mutation {
                Mutation::Empty => graph.modules.get_mut(&root).unwrap().clear(),
                Mutation::UnknownModule => {
                    graph.modules.insert(
                        RustDeclarationId {
                            definition_path_hash: 500,
                            ..root
                        },
                        Default::default(),
                    );
                }
                _ => {
                    let entries = graph.modules.get_mut(&root).unwrap();
                    let name = entries.keys().next().unwrap().clone();
                    let mut target = entries.remove(&name).unwrap();
                    let mut name = name;
                    match mutation {
                        Mutation::MissingWitness => {
                            target = RustExportTarget::Declaration(RustDeclarationId {
                                crate_id: 999,
                                definition_path_hash: 99,
                            })
                        }
                        Mutation::WrongNamespace => name.namespace = RustExportNamespace::Type,
                        Mutation::ForeignModule => {
                            name.namespace = RustExportNamespace::Type;
                            target = RustExportTarget::Module(RustDeclarationId {
                                crate_id: 999,
                                definition_path_hash: 1,
                            });
                        }
                        Mutation::ForeignFunction => {
                            target = RustExportTarget::Declaration(foreign_function)
                        }
                        _ => unreachable!(),
                    }
                    entries.insert(name, target);
                }
            }
        });
        assert!(
            project_c_package(fixture.registry, fixture.files).is_err(),
            "{mutation:?}"
        );
    }
}

#[test]
fn selected_source_identity_cannot_import_itself_without_owned_declarations() {
    let producer = producer(Shape::ConstantsOnly);
    let values: Vec<_> = producer.constants().cloned().collect();
    let fixture = configured(producer.source_root().unwrap().crate_id, &values, |_, _| {});
    assert!(project_c_package(fixture.registry, fixture.files).is_err());
}
