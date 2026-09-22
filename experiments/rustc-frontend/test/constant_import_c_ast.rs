//! Observe production imported-object reads and test missing/wrong mappings.
use super::{Mapping, PublicConstantReadInput};
use crate::c_lower::Reader;
use portable_backend_c::ast::*;

pub(super) fn read<'tcx>(
    reader: &mut Reader<'tcx>,
    input: PublicConstantReadInput<'tcx>,
    value: &CValue,
) {
    let CValueKind::Read(place) = value.kind() else {
        panic!("foreign constant must be a read")
    };
    let CPlaceKind::Global(object) = place.kind() else {
        panic!("foreign global reference")
    };
    let (registered, proof) = &reader.foreign_constants[&input.definition()];
    assert_eq!(object, registered);
    assert_eq!(reader.registry.imported_constant(object).unwrap(), proof);
    assert_eq!(
        proof.declaration(),
        crate::source_origin::identity(reader.tcx, input.definition())
    );
    assert_eq!(
        proof.value(),
        &crate::c_lower::constants::value(input.value())
    );
    assert_eq!(value.ty(), proof.read_type());
    assert_eq!(place.ty().constness(), CConstness::Const);
    assert_eq!(object.file(), proof.public_header().file());
    assert_ne!(Some(object.file()), reader.header.as_ref());
    assert!(!reader.constants.contains_key(&input.definition()));
    let saved = reader
        .foreign_constants
        .remove(&input.definition())
        .unwrap();
    assert!(super::CPublicConstantReads.lower(reader, input).is_err());
    for other in reader
        .foreign_constants
        .values()
        .cloned()
        .collect::<Vec<_>>()
    {
        reader.foreign_constants.insert(input.definition(), other);
        assert!(super::CPublicConstantReads.lower(reader, input).is_err());
    }
    reader.foreign_constants.insert(input.definition(), saved);
    println!("CONSTANT_IMPORT_READ\tc\t{:?}", input.value());
}
