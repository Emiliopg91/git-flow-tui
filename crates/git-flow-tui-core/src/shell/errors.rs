use std::{error::Error, fmt};

#[derive(Debug)]
pub enum ShellError {
    Io(std::io::Error),

    BadExitStatus(i32, String),

    UnexpectedExecutionEnd(),
}

impl fmt::Display for ShellError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ShellError::Io(err) => {
                write!(f, "IO error: {}", err)
            }

            ShellError::BadExitStatus(status, stderr) => {
                write!(
                    f,
                    "Program finished with status {}. Error output:\n  {}",
                    status, stderr
                )
            }

            ShellError::UnexpectedExecutionEnd() => {
                write!(f, "Unexpected execution end")
            }
        }
    }
}

impl Error for ShellError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            ShellError::Io(err) => Some(err),

            ShellError::BadExitStatus(_, _) => None,

            ShellError::UnexpectedExecutionEnd() => None,
        }
    }
}

impl From<std::io::Error> for ShellError {
    fn from(err: std::io::Error) -> Self {
        ShellError::Io(err)
    }
}
