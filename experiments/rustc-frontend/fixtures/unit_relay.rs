//! Transitive effect calls retain producer identities.
pub fn relay(value: i32, flag: bool) {
    unit_leaf::observe(unit_leaf::first(value), unit_leaf::second(value), flag);
    if unit_leaf::predicate(flag) {
        {
            unit_leaf::empty();
        }
        unit_leaf::explicit();
    }
}
