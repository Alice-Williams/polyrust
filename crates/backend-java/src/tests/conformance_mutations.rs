//! Whole conformance edges, including zero-method edges, are checked without call sites.

use super::Lowering;
use super::lowering_fixture::verify_members_with;
use crate::ast::{JavaHeritage, JavaMember, JavaMethodDeclaration, JavaType, JavaTypeName};
use crate::capabilities::java_capabilities;
use crate::preflight::JavaCapabilitySelection;
use portable_build::{ModuleBuilder, Type, Value, Visibility};
use portable_core_ir::CoreDeclaration;

#[derive(Clone, Copy, Debug)]
enum Mutation {
    Valid,
    RemoveUnusedImplementation,
    AddZeroMethodImplementation,
    AddEntryImplementation,
}

#[test]
fn complete_checked_conformance_inventory_is_required() {
    for mutation in [
        Mutation::Valid,
        Mutation::RemoveUnusedImplementation,
        Mutation::AddZeroMethodImplementation,
        Mutation::AddEntryImplementation,
    ] {
        let mut module = ModuleBuilder::new("conformance_inventory");
        let (first, ()) = module.record("First", Visibility::Public, vec![], |_| ());
        let (second, ()) = module.record("Second", Visibility::Public, vec![], |_| ());
        let (readable, read) =
            module.interface("Readable", Visibility::Public, vec![], |interface| {
                interface.method("read", vec![], vec![], Some(Type::i32()))
            });
        let (empty, ()) = module.interface("Empty", Visibility::Public, vec![], |_| ());
        for (name, record) in [("FirstReadable", first), ("SecondReadable", second)] {
            module.implementation(
                name,
                Visibility::Package,
                vec![],
                readable,
                record,
                |implementation| {
                    implementation.method("read", read, vec![], |method| {
                        method.returns(Type::i32());
                        method.body(|body| {
                            let value = body.literal(Value::i32(7));
                            body.block([], Some(value))
                        });
                    })
                },
            );
        }
        module.implementation(
            "SecondEmpty",
            Visibility::Package,
            vec![],
            empty,
            second,
            |_| (),
        );
        let checked = module.finish().unwrap();
        let core = portable_core_ir::lower_checked(&checked).unwrap();
        let selection = JavaCapabilitySelection::for_test(&core);
        let mut lowering = Lowering::new(&core, &selection, java_capabilities());
        lowering.register_types();
        lowering.register_values_and_callables().unwrap();
        let mut declarations = Vec::new();
        for declaration in &core.module().declarations {
            match *declaration {
                CoreDeclaration::Record(id) => {
                    declarations.push(lowering.record_declaration(id).unwrap())
                }
                CoreDeclaration::Interface(id) => {
                    declarations.extend(lowering.interface_declarations(id).unwrap())
                }
                CoreDeclaration::Implementation(_) => {}
                _ => unreachable!(),
            }
        }
        let record_id = declarations[0].declared.unwrap();
        let interface_index = match mutation {
            Mutation::Valid | Mutation::RemoveUnusedImplementation => 2,
            Mutation::AddZeroMethodImplementation | Mutation::AddEntryImplementation => 3,
        };
        let interface_id = declarations[interface_index].declared.unwrap();
        let interface_type = JavaType::Reference(JavaTypeName::Generated(interface_id));
        let record_type = JavaType::Reference(JavaTypeName::Generated(record_id));
        let entry_type = JavaType::Reference(JavaTypeName::Generated(lowering.entry.unwrap()));
        let JavaHeritage::Interfaces(heritage) = &mut declarations[0].heritage else {
            unreachable!()
        };
        match mutation {
            Mutation::Valid => {}
            Mutation::RemoveUnusedImplementation => {
                heritage.retain(|value| value != &interface_type);
                declarations[0].members.retain(|member| {
                    !matches!(member, JavaMember::Method(method)
                    if matches!(method.declared, JavaMethodDeclaration::Implementation { .. }))
                });
                declarations[interface_index]
                    .permits
                    .retain(|value| value != &record_type);
            }
            Mutation::AddZeroMethodImplementation => {
                heritage.push(interface_type.clone());
                declarations[interface_index].permits.push(record_type);
            }
            Mutation::AddEntryImplementation => {
                declarations[interface_index].permits.push(entry_type)
            }
        }
        let result = verify_members_with(
            lowering,
            declarations
                .into_iter()
                .map(JavaMember::NestedType)
                .collect(),
            |entry| {
                if matches!(mutation, Mutation::AddEntryImplementation) {
                    entry.heritage = JavaHeritage::Interfaces(vec![interface_type]);
                }
            },
        );
        assert_eq!(
            result.is_ok(),
            matches!(mutation, Mutation::Valid),
            "{mutation:?}: {result:?}"
        );
    }
}
