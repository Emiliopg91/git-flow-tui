pub mod errors;

use std::{
    fs,
    path::PathBuf,
    process::Command,
    sync::{Mutex, OnceLock},
};

use crate::git::errors::GitError;

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
        GIT_WRAPPER_INST.get().unwrap()
    }

    pub fn initialize(path_buf: PathBuf) -> Result<(), GitError> {
        if !fs::exists(&path_buf).unwrap() {
            return Err(GitError::NotAGitRepository);
        }

        let path = path_buf.display().to_string();

        let res = Self::run_git_command(["rev-parse", "--show-toplevel"], &path, false).unwrap();
        if res.status != 0 {
            return Err(GitError::NotAGitRepository);
        }

        let inst = Self { path: res.stdout };

        GIT_WRAPPER_INST.get_or_init(|| Mutex::new(inst));

        Ok(())
    }

    fn run_git_command<I, S>(
        args: I,
        cwd: &String,
        check: bool,
    ) -> Result<GitCmdResult, std::io::Error>
    where
        I: IntoIterator<Item = S>,
        S: AsRef<str>,
    {
        let args_vec: Vec<String> = args.into_iter().map(|s| s.as_ref().to_string()).collect();

        let res = Command::new("git")
            .args(&args_vec)
            .current_dir(cwd)
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
        Ok(
            Self::run_git_command(["status", "--porcelain"], &self.path, true)
                .map_err(|e| GitError::from(e))
                .unwrap()
                .stdout
                .lines()
                .map(|s| s.to_string())
                .collect::<Vec<String>>()
                .len()
                > 0,
        )
    }

    pub fn get_branch(&self) -> Result<String, GitError> {
        Ok(
            Self::run_git_command(["rev-parse", "--abbrev-ref", "HEAD"], &self.path, true)
                .map_err(|e| GitError::from(e))
                .unwrap()
                .stdout,
        )
    }

    pub fn checkout(&self, branch: &String) -> Result<(), GitError> {
        let res = Self::run_git_command(["checkout", branch], &self.path, false).unwrap();
        if res.status != 0 {
            return Err(GitError::CheckoutFailed {
                branch: branch.clone(),
            });
        }

        Ok(())
    }

    pub fn merge(&self, branch: &String) -> Result<(), GitError> {
        let res = Self::run_git_command(["merge", branch], &self.path, false).unwrap();
        if res.status != 0 {
            return Err(GitError::MergeFailed {
                branch: branch.clone(),
            });
        }

        Ok(())
    }

    pub fn pull(&self) -> Result<(), GitError> {
        let res = Self::run_git_command(["pull"], &self.path, false).unwrap();
        if res.status != 0 {
            return Err(GitError::PullFailed {
                branch: self.get_branch().unwrap(),
            });
        }

        Ok(())
    }

    pub fn push(&self) -> Result<(), GitError> {
        let cur_branch = self.get_branch().unwrap();
        let mut args = ["push".to_string()].to_vec();

        if !self.get_branches().unwrap().contains(&cur_branch) {
            args.push("--set-upstream".to_string());
            args.push("origin".to_string());
            args.push(self.get_branch().unwrap());
        }

        let res = Self::run_git_command(&args, &self.path, false).unwrap();
        if res.status != 0 {
            return Err(GitError::PushFailed {
                branch: self.get_branch().unwrap(),
            });
        }

        Ok(())
    }

    pub fn push_tags(&self) -> Result<(), GitError> {
        let res = Self::run_git_command(
            ["push".to_string(), "--tags".to_string()],
            &self.path,
            false,
        )
        .unwrap();
        if res.status != 0 {
            return Err(GitError::PushTagsFailed);
        }

        Ok(())
    }

    pub fn create_branch(&self, branch: &String) -> Result<(), GitError> {
        let res = Self::run_git_command(["checkout", "-b", branch], &self.path, false).unwrap();
        if res.status != 0 {
            return Err(GitError::BranchFailed {
                branch: self.get_branch().unwrap(),
            });
        }

        Ok(())
    }

    pub fn delete_branch(&self, branch: &String) -> Result<(), GitError> {
        let res = Self::run_git_command(["branch", "-D", branch], &self.path, false).unwrap();
        if res.status != 0 {
            return Err(GitError::BranchDeletionFailed {
                branch: self.get_branch().unwrap(),
            });
        }

        Ok(())
    }

    pub fn get_branches(&self) -> Result<Vec<String>, GitError> {
        let res = Self::run_git_command(["branch"], &self.path, false).unwrap();
        if res.status != 0 {
            return Err(GitError::ListBranchFailed);
        }

        Ok(res.stdout.lines().map(|s| s.to_string()).collect())
    }

    pub fn get_remote_branches(&self) -> Result<Vec<String>, GitError> {
        let res = Self::run_git_command(["branch", "-r"], &self.path, false).unwrap();
        if res.status != 0 {
            return Err(GitError::ListBranchFailed);
        }

        Ok(res.stdout.lines().map(|s| s.to_string()).collect())
    }

    pub fn get_features(&self) -> Result<Vec<String>, GitError> {
        Ok(self
            .get_branches()
            .unwrap()
            .iter()
            .filter_map(|s| {
                let s = s.replace("*", " ").replace("  ", "");
                if s.starts_with("feature/") {
                    Some(s.replace("feature/", ""))
                } else {
                    None
                }
            })
            .collect())
    }

    pub fn get_releases(&self) -> Result<Vec<String>, GitError> {
        Ok(self
            .get_branches()
            .unwrap()
            .iter()
            .filter_map(|s| {
                let s = s.replace("*", " ").replace("  ", "");
                if s.starts_with("release/") {
                    Some(s.replace("release/", ""))
                } else {
                    None
                }
            })
            .collect())
    }

    pub fn get_remote_releases(&self) -> Result<Vec<String>, GitError> {
        Ok(self
            .get_remote_branches()
            .unwrap()
            .iter()
            .filter_map(|s| {
                let s = s.replace("*", " ").replace("  ", "");
                if s.starts_with("release/") {
                    Some(s.replace("release/", ""))
                } else {
                    None
                }
            })
            .collect())
    }

    pub fn commit(&self, msg: &String) -> Result<(), GitError> {
        let res = Self::run_git_command(["commit", "-m", msg], &self.path, false).unwrap();
        if res.status != 0 {
            return Err(GitError::CommitFailed {
                branch: self.get_branch().unwrap(),
            });
        }

        Ok(())
    }

    pub fn commit_empty(&self, msg: &String) -> Result<(), GitError> {
        let res = Self::run_git_command(["commit", "--allow-empty", "-m", msg], &self.path, false)
            .unwrap();
        if res.status != 0 {
            return Err(GitError::CommitFailed {
                branch: self.get_branch().unwrap(),
            });
        }

        Ok(())
    }

    pub fn tag(&self, name: &String) -> Result<(), GitError> {
        let res = Self::run_git_command(["tag", name], &self.path, false).unwrap();
        if res.status != 0 {
            return Err(GitError::TagFailed { tag: name.clone() });
        }

        Ok(())
    }
}
