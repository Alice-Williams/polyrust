//! Compiler-produced frame evidence for native oracle fixtures, not generation.
use std::{fs, path::Path};

pub(super) fn check(directory: &Path, source_bound: u64) {
    let mut reports = 0;
    let mut generated = 0;
    let mut maximum = 0u64;
    for entry in fs::read_dir(directory).unwrap() {
        let path = entry.unwrap().path();
        if path.extension().is_none_or(|extension| extension != "su") {
            continue;
        }
        reports += 1;
        for line in fs::read_to_string(&path).unwrap().lines() {
            let columns = line.split('\t').collect::<Vec<_>>();
            if columns
                .first()
                .is_none_or(|name| !name.ends_with(":poly_identity"))
            {
                continue;
            }
            assert_eq!(
                columns.len(),
                3,
                "unrecognized compiler frame report: {line}"
            );
            let bytes = columns[1].parse::<u64>().unwrap();
            assert!(
                matches!(columns[2], "static" | "dynamic,bounded"),
                "unbounded native frame: {line}"
            );
            maximum = maximum.max(bytes);
            generated += 1;
        }
    }
    assert!(
        reports > 0,
        "compiler produced no .su report in {}",
        directory.display()
    );
    assert!(
        generated > 0,
        "frame report omitted the generated function in {}",
        directory.display()
    );
    // Candidate native-path budget. This empirical check is not yet the
    // conservative source-derived spill/frame admission rule.
    assert!(
        maximum <= 1024 * 1024,
        "candidate native frame budget exceeded"
    );
    assert!(
        maximum <= source_bound,
        "native frame {maximum} exceeds source-derived bound {source_bound}"
    );
    eprintln!(
        "C native frame {}: {maximum} bytes",
        directory.file_name().unwrap().to_string_lossy()
    );
}
