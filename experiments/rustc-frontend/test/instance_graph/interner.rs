//! Same-graph observation authority, deliberately unable to issue target code.
use super::shape::ResultShape;
use portable_codegen::{
    RustCanonicalErrorKindFacts as ErrorFacts, RustCanonicalInstanceFacts as Facts,
    RustCanonicalInstanceKey as Key,
};
use std::{collections::BTreeMap, sync::Arc};

const MAX_OWNERS: usize = 1024;
const MAX_USES: usize = 100_000;

pub(super) struct Observation {
    facts: ErrorFacts,
}

impl Observation {
    pub(super) fn facts(&self) -> Facts {
        self.facts.instance()
    }

    pub(super) fn error_facts(&self) -> ErrorFacts {
        self.facts
    }
}

pub(super) struct Interner {
    owners: BTreeMap<Key, Arc<Observation>>,
    source_owners: usize,
    uses: usize,
    failed: bool,
}

impl Interner {
    pub(super) fn new(source_owners: usize) -> Result<Self, String> {
        if !(1..=MAX_OWNERS).contains(&source_owners) {
            return Err("instance graph source owner budget exceeded".into());
        }
        Ok(Self {
            owners: BTreeMap::new(),
            source_owners,
            uses: 0,
            failed: false,
        })
    }

    pub(super) fn intern(&mut self, witness: &ResultShape<'_>) -> Result<Arc<Observation>, String> {
        let facts = witness.error_facts();
        #[cfg(instance_graph_late_conflict)]
        let facts = if self.uses >= 2 {
            let instance = facts.instance();
            let changed = Facts::new(
                instance.key(),
                instance.core_root(),
                instance.err(),
                instance.ok(),
            )
            .unwrap();
            with_instance(facts, changed)
        } else {
            facts
        };
        self.intern_facts(facts)
    }

    // Only this module and its adversarial controls can supply unauthenticated
    // facts. The graph adapter can register only a checked compiler witness.
    fn intern_facts(&mut self, facts: ErrorFacts) -> Result<Arc<Observation>, String> {
        if self.failed {
            return Err("instance graph transaction already failed".into());
        }
        let result = self.register(facts);
        if result.is_err() {
            self.failed = true;
        }
        result
    }

    fn register(&mut self, facts: ErrorFacts) -> Result<Arc<Observation>, String> {
        if self.uses == MAX_USES {
            return Err("instance graph use budget exceeded".into());
        }
        if let Some(previous) = self.owners.get(&facts.instance().key()) {
            if previous.facts != facts {
                return Err("instance graph original facts conflict".into());
            }
            self.uses += 1;
            return Ok(Arc::clone(previous));
        }
        if self.source_owners + self.owners.len() == MAX_OWNERS {
            return Err("instance graph total owner budget exceeded".into());
        }
        let observation = Arc::new(Observation { facts });
        self.owners
            .insert(facts.instance().key(), Arc::clone(&observation));
        self.uses += 1;
        Ok(observation)
    }

    pub(super) fn freeze(self) -> Result<Frozen, String> {
        if self.failed {
            return Err("instance graph transaction already failed".into());
        }
        Ok(Frozen {
            owners: self.owners,
            uses: self.uses,
        })
    }
}

pub(super) struct Frozen {
    owners: BTreeMap<Key, Arc<Observation>>,
    uses: usize,
}

impl Frozen {
    pub(super) fn owners(&self) -> impl Iterator<Item = (&Key, &Arc<Observation>)> {
        self.owners.iter()
    }

    pub(super) fn uses(&self) -> usize {
        self.uses
    }

    pub(super) fn contains_original(&self, observation: &Arc<Observation>) -> bool {
        self.owners
            .get(&observation.facts.instance().key())
            .is_some_and(|original| Arc::ptr_eq(original, observation))
    }
}

#[cfg(instance_graph_controls)]
#[path = "controls.rs"]
mod controls;

#[cfg(instance_graph_controls)]
pub(super) fn run_controls() {
    controls::run();
}

#[cfg(instance_graph_late_conflict)]
fn with_instance(facts: ErrorFacts, instance: Facts) -> ErrorFacts {
    use portable_codegen::{RustIntegerErrorKind as Kind, RustIntegerErrorVariants as Variants};
    ErrorFacts::new(
        instance,
        facts.wrapper_field(),
        facts.kind_definition(),
        Variants {
            empty: facts.variant(Kind::Empty),
            invalid_digit: facts.variant(Kind::InvalidDigit),
            positive_overflow: facts.variant(Kind::PosOverflow),
            negative_overflow: facts.variant(Kind::NegOverflow),
            zero: facts.variant(Kind::Zero),
            not_a_power_of_two: facts.variant(Kind::NotAPowerOfTwo),
        },
    )
    .unwrap()
}
