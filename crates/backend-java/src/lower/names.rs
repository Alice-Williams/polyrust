//! Allocate Java member spellings without changing portable declaration identities.

mod declarations;
mod locals;

use crate::ast::JavaIdentifier;

use crate::ast::qualifier_names::reserved_type_names;
use portable_core_ir::{
    CoreConstantId, CoreDeclaration, CoreFieldId, CoreFunctionId, CoreInterfaceId,
    CoreInterfaceMethodId, CoreProgram, CoreVariantId,
};
use std::collections::{BTreeMap, BTreeSet};

const OBJECT_MEMBERS: &[&str] = &[
    "clone",
    "finalize",
    "getClass",
    "hashCode",
    "notify",
    "notifyAll",
    "toString",
    "wait",
];

#[derive(Default)]
pub(super) struct JavaPortableNames {
    fields: BTreeMap<CoreFieldId, JavaIdentifier>,
    functions: BTreeMap<CoreFunctionId, JavaIdentifier>,
    methods: BTreeMap<CoreInterfaceMethodId, JavaIdentifier>,
    declarations: BTreeMap<CoreDeclaration, JavaIdentifier>,
    variants: BTreeMap<CoreVariantId, JavaIdentifier>,
    constants: BTreeMap<CoreConstantId, JavaIdentifier>,
    enum_values: BTreeMap<CoreVariantId, JavaIdentifier>,
    uninhabited: BTreeMap<CoreInterfaceId, JavaIdentifier>,
    locals: Vec<JavaIdentifier>,
}

impl JavaPortableNames {
    pub(super) fn new(core: &CoreProgram) -> Self {
        let mut names = Self::default();
        names.allocate_types(core);
        names.allocate_locals(core);
        for fields in core
            .records()
            .iter()
            .map(|record| &record.fields)
            .chain(core.variants().iter().map(|variant| &variant.fields))
        {
            let requested = fields.iter().map(|id| {
                core.field(*id)
                    .expect("verified field")
                    .header
                    .name
                    .as_str()
            });
            let mut pool = NamePool::new(requested, OBJECT_MEMBERS.iter().copied());
            pool.requested.extend(
                core.interface_methods()
                    .iter()
                    .map(|method| normalize(&method.header.name)),
            );
            pool.occupied.extend(names.qualifier_names());
            for id in fields {
                names.fields.insert(
                    *id,
                    pool.allocate(&core.field(*id).expect("verified field").header.name),
                );
            }
        }

        let mut pool = names.value_pool(core);
        pool.occupied
            .extend(OBJECT_MEMBERS.iter().map(|name| normalize(name)));
        for declaration in &core.module().declarations {
            if let CoreDeclaration::Function(id) = *declaration {
                names.functions.insert(
                    id,
                    pool.allocate(&core.function(id).expect("verified function").header.name),
                );
            }
        }
        // A zero-argument interface method must not accidentally become a record
        // accessor. Conservative reservation also leaves future overloads safe.
        pool.occupied.extend(names.fields.values().cloned());
        // Interfaces must also be implementable by the private zero-constant
        // enum. Reserve inherited final and compiler-generated enum methods.
        pool.occupied.extend(
            [
                "name",
                "ordinal",
                "compareTo",
                "getDeclaringClass",
                "describeConstable",
                "values",
                "valueOf",
            ]
            .into_iter()
            .map(normalize),
        );
        for interface in core.interfaces() {
            for id in &interface.methods {
                names.methods.insert(
                    *id,
                    pool.allocate(
                        &core
                            .interface_method(*id)
                            .expect("verified interface method")
                            .header
                            .name,
                    ),
                );
            }
        }
        names
    }

    pub(super) fn field(&self, id: CoreFieldId) -> &JavaIdentifier {
        &self.fields[&id]
    }

    fn qualifier_names(&self) -> BTreeSet<JavaIdentifier> {
        reserved_type_names()
            .map(normalize)
            .chain(self.declarations.values().cloned())
            .chain(self.variants.values().cloned())
            .chain(self.uninhabited.values().cloned())
            .collect()
    }

    pub(super) fn function(&self, id: CoreFunctionId) -> &JavaIdentifier {
        &self.functions[&id]
    }

    pub(super) fn method(&self, id: CoreInterfaceMethodId) -> &JavaIdentifier {
        &self.methods[&id]
    }
}

struct NamePool {
    requested: BTreeSet<JavaIdentifier>,
    occupied: BTreeSet<JavaIdentifier>,
}

impl NamePool {
    fn new<'a, 'b>(
        requested: impl Iterator<Item = &'a str>,
        reserved: impl Iterator<Item = &'b str>,
    ) -> Self {
        Self {
            requested: requested.map(normalize).collect(),
            occupied: reserved.map(normalize).collect(),
        }
    }

    fn allocate(&mut self, candidate: &str) -> JavaIdentifier {
        let preferred = normalize(candidate);
        if self.occupied.insert(preferred.clone()) {
            return preferred;
        }
        for suffix in 1_u64.. {
            let name = JavaIdentifier::new(format!("{}_{suffix}", preferred.as_str()))
                .expect("identifier plus numeric suffix remains valid");
            if !self.requested.contains(&name) && self.occupied.insert(name.clone()) {
                return name;
            }
        }
        unreachable!("a finite CoreIR namespace cannot exhaust u64 suffixes")
    }
}

fn normalize(candidate: &str) -> JavaIdentifier {
    let value = JavaIdentifier::from_portable(candidate);
    if let Some(suffix) = value.as_str().strip_prefix("__polyrust_") {
        JavaIdentifier::new(format!("poly_user_{suffix}")).expect("normalized portable identifier")
    } else {
        value
    }
}
