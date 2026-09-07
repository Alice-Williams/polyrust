//! Namespaces for nominal declarations and static values, keyed by CoreIR identities.

use super::{JavaPortableNames, NamePool};
use crate::ast::JavaIdentifier;
use portable_core_ir::{
    CoreConstantId, CoreDeclaration, CoreEnumId, CoreInterfaceId, CoreProgram, CoreRecordId,
    CoreVariantId,
};

impl JavaPortableNames {
    pub(super) fn allocate_types(&mut self, core: &CoreProgram) {
        let declarations = core
            .module()
            .declarations
            .iter()
            .filter_map(|declaration| {
                let name = match *declaration {
                    CoreDeclaration::Record(id) => {
                        &core.record(id).expect("verified record").header.name
                    }
                    CoreDeclaration::Enum(id) => {
                        &core.enumeration(id).expect("verified enum").header.name
                    }
                    CoreDeclaration::Interface(id) => {
                        &core.interface(id).expect("verified interface").header.name
                    }
                    _ => return None,
                };
                Some((*declaration, name.as_str()))
            })
            .collect::<Vec<_>>();
        let variants = core
            .enums()
            .iter()
            .filter(|enumeration| {
                enumeration.variants.iter().any(|id| {
                    !core
                        .variant(*id)
                        .expect("verified variant")
                        .fields
                        .is_empty()
                })
            })
            .flat_map(|enumeration| {
                enumeration.variants.iter().map(|id| {
                    (
                        *id,
                        format!(
                            "{}{}",
                            enumeration.header.name,
                            core.variant(*id).expect("verified variant").header.name
                        ),
                    )
                })
            })
            .collect::<Vec<_>>();
        let requested = declarations
            .iter()
            .map(|(_, name)| *name)
            .chain(variants.iter().map(|(_, name)| name.as_str()));
        let reserved = super::reserved_type_names();
        let mut pool = NamePool::new(requested, reserved);
        for (id, name) in declarations {
            self.declarations.insert(id, pool.allocate(name));
        }
        for (id, name) in variants {
            self.variants.insert(id, pool.allocate(&name));
        }
        let mut synthetic = Vec::new();
        for declaration in &core.module().declarations {
            if let CoreDeclaration::Interface(id) = *declaration
                && !core
                    .implementations()
                    .iter()
                    .any(|value| value.interface == id)
            {
                let preferred = format!("Uninhabited{}", self.interface(id).as_str());
                synthetic.push((id, preferred));
            }
        }
        pool.requested
            .extend(synthetic.iter().map(|(_, name)| super::normalize(name)));
        for (id, preferred) in synthetic {
            self.uninhabited.insert(id, pool.allocate(&preferred));
        }
    }

    pub(super) fn value_pool(&mut self, core: &CoreProgram) -> NamePool {
        let requested = core
            .constants()
            .iter()
            .map(|value| value.header.name.as_str())
            .chain(
                core.variants()
                    .iter()
                    .map(|value| value.header.name.as_str()),
            )
            .chain(
                core.functions()
                    .iter()
                    .map(|value| value.header.name.as_str()),
            )
            .chain(
                core.interface_methods()
                    .iter()
                    .map(|value| value.header.name.as_str()),
            );
        let mut pool = NamePool::new(requested, std::iter::empty());
        pool.occupied.extend(self.qualifier_names());
        for declaration in &core.module().declarations {
            if let CoreDeclaration::Constant(id) = *declaration {
                self.constants.insert(
                    id,
                    pool.allocate(&core.constant(id).expect("verified constant").header.name),
                );
            }
        }
        for enumeration in core.enums().iter().filter(|enumeration| {
            enumeration.variants.iter().all(|id| {
                core.variant(*id)
                    .expect("verified variant")
                    .fields
                    .is_empty()
            })
        }) {
            for id in &enumeration.variants {
                self.enum_values.insert(
                    *id,
                    pool.allocate(&core.variant(*id).expect("verified variant").header.name),
                );
            }
        }
        pool
    }

    pub(in crate::lower) fn record(&self, id: CoreRecordId) -> &JavaIdentifier {
        &self.declarations[&CoreDeclaration::Record(id)]
    }
    pub(in crate::lower) fn enumeration(&self, id: CoreEnumId) -> &JavaIdentifier {
        &self.declarations[&CoreDeclaration::Enum(id)]
    }
    pub(in crate::lower) fn interface(&self, id: CoreInterfaceId) -> &JavaIdentifier {
        &self.declarations[&CoreDeclaration::Interface(id)]
    }
    pub(in crate::lower) fn uninhabited(&self, id: CoreInterfaceId) -> &JavaIdentifier {
        &self.uninhabited[&id]
    }
    pub(in crate::lower) fn variant(&self, id: CoreVariantId) -> &JavaIdentifier {
        &self.variants[&id]
    }
    pub(in crate::lower) fn constant(&self, id: CoreConstantId) -> &JavaIdentifier {
        &self.constants[&id]
    }
    pub(in crate::lower) fn enum_value(&self, id: CoreVariantId) -> &JavaIdentifier {
        &self.enum_values[&id]
    }
}
