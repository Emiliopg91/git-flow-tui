use std::{collections::HashMap, env, process::exit};

use color_eyre::eyre::Result;

use crate::{
    git::GitWrapper,
    others::{exit_code::ExitCode, whiteboard::WHITEBOARD},
    ui::main_loop,
};

mod git;
mod logic;
mod others;
mod ui;

fn main() -> Result<()> {
    let repo_path = env::current_dir()?;
    GitWrapper::initialize(repo_path).unwrap_or_else(|e| {
        eprintln!("Error: {}", e);
        exit(ExitCode::NotAGitRepository.code());
    });

    let git = GitWrapper::global().lock().unwrap();
    let has_changes = git.has_changes().unwrap();
    let main_branch = git.main_branch.clone();
    drop(git);

    println!("Primary branch detected: {}", main_branch);

    if has_changes {
        eprintln!("Found uncommited changes on repository:");
        eprintln!("Fix it and try again.");
        exit(ExitCode::UncommitedChanges.code());
    }

    WHITEBOARD.get_or_init(|| HashMap::new().into());

    main_loop()
}
