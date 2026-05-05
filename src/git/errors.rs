use std::fmt;

#[derive(Debug)]
pub enum GitError {
    NotAGitRepository,
    CommandFailed { command: String, code: i32 },
    Io(std::io::Error),
    MergeFailed { branch: String },
    CheckoutFailed { branch: String },
    PullFailed { branch: String },
    PushFailed { branch: String },
    PushTagsFailed,
    BranchFailed { branch: String },
    BranchDeletionFailed { branch: String },
    ListBranchFailed,
    CommitFailed { branch: String },
    TagFailed { tag: String },
}

impl fmt::Display for GitError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            GitError::NotAGitRepository => {
                write!(f, "Not a git repository")
            }
            GitError::CommandFailed { command, code } => {
                write!(f, "Command '{}' failed ({})", command, code)
            }
            GitError::Io(err) => write!(f, "IO error: {}", err),
            GitError::MergeFailed { branch } => {
                write!(f, "Could not merge {} branch.", branch)
            }
            GitError::CheckoutFailed { branch } => {
                write!(f, "Could not checkout {} branch.", branch)
            }
            GitError::PullFailed { branch } => {
                write!(f, "Could not pull {} branch from remote.", branch)
            }
            GitError::PushFailed { branch } => {
                write!(f, "Could not push {} branch to remote.", branch)
            }
            GitError::PushTagsFailed => {
                write!(f, "Could not push tags")
            }
            GitError::BranchFailed { branch } => {
                write!(f, "Could not create {} branch.", branch)
            }
            GitError::ListBranchFailed => {
                write!(f, "Could not list local branches.")
            }
            GitError::BranchDeletionFailed { branch } => {
                write!(f, "Could not delete local branch {}.", branch)
            }
            GitError::CommitFailed { branch } => {
                write!(f, "Could not commit local branch {}.", branch)
            }
            GitError::TagFailed { tag } => {
                write!(f, "Could not create tag {}.", tag)
            }
        }
    }
}

impl std::error::Error for GitError {}

impl From<std::io::Error> for GitError {
    fn from(err: std::io::Error) -> Self {
        GitError::Io(err)
    }
}
