use std::{collections::HashMap, process::exit};

use color_eyre::eyre::Result;
use git_flow_tui_core::{
    git::GitWrapper,
    others::{exit_code::ExitCode, whiteboard::WHITEBOARD},
};

use crate::ui::main_loop;
mod ui;

fn initialize_and_validate() -> Result<()> {
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

fn main() -> Result<()> {
    initialize_and_validate()?;
    WHITEBOARD.get_or_init(|| HashMap::new().into());

    main_loop()
}
