pub mod errors;

use std::{
    process::{Command, exit},
    sync::{Mutex, OnceLock},
};

use crate::{git::errors::GitError, others::exit_code::ExitCode};

pub struct GitWrapper {
    pub path: String,
}

pub struct GitCmdResult {
    pub status: i32,
    pub stdout: String,
    pub stderr: String,
}

static GIT_WRAPPER_INST: OnceLock<Mutex<GitWrapper>> = OnceLock::new();

impl GitWrapper {
    pub fn global() -> &'static Mutex<GitWrapper> {
        GIT_WRAPPER_INST.get_or_init(|| {
            Mutex::new(Self::initialize().unwrap_or_else(|e| {
                println!("{}", e);
                exit(ExitCode::NotAGitRepository.code())
            }))
        })
    }

    pub fn initialize() -> Result<Self, GitError> {
        let res = Self::run_git_command(["rev-parse", "--show-toplevel"], false).unwrap();
        if res.status != 0 {
            return Err(GitError::NotAGitRepository);
        }

        Ok(Self { path: res.stdout })
    }

    fn run_git_command<I, S>(args: I, check: bool) -> Result<GitCmdResult, std::io::Error>
    where
        I: IntoIterator<Item = S>,
        S: AsRef<str>,
    {
        let args_vec: Vec<String> = args.into_iter().map(|s| s.as_ref().to_string()).collect();

        let res = Command::new("git")
            .args(&args_vec)
            .env("LANG", "C")
            .output()?;

        let status = res.status.code().ok_or_else(|| {
            std::io::Error::new(
                std::io::ErrorKind::Other,
                format!(
                    "Command 'git {}' terminated without exit code",
                    args_vec.join(" ")
                ),
            )
        })?;

        if check && status != 0 {
            return Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!(
                    "Command 'git {}' failed (exit code {}): {}",
                    args_vec.join(" "),
                    status,
                    String::from_utf8_lossy(&res.stderr).trim()
                ),
            ));
        }

        Ok(GitCmdResult {
            status,
            stdout: String::from_utf8_lossy(&res.stdout).trim().to_string(),
            stderr: String::from_utf8_lossy(&res.stderr).trim().to_string(),
        })
    }

    pub fn has_changes(&self) -> Result<bool, GitError> {
        Ok(Self::run_git_command(["status", "--porcelain"], true)
            .map_err(|e| GitError::from(e))
            .unwrap()
            .stdout
            .lines()
            .map(|s| s.to_string())
            .collect::<Vec<String>>()
            .len()
            > 0)
    }

    pub fn get_branch(&self) -> Result<String, GitError> {
        Ok(
            Self::run_git_command(["rev-parse", "--abbrev-ref", "HEAD"], true)
                .map_err(|e| GitError::from(e))
                .unwrap()
                .stdout,
        )
    }
}
