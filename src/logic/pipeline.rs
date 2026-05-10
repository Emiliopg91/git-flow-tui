use std::{sync::mpsc::Sender, time::Instant};

use crate::{git::GitWrapper, logic::errors::PipelineError};

pub enum Precondition {
    RequiresMissingBranch(String),
    RequiresExistingLocalBranch(String),
}

pub enum Step {
    CreateBranch(String),
    Checkout(String),
    Commit(String),
    DeleteBranch(String),
    Merge(String),
    Pull(),
    Push(),
    PushTags(),
    Tag(String),
}

pub struct LogicPipeline {}

impl LogicPipeline {
    fn execute_precondition(
        precondition: &Precondition,
        git: &mut GitWrapper,
        sender: &Sender<String>,
    ) -> Result<(), PipelineError> {
        match precondition {
            Precondition::RequiresMissingBranch(branch) => {
                Self::send(sender, &format!("    Checking branch {}", branch));
                let local = git.get_branches().map_err(|e| {
                    PipelineError::PreconditionFailed(
                        "Could not get local branches".to_string(),
                        Some(Box::new(e)),
                    )
                })?;
                let remotes = git.get_remote_branches().map_err(|e| {
                    PipelineError::PreconditionFailed(
                        "Could not get remote branches".to_string(),
                        Some(Box::new(e)),
                    )
                })?;

                if local.contains(branch) || remotes.contains(branch) {
                    return Err(PipelineError::PreconditionFailed(
                        format!("Branch {} already exists", branch),
                        None,
                    ));
                }

                Ok(())
            }
            Precondition::RequiresExistingLocalBranch(branch) => {
                let local = git.get_branches().map_err(|e| {
                    PipelineError::PreconditionFailed(
                        "Could not get local branches".to_string(),
                        Some(Box::new(e)),
                    )
                })?;
                if !local.contains(branch) {
                    return Err(PipelineError::PreconditionFailed(
                        format!("Branch {} not found", branch),
                        None,
                    ));
                }
                Ok(())
            }
        }
    }

    fn execute_step(
        step: &Step,
        git: &mut GitWrapper,
        sender: &Sender<String>,
    ) -> Result<(), PipelineError> {
        match step {
            Step::CreateBranch(branch) => {
                Self::send(sender, &format!("    Creating branch {}", branch));
                git.create_branch(branch)
                    .map_err(|e| PipelineError::ExecutionFailed(Box::new(e)))?;
            }
            Step::Checkout(branch) => {
                if git.get_branch() != branch {
                    Self::send(sender, &format!("    Checking out {} branch", branch));
                    git.checkout(branch)
                        .map_err(|e| PipelineError::ExecutionFailed(Box::new(e)))?;
                }
            }
            Step::Commit(message) => {
                Self::send(sender, &format!("    Creating commit '{}'", message));
                git.commit(message)
                    .map_err(|e| PipelineError::ExecutionFailed(Box::new(e)))?;
            }
            Step::DeleteBranch(branch) => {
                Self::send(sender, &format!("    Deleting {} branch", branch));
                git.delete_branch(branch)
                    .map_err(|e| PipelineError::ExecutionFailed(Box::new(e)))?;
            }
            Step::Merge(branch) => {
                Self::send(sender, &format!("    Merging {} branch", branch));
                git.merge(branch)
                    .map_err(|e| PipelineError::ExecutionFailed(Box::new(e)))?;
            }
            Step::Pull() => {
                if git
                    .get_remote_branches()
                    .map_err(|e| PipelineError::ExecutionFailed(Box::new(e)))?
                    .contains(git.get_branch())
                {
                    Self::send(sender, "    Pulling from remote");
                    git.pull()
                        .map_err(|e| PipelineError::ExecutionFailed(Box::new(e)))?;
                }
            }
            Step::Push() => {
                Self::send(sender, "    Push to remote");
                git.push()
                    .map_err(|e| PipelineError::ExecutionFailed(Box::new(e)))?;
            }
            Step::PushTags() => {
                Self::send(sender, "    Push tags to remote");
                git.push_tags()
                    .map_err(|e| PipelineError::ExecutionFailed(Box::new(e)))?;
            }
            Step::Tag(tag) => {
                Self::send(sender, &format!("    Creating tag {}", tag));
                git.tag(tag)
                    .map_err(|e| PipelineError::ExecutionFailed(Box::new(e)))?;
            }
        }

        Ok(())
    }

    pub fn execute_pipeline(
        preconditions: &[Precondition],
        steps: &[Step],
        sender: &Sender<String>,
    ) -> Result<(), PipelineError> {
        let t0 = Instant::now();

        Self::send(sender, "  Starting pipeline");
        let mut git = GitWrapper::global().lock().unwrap();
        for precondition in preconditions {
            match Self::execute_precondition(precondition, &mut git, sender) {
                Ok(_) => continue,
                Err(e) => {
                    return Err(e);
                }
            }
        }
        for step in steps {
            match Self::execute_step(step, &mut git, sender)
                .map_err(|e| PipelineError::ExecutionFailed(Box::new(e)))
            {
                Ok(_) => continue,
                Err(e) => {
                    return Err(e);
                }
            }
        }
        Self::send(
            sender,
            &format!(
                "  Pipeline finished after {:.3}",
                t0.elapsed().as_secs_f64()
            ),
        );

        Ok(())
    }

    fn send(sender: &Sender<String>, msg: &str) {
        let _ = sender.send(msg.to_string());
    }
}
