pub mod errors;

use std::{
    fs,
    path::PathBuf,
    sync::{Mutex, OnceLock},
};

use git2::{FetchOptions, RemoteCallbacks, Repository, Status, StatusOptions};

use crate::git::errors::{GitError, MapGitError};

pub struct GitWrapper {
    repo: Repository,
}

static GIT_WRAPPER_INST: OnceLock<Mutex<GitWrapper>> = OnceLock::new();

impl GitWrapper {
    pub fn global() -> &'static Mutex<GitWrapper> {
        GIT_WRAPPER_INST.get().unwrap()
    }

    pub fn initialize(path_buf: PathBuf) -> Result<(), GitError> {
        let mut curr_path = path_buf.clone();
        loop {
            if let Ok(exists) = fs::exists(curr_path.join(".git"))
                && exists
            {
                break;
            }
            curr_path = curr_path.parent().unwrap().to_path_buf();
            if curr_path.display().to_string() == "/" {
                return Err(GitError::NotAGitRepository);
            }
        }

        let inst = Self {
            repo: Repository::open(curr_path).map_err(|e| GitError::GitError {
                op: errors::GitOp::Open,
                error: e,
            })?,
        };

        GIT_WRAPPER_INST.get_or_init(|| Mutex::new(inst));

        Ok(())
    }

    pub fn get_changes(&self) -> Result<Vec<(String, Status)>, GitError> {
        let mut opts = StatusOptions::new();
        opts.include_untracked(true);

        let statuses = self
            .repo
            .statuses(Some(&mut opts))
            .to_git_error(errors::GitOp::Status)?;

        Ok(statuses
            .iter()
            .map(|e| (e.path().unwrap_or("").to_string(), e.status()))
            .collect())
    }

    pub fn get_branch(&self) -> Result<String, GitError> {
        let head = self.repo.head().to_git_error(errors::GitOp::RevParse)?;

        Ok(head.shorthand().unwrap_or("HEAD (detached)").to_string())
    }

    pub fn checkout(&self, branch: &str) -> Result<(), GitError> {
        let (object, reference) = self
            .repo
            .revparse_ext(branch)
            .to_git_error(errors::GitOp::Merge)?;

        self.repo
            .checkout_tree(&object, None)
            .to_git_error(errors::GitOp::Checkout)?;

        if let Some(r) = reference {
            let name = r.name().ok_or_else(|| GitError::GitError {
                op: errors::GitOp::Checkout,
                error: git2::Error::from_str("Invalid reference name"),
            })?;

            self.repo
                .set_head(name)
                .to_git_error(errors::GitOp::Checkout)?;

            return Ok(());
        }

        self.repo
            .set_head_detached(object.id())
            .to_git_error(errors::GitOp::Checkout)?;

        Ok(())
    }

    pub fn merge(&self, branch: &str) -> Result<(), GitError> {
        let reference = self
            .repo
            .find_reference(&format!("refs/heads/{}", branch))
            .to_git_error(errors::GitOp::Merge)?;

        let annotated = self
            .repo
            .reference_to_annotated_commit(&reference)
            .to_git_error(errors::GitOp::Merge)?;

        let head = self.repo.head().to_git_error(errors::GitOp::Merge)?;

        let head_commit = self
            .repo
            .reference_to_annotated_commit(&head)
            .to_git_error(errors::GitOp::Merge)?;

        let head_commit = self
            .repo
            .find_commit(head_commit.id())
            .to_git_error(errors::GitOp::Merge)?;

        let analysis = self
            .repo
            .merge_analysis(&[&annotated])
            .to_git_error(errors::GitOp::Merge)?;

        let annotated = self
            .repo
            .find_commit(annotated.id())
            .to_git_error(errors::GitOp::Merge)?;

        if analysis.0.is_up_to_date() {
            return Ok(());
        }

        if analysis.0.is_fast_forward() {
            let refname = "HEAD";

            let mut r = self
                .repo
                .find_reference(refname)
                .to_git_error(errors::GitOp::Merge)?;

            r.set_target(annotated.id(), "Fast-Forward")
                .to_git_error(errors::GitOp::Merge)?;

            self.repo
                .set_head(refname)
                .to_git_error(errors::GitOp::Merge)?;

            self.repo
                .checkout_head(Some(git2::build::CheckoutBuilder::default().force()))
                .to_git_error(errors::GitOp::Merge)?;

            return Ok(());
        }

        if analysis.0.is_normal() {
            let mut idx = self
                .repo
                .merge_commits(&head_commit, &annotated, None)
                .to_git_error(errors::GitOp::Merge)?;

            if idx.has_conflicts() {
                return Err(GitError::GitError {
                    op: errors::GitOp::Merge,
                    error: git2::Error::from_str("Merge conflict detected"),
                });
            }

            let tree_id = idx
                .write_tree_to(&self.repo)
                .to_git_error(errors::GitOp::Merge)?;

            let result_tree = self
                .repo
                .find_tree(tree_id)
                .to_git_error(errors::GitOp::Merge)?;

            let sig = self.repo.signature().to_git_error(errors::GitOp::Merge)?;

            let head_commit = self
                .repo
                .find_commit(head_commit.id())
                .to_git_error(errors::GitOp::Merge)?;

            let other_commit = self
                .repo
                .find_commit(annotated.id())
                .to_git_error(errors::GitOp::Merge)?;

            self.repo
                .commit(
                    Some("HEAD"),
                    &sig,
                    &sig,
                    "Merge commit",
                    &result_tree,
                    &[&head_commit, &other_commit],
                )
                .to_git_error(errors::GitOp::Merge)?;

            return Ok(());
        }

        Err(GitError::GitError {
            op: errors::GitOp::Merge,
            error: git2::Error::from_str("Unknown merge state"),
        })
    }

    pub fn pull(&self) -> Result<(), GitError> {
        let repo = &self.repo;
        let branch = self.get_branch()?;

        let mut remote = repo
            .find_remote("origin")
            .to_git_error(errors::GitOp::Pull)?;

        let callbacks = RemoteCallbacks::new();

        let mut fetch_opts = FetchOptions::new();
        fetch_opts.remote_callbacks(callbacks);

        remote
            .fetch(&[&branch], Some(&mut fetch_opts), None)
            .to_git_error(errors::GitOp::Pull)?;

        let fetch_head = repo
            .find_reference(&format!("refs/remotes/origin/{}", branch))
            .to_git_error(errors::GitOp::Pull)?;

        let annotated = repo
            .reference_to_annotated_commit(&fetch_head)
            .to_git_error(errors::GitOp::Pull)?;

        let head = repo.head().to_git_error(errors::GitOp::Pull)?;

        let head_commit = repo
            .reference_to_annotated_commit(&head)
            .to_git_error(errors::GitOp::Pull)?;

        let analysis = repo
            .merge_analysis(&[&annotated])
            .to_git_error(errors::GitOp::Pull)?;

        if analysis.0.is_up_to_date() {
            return Ok(());
        }

        if analysis.0.is_fast_forward() {
            let refname = "HEAD";

            let mut r = repo
                .find_reference(refname)
                .to_git_error(errors::GitOp::Pull)?;

            r.set_target(annotated.id(), "Fast-forward")
                .to_git_error(errors::GitOp::Pull)?;

            repo.set_head(refname).to_git_error(errors::GitOp::Pull)?;

            repo.checkout_head(Some(git2::build::CheckoutBuilder::default().force()))
                .to_git_error(errors::GitOp::Pull)?;

            return Ok(());
        }

        if analysis.0.is_normal() {
            let head_commit = repo
                .find_commit(head_commit.id())
                .to_git_error(errors::GitOp::Pull)?;

            let fetch_commit = repo
                .find_commit(annotated.id())
                .to_git_error(errors::GitOp::Pull)?;

            let mut idx = repo
                .merge_commits(&head_commit, &fetch_commit, None)
                .to_git_error(errors::GitOp::Pull)?;

            if idx.has_conflicts() {
                return Err(GitError::GitError {
                    op: errors::GitOp::Pull,
                    error: git2::Error::from_str("Merge conflicts"),
                });
            }

            let tree_id = idx.write_tree_to(repo).to_git_error(errors::GitOp::Pull)?;

            let result_tree = repo.find_tree(tree_id).to_git_error(errors::GitOp::Pull)?;

            let sig = repo.signature().to_git_error(errors::GitOp::Pull)?;

            repo.commit(
                Some("HEAD"),
                &sig,
                &sig,
                "Merge remote-tracking branch",
                &result_tree,
                &[&head_commit, &fetch_commit],
            )
            .to_git_error(errors::GitOp::Pull)?;

            return Ok(());
        }

        Err(GitError::GitError {
            op: errors::GitOp::Pull,
            error: git2::Error::from_str("Unknown pull state"),
        })
    }

    pub fn push(&self) -> Result<(), GitError> {
        let branch = self.get_branch()?;

        let mut remote = self
            .repo
            .find_remote("origin")
            .to_git_error(errors::GitOp::Push)?;

        let refspec = format!("refs/heads/{0}:refs/heads/{0}", branch);

        let mut options = git2::PushOptions::new();
        options.remote_callbacks(git2::RemoteCallbacks::new());

        remote
            .push(&[&refspec], Some(&mut options))
            .to_git_error(errors::GitOp::Push)?;

        let mut local_branch = self
            .repo
            .find_branch(&branch, git2::BranchType::Local)
            .to_git_error(errors::GitOp::Push)?;

        local_branch
            .set_upstream(Some(&format!("origin/{}", branch)))
            .ok();

        Ok(())
    }

    pub fn push_tags(&self) -> Result<(), GitError> {
        let mut remote = self
            .repo
            .find_remote("origin")
            .to_git_error(errors::GitOp::Push)?;

        let callbacks = git2::RemoteCallbacks::new();

        let mut options = git2::PushOptions::new();
        options.remote_callbacks(callbacks);

        remote
            .push(&["refs/tags/*:refs/tags/*"], Some(&mut options))
            .to_git_error(errors::GitOp::Push)?;

        Ok(())
    }

    pub fn create_branch(&self, branch: &str) -> Result<(), GitError> {
        let head = self.repo.head().to_git_error(errors::GitOp::Branch)?;

        let target_commit = head.peel_to_commit().to_git_error(errors::GitOp::Branch)?;

        self.repo
            .branch(branch, &target_commit, false)
            .to_git_error(errors::GitOp::Branch)?;

        self.checkout(branch)?;

        Ok(())
    }

    pub fn delete_branch(&self, branch: &str) -> Result<(), GitError> {
        if self.get_branch()? == branch {
            return Ok(());
        }

        let mut b = self
            .repo
            .find_branch(branch, git2::BranchType::Local)
            .to_git_error(errors::GitOp::Branch)?;

        b.delete().to_git_error(errors::GitOp::Branch)?;

        Ok(())
    }
    pub fn commit(&self, msg: &str) -> Result<(), GitError> {
        let repo = &self.repo;

        let sig = repo.signature().to_git_error(errors::GitOp::Commit)?;

        let head = repo.head().ok();
        let parent = head.and_then(|h| h.peel_to_commit().ok());

        let tree = match &parent {
            Some(c) => c.tree().to_git_error(errors::GitOp::Commit)?,

            None => {
                let oid = repo
                    .treebuilder(None)
                    .to_git_error(errors::GitOp::Commit)?
                    .write()
                    .to_git_error(errors::GitOp::Commit)?;

                repo.find_tree(oid).to_git_error(errors::GitOp::Commit)?
            }
        };

        let parents: Vec<&git2::Commit> = parent.iter().collect();

        repo.commit(Some("HEAD"), &sig, &sig, msg, &tree, &parents)
            .to_git_error(errors::GitOp::Commit)?;

        Ok(())
    }

    pub fn tag(&self, name: &str) -> Result<(), GitError> {
        let obj = self
            .repo
            .head()
            .and_then(|h| h.peel(git2::ObjectType::Commit))
            .to_git_error(errors::GitOp::Tag)?;

        let commit = obj.as_commit().expect("HEAD must be a commit");

        self.repo
            .tag_lightweight(name, commit.as_object(), false)
            .to_git_error(errors::GitOp::Tag)?;

        Ok(())
    }

    pub fn fetch(&self, all: bool, prune: bool) -> Result<(), GitError> {
        let repo = &self.repo;

        let callbacks = git2::RemoteCallbacks::new();

        let mut fetch_opts = git2::FetchOptions::new();
        fetch_opts.remote_callbacks(callbacks);

        fetch_opts.prune(if prune {
            git2::FetchPrune::On
        } else {
            git2::FetchPrune::Off
        });

        let mut fetch_remote = |name: &str| -> Result<(), GitError> {
            let mut remote = repo.find_remote(name).to_git_error(errors::GitOp::Fetch)?;

            remote
                .fetch(&[] as &[&str], Some(&mut fetch_opts), None)
                .to_git_error(errors::GitOp::Fetch)?;

            Ok(())
        };

        if all {
            let remotes = repo.remotes().to_git_error(errors::GitOp::Fetch)?;

            for i in 0..remotes.len() {
                if let Some(name) = remotes.get(i) {
                    fetch_remote(name)?;
                }
            }
        } else {
            fetch_remote("origin")?;
        }

        Ok(())
    }

    pub fn get_branches(&self) -> Result<Vec<String>, GitError> {
        let branches = self
            .repo
            .branches(Some(git2::BranchType::Local))
            .to_git_error(errors::GitOp::Branch)?;

        let mut result = Vec::new();

        for branch in branches {
            let (branch, _) = branch.to_git_error(errors::GitOp::Branch)?;

            if let Some(name) = branch.name().to_git_error(errors::GitOp::Branch)? {
                result.push(name.to_string());
            }
        }

        Ok(result)
    }

    pub fn get_remote_branches(&self) -> Result<Vec<String>, GitError> {
        let branches = self
            .repo
            .branches(Some(git2::BranchType::Remote))
            .to_git_error(errors::GitOp::Branch)?;

        let mut result = Vec::new();

        for branch in branches {
            let (branch, _) = branch.to_git_error(errors::GitOp::Branch)?;

            if let Some(name) = branch.name().to_git_error(errors::GitOp::Branch)? {
                result.push(name.to_string());
            }
        }

        Ok(result)
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
}
