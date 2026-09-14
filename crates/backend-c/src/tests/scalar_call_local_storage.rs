//! A closed helper may read its own nonescaping record through a shared pointer.
use super::fixture::{Fixture, key};
use crate::ast::*;
use crate::ownership::context_facts::ContextFacts;

#[test]
fn called_helper_keeps_initialized_local_record_and_shared_pointer_evidence() {
    let mut f = Fixture::new(&[1, 1]);
    let record = f.registry.declare_struct(&f.file, key("record")).unwrap();
    let owner = CAggregateRef::Struct(record.clone());
    let member = f
        .registry
        .register_member(&owner, key("value"), CObjectType::scalar(CScalarType::I32))
        .unwrap();
    f.registry
        .define_aggregate(&owner, vec![member.clone()])
        .unwrap();
    let ty = CObjectType::structure(record.clone());
    let local = f
        .registry
        .register_local(&f.scopes[1], key("record_value"), ty.clone())
        .unwrap();
    let pointer_ty = CObjectType::pointer(CPointerTarget::Object(Box::new(
        ty.with_constness(CConstness::Const).unwrap(),
    )));
    let pointer = f
        .registry
        .register_local(&f.scopes[1], key("reference"), pointer_ty.clone())
        .unwrap();
    let initial = f
        .values()
        .struct_initializer(
            record,
            vec![(
                member.clone(),
                f.values().expression_initializer(f.input(1, 0)).unwrap(),
            )],
        )
        .unwrap();
    let address = f
        .values()
        .address_of(f.values().local(local.clone()).unwrap())
        .unwrap();
    let address = f.values().add_const(pointer_ty, address).unwrap();
    let borrowed = f.values().expression_initializer(address).unwrap();
    let read_pointer = f
        .values()
        .read(f.values().local(pointer.clone()).unwrap())
        .unwrap();
    let place = f
        .values()
        .member(f.values().dereference(read_pointer).unwrap(), member)
        .unwrap();
    let mut helper = vec![
        f.statements(1).declare(local, Some(initial)).unwrap(),
        f.statements(1).declare(pointer, Some(borrowed)).unwrap(),
    ];
    helper.extend(f.returning(1, f.values().read(place).unwrap()));
    let source = f.source(vec![f.returning(0, f.call(1, vec![f.input(0, 0)])), helper]);
    let declarations = CDeclarations::new(&f.registry, f.file.clone()).unwrap();
    let mut items = vec![CFileItem::Declaration(
        declarations.aggregate(owner).unwrap(),
    )];
    items.extend(source.items().iter().cloned());
    let source = declarations.source_file(items).unwrap();
    let facts = ContextFacts::check(&f.registry, std::slice::from_ref(&source)).unwrap();
    assert!(
        f.functions
            .iter()
            .all(|function| facts.scalar_call(&f.values().direct(function.clone()).unwrap()))
    );
    f.registry.check_storage_paths(&[source]).unwrap();
}
