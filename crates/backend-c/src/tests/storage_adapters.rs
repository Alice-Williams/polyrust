//! Real erase/restore operands retain original object type, extent and alignment.
use super::{
    contextual_reconstruction::key, index_extent_fixture as arrays, numeric_fixture::Fixture,
    storage_fixture::*, *,
};
use portable_core_ir::CoreDeclaration;

#[test]
fn adapter_restore_cannot_reinterpret_wrong_nominal_or_under_aligned_storage() {
    for shape in 0..3 {
        for qualifier in [CConstness::Unqualified, CConstness::Const] {
            let mut f = Fixture::new(&[]);
            let record = f.registry.declare_struct(&f.file, key("Record")).unwrap();
            let owner = CAggregateRef::Struct(record.clone());
            let field = f
                .registry
                .register_member(&owner, key("data"), CObjectType::scalar(CScalarType::I64))
                .unwrap();
            f.registry
                .define_aggregate(&owner, vec![field.clone()])
                .unwrap();
            let other = f.registry.declare_struct(&f.file, key("Other")).unwrap();
            let other_owner = CAggregateRef::Struct(other.clone());
            let other_field = f
                .registry
                .register_member(&other_owner, key("data"), field.ty().clone())
                .unwrap();
            f.registry
                .define_aggregate(&other_owner, vec![other_field])
                .unwrap();
            let interface = f
                .registry
                .declare_struct(&f.file, key("Interface"))
                .unwrap();
            let table_type = f.registry.declare_struct(&f.file, key("Table")).unwrap();
            let table_owner = CAggregateRef::Struct(table_type.clone());
            let table_field = f
                .registry
                .register_member(
                    &table_owner,
                    key("reserved"),
                    CObjectType::scalar(CScalarType::Int),
                )
                .unwrap();
            f.registry
                .define_aggregate(&table_owner, vec![table_field])
                .unwrap();
            let table_object = f
                .registry
                .register_object(
                    &f.file,
                    key("table"),
                    CObjectType::structure(table_type)
                        .with_constness(CConstness::Const)
                        .unwrap(),
                )
                .unwrap();
            let core = super::contextual_origins::core();
            let implementation = core
                .module()
                .declarations
                .iter()
                .find_map(|d| match d {
                    CoreDeclaration::Implementation(id) => Some(*id),
                    _ => None,
                })
                .unwrap();
            let methods = core.implementations()[implementation.index()]
                .methods
                .iter()
                .map(|id| CWitnessMethod {
                    interface_method: core.implementation_methods()[id.index()].interface_method,
                    implementation_method: *id,
                    concrete: f.function.clone(),
                    adapter: f.function.clone(),
                })
                .collect();
            let witness = f
                .registry
                .register_interface_witness(
                    key("Witness"),
                    implementation,
                    &interface,
                    &record,
                    methods,
                )
                .unwrap();
            let table = f
                .registry
                .register_interface_table(
                    key("TableBinding"),
                    &witness,
                    &table_object,
                    &f.function,
                    &f.function,
                )
                .unwrap();
            let adapter = f
                .registry
                .register_interface_adapter(key("Adapter"), &table)
                .unwrap();
            let ty = match shape {
                0 => CObjectType::structure(record),
                1 => CObjectType::structure(other),
                _ => CObjectType::scalar(CScalarType::I8),
            };
            let storage = local(&mut f, ty, "storage");
            let address = address(&f, f.values().local(storage.clone()).unwrap());
            let qualified = if qualifier == CConstness::Const {
                f.values()
                    .add_const(
                        pointer_type(storage.ty().clone().with_constness(qualifier).unwrap()),
                        address,
                    )
                    .unwrap()
            } else {
                address
            };
            let erased = if shape == 0 {
                f.values()
                    .adapter_erase(adapter.clone(), qualified)
                    .unwrap()
            } else {
                f.values()
                    .object_to_void(
                        CObjectType::pointer(CPointerTarget::Void(qualifier)),
                        qualified,
                    )
                    .unwrap()
            };
            let restored = f.values().adapter_restore(adapter, erased).unwrap();
            let read = f
                .values()
                .read(
                    f.values()
                        .member(f.values().dereference(restored).unwrap(), field)
                        .unwrap(),
                )
                .unwrap();
            let mut source = f.source(vec![arrays::declare(&f, &storage), f.discard(read)]);
            let declarations = CDeclarations::new(&f.registry, f.file.clone()).unwrap();
            for aggregate in [owner, other_owner, table_owner] {
                source.items.insert(
                    0,
                    CFileItem::Declaration(declarations.aggregate(aggregate).unwrap()),
                );
            }
            source.items.insert(
                0,
                CFileItem::Declaration(
                    declarations
                        .forward_tag(CAggregateRef::Struct(interface))
                        .unwrap(),
                ),
            );
            source.items.push(CFileItem::Definition(
                declarations
                    .object_definition(
                        table_object.clone(),
                        CLinkage::Internal,
                        f.values()
                            .zero_initializer(table_object.ty().clone())
                            .unwrap(),
                    )
                    .unwrap(),
            ));
            f.registry
                .check_numeric_flow(std::slice::from_ref(&source))
                .unwrap();
            assert_eq!(
                f.registry.check_storage_paths(&[source]),
                if shape == 0 {
                    Ok(())
                } else {
                    Err(CSafetyError::StorageTypeMismatch)
                }
            );
        }
    }
}
