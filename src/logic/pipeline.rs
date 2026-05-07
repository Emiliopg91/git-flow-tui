use std::sync::{MutexGuard, mpsc::Sender};

use crate::git::{GitWrapper, errors::GitError};

pub enum StepKind {
    CreateBranch { branch: String },
    Checkout { branch: String },
    Commit { message: String },
    DeleteBranch { branch: String },
    Merge { branch: String },
    Pull,
    Push,
    PushTags,
    Tag { tag: String },
}

pub struct LogicPipeline {}

impl LogicPipeline {
    fn execute_step(
        step: &StepKind,
        git: MutexGuard<'_, GitWrapper>,
        sender: Sender<String>,
    ) -> Result<(), GitError> {
        let send = |msg: &str| {
            let _ = sender.send(msg.into());
        };
        match step {
            StepKind::CreateBranch { branch } => {
                send(&format!("  Creating branch {}", branch));
                git.create_branch(branch)?;
            }
            StepKind::Checkout { branch } => {
                if git.get_branch()? != *branch {
                    send(&format!("  Checking out {} branch", branch));
                    git.checkout(branch)?;
                }
            }
            StepKind::Commit { message } => {
                send(&format!("  Creating commit '{}'", message));
                git.commit(message)?;
            }
            StepKind::DeleteBranch { branch } => {
                send(&format!("  Deleting {} branch", branch));
                git.delete_branch(branch)?;
            }
            StepKind::Merge { branch } => {
                send(&format!("  Merging {} branch", branch));
                git.merge(branch)?;
            }
            StepKind::Pull => {
                if git.get_remote_branches()?.contains(&git.get_branch()?) {
                    send("  Pulling from remote");
                    git.pull()?;
                }
            }
            StepKind::Push => {
                send("  Push to remote");
                git.push()?;
            }
            StepKind::PushTags => {
                send("  Push tags to remote");
                git.push_tags()?;
            }
            StepKind::Tag { tag } => {
                send(&format!("  Creating tag {}", tag));
                git.tag(tag)?;
            }
        }

        Ok(())
    }

    pub fn execute_pipeline(steps: &[StepKind], sender: Sender<String>) -> Result<(), GitError> {
        for step in steps {
            let git = GitWrapper::global().lock().unwrap();
            match Self::execute_step(step, git, sender.clone()) {
                Ok(_) => continue,
                Err(e) => {
                    return Err(e);
                }
            }
        }

        Ok(())
    }
}
