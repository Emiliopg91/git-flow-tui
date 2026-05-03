use std::{env, path::PathBuf, process::exit};

use color_eyre::eyre::Result;

use crate::{git::GitWrapper, others::exit_code::ExitCode, ui::main_loop};
use clap::Parser;

mod git;
mod others;
mod ui;

#[derive(Parser, Debug)]
#[command(author, version, about)]
struct Args {
    #[arg(short, long)]
    repository: Option<PathBuf>,
}

fn main() -> Result<()> {
    let args = Args::parse();

    let cwd = env::current_dir()?;

    let repo_path = match args.repository {
        Some(repo) => {
            if repo.is_absolute() {
                repo
            } else {
                cwd.join(repo)
            }
        }
        None => cwd,
    };
    println!("Using repository {:?}", repo_path);
    GitWrapper::initialize(repo_path).unwrap_or_else(|e| {
        eprintln!("Error: {}", e);
        exit(ExitCode::NotAGitRepository.code());
    });

    let has_changes = GitWrapper::global().lock().unwrap().has_changes().unwrap();
    if has_changes {
        eprintln!("Found uncommited changes on repository.");
        eprintln!("Fix it and try again.");
        exit(ExitCode::UncommitedChanges.code());
    }

    main_loop()
}
