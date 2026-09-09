//! Internal mutation controls for the private composed proof's site associations.
use super::*;
use crate::ast::*;
use crate::ast::{CBinaryOperator as B, CScalarType, numeric_fixture::Fixture};
use crate::dialect::CKnownCall;
use crate::ownership::numeric_flow::Obligation;

#[test]
fn index_obligation_retains_its_actual_arithmetic_history() {
    for wrapped in [false, true] {
        let mut f = Fixture::new(&[CScalarType::Size]);
        let ty = CObjectType::array(
            CObjectType::scalar(CScalarType::Int),
            CArrayLength::new(1).unwrap(),
        )
        .unwrap();
        let array = f
            .registry
            .register_local(
                &f.scope,
                CDeclarationKey {
                    name: CIdentifier::new("array").unwrap(),
                    origin: CGeneratedOrigin::Synthesized(CSynthesisReason::TestHarness),
                },
                ty,
            )
            .unwrap();
        let index = if wrapped {
            f.binary(B::Add, f.input(0), f.size(1))
        } else {
            f.size(0)
        };
        let place = f
            .values()
            .index(
                CIndexBase::Array(Box::new(f.values().local(array.clone()).unwrap())),
                index,
            )
            .unwrap();
        let files = [f.source(vec![
            f.ast().declare(array, None).unwrap(),
            f.discard(f.values().address_of(place).unwrap()),
        ])];
        let facts = NumericFacts::check(&f.registry, &files).unwrap();
        let (place, number) = facts
            .analysis
            .obligations
            .iter()
            .find_map(|entry| match &entry.kind {
                Obligation::Index { place, index } => Some((*place, index)),
                _ => None,
            })
            .unwrap();
        if wrapped {
            let CPlaceKind::Index { index, .. } = place.kind() else {
                panic!("index");
            };
            assert_eq!(number.losses.len(), 1);
            let crate::ownership::numeric_flow::provenance::Origin::Arithmetic(actual) =
                &number.losses[0]
            else {
                panic!("arithmetic history");
            };
            assert!(std::ptr::eq(*actual, index.as_ref()));
        } else {
            assert!(number.losses.is_empty());
            assert_eq!(number.domain.integer_bounds(), Some((0, 0)));
        }
    }
}

#[test]
fn facts_retain_actual_context_point_and_calculation_identity() {
    let f = Fixture::new(&[CScalarType::Size]);
    let expression = f.binary(B::Add, f.input(0), f.size(1));
    let files = [f.source(vec![f.discard(expression)])];
    let facts = NumericFacts::check(&f.registry, &files).unwrap();
    assert!(std::ptr::eq(facts.context.registry(), &f.registry));
    assert!(std::ptr::eq(facts.context.files(), files.as_slice()));
    assert_eq!(facts.analysis.functions.len(), 1);
    let obligation = facts
        .analysis
        .obligations
        .iter()
        .find(|entry| matches!(entry.kind, Obligation::Calculation { .. }))
        .unwrap();
    let graph = &facts.context.functions()[0];
    assert_eq!(obligation.site.function, graph.function());
    let crate::ast::contextual::flow_graph::Action::Discard(actual) =
        graph.node(obligation.site.point).action()
    else {
        panic!("actual discard");
    };
    let Obligation::Calculation { value, number } = &obligation.kind else {
        panic!("calculation");
    };
    assert!(std::ptr::eq(*value, *actual));
    assert_eq!(number.losses.len(), 1);
    let crate::ownership::numeric_flow::provenance::Origin::Arithmetic(origin) = &number.losses[0]
    else {
        panic!("arithmetic origin");
    };
    assert!(std::ptr::eq(*origin, *actual));
}

#[test]
fn missing_or_unreachable_function_sites_cannot_validate() {
    let f = Fixture::new(&[CScalarType::Size]);
    let files = [f.source(vec![f.discard(f.binary(B::Add, f.input(0), f.size(1)))])];
    let mut facts = NumericFacts::check(&f.registry, &files).unwrap();
    let index = facts.analysis.obligations[0].site.point.index();
    facts.analysis.functions[0].incoming[index] = None;
    assert_eq!(facts.validate_sites(), Err(E::InvalidNumericSite));
    let mut facts = NumericFacts::check(&f.registry, &files).unwrap();
    facts.analysis.functions.clear();
    assert_eq!(facts.validate_sites(), Err(E::InvalidNumericSite));
}

