#![forbid(unsafe_code)]
fn main() {
    let arguments: Vec<String> = std::env::args().skip(1).collect();
    let arguments = if arguments.is_empty() {
        vec!["--all-targets".into()]
    } else {
        arguments
    };
    if arguments
        .iter()
        .any(|argument| !matches!(argument.as_str(), "--all-targets" | "--determinism"))
    {
        eprintln!("usage: polyrust-conformance [--all-targets] [--determinism]");
        std::process::exit(2)
    }
    let all_targets = arguments.iter().any(|argument| argument == "--all-targets");
    let determinism = arguments.iter().any(|argument| argument == "--determinism");
    let result: Result<String, Box<portable_conformance::Mismatch>> = (|| {
        let mut summaries = Vec::new();
        if all_targets {
            summaries.push(portable_conformance::run_all()?);
        }
        if determinism {
            portable_conformance::verify_determinism(&portable_conformance::checked_fixture())?;
            summaries.push(
                "seven target manifests are byte-identical across repeated generation".into(),
            );
        }
        Ok(summaries.join("; "))
    })();
    match result {
        Ok(summary) => println!("{summary}"),
        Err(error) => {
            eprintln!("{error}");
            std::process::exit(1)
        }
    }
}
