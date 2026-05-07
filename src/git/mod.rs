pub mod errors;

use std::{
    fs,
    path::PathBuf,
    process::Command,
    sync::{Mutex, OnceLock},
};

use crate::git::errors::GitError;

pub struct GitWrapper {
    pub main_branch: String,
}

pub struct GitCmdResult {
    pub status: i32,
    pub stdout: String,
    #[allow(dead_code)]
    pub stderr: String,
}

static GIT_WRAPPER_INST: OnceLock<Mutex<GitWrapper>> = OnceLock::new();

impl GitWrapper {
    pub fn global() -> &'static Mutex<GitWrapper> {
        GIT_WRAPPER_INST.get().unwrap()
    }

    pub fn initialize(path_buf: PathBuf) -> Result<(), GitError> {
        if !fs::exists(&path_buf)? {
            return Err(GitError::NotAGitRepository);
        }

        let res = Self::run_git_command(["symbolic-ref", "refs/remotes/origin/HEAD"], false)
            .unwrap_or(GitCmdResult {
                status: 0,
                stdout: "main".to_string(),
                stderr: "".to_string(),
            });

        let inst = Self {
            main_branch: res.stdout.rsplit('/').next().unwrap_or("main").to_string(),
        };

        GIT_WRAPPER_INST.get_or_init(|| Mutex::new(inst));

        Ok(())
    }

    fn run_git_command<I, S>(args: I, check: bool) -> Result<GitCmdResult, GitError>
    where
        I: IntoIterator<Item = S>,
        S: AsRef<str>,
    {
        let args_vec: Vec<String> = args.into_iter().map(|s| s.as_ref().to_string()).collect();

        let res = Command::new("git")
            .args(&args_vec)
            .env("LANG", "C")
            .output()?;

        let status = res.status.code().ok_or_else(|| GitError::CommandFailed {
            command: format!("git {}", args_vec.join(" ")),
            code: -1,
        })?;

        if check && status != 0 {
            return Err(GitError::CommandFailed {
                command: format!("git {}", args_vec.join(" ")),
                code: status,
            });
        }

        Ok(GitCmdResult {
            status,
            stdout: String::from_utf8_lossy(&res.stdout).trim().to_string(),
            stderr: String::from_utf8_lossy(&res.stderr).trim().to_string(),
        })
    }

    pub fn has_changes(&self) -> Result<bool, GitError> {
        Ok(!Self::run_git_command(["status", "--porcelain"], true)?
            .stdout
            .lines()
            .map(|s| s.to_string())
            .collect::<Vec<String>>()
            .is_empty())
    }

    pub fn get_branch(&self) -> Result<String, GitError> {
        Ok(Self::run_git_command(["rev-parse", "--abbrev-ref", "HEAD"], true)?.stdout)
    }

    pub fn checkout(&self, branch: &str) -> Result<(), GitError> {
        let res = Self::run_git_command(["checkout", branch], false)?;
        if res.status != 0 {
            return Err(GitError::CheckoutFailed {
                branch: branch.to_owned(),
            });
        }

        Ok(())
    }

    pub fn merge(&self, branch: &str) -> Result<(), GitError> {
        let res = Self::run_git_command(["merge", branch], false)?;
        if res.status != 0 {
            return Err(GitError::MergeFailed {
                branch: branch.to_owned(),
            });
        }

        Ok(())
    }

    pub fn pull(&self) -> Result<(), GitError> {
        let res = Self::run_git_command(["pull"], false)?;
        if res.status != 0 {
            return Err(GitError::PullFailed {
                branch: self.get_branch()?,
            });
        }

        Ok(())
    }

    pub fn push(&self) -> Result<(), GitError> {
        let cur_branch = self.get_branch()?;
        let mut args = ["push".to_string()].to_vec();

        if !self.get_remote_branches()?.contains(&cur_branch) {
            args.push("--set-upstream".to_string());
            args.push("origin".to_string());
            args.push(self.get_branch()?);
        }

        let res = Self::run_git_command(&args, false)?;
        if res.status != 0 {
            return Err(GitError::PushFailed {
                branch: self.get_branch()?,
            });
        }

        Ok(())
    }

    pub fn push_tags(&self) -> Result<(), GitError> {
        let res = Self::run_git_command(["push".to_string(), "--tags".to_string()], false)?;
        if res.status != 0 {
            return Err(GitError::PushTagsFailed);
        }

        Ok(())
    }

    pub fn create_branch(&self, branch: &str) -> Result<(), GitError> {
        let res = Self::run_git_command(["checkout", "-b", branch], false)?;
        if res.status != 0 {
            return Err(GitError::BranchFailed {
                branch: self.get_branch()?,
            });
        }

        Ok(())
    }

    pub fn delete_branch(&self, branch: &str) -> Result<(), GitError> {
        let res = Self::run_git_command(["branch", "-D", branch], false)?;
        if res.status != 0 {
            return Err(GitError::BranchDeletionFailed {
                branch: self.get_branch()?,
            });
        }

        Ok(())
    }

    pub fn get_branches(&self) -> Result<Vec<String>, GitError> {
        let res = Self::run_git_command(["branch"], false)?;
        if res.status != 0 {
            return Err(GitError::ListBranchFailed);
        }

        Ok(res
            .stdout
            .lines()
            .map(|s| s.replace("* ", "").trim().to_string())
            .collect())
    }

    pub fn get_remote_branches(&self) -> Result<Vec<String>, GitError> {
        let res = Self::run_git_command(["branch", "-r"], false)?;
        if res.status != 0 {
            return Err(GitError::ListBranchFailed);
        }

        Ok(res
            .stdout
            .lines()
            .map(|s| {
                s.replace("* ", "")
                    .replace("origin/", "")
                    .trim()
                    .to_string()
            })
            .collect())
    }

    fn filter_local_branches(&self, branches: &[String], tpe: &String) -> Vec<String> {
        branches
            .iter()
            .filter_map(|s| {
                let cleaned = s.trim().trim_start_matches('*').trim();

                let (kind, name) = cleaned.split_once('/')?;

                if kind == tpe {
                    Some(name.to_string())
                } else {
                    None
                }
            })
            .collect()
    }

    pub fn get_releases(&self) -> Result<Vec<String>, GitError> {
        let branches = self.get_branches()?;
        Ok(self.filter_local_branches(&branches, &"release".to_string()))
    }

    pub fn get_features(&self) -> Result<Vec<String>, GitError> {
        let branches = self.get_branches()?;
        Ok(self.filter_local_branches(&branches, &"feature".to_string()))
    }

    pub fn get_bugfixes(&self) -> Result<Vec<String>, GitError> {
        let branches = self.get_branches()?;
        Ok(self.filter_local_branches(&branches, &"bugfix".to_string()))
    }

    pub fn get_hotfixes(&self) -> Result<Vec<String>, GitError> {
        let branches = self.get_branches()?;
        Ok(self.filter_local_branches(&branches, &"hotfix".to_string()))
    }

    fn filter_remote_branches(&self, branches: &[String], tpe: &String) -> Vec<String> {
        branches
            .iter()
            .filter_map(|s| {
                let cleaned = s.trim().trim_start_matches('*').trim();

                let mut parts = cleaned.splitn(3, '/');

                let _remote = parts.next()?;
                let kind = parts.next()?;
                let name = parts.next()?;

                if kind == tpe {
                    Some(name.to_string())
                } else {
                    None
                }
            })
            .collect()
    }

    pub fn get_remote_releases(&self) -> Result<Vec<String>, GitError> {
        let branches = self.get_remote_branches()?;
        Ok(self.filter_remote_branches(&branches, &"release".to_string()))
    }

    pub fn get_remote_features(&self) -> Result<Vec<String>, GitError> {
        let branches = self.get_remote_branches()?;
        Ok(self.filter_remote_branches(&branches, &"feature".to_string()))
    }

    pub fn get_remote_bugfixes(&self) -> Result<Vec<String>, GitError> {
        let branches = self.get_remote_branches()?;
        Ok(self.filter_remote_branches(&branches, &"bugfix".to_string()))
    }

    pub fn get_remote_hotfixes(&self) -> Result<Vec<String>, GitError> {
        let branches = self.get_remote_branches()?;
        Ok(self.filter_remote_branches(&branches, &"hotfix".to_string()))
    }

    pub fn commit(&self, msg: &str) -> Result<(), GitError> {
        let res = Self::run_git_command(["commit", "--allow-empty", "-m", msg], false)?;
        if res.status != 0 {
            return Err(GitError::CommitFailed {
                branch: self.get_branch()?,
            });
        }

        Ok(())
    }

    pub fn tag(&self, name: &str) -> Result<(), GitError> {
        let res = Self::run_git_command(["tag", name], false)?;
        if res.status != 0 {
            return Err(GitError::TagFailed {
                tag: name.to_owned(),
            });
        }

        Ok(())
    }

    pub fn fetch(&self, all: bool, prune: bool) -> Result<(), GitError> {
        let mut args = ["fetch".to_string()].to_vec();

        if all {
            args.push("--all".to_string());
        }

        if prune {
            args.push("--prune".to_string());
        }

        let res = Self::run_git_command(&args, false)?;
        if res.status != 0 {
            return Err(GitError::FetchFailed);
        }

        Ok(())
    }
}
