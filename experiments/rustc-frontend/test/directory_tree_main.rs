#![forbid(unsafe_code)]
use portable_directory_publication::{PathPolicy, TreeLimits, publish};

fn main() -> std::process::ExitCode {
    let args: Vec<_> = std::env::args().collect();
    let mode = args[1].as_str();
    let token = args.get(3).map(String::as_str).unwrap_or("one");
    let mut files: Vec<_> = ["a/b/one", "a/two", "c/three"]
        .into_iter()
        .map(|name| (name.to_owned(), token.to_owned()))
        .collect();
    let mut limits = TreeLimits {
        files: 3,
        directories: 3,
        depth: 2,
        path_bytes: 7,
        bytes: 9,
    };
    match mode {
        "files" => limits.files -= 1,
        "directories" => limits.directories -= 1,
        "depth" => limits.depth -= 1,
        "path-bytes" => limits.path_bytes -= 1,
        "bytes" => limits.bytes -= 1,
        "duplicate" => files[1].0 = files[0].0.clone(),
        "prefix" => files[1].0 = "a".into(),
        "parent" => files[1].0 = "../out".into(),
        "absolute" => files[1].0 = "/out".into(),
        "dot" => files[1].0 = "a/./x".into(),
        "empty-component" => files[1].0 = "a//x".into(),
        "directory-failure" => {
            files[0].0 = format!("a/{}/one", "x".repeat(300));
            limits.path_bytes = 4096;
            limits.directories = 10;
        }
        "file-failure" => {
            files[1].0 = format!("a/{}", "x".repeat(300));
            limits.path_bytes = 4096;
        }
        "tree" => {}
        _ => panic!("unknown test mode"),
    }
    match publish(
        std::path::Path::new(&args[2]),
        &files,
        PathPolicy::RelativeTree(limits),
    ) {
        Ok(()) => std::process::ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("{error}");
            std::process::ExitCode::FAILURE
        }
    }
}
