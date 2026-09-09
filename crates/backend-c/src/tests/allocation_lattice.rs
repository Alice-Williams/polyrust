//! Exhaustive finite outcome model, independent of the production join cases.
use super::Status;

const ALL: [Status; 6] = [
    Status::Possible,
    Status::Live,
    Status::Null,
    Status::Released,
    Status::Settled,
    Status::Unproved,
];
fn outcomes(value: Status) -> u8 {
    match value {
        Status::Null => 1,
        Status::Live => 2,
        Status::Released => 4,
        Status::Possible => 3,
        Status::Settled => 5,
        Status::Unproved => 7,
    }
}
#[test]
fn every_join_contains_all_paths_and_forms_a_finite_monotone_lattice() {
    for a in ALL {
        assert_eq!(a.join(a), a);
        assert_eq!(a.outstanding(), outcomes(a) & 2 != 0);
        for b in ALL {
            let union = outcomes(a) | outcomes(b);
            assert_eq!(outcomes(a.join(b)) & union, union);
            assert_eq!(a.join(b), b.join(a));
            for c in ALL {
                assert_eq!(a.join(b).join(c), a.join(b.join(c)));
            }
        }
    }
}
