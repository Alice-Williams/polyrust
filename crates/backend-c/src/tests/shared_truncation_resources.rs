//! Library stack reserves remain live in certificate and budget accounting.
use super::super::super::{call_fixture, resources};
use super::*;
use portable_diagnostics::DiagnosticCode;

fn graph(count: usize, repetitions: usize) -> LinkedTargetPackage<CDialect> {
    let graph: Vec<_> = (0..count)
        .map(|index| {
            if index + 1 < count {
                vec![index + 1]
            } else {
                vec![]
            }
        })
        .collect();
    let (registry, source) =
        call_fixture::scalar_fixture(&graph, 0, true, &vec![1; count], CScalarType::F64);
    let e = CExpressions::new(registry.registrations());
    let declarations =
        CDeclarations::new(registry.registrations(), source.identity().clone()).unwrap();
    let items = source
        .items()
        .iter()
        .map(|item| {
            let CFileItem::Definition(definition) = item else {
                return item.clone();
            };
            let CDefinitionKind::Function {
                function,
                linkage,
                parameters,
                body,
            } = definition.kind()
            else {
                return item.clone();
            };
            let statements = CStatements::new(registry.registrations(), function.clone()).unwrap();
            let operand = e.read(e.parameter(parameters[0].clone()).unwrap()).unwrap();
            let call = e
                .call_value(e.known(CKnownCall::FloatTruncate), vec![operand])
                .unwrap();
            let mut contents = vec![statements.discard(call).unwrap(); repetitions];
            contents.extend_from_slice(body.statements());
            let body = statements.block(body.scope().clone(), contents).unwrap();
            CFileItem::Definition(
                declarations
                    .function_definition(function.clone(), *linkage, parameters.clone(), body)
                    .unwrap(),
            )
        })
        .collect();
    let source = declarations.source_file(items).unwrap();
    let package = crate::dialect::project_c_package(registry, vec![source]).unwrap();
    let checked = verify_unresolved_package(&CDialect, package).unwrap();
    TargetLinker::new(CDialect).link_ast(&checked).unwrap()
}

#[test]
fn known_stack_cost_is_nonzero_once_per_frame_and_transitive() {
    let reserve = CKnownCall::FloatTruncate.stack_bound_bytes().unwrap().get();
    assert_eq!(reserve, 65536);
    for known in CKnownCall::ALL {
        assert_eq!(
            known.stack_bound_bytes().is_some(),
            known == CKnownCall::FloatTruncate
        );
    }
    for repetitions in [1, 2, 4] {
        let package = graph(1, repetitions);
        let measured = resources::measure_package(&package).unwrap();
        assert_eq!(
            measured.total.frame_bound,
            resources::frame_bound(&measured.total).unwrap() + reserve
        );
        assert!(certify_resolved_package(&CDialect, package).is_ok());
    }
    let owners = chain();
    let bounds: Vec<_> = owners
        .iter()
        .map(|owner| owner.functions().next().unwrap().stack_bound_bytes())
        .collect();
    assert!(bounds[0] > reserve);
    assert!(bounds[1] > bounds[0] && bounds[2] > bounds[1]);
}

#[test]
fn actual_truncation_call_chain_budget_and_missing_cost_mutants() {
    let reserve = CKnownCall::FloatTruncate.stack_bound_bytes().unwrap().get();
    let limit = resources::policy::CResourceKind::NativeFrameBytes.limit();
    let mut previous = None;
    for count in 1..32 {
        let package = graph(count, 1);
        let measured = resources::measure_package(&package).unwrap();
        let result = certify_resolved_package(&CDialect, package);
        if measured.total.frame_bound <= limit {
            assert!(result.is_ok(), "count={count}");
            previous = Some(measured.total.frame_bound);
            continue;
        }
        assert!(previous.is_some_and(|bound| bound <= limit));
        let errors = result.unwrap_err();
        assert!(
            errors
                .iter()
                .any(|error| error.code == DiagnosticCode::TargetResourceLimit
                    && error.message.contains("NativeFrameBytes"))
        );
        let source = portable_diagnostics::SourceRef::logical(["c", "truncation-cost-control"]);
        let mut missing = measured.total.clone();
        missing.frame_bound -= count as u64 * reserve;
        assert!(
            resources::policy::check(&missing, source.clone()).is_empty(),
            "missing reserve must change admission"
        );
        let mut underestimated = measured.total.clone();
        underestimated.frame_bound -= count as u64 * (reserve - 1);
        assert!(
            resources::policy::check(&underestimated, source.clone()).is_empty(),
            "one-byte reserve must change admission"
        );
        let mut edge = measured.total.clone();
        edge.frame_bound = limit;
        assert!(resources::policy::check(&edge, source.clone()).is_empty());
        edge.frame_bound += 1;
        assert!(
            resources::policy::check(&edge, source)
                .iter()
                .any(|error| error.kind() == resources::policy::CResourceKind::NativeFrameBytes)
        );
        eprintln!(
            "truncation stack chain: {} admitted, {count} rejected at {}, preceding {}",
            count - 1,
            measured.total.frame_bound,
            previous.unwrap()
        );
        return;
    }
    panic!("missing actual-AST stack boundary");
}
