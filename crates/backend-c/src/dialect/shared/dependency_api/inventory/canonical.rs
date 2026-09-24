//! Type-only exports retain original handles and the certificate's typed owner.
use super::{CDialect, CGeneratedHeader, Inventory, structs};
use portable_codegen::RenderReadyPackage;

pub(super) fn collect(package: &RenderReadyPackage<CDialect>) -> Result<Option<Inventory>, String> {
    let Some(descriptor) = crate::dialect::c_canonical_type_package(package) else {
        return Ok(None);
    };
    let registry = package
        .ast()
        .files()
        .first()
        .and_then(|file| file.items().first())
        .map(|item| item.unit.projection.registry.registrations())
        .ok_or("C canonical dependency lacks original registry authority")?;
    let structs = structs::collect(package)?;
    let export = structs
        .get(descriptor.record())
        .ok_or("C canonical dependency lacks its original record")?;
    if structs.len() != 1
        || export.members != [descriptor.tag().clone(), descriptor.value().clone()]
    {
        return Err("C canonical dependency role inventory differs from its certificate".into());
    }
    let header =
        CGeneratedHeader::resolve(registry, descriptor.implementation(), descriptor.header())
            .map_err(|error| error.message)?;
    Ok(Some(Inventory {
        owner: descriptor.owner(),
        header,
        implementation: descriptor.implementation().clone(),
        structs,
        functions: Default::default(),
        constants: Default::default(),
        foreign_constants: Vec::new(),
        foreign_structs: Vec::new(),
    }))
}
