use std::{
    collections::HashMap,
    env,
    process::exit,
    sync::mpsc::{self, Sender},
    thread,
};

use color_eyre::eyre::Result;

use clap::{Parser, ValueEnum};

use crate::{
    git::{GitWrapper, errors::GitError},
    logic::{
        feature::{feature_finish, feature_start},
        hotfix::{hotfix_finish, hotfix_start},
        release::{release_finish, release_start},
    },
    others::{exit_code::ExitCode, whiteboard::WHITEBOARD},
    ui::main_loop,
};

mod git;
mod logic;
mod others;
mod ui;

#[derive(Clone, Debug, ValueEnum)]
enum BranchKind {
    Feature,
    Release,
    Bugfix,
    Hotfix,
}

#[derive(Clone, Debug, ValueEnum)]
enum Action {
    Start,
    Finish,
}

#[derive(Parser, Debug)]
struct CliArguments {
    #[arg(value_enum)]
    pub kind: BranchKind,

    pub name: String,

    #[arg(value_enum)]
    pub action: Action,
}

fn initialize_and_validate() -> Result<()> {
    let repo_path = env::current_dir()?;
    GitWrapper::initialize(repo_path).unwrap_or_else(|e| {
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
    let args = env::args().collect::<Vec<String>>();
    if args.len() > 1 {
        let cli = CliArguments::parse();
        initialize_and_validate()?;

        let function: Option<fn(&str, Sender<String>) -> Result<(), GitError>> =
            match (cli.kind, cli.action) {
                (BranchKind::Feature, Action::Start) => Some(feature_start),
                (BranchKind::Feature, Action::Finish) => Some(feature_finish),
                (BranchKind::Release, Action::Start) => Some(release_start),
                (BranchKind::Release, Action::Finish) => Some(release_finish),
                (BranchKind::Hotfix, Action::Start) => Some(hotfix_start),
                (BranchKind::Hotfix, Action::Finish) => Some(hotfix_finish),
                (_, _) => None,
            };

        if let Some(fnc) = function {
            let (tx, rx) = mpsc::channel::<String>();
            let worker = thread::spawn(move || {
                if let Err(e) = fnc(&cli.name, tx.clone()) {
                    tx.send(format!("{}", e)).unwrap()
                }
            });

            loop {
                while let Ok(msg) = rx.try_recv() {
                    println!("{}", msg);
                }
                if worker.is_finished() {
                    break;
                }
            }
        }

        Ok(())
    } else {
        initialize_and_validate()?;
        WHITEBOARD.get_or_init(|| HashMap::new().into());

        main_loop()
    }
}
