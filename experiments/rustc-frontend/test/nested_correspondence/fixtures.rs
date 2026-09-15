//! Independent expected constructor provenance and remaining source paths.
pub(super) struct Expected {
    pub drops: &'static [usize],
    pub paths: &'static [(usize, &'static [usize])],
    pub read: usize,
    pub moves: usize,
}
pub(super) fn expected(name: &str) -> Expected {
    let (drops, paths, read, moves): (&[usize], &[(usize, &[usize])], _, _) = match name {
        "first" | "reversed" | "explicit_return" | "alternate::same" | "permuted" => {
            (&[0, 1, 2], &[(5, &[]), (4, &[0, 1]), (4, &[1])], 0, 1)
        }
        "second" => (&[1, 0, 2], &[(5, &[]), (4, &[0, 0]), (4, &[1])], 1, 1),
        "multiple" => (&[1, 0, 2], &[(6, &[]), (5, &[]), (4, &[1])], 0, 2),
        "whole_inner" => (&[0, 1, 2], &[(6, &[]), (5, &[1]), (4, &[1])], 0, 2),
        "spare" => (&[2, 0, 1], &[(5, &[]), (4, &[0, 0]), (4, &[0, 1])], 2, 1),
        "whole_unopened" => (&[2, 0, 1], &[(6, &[]), (5, &[0]), (5, &[1])], 2, 2),
        "all_leaves" => (&[2, 1, 0], &[(7, &[]), (6, &[]), (5, &[])], 1, 3),
        "reverse_extractions" => (&[0, 1, 2], &[(6, &[]), (5, &[]), (4, &[1])], 1, 2),
        "whole_multiple" => (&[1, 0, 2], &[(7, &[]), (6, &[]), (4, &[1])], 1, 3),
        "shadowed" => (&[0, 1, 2], &[(7, &[]), (5, &[1]), (4, &[1])], 0, 3),
        _ => panic!("unasserted nested correspondence fixture {name}"),
    };
    Expected {
        drops,
        paths,
        read,
        moves,
    }
}
