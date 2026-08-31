#![forbid(unsafe_code)]
fn main() {
    let mode = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "--all-targets".into());
    let result = match mode.as_str() {
        "--all-targets" => portable_conformance::run_all(),
        "--determinism" => portable_conformance::verify_determinism(
            &portable_conformance::checked_fixture(),
        )
        .map(|()| "four target manifests are byte-identical across repeated generation".into()),
        _ => {
            eprintln!("usage: polyrust-conformance [--all-targets|--determinism]");
            std::process::exit(2)
        }
    };
    match result {
        Ok(summary) => println!("{summary}"),
        Err(error) => {
            eprintln!("{error}");
            std::process::exit(1)
        }
    }
}
