use std::fmt;

use git2::Error;

#[derive(Debug)]
pub enum GitOp {
    Merge,
    Checkout,
    Pull,
    Push,
    Commit,
    Tag,
    Fetch,
    Open,
    Branch,
    Status,
    RevParse,
}

#[derive(Debug)]
pub enum GitError {
    GitError { op: GitOp, error: Error },
    NotAGitRepository,
}

impl fmt::Display for GitError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            GitError::NotAGitRepository => {
                write!(f, "Not a git repository")
            }
            GitError::GitError { op, error } => {
                write!(f, "Error while {:?}: {}", op, error)
            }
        }
    }
}
pub trait MapGitError<T> {
    fn to_git_error(self, op: GitOp) -> Result<T, GitError>;
}

impl<T> MapGitError<T> for Result<T, Error> {
    fn to_git_error(self, op: GitOp) -> Result<T, GitError> {
        self.map_err(|e| GitError::GitError { op, error: e })
    }
}
