use super::*;
use crate::tests::source_dependency_fixture::*;
use portable_codegen::{GeneratedOrigin, RustExportName, RustExportNamespace, RustExportTarget};

#[test]
fn large_shared_export_graph_is_compared_once_per_distinct_allocation() {
    let draft = package_with_exports(7, functions(42), |exports| {
        for index in 0..4096 {
            exports.modules.get_mut(&id(7, 1)).unwrap().insert(
                RustExportName {
                    namespace: RustExportNamespace::Value,
                    name: format!("alias{index}"),
                },
                RustExportTarget::Declaration(id(7, 10)),
            );
        }
    });
    let GeneratedOrigin::RustSource(origin) = &draft.callables().next().unwrap().origin else {
        panic!("source")
    };
    let expected = &origin.crate_exports;
    let equal = Arc::new(expected.as_ref().clone());
    let mut different = expected.as_ref().clone();
    different
        .modules
        .get_mut(&id(7, 1))
        .unwrap()
        .remove(&RustExportName {
            namespace: RustExportNamespace::Value,
            name: "alias0".into(),
        });
    let different = Arc::new(different);
    let mut agreement = Agreement::new(expected);
    for _ in 0..10_000 {
        agreement.check(expected).unwrap();
        agreement.check(&equal).unwrap();
    }
    assert_eq!(
        agreement.comparisons, 1,
        "shared fields/declarations must not rescan the graph"
    );
    assert!(
        agreement
            .check(&different)
            .unwrap_err()
            .contains("source exports")
    );
    assert_eq!(agreement.comparisons, 2);
    assert!(
        !agreement.checked.contains(&Arc::as_ptr(&different)),
        "failed comparison cannot be cached as agreement"
    );
}
