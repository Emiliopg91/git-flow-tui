use std::{
    env,
    error::Error,
    io::stdout,
    process::exit,
    sync::mpsc::{self, Sender},
    thread,
};

use clap_complete::{aot::Bash, generate};

use clap::{CommandFactory, Parser, Subcommand, ValueEnum};
use git_flow_tui_core::{
    git::GitWrapper,
    logic::{
        bugfix::{bugfix_finish, bugfix_start},
        errors::PipelineError,
        feature::{feature_finish, feature_start},
        hotfix::{hotfix_finish, hotfix_start},
        release::{release_finish, release_start},
    },
    others::exit_code::ExitCode,
};

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

fn initialize_and_validate() -> Result<(), Box<dyn Error>> {
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

fn main() -> Result<(), Box<dyn Error>> {
    if env::var("GFT_GEN_COMPLETION").is_ok() {
        let mut cmd = CliArguments::command();
        generate(Bash, &mut cmd, "git-flow", &mut stdout());
    } else {
        let cli: CliArguments = CliArguments::parse();

        let command = cli.command;
        {
            type StepFn = fn(&str, Sender<String>) -> Result<(), PipelineError>;

            let exec: Option<(String, StepFn)> = match command {
                Commands::Feature { name, action } => Some((
                    name,
                    match action {
                        Action::Start => feature_start,
                        Action::Finish => feature_finish,
                    },
                )),

                Commands::Release { name, action } => Some((
                    name,
                    match action {
                        Action::Start => release_start,
                        Action::Finish => release_finish,
                    },
                )),

                Commands::Bugfix { name, action } => Some((
                    name,
                    match action {
                        Action::Start => bugfix_start,
                        Action::Finish => bugfix_finish,
                    },
                )),

                Commands::Hotfix { name, action } => Some((
                    name,
                    match action {
                        Action::Start => hotfix_start,
                        Action::Finish => hotfix_finish,
                    },
                )),
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
    Ok(())
}
