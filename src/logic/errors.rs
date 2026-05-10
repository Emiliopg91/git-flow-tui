use std::{error::Error, fmt};

#[derive(Debug)]
pub enum PipelineError {
    PreconditionFailed(String, Option<Box<dyn Error>>),
    ExecutionFailed(Box<dyn Error>),
}

impl Error for PipelineError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            PipelineError::PreconditionFailed(_, e) => e.as_deref(),
            PipelineError::ExecutionFailed(e) => Some(e.as_ref()),
        }
    }
}

impl fmt::Display for PipelineError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut message = match self {
            PipelineError::PreconditionFailed(msg, _) => format!("Precondition failed: {}", msg),
            PipelineError::ExecutionFailed(e) => format!("Execution failed: {}", e),
        };

        if let Some(err) = self.source() {
            message.push_str(&format!("\n  Caused by: {}", err));
        }

        write!(f, "{message}")?;

        Ok(())
    }
}
