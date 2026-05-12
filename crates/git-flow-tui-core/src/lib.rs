use std::{error::Error, process::exit};

use crate::{git::GitWrapper, others::exit_code::ExitCode};

pub mod git;
pub mod logic;
pub mod others;
pub mod shell;

pub fn initialize_and_validate() -> Result<(), Box<dyn Error>> {
    GitWrapper::initialize().unwrap_or_else(|e| {
        eprintln!("Error: {}", e);
        exit(ExitCode::NotAGitRepository.code());
    });

    let git = GitWrapper::global().lock().unwrap();
    let has_changes = git.has_changes().unwrap();
    drop(git);

    if has_changes {
        eprintln!("Found uncommited changes on repository:");
        eprintln!("Fix it and try again.");
        exit(ExitCode::UncommitedChanges.code());
    }

    Ok(())
}
