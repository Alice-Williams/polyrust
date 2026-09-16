//! Header-only dependency claims are reconstructed from source-package metadata.
use super::{
    CDialect, CFileGrammar, constant_consumer_fixture::producer, constant_export_fixture::facade,
    owned_constant_fixture::Shape, project_c_package,
};
use portable_codegen::*;
use std::sync::Arc;

#[derive(Clone, Copy, Debug)]
enum Fault {
    None,
    MissingHeaderReference,
    ExtraSourceReference,
    CoupledMissing,
    SwappedBindings,
}

#[test]
fn header_only_export_dependencies_cannot_be_deleted_retargeted_or_moved() {
    let producer = producer(Shape::ConstantsOnly);
    let values: Vec<_> = producer.constants().cloned().collect();
    let fixture = facade(96, &values);
    let package = project_c_package(fixture.registry, fixture.files).unwrap();
    for fault in [
        Fault::None,
        Fault::MissingHeaderReference,
        Fault::ExtraSourceReference,
        Fault::CoupledMissing,
        Fault::SwappedBindings,
    ] {
        let mut builder = TargetAstBuilder::new(CDialect);
        assert!(
            package.generated_types().len() == 0
                && package.callables().len() == 0
                && package.values().len() == 0
        );
        for file in package.files() {
            let mut units = file.items().to_vec();
            let header = matches!(file.source_kind(), CFileGrammar::Header(_));
            if header {
                let bindings = &mut Arc::make_mut(&mut units[0].data).bindings;
                match fault {
                    Fault::MissingHeaderReference | Fault::CoupledMissing => {
                        bindings.imported_values.clear()
                    }
                    Fault::SwappedBindings => {
                        let entries: Vec<_> = bindings
                            .imported_values
                            .iter()
                            .map(|(a, b)| (a.clone(), b.clone()))
                            .collect();
                        bindings
                            .imported_values
                            .insert(entries[0].0.clone(), entries[1].1.clone());
                        bindings
                            .imported_values
                            .insert(entries[1].0.clone(), entries[0].1.clone());
                    }
                    _ => {}
                }
            } else if matches!(fault, Fault::ExtraSourceReference) {
                let imports = units[0].projection.bindings.imported_values.clone();
                Arc::make_mut(&mut units[0].data).bindings.imported_values = imports;
            }
            if matches!(fault, Fault::CoupledMissing) {
                Arc::make_mut(&mut units[0].projection)
                    .bindings
                    .imported_values
                    .clear();
            }
            builder.file(TargetFile::new(
                file.path().clone(),
                file.role(),
                file.module().clone(),
                *file.placement(),
                units,
                file.source_kind().clone(),
                file.source().clone(),
            ));
        }
        for group in package.groups() {
            builder.group(group.clone());
        }
        assert_eq!(
            verify_unresolved_package(&CDialect, builder.build()).is_ok(),
            matches!(fault, Fault::None),
            "{fault:?}"
        );
    }
}
