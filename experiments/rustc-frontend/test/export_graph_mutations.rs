//! Shared verification cannot accept changed compiler API metadata.
use portable_backend_c::dialect::CDialect;
use portable_codegen::*;
use std::sync::Arc;

#[derive(Clone, Copy, Debug)]
enum Mutation {
    Unchanged,
    MissingModules,
    ForeignRoot,
    WrongTarget,
    WrongNamespace,
}

pub(super) fn check(package: &TargetAstPackage<CDialect>) {
    for mutation in [
        Mutation::Unchanged,
        Mutation::MissingModules,
        Mutation::ForeignRoot,
        Mutation::WrongTarget,
        Mutation::WrongNamespace,
    ] {
        let mut builder = TargetAstBuilder::new(CDialect);
        for ty in package.generated_types() {
            builder.generated_type(ty.clone());
        }
        for function in package.callables() {
            let mut function = function.clone();
            let GeneratedOrigin::RustSource(origin) = &mut function.origin else {
                panic!("source function");
            };
            let graph = Arc::make_mut(&mut Arc::make_mut(origin).crate_exports);
            match mutation {
                Mutation::Unchanged => {}
                Mutation::MissingModules => graph.modules.clear(),
                Mutation::ForeignRoot => graph.root.crate_id ^= 1,
                Mutation::WrongTarget => {
                    *graph
                        .modules
                        .get_mut(&graph.root)
                        .unwrap()
                        .values_mut()
                        .next()
                        .unwrap() = RustExportTarget::Declaration(graph.root);
                }
                Mutation::WrongNamespace => {
                    let root = graph.modules.get_mut(&graph.root).unwrap();
                    let (mut name, target) = root.pop_first().unwrap();
                    name.namespace = RustExportNamespace::Macro;
                    root.insert(name, target);
                }
            }
            builder.callable(function);
        }
        for value in package.values() {
            builder.value(value.clone());
        }
        for file in package.files() {
            builder.file(file.clone());
        }
        for group in package.groups() {
            builder.group(group.clone());
        }
        let rebuilt = builder.build();
        if matches!(mutation, Mutation::Unchanged) {
            assert_eq!(&rebuilt, package);
            assert!(verify_unresolved_package(&CDialect, rebuilt).is_ok());
        } else {
            assert!(
                verify_unresolved_package(&CDialect, rebuilt).is_err(),
                "{mutation:?}"
            );
        }
    }
}
