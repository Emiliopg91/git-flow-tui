use std::process::exit;

use color_eyre::eyre::Result;

use crate::{git::GitWrapper, others::exit_code::ExitCode, ui::main_loop};

mod git;
mod others;
mod ui;

fn main() -> Result<()> {
    let has_changes = GitWrapper::global().lock().unwrap().has_changes().unwrap();
    if has_changes {
        eprintln!("Found uncommited changes on repository.");
        eprintln!("Fix it and try again.");
        exit(ExitCode::UncommitedChanges.code());
    }

    main_loop()
}
