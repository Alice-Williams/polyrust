use super::{PathPolicy, TreeLimits, paths::validate};

fn limits() -> TreeLimits {
    TreeLimits {
        files: 3,
        directories: 3,
        depth: 2,
        path_bytes: 7,
        bytes: 9,
    }
}
fn payload() -> Vec<(String, String)> {
    ["a/b/one", "a/two", "c/three"]
        .into_iter()
        .map(|path| (path.into(), "one".into()))
        .collect()
}

#[test]
fn exact_and_one_over_each_inventory_bound() {
    let files = payload();
    assert_eq!(
        validate(&files, PathPolicy::RelativeTree(limits())).unwrap(),
        ["a", "a/b", "c"]
    );
    for lower in [
        TreeLimits {
            files: 2,
            ..limits()
        },
        TreeLimits {
            directories: 2,
            ..limits()
        },
        TreeLimits {
            depth: 1,
            ..limits()
        },
        TreeLimits {
            path_bytes: 6,
            ..limits()
        },
        TreeLimits {
            bytes: 8,
            ..limits()
        },
    ] {
        assert!(validate(&files, PathPolicy::RelativeTree(lower)).is_err());
    }
    assert!(validate(&[], PathPolicy::Flat).is_err());
}

#[test]
fn unsafe_spelling_and_order_independent_prefix_conflicts() {
    for path in [
        "",
        ".",
        "..",
        "../escape",
        "/absolute",
        "a//b",
        "a/./b",
        "a/../b",
        "a/",
        "a\\b",
        "C:/b",
        "a b",
        "a\0b",
        "a/é",
    ] {
        let files = [(path.into(), String::new())];
        assert!(
            validate(&files, PathPolicy::RelativeTree(limits())).is_err(),
            "{path:?}"
        );
    }
    for paths in [["a", "a/b"], ["a/b", "a"], ["a/b", "a/b"]] {
        let files: Vec<_> = paths
            .into_iter()
            .map(|name| (name.into(), String::new()))
            .collect();
        assert!(validate(&files, PathPolicy::RelativeTree(limits())).is_err());
    }
    assert!(validate(&payload(), PathPolicy::Flat).is_err());
}

#[test]
fn hard_ceilings_cannot_be_relaxed_by_a_caller() {
    for invalid in [
        TreeLimits {
            files: 3074,
            ..limits()
        },
        TreeLimits {
            directories: 4097,
            ..limits()
        },
        TreeLimits {
            depth: 33,
            ..limits()
        },
        TreeLimits {
            path_bytes: 4097,
            ..limits()
        },
        TreeLimits {
            bytes: 256 * 1024 * 1024 + 1,
            ..limits()
        },
        TreeLimits {
            files: usize::MAX,
            ..limits()
        },
        TreeLimits {
            directories: usize::MAX,
            ..limits()
        },
        TreeLimits {
            depth: usize::MAX,
            ..limits()
        },
        TreeLimits {
            path_bytes: usize::MAX,
            ..limits()
        },
        TreeLimits {
            bytes: usize::MAX,
            ..limits()
        },
    ] {
        assert!(validate(&payload(), PathPolicy::RelativeTree(invalid)).is_err());
    }
}

#[test]
fn full_java_directory_inventory_and_c_file_limit() {
    let mut files = Vec::new();
    for owner in 0..1024 {
        files.push((
            format!("src/main/java/org/polyrust/generated/r{owner:016x}/Generated.java"),
            String::new(),
        ));
        files.push((format!("owner-{owner:016x}.json"), String::new()));
    }
    files.push(("bundle.json".into(), String::new()));
    let java = TreeLimits {
        files: 2049,
        directories: 1030,
        depth: 7,
        path_bytes: 256,
        bytes: 0,
    };
    assert_eq!(
        validate(&files, PathPolicy::RelativeTree(java))
            .unwrap()
            .len(),
        1030
    );
    assert!(
        validate(
            &files,
            PathPolicy::RelativeTree(TreeLimits {
                directories: 1029,
                ..java
            })
        )
        .is_err()
    );
    let mut flat: Vec<_> = (0..3073)
        .map(|i| (format!("f{i}"), String::new()))
        .collect();
    assert!(validate(&flat, PathPolicy::Flat).unwrap().is_empty());
    flat.push(("extra".into(), String::new()));
    assert!(validate(&flat, PathPolicy::Flat).is_err());
}
