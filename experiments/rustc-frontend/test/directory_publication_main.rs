#![forbid(unsafe_code)]
#[path = "../src/output/publication.rs"]
mod publication;

fn main() -> std::process::ExitCode {
    let args: Vec<_> = std::env::args().collect();
    let mode = &args[1];
    let count = if mode == "single" { 3 } else { 4 };
    let mut files: Vec<_> = (0..count)
        .map(|index| (format!("file{index}"), format!("contents{index}")))
        .collect();
    match mode.as_str() {
        "duplicate" => files[1].0 = files[0].0.clone(),
        "nested" => files[1].0 = "../escape".into(),
        "write-failure" => files[1].0 = "x".repeat(300),
        _ => {}
    }
    let result = if mode == "single" || mode == "wrong-single-count" {
        publication::new_directory(std::path::Path::new(&args[2]), &files)
    } else {
        let members = match mode.as_str() {
            "zero-members" => 0,
            "too-many-members" => 1025,
            _ => 1,
        };
        publication::bundle_directory(std::path::Path::new(&args[2]), &files, members)
    };
    match result {
        Ok(()) => std::process::ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("{error}");
            std::process::ExitCode::FAILURE
        }
    }
}
