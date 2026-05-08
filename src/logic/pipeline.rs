use std::{sync::mpsc::Sender, time::Instant};

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
        git: &mut GitWrapper,
        sender: &Sender<String>,
    ) -> Result<(), GitError> {
        match step {
            StepKind::CreateBranch { branch } => {
                Self::send(sender, &format!("    Creating branch {}", branch));
                git.create_branch(branch)?;
            }
            StepKind::Checkout { branch } => {
                if git.get_branch() != branch {
                    Self::send(sender, &format!("    Checking out {} branch", branch));
                    git.checkout(branch)?;
                }
            }
            StepKind::Commit { message } => {
                Self::send(sender, &format!("    Creating commit '{}'", message));
                git.commit(message)?;
            }
            StepKind::DeleteBranch { branch } => {
                Self::send(sender, &format!("    Deleting {} branch", branch));
                git.delete_branch(branch)?;
            }
            StepKind::Merge { branch } => {
                Self::send(sender, &format!("    Merging {} branch", branch));
                git.merge(branch)?;
            }
            StepKind::Pull => {
                if git.get_remote_branches()?.contains(git.get_branch()) {
                    Self::send(sender, "    Pulling from remote");
                    git.pull()?;
                }
            }
            StepKind::Push => {
                Self::send(sender, "    Push to remote");
                git.push()?;
            }
            StepKind::PushTags => {
                Self::send(sender, "    Push tags to remote");
                git.push_tags()?;
            }
            StepKind::Tag { tag } => {
                Self::send(sender, &format!("    Creating tag {}", tag));
                git.tag(tag)?;
            }
        }

        Ok(())
    }

    pub fn execute_pipeline(steps: &[StepKind], sender: &Sender<String>) -> Result<(), GitError> {
        let t0 = Instant::now();

        let _ = Self::send(sender, "  Starting pipeline");
        let mut git = GitWrapper::global().lock().unwrap();
        for step in steps {
            match Self::execute_step(step, &mut git, sender) {
                Ok(_) => continue,
                Err(e) => {
                    return Err(e);
                }
            }
        }
        let _ = Self::send(
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
