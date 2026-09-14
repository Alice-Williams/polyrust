//! Native exports, imports and all reported frames must match exact packages.
use super::*;

pub(super) fn symbols(object: &Path, package: &Package) {
    for (flag, expected) in [
        (
            "--defined-only",
            package.exports.iter().cloned().collect::<BTreeSet<_>>(),
        ),
        ("--undefined-only", package.imports.clone()),
    ] {
        let output = Command::new("nm")
            .args([flag, "--extern-only"])
            .arg(object)
            .output()
            .unwrap();
        assert!(output.status.success());
        let text = String::from_utf8(output.stdout).unwrap();
        let symbols: BTreeSet<_> = text
            .lines()
            .filter_map(|line| line.split_whitespace().last())
            .filter(|name| name.starts_with("poly_"))
            .map(str::to_owned)
            .collect();
        assert_eq!(symbols, expected, "{object:?}: {text}");
    }
}

pub(super) fn frames(directory: &Path, packages: &[Package]) {
    let expected: BTreeMap<_, _> = packages
        .iter()
        .flat_map(|package| package.frames.clone())
        .collect();
    let mut actual = BTreeMap::new();
    let mut derivatives = 0u64;
    for entry in fs::read_dir(directory).unwrap() {
        let path = entry.unwrap().path();
        if path.extension().is_none_or(|extension| extension != "su") {
            continue;
        }
        for line in fs::read_to_string(path).unwrap().lines() {
            let fields: Vec<_> = line.split('\t').collect();
            let name = fields[0].rsplit(':').next().unwrap();
            let Some(bound) =
                crate::dialect::shared::call_native_reports::frame_bound(name, &expected).unwrap()
            else {
                continue;
            };
            assert_eq!(fields.len(), 3, "{line}");
            assert!(matches!(fields[2], "static" | "dynamic,bounded"), "{line}");
            let bytes: u64 = fields[1].parse().unwrap();
            assert!(bytes <= bound, "{name}: native {bytes} exceeds {bound}");
            if expected.contains_key(name) {
                assert!(actual.insert(name.to_owned(), bytes).is_none());
            } else {
                derivatives = derivatives.checked_add(bytes).unwrap();
            }
        }
    }
    assert_eq!(
        actual.keys().collect::<Vec<_>>(),
        expected.keys().collect::<Vec<_>>()
    );
    let worst = (0..2)
        .map(|scalar| {
            packages
                .iter()
                .map(|package| actual[&package.exports[scalar]])
                .sum::<u64>()
        })
        .max()
        .unwrap()
        .checked_add(derivatives)
        .unwrap();
    assert!(worst <= packages[2].bound);
    eprintln!(
        "C separate objects {}: native path {worst}, certified bound {}",
        directory.display(),
        packages[2].bound
    );
}
