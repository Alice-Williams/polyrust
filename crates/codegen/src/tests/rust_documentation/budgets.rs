use super::*;

fn exact_limits() -> Limits {
    Limits {
        declarations: 1,
        ancestry_nodes: 2,
        ancestry_depth: 1,
        export_modules: 1,
        export_bindings: 1,
        attributes: 3,
        // Declaration path, canonical root path, three docs and one alias name.
        text_bytes: 6 + 6 + 5 + 5 + 6 + 5,
    }
}

fn budget_fixture() -> RustSourceOrigin {
    let mut origin = fixture();
    Arc::make_mut(&mut origin.crate_exports)
        .modules
        .get_mut(&id(1))
        .unwrap()
        .insert(name("cycle"), RustExportTarget::Module(id(1)));
    origin
}

#[test]
fn exact_and_one_over_every_input_budget() {
    let origin = budget_fixture();
    let exact = exact_limits();
    check([&origin], exact).unwrap();
    let mut cases = Vec::new();
    let mut limit = exact;
    limit.declarations -= 1;
    cases.push((limit, "declarations"));
    let mut limit = exact;
    limit.ancestry_nodes -= 1;
    cases.push((limit, "ancestry nodes"));
    let mut limit = exact;
    limit.ancestry_depth -= 1;
    cases.push((limit, "ancestry depth"));
    let mut limit = exact;
    limit.export_modules -= 1;
    cases.push((limit, "export modules"));
    let mut limit = exact;
    limit.export_bindings -= 1;
    cases.push((limit, "export bindings"));
    let mut limit = exact;
    limit.attributes -= 1;
    cases.push((limit, "attributes"));
    let mut limit = exact;
    limit.text_bytes -= 1;
    cases.push((limit, "metadata text"));
    for (limit, expected) in cases {
        assert_eq!(
            check([&origin], limit).unwrap_err(),
            RustDocumentationError::Budget(expected)
        );
    }
}

#[test]
fn shared_payloads_are_scanned_once_and_distinct_copies_are_charged() {
    let first = budget_fixture();
    let mut second = first.clone();
    second.declaration = id(3);
    let mut limits = exact_limits();
    limits.declarations = 2;
    limits.ancestry_nodes = 3;
    limits.attributes += 2;
    limits.text_bytes += 6 + 5 + 6;
    check([&first, &second], limits).unwrap();
    let mut independent = budget_fixture();
    independent.declaration = id(3);
    assert_eq!(
        check([&first, &independent], limits).unwrap_err(),
        RustDocumentationError::Budget("export modules")
    );
    limits.export_modules = 2;
    limits.export_bindings = 2;
    limits.ancestry_nodes = 4;
    limits.attributes += 2; // The distinct root copy is visited in both ancestries.
    limits.text_bytes += 5 + 2 * (6 + 5);
    check([&first, &independent], limits).unwrap();
}

#[test]
fn production_policy_is_explicit_and_fixed() {
    let limits = Limits::PRODUCTION;
    assert_eq!(limits.declarations, 100_000);
    assert_eq!(limits.ancestry_nodes, 100_000);
    assert_eq!(limits.ancestry_depth, 128);
    assert_eq!(limits.export_modules, 100_000);
    assert_eq!(limits.export_bindings, 100_000);
    assert_eq!(limits.attributes, 100_000);
    assert_eq!(limits.text_bytes, 16 * 1024 * 1024);
}

#[test]
fn declaration_budget_stops_a_lazy_inspection_before_the_panic_tail() {
    let first = fixture();
    let mut second = first.clone();
    second.declaration = id(3);
    let calls = std::cell::Cell::new(0);
    let origins = [&first, &second]
        .into_iter()
        .chain(std::iter::from_fn(|| {
            panic!("read beyond declaration cutoff")
        }))
        .inspect(|_| calls.set(calls.get() + 1));
    let limits = Limits {
        declarations: 1,
        ..Limits::PRODUCTION
    };
    assert_eq!(
        check(origins, limits).unwrap_err(),
        RustDocumentationError::Budget("declarations")
    );
    assert_eq!(calls.get(), limits.declarations + 1);
}
