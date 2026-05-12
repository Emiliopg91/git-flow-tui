use std::{error::Error, fmt};

use crate::shell::errors::ShellError;

#[derive(Debug)]
pub enum GitError {
    NotAGitRepository(),

    CommandFailed(String, ShellError),

    MergeFailed(String, ShellError),

    CheckoutFailed(String, ShellError),

    PullFailed(String, ShellError),

    PushFailed(String, ShellError),

    PushTagsFailed(ShellError),

    BranchFailed(String, ShellError),

    BranchDeletionFailed(String, ShellError),

    ListBranchFailed(ShellError),

    CommitFailed(String, ShellError),

    TagFailed(String, ShellError),

    FetchFailed(ShellError),
}

impl Error for GitError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            GitError::NotAGitRepository() => None,
            GitError::CommandFailed(_, err)
            | GitError::MergeFailed(_, err)
            | GitError::CheckoutFailed(_, err)
            | GitError::PullFailed(_, err)
            | GitError::PushFailed(_, err)
            | GitError::PushTagsFailed(err)
            | GitError::BranchFailed(_, err)
            | GitError::BranchDeletionFailed(_, err)
            | GitError::ListBranchFailed(err)
            | GitError::CommitFailed(_, err)
            | GitError::TagFailed(_, err)
            | GitError::FetchFailed(err) => Some(err),
        }
    }
}

impl fmt::Display for GitError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut message = match self {
            GitError::NotAGitRepository() => "Not a git repository".to_string(),

            GitError::CommandFailed(cmd, _) => {
                format!("Command '{}' failed", cmd)
            }

            GitError::MergeFailed(branch, _) => {
                format!("Could not merge {} branch", branch)
            }

            GitError::CheckoutFailed(branch, _) => {
                format!("Could not checkout {} branch", branch)
            }

            GitError::PullFailed(branch, _) => {
                format!("Could not pull {} branch from remote", branch)
            }

            GitError::PushFailed(branch, _) => {
                format!("Could not push {} branch to remote", branch)
            }

            GitError::PushTagsFailed(_) => "Could not push tags".to_string(),

            GitError::BranchFailed(branch, _) => {
                format!("Could not create {} branch", branch)
            }

            GitError::BranchDeletionFailed(branch, _) => {
                format!("Could not delete local branch {}", branch)
            }

            GitError::ListBranchFailed(_) => "Could not list local branches".to_string(),

            GitError::CommitFailed(branch, _) => {
                format!("Could not commit local branch {}", branch)
            }

            GitError::TagFailed(tag, _) => {
                format!("Could not create tag {}", tag)
            }

            GitError::FetchFailed(_) => "Could not fetch remotes".to_string(),
        };

        if let Some(err) = self.source() {
            message.push_str(&format!("\n  Caused by: {}", err));
        }

        write!(f, "{message}")?;

        Ok(())
    }
}
