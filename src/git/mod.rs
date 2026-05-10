pub mod errors;

use std::sync::{Mutex, OnceLock};

use crate::{
    git::errors::GitError,
    shell::{ShellResult, run_shell_command},
};

pub struct GitWrapper {
    pub main_branch: String,
    cur_branch: String,
}

static GIT_WRAPPER_INST: OnceLock<Mutex<GitWrapper>> = OnceLock::new();

impl GitWrapper {
    pub fn global() -> &'static Mutex<GitWrapper> {
        GIT_WRAPPER_INST.get().unwrap()
    }

    pub fn initialize() -> Result<(), GitError> {
        if run_shell_command("git", ["status"], true).is_err() {
            return Err(GitError::NotAGitRepository());
        }

        let res = run_shell_command("git", ["symbolic-ref", "refs/remotes/origin/HEAD"], true)
            .unwrap_or(ShellResult {
                status: 0,
                stdout: "main".to_string(),
                stderr: "".to_string(),
            });
        let main_branch = res.stdout.rsplit('/').next().unwrap_or("main").to_string();

        let res2 = run_shell_command("git", ["rev-parse", "--abbrev-ref", "HEAD"], true).unwrap_or(
            ShellResult {
                status: 0,
                stdout: res.stdout,
                stderr: "".to_string(),
            },
        );
        let cur_branch = res2.stdout;

        let inst = Self {
            main_branch,
            cur_branch,
        };

        GIT_WRAPPER_INST.get_or_init(|| Mutex::new(inst));

        Ok(())
    }

    pub fn has_changes(&self) -> Result<bool, GitError> {
        let args = ["status", "--porcelain"];
        Ok(!run_shell_command("git", &args, true)
            .map_err(|e| GitError::CommandFailed("git status --porcelain".to_string(), e))?
            .stdout
            .lines()
            .map(|s| s.to_string())
            .collect::<Vec<String>>()
            .is_empty())
    }

    pub fn get_branch(&self) -> &String {
        &self.cur_branch
    }

    pub fn checkout(&mut self, branch: &str) -> Result<(), GitError> {
        run_shell_command("git", ["checkout", branch], true)
            .map_err(|e| GitError::CheckoutFailed(branch.into(), e))?;
        self.cur_branch = branch.to_string();
        Ok(())
    }

    pub fn merge(&self, branch: &str) -> Result<(), GitError> {
        run_shell_command("git", ["merge", branch], true)
            .map_err(|e| GitError::MergeFailed(branch.into(), e))?;

        Ok(())
    }

    pub fn pull(&self) -> Result<(), GitError> {
        run_shell_command("git", ["pull"], true)
            .map_err(|e| GitError::PullFailed(self.get_branch().into(), e))?;

        Ok(())
    }

    pub fn push(&self) -> Result<(), GitError> {
        let cur_branch = self.get_branch();
        let mut args = ["push".to_string()].to_vec();

        if !self.get_remote_branches()?.contains(cur_branch) {
            args.push("--set-upstream".to_string());
            args.push("origin".to_string());
            args.push(cur_branch.clone());
        }

        run_shell_command("git", &args, true)
            .map_err(|e| GitError::PushFailed(self.get_branch().into(), e))?;

        Ok(())
    }

    pub fn push_tags(&self) -> Result<(), GitError> {
        run_shell_command("git", ["push".to_string(), "--tags".to_string()], true)
            .map_err(|e| GitError::PushTagsFailed(e))?;

        Ok(())
    }

    pub fn commit(&self, msg: &str) -> Result<(), GitError> {
        run_shell_command("git", ["commit", "--allow-empty", "-m", msg], true)
            .map_err(|e| GitError::CommitFailed(self.get_branch().into(), e))?;

        Ok(())
    }

    pub fn tag(&self, name: &str) -> Result<(), GitError> {
        run_shell_command("git", ["tag", name], true)
            .map_err(|e| GitError::TagFailed(name.to_string(), e))?;

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

        run_shell_command("git", &args, true).map_err(|e| GitError::FetchFailed(e))?;

        Ok(())
    }

    pub fn create_branch(&mut self, branch: &str) -> Result<(), GitError> {
        run_shell_command("git", ["checkout", "-b", branch], true)
            .map_err(|e| GitError::BranchFailed(branch.to_string(), e))?;
        self.cur_branch = branch.to_string();
        Ok(())
    }

    pub fn delete_branch(&self, branch: &str) -> Result<(), GitError> {
        run_shell_command("git", ["branch", "-D", branch], true)
            .map_err(|e| GitError::BranchDeletionFailed(branch.to_string(), e))?;

        Ok(())
    }

    pub fn get_branches(&self) -> Result<Vec<String>, GitError> {
        let res = run_shell_command("git", ["branch"], true)
            .map_err(|e| GitError::ListBranchFailed(e))?;

        Ok(res
            .stdout
            .lines()
            .map(|s| s.replace("* ", "").trim().to_string())
            .collect())
    }

    pub fn get_remote_branches(&self) -> Result<Vec<String>, GitError> {
        let res = run_shell_command("git", ["branch", "-r"], true)
            .map_err(|e| GitError::ListBranchFailed(e))?;

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
}
