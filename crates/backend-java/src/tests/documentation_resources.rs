use super::*;
use crate::ast::JavaDocumentationStyle;
use crate::tests::documentation_fixture as fixture;

#[test]
fn exact_and_one_over_package_presentation_budgets() {
    let first = fixture::docs(
        1,
        JavaDocumentationStyle::Declaration,
        &["first\nsecond", "*"],
    );
    let second = fixture::docs(0, JavaDocumentationStyle::ExplanatoryModule, &["module"]);
    let bytes = first
        .iter()
        .chain(second.iter())
        .map(|(_, attachment)| attachment.presentation_len())
        .sum();
    check([&first, &second], Limits { owners: 2, bytes }).unwrap();
    assert_eq!(
        check([&first, &second], Limits { owners: 1, bytes }).unwrap_err(),
        "Java documentation attachment limit exceeded"
    );
    assert_eq!(
        check(
            [&first, &second],
            Limits {
                owners: 2,
                bytes: bytes - 1
            }
        )
        .unwrap_err(),
        "Java documentation presentation byte limit exceeded"
    );
    check(
        [&JavaDocumentation::default()],
        Limits {
            owners: 0,
            bytes: 0,
        },
    )
    .unwrap();
}

#[test]
fn overflowing_presentation_size_rejects_without_rendering_or_allocation() {
    let docs = fixture::docs(usize::MAX, JavaDocumentationStyle::Declaration, &[""]);
    assert_eq!(
        check([&docs], LIMITS).unwrap_err(),
        "Java documentation presentation byte limit exceeded"
    );
    assert_eq!(LIMITS.owners, 100_000);
    assert_eq!(LIMITS.bytes, 64 * 1024 * 1024);
}
