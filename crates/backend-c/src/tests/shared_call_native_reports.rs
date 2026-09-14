//! Require every generated frame and prove private/public native linkage.
use super::call_native_tests::Probe;
use std::{collections::BTreeMap, fs, path::Path, process::Command};

pub(super) fn check(directory: &Path, object: &Path, probe: &Probe, graph: &[Vec<usize>]) {
    let mut actual = BTreeMap::new();
    let mut derivatives = BTreeMap::new();
    for entry in fs::read_dir(directory).unwrap() {
        let path = entry.unwrap().path();
        if path.extension().is_none_or(|extension| extension != "su") {
            continue;
        }
        for line in fs::read_to_string(path).unwrap().lines() {
            let fields: Vec<_> = line.split('\t').collect();
            let Some(name) = fields.first().and_then(|field| field.rsplit(':').next()) else {
                continue;
            };
            let Some(bound) =
                frame_bound(name, &probe.frames).unwrap_or_else(|error| panic!("{error}: {line}"))
            else {
                continue;
            };
            assert_eq!(fields.len(), 3, "unrecognized frame report: {line}");
            assert!(
                matches!(fields[2], "static" | "dynamic,bounded"),
                "unbounded frame: {line}"
            );
            let bytes = fields[1].parse::<u64>().unwrap();
            assert!(
                bytes <= bound,
                "{name}: native {bytes} exceeds source bound {bound}"
            );
            let inventory = if probe.frames.contains_key(name) {
                &mut actual
            } else {
                &mut derivatives
            };
            assert!(
                inventory.insert(name.to_owned(), bytes).is_none(),
                "duplicate native frame: {name}"
            );
        }
    }
    assert_eq!(
        actual.keys().collect::<Vec<_>>(),
        probe.frames.keys().collect::<Vec<_>>(),
        "missing generated frame report"
    );
    fn path(
        node: usize,
        graph: &[Vec<usize>],
        probe: &Probe,
        frames: &BTreeMap<String, u64>,
    ) -> u64 {
        frames[&probe.names[node]]
            .checked_add(
                graph[node]
                    .iter()
                    .map(|next| path(*next, graph, probe, frames))
                    .max()
                    .unwrap_or(0),
            )
            .unwrap()
    }
    let worst = (0..graph.len())
        .map(|node| path(node, graph, probe, &actual))
        .max()
        .unwrap();
    // Compiler clones/split fragments may participate in the executed path.
    // Charge ALL reported derivatives in addition to the source-graph path,
    // including derivatives of non-live branches. Never treat a clone as zero.
    let worst = worst.checked_add(derivative_total(&derivatives)).unwrap();
    assert!(worst <= probe.bound);
    eprintln!(
        "C call path {}: native {worst}, source {}",
        directory.display(),
        probe.bound
    );
    let output = Command::new("nm")
        .arg("--defined-only")
        .arg(object)
        .output()
        .unwrap();
    assert!(output.status.success());
    let output = String::from_utf8(output.stdout).unwrap();
    let symbols: BTreeMap<_, _> = output
        .lines()
        .filter_map(|line| {
            let fields: Vec<_> = line.split_whitespace().collect();
            (fields.len() == 3).then(|| (fields[2], fields[1]))
        })
        .collect();
    for (index, name) in probe.names.iter().enumerate() {
        assert_eq!(
            symbols.get(name.as_str()).copied(),
            Some(if index == 0 { "T" } else { "t" }),
            "wrong native linkage for {name}"
        );
    }
}

pub(super) fn frame_bound(
    name: &str,
    frames: &BTreeMap<String, u64>,
) -> Result<Option<u64>, String> {
    if let Some(bound) = frames.get(name) {
        return Ok(Some(*bound));
    }
    for (original, bound) in frames {
        if let Some(suffix) = name.strip_prefix(original.as_str())
            && [".constprop", ".isra", ".cold"].iter().any(|prefix| {
                suffix.strip_prefix(prefix).is_some_and(|rest| {
                    rest.is_empty()
                        || rest.strip_prefix('.').is_some_and(|digits| {
                            !digits.is_empty() && digits.bytes().all(|byte| byte.is_ascii_digit())
                        })
                })
            })
        {
            return Ok(Some(*bound));
        }
    }
    // Only the test consumer and pinned ASan process-startup/teardown hooks
    // sit outside the generated graph. Unknown clones, splits and thunks fail.
    if matches!(
        name,
        "main" | "_sub_I_00099_0" | "_sub_I_00099_1" | "_sub_D_00099_0" | "_sub_D_00099_1"
    ) {
        Ok(None)
    } else {
        Err(format!("unaccounted native function frame {name}"))
    }
}

fn derivative_total(derivatives: &BTreeMap<String, u64>) -> u64 {
    derivatives
        .values()
        .try_fold(0u64, |sum, frame| sum.checked_add(*frame))
        .unwrap()
}

#[test]
fn compiler_created_frames_cannot_silently_escape_the_native_oracle() {
    let frames = BTreeMap::from([("poly_call0".into(), 42)]);
    assert_eq!(frame_bound("poly_call0", &frames), Ok(Some(42)));
    assert_eq!(frame_bound("main", &frames), Ok(None));
    assert_eq!(frame_bound("_sub_I_00099_1", &frames), Ok(None));
    for name in [
        "poly_call0.constprop",
        "poly_call0.constprop.0",
        "poly_call0.isra.0",
        "poly_call0.cold",
    ] {
        assert_eq!(frame_bound(name, &frames), Ok(Some(42)));
    }
    let derivatives = BTreeMap::from([
        ("poly_call0.constprop".into(), 8),
        ("poly_call0.cold".into(), 16),
    ]);
    assert_eq!(derivative_total(&derivatives), 24);
    for name in [
        "poly_call0.constprop.evil",
        "poly_call0.coldish",
        "poly_call0.thunk",
        "unrelated_compiler_helper",
    ] {
        assert!(frame_bound(name, &frames).is_err(), "{name}");
    }
}
