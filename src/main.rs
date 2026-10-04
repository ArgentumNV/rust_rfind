use std::fs;
use std::process::ExitCode;

use clap::Parser;
use std::path::{Path, PathBuf};

#[derive(Parser)]
#[command(version, about = "Find files by part of their name")]
struct Args {
    /// Directory to search recursively
    path: PathBuf,
    /// File name or substring to search for
    file_to_find: String,
    /// Match the full file name exactly instead of searching for a substring
    #[arg(short = 'e', long = "exact")]
    exact: bool,
}

/// Checks that the search path in the parsed arguments is a directory.
///
/// Argument parsing is handled by clap. The search string is not validated.
///
/// # Errors
///
/// Returns an error if the path is not a directory or its metadata cannot be
/// read. This check does not guarantee that the directory contents can be read.
fn validate_args(args: &Args) -> Result<(), String> {
    if !args.path.is_dir() {
        return Err(format!(
            "Error: '{}' is not a directory",
            args.path.display()
        ));
    }

    Ok(())
}

/// Recursively searches `path` and prints matching regular file paths to stdout.
///
/// Matches the full file name when `args.exact` is set; otherwise, searches for
/// `args.file_to_find` as a substring. Both modes are case-sensitive. Substring
/// matching replaces invalid UTF-8 sequences in file names with replacement
/// characters. Symbolic links are neither matched nor followed.
///
/// Read errors are printed to stderr. Unreadable directories and entries are
/// skipped, and the search continues where possible. No result is returned.
fn walk_dir(path: &Path, args: &Args) {
    let entries = match fs::read_dir(path) {
        Ok(entries) => entries,
        Err(err) => {
            eprintln!("Error reading '{}': {}", path.display(), err);
            return;
        }
    };

    for entry in entries {
        let entry = match entry {
            Ok(entry) => entry,
            // Skip entries that could not be read.
            Err(err) => {
                eprintln!("Error reading an entry in '{}': {}", path.display(), err);
                continue;
            }
        };

        match entry.file_type() {
            Ok(file_type) => {
                if file_type.is_file() {
                    let name = entry.file_name();

                    if args.exact {
                        if name == std::ffi::OsStr::new(&args.file_to_find) {
                            println!("{}", entry.path().display());
                        }
                    } else {
                        if name.to_string_lossy().contains(&args.file_to_find) {
                            println!("{}", entry.path().display());
                        }
                    }
                } else if file_type.is_dir() {
                    walk_dir(&entry.path(), args)
                }
            }
            Err(err) => {
                eprintln!("Error reading '{}': {}", entry.path().display(), err);
            }
        }
    }
}

/// Parses arguments, validates the search directory, and starts the search.
///
/// Returns failure if validation fails, otherwise success even if read errors
/// occur during traversal. clap handles help, version, and parsing errors.
fn main() -> ExitCode {
    let args = Args::parse();

    match validate_args(&args) {
        Ok(()) => {}
        Err(err) => {
            eprintln!("{}", err);
            return ExitCode::FAILURE;
        }
    }
    walk_dir(&args.path, &args);

    ExitCode::SUCCESS
}
