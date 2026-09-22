//! Check actual registered objects and read nodes, not matching rendered text.
use super::{ConstantDeclarationInput, Mapping, PublicConstantReadInput};
use crate::c_lower::{Reader, package::State};
use portable_backend_c::ast::*;
pub(super) fn declaration(state: &State, input: ConstantDeclarationInput<'_>, object: &CObjectRef) {
    let CGeneratedOrigin::RustSource(origin) = &object.key().origin else {
        panic!("source origin")
    };
    assert_eq!(
        origin.declaration,
        crate::source_origin::identity(input.tcx(), input.definition())
    );
    assert!(origin.externally_reachable);
    assert_eq!(Some(object.file()), state.header.as_ref());
    assert_eq!(
        state.constants[&input.definition()],
        (object.clone(), input.value())
    );
    println!("PUBLIC_CONSTANT_DECL\tc\t{:?}", input.value());
}
pub(super) fn read<'tcx>(
    reader: &mut Reader<'tcx>,
    input: PublicConstantReadInput<'tcx>,
    value: &CValue,
) {
    let CValueKind::Read(place) = value.kind() else {
        panic!("public constant must be a read")
    };
    let CPlaceKind::Global(object) = place.kind() else {
        panic!("owned object")
    };
    assert_eq!(object, &reader.constants[&input.definition()].0);
    assert_eq!(place.ty().constness(), CConstness::Const);
    assert_eq!(value.ty().constness(), CConstness::Unqualified);

    let saved = reader.constants.remove(&input.definition()).unwrap();
    assert!(super::CPublicConstantReads.lower(reader, input).is_err());
    reader.constants.insert(input.definition(), saved.clone());
    reader.constants.get_mut(&input.definition()).unwrap().1 = match input.value() {
        super::ScalarConstantValue::Bool(v) => super::ScalarConstantValue::Bool(!v),
        super::ScalarConstantValue::I32(v) => super::ScalarConstantValue::I32(v.wrapping_add(1)),
        super::ScalarConstantValue::I64(v) => super::ScalarConstantValue::I64(v.wrapping_add(1)),
        super::ScalarConstantValue::F64(v) => super::ScalarConstantValue::F64(
            portable_binary64::FiniteBinary64::from_bits(v.to_bits() ^ (1_u64 << 63)).unwrap(),
        ),
        super::ScalarConstantValue::Infinity(sign) => {
            super::ScalarConstantValue::Infinity(match sign {
                portable_binary64::Binary64Sign::Positive => {
                    portable_binary64::Binary64Sign::Negative
                }
                portable_binary64::Binary64Sign::Negative => {
                    portable_binary64::Binary64Sign::Positive
                }
            })
        }
    };
    assert!(super::CPublicConstantReads.lower(reader, input).is_err());
    reader.constants.insert(input.definition(), saved.clone());
    if let Some(other) = reader
        .constants
        .iter()
        .find(|(id, c)| **id != input.definition() && c.1 == input.value())
        .map(|(_, c)| c.clone())
    {
        reader.constants.insert(input.definition(), other);
        assert!(super::CPublicConstantReads.lower(reader, input).is_err());
        reader.constants.insert(input.definition(), saved);
    }
    println!("PUBLIC_CONSTANT_READ\tc\t{:?}", input.value());
}
