use std::sync::mpsc::Sender;

use crate::{
    git::{GitWrapper, errors::GitError},
    logic::pipeline::{LogicPipeline, StepKind},
};

pub fn hotfix_start(name: &str, sender: Sender<String>) -> Result<(), GitError> {
    let branch = format!("hotfix/{}", name);
    let git = GitWrapper::global().lock().unwrap();

    let send = |msg: &str| {
        let _ = sender.send(msg.into());
    };

    send(&format!("Starting creation of hotfix {}...", name));

    LogicPipeline::execute_pipeline(
        &[
            StepKind::Checkout {
                branch: git.main_branch.clone(),
            },
            StepKind::Pull,
            StepKind::CreateBranch { branch },
        ],
        sender.clone(),
    )?;

    send("Hotfix started succesfully");

    Ok(())
}

pub fn hotfix_finish(name: &str, sender: Sender<String>) -> Result<(), GitError> {
    let branch = format!("hotfix/{}", name);
    let git = GitWrapper::global().lock().unwrap();

    let send = |msg: &str| {
        let _ = sender.send(msg.into());
    };

    send(&format!("Finishing hotfix {}...", name));

    LogicPipeline::execute_pipeline(
        &[
            StepKind::Checkout {
                branch: branch.clone(),
            },
            StepKind::Pull,
            StepKind::Push,
            StepKind::Checkout {
                branch: git.main_branch.clone(),
            },
            StepKind::Pull,
            StepKind::Merge {
                branch: branch.clone(),
            },
            StepKind::Commit {
                message: format!("Merge after {} hotfix merge", name),
            },
            StepKind::Push,
            StepKind::Checkout {
                branch: "develop".to_string(),
            },
            StepKind::Pull,
            StepKind::Merge {
                branch: git.main_branch.clone(),
            },
            StepKind::Push,
            StepKind::DeleteBranch {
                branch: branch.clone(),
            },
        ],
        sender.clone(),
    )?;

    send("Hotfix finished succesfully");

    Ok(())
}
