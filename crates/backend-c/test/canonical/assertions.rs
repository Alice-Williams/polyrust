//! Callers cannot customize a canonical owner's platform obligation text/order.
use super::fixture::*;
use portable_backend_c::ast::*;

#[derive(Clone, Copy, Debug)]
enum Fault {
    Partial,
    Message,
    WrongLayout,
    Late,
    Duplicate,
}

#[test]
fn noncanonical_platform_assertions_cannot_change_owner_bytes() {
    for fault in [
        Fault::Partial,
        Fault::Message,
        Fault::WrongLayout,
        Fault::Late,
        Fault::Duplicate,
    ] {
        let mut value = fixture(false, super::fixture::Fault::None);
        register(&mut value, super::fixture::Fault::None).unwrap();
        let d = CDeclarations::new(&value.registry, value.header.clone()).unwrap();
        let e = CExpressions::new(&value.registry);
        let left = e.size_of(CObjectType::scalar(CScalarType::Bool)).unwrap();
        let right = e
            .literal(CLiteral::Unsigned(CUnsignedLiteral::Size(
                if matches!(fault, Fault::WrongLayout) {
                    2
                } else {
                    1
                },
            )))
            .unwrap();
        let condition = e.binary(CBinaryOperator::Equal, left, right).unwrap();
        let assertion = CFileItem::StaticAssert(
            d.static_assert(
                condition,
                CAssertDiagnostic::new(if matches!(fault, Fault::Message) {
                    b"caller text".to_vec()
                } else {
                    b"C profile Bool Size".to_vec()
                }),
            )
            .unwrap(),
        );
        let mut items = value.files[0].items().to_vec();
        if matches!(fault, Fault::Late) {
            items.push(assertion);
        } else {
            items.insert(0, assertion.clone());
            if matches!(fault, Fault::Duplicate) {
                items.insert(0, assertion);
            }
        }
        value.files[0] = d.source_file(items).unwrap();
        let error = certify(value).unwrap_err();
        assert!(
            error.contains("canonical") || error.contains("platform"),
            "{fault:?}: {error}"
        );
    }
}
