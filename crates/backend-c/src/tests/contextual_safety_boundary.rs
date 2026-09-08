//! Explicit staging controls: contextual success is not final C validity.
use super::contextual_reconstruction::{fixture, int, key, package};
use super::*;

#[test]
fn layout_operand_completeness_and_constant_truth_are_forwarded_to_safety() {
    let (mut registry, file, function, scope) = fixture();
    let record = registry.declare_struct(&file, key("Opaque")).unwrap();
    let ty = CObjectType::structure(record.clone());
    let values = CExpressions::new(&registry);
    let statements = CStatements::new(&registry, function.clone()).unwrap();
    let mut source = package(
        &registry,
        file.clone(),
        function,
        scope,
        vec![
            statements
                .discard(values.size_of(ty.clone()).unwrap())
                .unwrap(),
            statements.discard(values.align_of(ty).unwrap()).unwrap(),
        ],
    );
    let declarations = CDeclarations::new(&registry, file).unwrap();
    source.items.insert(
        0,
        CFileItem::Declaration(
            declarations
                .forward_tag(CAggregateRef::Struct(record))
                .unwrap(),
        ),
    );
    let zero = values
        .literal(CLiteral::Signed(CSignedLiteral::I32(0)))
        .unwrap();
    let unsafe_constant = values
        .binary(CBinaryOperator::Divide, int(&values), zero.clone())
        .unwrap();
    for condition in [zero, unsafe_constant] {
        source.items.push(CFileItem::StaticAssert(
            declarations
                .static_assert(condition, CAssertDiagnostic::new("deferred safety"))
                .unwrap(),
        ));
    }
    // These inputs must NEVER receive a final certificate. This diagnostic-only
    // stage forwards layout operand completeness, arithmetic and assertion truth.
    registry.check_context(&[source]).unwrap();
}
