use std::{sync::mpsc::Sender, time::Instant};

use crate::git::{GitWrapper, errors::GitError};

pub enum Step {
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
        step: &Step,
        git: &mut GitWrapper,
        sender: &Sender<String>,
    ) -> Result<(), GitError> {
        match step {
            Step::CreateBranch { branch } => {
                Self::send(sender, &format!("    Creating branch {}", branch));
                git.create_branch(branch)?;
            }
            Step::Checkout { branch } => {
                if git.get_branch() != branch {
                    Self::send(sender, &format!("    Checking out {} branch", branch));
                    git.checkout(branch)?;
                }
            }
            Step::Commit { message } => {
                Self::send(sender, &format!("    Creating commit '{}'", message));
                git.commit(message)?;
            }
            Step::DeleteBranch { branch } => {
                Self::send(sender, &format!("    Deleting {} branch", branch));
                git.delete_branch(branch)?;
            }
            Step::Merge { branch } => {
                Self::send(sender, &format!("    Merging {} branch", branch));
                git.merge(branch)?;
            }
            Step::Pull => {
                if git.get_remote_branches()?.contains(git.get_branch()) {
                    Self::send(sender, "    Pulling from remote");
                    git.pull()?;
                }
            }
            Step::Push => {
                Self::send(sender, "    Push to remote");
                git.push()?;
            }
            Step::PushTags => {
                Self::send(sender, "    Push tags to remote");
                git.push_tags()?;
            }
            Step::Tag { tag } => {
                Self::send(sender, &format!("    Creating tag {}", tag));
                git.tag(tag)?;
            }
        }

        Ok(())
    }

    pub fn execute_pipeline(steps: &[Step], sender: &Sender<String>) -> Result<(), GitError> {
        let t0 = Instant::now();

        let _ = Self::send(sender, "  Starting pipeline");
        for step in steps {
            let mut git = GitWrapper::global().lock().unwrap();
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
