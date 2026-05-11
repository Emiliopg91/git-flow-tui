use std::{
    collections::HashMap,
    env,
    io::stdout,
    path::Path,
    process::exit,
    sync::mpsc::{self, Sender},
    thread,
};

use clap_complete::{aot::Bash, generate};
use color_eyre::eyre::Result;

use clap::{CommandFactory, Parser, Subcommand, ValueEnum};

use crate::{
    git::GitWrapper,
    logic::{
        bugfix::{bugfix_finish, bugfix_start},
        errors::PipelineError,
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
mod shell;
mod ui;

#[derive(Clone, Debug, ValueEnum, PartialEq)]
enum Action {
    Start,
    Finish,
}

#[derive(Debug, Subcommand)]
enum Commands {
    Feature {
        name: String,

        #[arg(value_enum)]
        action: Action,
    },

    Release {
        name: String,

        #[arg(value_enum)]
        action: Action,
    },

    Hotfix {
        name: String,

        #[arg(value_enum)]
        action: Action,
    },

    Bugfix {
        name: String,

        #[arg(value_enum)]
        action: Action,
    },
}

#[derive(Parser, Debug)]
struct CliArguments {
    #[command(subcommand)]
    command: Commands,
}

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
    let args = env::args().collect::<Vec<String>>();
    let exe_name = Path::new(&args[0])
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("");

    let is_git_flow = exe_name.ends_with("git-flow-tui");

    if is_git_flow {
        initialize_and_validate()?;
        WHITEBOARD.get_or_init(|| HashMap::new().into());

        main_loop()
    } else {
        if env::var("GFT_GEN_COMPLETION").is_ok() {
            let mut cmd = CliArguments::command();
            generate(Bash, &mut cmd, "git-flow", &mut stdout());
        } else {
            let cli: CliArguments = CliArguments::parse();

            match cli.command {
                command => {
                    type StepFn = fn(&str, Sender<String>) -> Result<(), PipelineError>;

                    let exec: Option<(String, StepFn)> = match command {
                        Commands::Feature { name, action } => {
                            if action == Action::Start {
                                Some((name, feature_start))
                            } else {
                                Some((name, feature_finish))
                            }
                        }
                        Commands::Release { name, action } => {
                            if action == Action::Start {
                                Some((name, release_start))
                            } else {
                                Some((name, release_finish))
                            }
                        }
                        Commands::Bugfix { name, action } => {
                            if action == Action::Start {
                                Some((name, bugfix_start))
                            } else {
                                Some((name, bugfix_finish))
                            }
                        }
                        Commands::Hotfix { name, action } => {
                            if action == Action::Start {
                                Some((name, hotfix_start))
                            } else {
                                Some((name, hotfix_finish))
                            }
                        }
                    };

                    if let Some(fnc) = exec {
                        initialize_and_validate()?;

                        let (tx, rx) = mpsc::channel::<String>();
                        let worker = thread::spawn(move || {
                            if let Err(e) = fnc.1(&fnc.0, tx.clone()) {
                                tx.send(format!("{}", e)).unwrap();
                                exit(1);
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
                }
            }
        }
        Ok(())

        /*
        match cli.command {
            Commands::Completion => {}
            Commands::Branch { kind, name, action } => {
                type StepFn = fn(&str, Sender<String>) -> Result<(), GitError>;
                let function: Option<StepFn> = match (kind, action) {
                    (BranchKind::Feature, Action::Start) => Some(feature_start),
                    (BranchKind::Feature, Action::Finish) => Some(feature_finish),
                    (BranchKind::Release, Action::Start) => Some(release_start),
                    (BranchKind::Release, Action::Finish) => Some(release_finish),
                    (BranchKind::Hotfix, Action::Start) => Some(hotfix_start),
                    (BranchKind::Hotfix, Action::Finish) => Some(hotfix_finish),
                    (BranchKind::Bugfix, Action::Start) => Some(bugfix_start),
                    (BranchKind::Bugfix, Action::Finish) => Some(bugfix_finish),
                };

                if let Some(fnc) = function {
                    initialize_and_validate()?;

                    let (tx, rx) = mpsc::channel::<String>();
                    let worker = thread::spawn(move || {
                        if let Err(e) = fnc(&name, tx.clone()) {
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
            }
        } */
    }
}
