//! Repeated standard names and original dependency names retain certification.
use super::*;

#[test]
fn rounding_source_reservation_charges_every_resolved_standard_name() {
    for count in [1, 32] {
        let mut bounds = Vec::new();
        for callable in [JavaKnownCallable::MathFloor, JavaKnownCallable::MathCeil] {
            let mut functions = functions(&[]);
            let mut statements: Vec<_> = (0..count)
                .map(|index| JavaStmt::Local {
                    finality: JavaLocalFinality::Final,
                    ty: double(),
                    name: f::name(&format!("value{index}")),
                    value: Some(primitive(
                        callable,
                        JavaExpr::local(double(), f::name("input")),
                    )),
                })
                .collect();
            statements.push(JavaStmt::Return(Some(JavaExpr::local(
                double(),
                f::name("input"),
            ))));
            functions[0].body = JavaBlock::new(statements);
            let api = JavaDependencyApi::from_certificate(f::certify(f::package(108, functions)))
                .unwrap();
            let bound = api.source_byte_bound().unwrap();
            let output = render_certified_package(&JavaStructuralRenderer, api.package()).unwrap();
            for file in output.files() {
                let OutputContents::Text(text) = file.contents() else {
                    panic!("text")
                };
                assert!(text.len() as u64 <= bound);
            }
            bounds.push(bound);
        }
        // "floor" is one byte longer than "ceil", independently per call.
        assert_eq!(bounds[0] - bounds[1], count);
    }
}

#[test]
fn rounding_original_dependency_names_cannot_be_substituted() {
    let first = owner(101, None);
    let second = owner(102, Some(&first));
    for (api, expected) in [(&first, 0), (&second, 3)] {
        let item = &api.package().ast().files()[0].items()[0];
        assert!(JavaDialect.verify_resolved_file_item(item).is_empty());
        let mut checked = 0;
        for symbol in item.names.keys() {
            if !matches!(symbol, TargetSymbolRef::DependencyCallable(_)) {
                continue;
            }
            checked += 1;
            let mut changed = item.clone();
            changed
                .names
                .insert(symbol.clone(), JavaResolvedName::Local(f::name("invented")));
            assert!(!JavaDialect.verify_resolved_file_item(&changed).is_empty());
        }
        assert_eq!(checked, expected);
    }
}