#[test]
fn index_obligation_retains_actual_place_and_rejects_another_reachable_site() {
    let mut f = Fixture::new(&[CScalarType::Size]);
    let ty = CObjectType::array(
        CObjectType::scalar(CScalarType::Int),
        CArrayLength::new(1).unwrap(),
    )
    .unwrap();
    let array = f
        .registry
        .register_local(
            &f.scope,
            CDeclarationKey {
                name: CIdentifier::new("array").unwrap(),
                origin: CGeneratedOrigin::Synthesized(CSynthesisReason::TestHarness),
            },
            ty,
        )
        .unwrap();
    let place = f
        .values()
        .index(
            CIndexBase::Array(Box::new(f.values().local(array.clone()).unwrap())),
            f.input(0),
        )
        .unwrap();
    let files = [f.source(vec![
        f.ast().declare(array, None).unwrap(),
        f.discard(f.values().address_of(place).unwrap()),
    ])];
    let mut facts = NumericFacts::check(&f.registry, &files).unwrap();
    let entry = facts
        .analysis
        .obligations
        .iter_mut()
        .find(|entry| matches!(entry.kind, Obligation::Index { .. }))
        .unwrap();
    let graph = &facts.context.functions()[0];
    let crate::ast::contextual::flow_graph::Action::Discard(value) =
        graph.node(entry.site.point).action()
    else {
        panic!("discard");
    };
    let CValueKind::AddressOf(actual) = value.kind() else {
        panic!("address");
    };
    let Obligation::Index { place, index } = &entry.kind else {
        panic!("index");
    };
    assert!(std::ptr::eq(*place, actual.as_ref()));
    assert_eq!(
        index.domain.integer_bounds(),
        Some((0, i128::from(u64::MAX)))
    );
    entry.site.point = graph
        .points()
        .find(|point| {
            matches!(
                graph.node(*point).action(),
                crate::ast::contextual::flow_graph::Action::FunctionEnd
            )
        })
        .unwrap();
    assert_eq!(facts.validate_sites(), Err(E::InvalidNumericSite));
}

#[test]
fn call_obligation_retains_actual_arguments_and_the_implicit_byte_product() {
    let f = Fixture::new(&[]);
    let null = |ty| {
        f.values()
            .literal(CLiteral::NullPointer(CNullPointer::new(ty).unwrap()))
            .unwrap()
    };
    let readable = CObjectType::pointer(CPointerTarget::Void(CConstness::Const));
    let stream = CObjectType::pointer(CPointerTarget::Object(Box::new(CObjectType::known(
        CKnownObject::File,
    ))));
    let call = f
        .values()
        .call_value(
            f.values().known(CKnownCall::WriteBytes),
            vec![null(readable), f.size(2), f.size(3), null(stream)],
        )
        .unwrap();
    let files = [f.source(vec![f.discard(call)])];
    let mut facts = NumericFacts::check(&f.registry, &files).unwrap();
    let entry = facts
        .analysis
        .obligations
        .iter_mut()
        .find(|entry| matches!(entry.kind, Obligation::Call { .. }))
        .unwrap();
    let graph = &facts.context.functions()[0];
    let crate::ast::contextual::flow_graph::Action::Discard(value) =
        graph.node(entry.site.point).action()
    else {
        panic!("discard");
    };
    let CValueKind::Call(actual) = value.kind() else {
        panic!("call");
    };
    let Obligation::Call {
        call,
        arguments,
        byte_product,
    } = &entry.kind
    else {
        panic!("call obligation");
    };
    assert!(std::ptr::eq(*call, actual));
    assert!(std::ptr::eq(
        call.arguments().as_ptr(),
        actual.arguments().as_ptr()
    ));
    assert_eq!(arguments.len(), 4);
    for index in [1, 2] {
        assert_eq!(
            arguments[index].as_ref().unwrap().domain.integer_bounds(),
            Some(((index + 1) as i128, (index + 1) as i128))
        );
    }
    assert_eq!(
        byte_product.as_ref().unwrap().domain.integer_bounds(),
        Some((6, 6))
    );
    assert_eq!(
        byte_product.as_ref().unwrap().loss,
        crate::ownership::ranges::NumericLoss::None
    );
    let point = entry.site.point.index();
    facts.analysis.functions[0].incoming[point] = None;
    assert_eq!(facts.validate_sites(), Err(E::InvalidNumericSite));
}
