use std::fmt;

#[derive(Debug)]
pub enum GitError {
    NotAGitRepository,
    CommandFailed { command: String, code: i32 },
    Io(std::io::Error),
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
        }
    }
}

impl std::error::Error for GitError {}

impl From<std::io::Error> for GitError {
    fn from(err: std::io::Error) -> Self {
        GitError::Io(err)
    }
}
