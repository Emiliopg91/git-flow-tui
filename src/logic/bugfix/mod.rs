use std::sync::mpsc::Sender;

use crate::{
    git::errors::GitError,
    logic::pipeline::{LogicPipeline, StepKind},
};

pub fn bugfix_start(name: &String, sender: Sender<String>) -> Result<(), GitError> {
    let branch = format!("bugfix/{}", name);
    let send = |msg: &str| {
        let _ = sender.send(msg.into());
    };

    send(&format!("Starting creation of bugfix {}...", name));

    LogicPipeline::execute_pipeline(
        &[
            StepKind::Checkout {
                branch: "develop".to_string(),
            },
            StepKind::Pull,
            StepKind::CreateBranch { branch },
        ],
        sender.clone(),
    )?;

    send("Bugfix started succesfully");

    Ok(())
}

pub fn bugfix_finish(name: &String, sender: Sender<String>) -> Result<(), GitError> {
    let branch = format!("bugfix/{}", name);
    let send = |msg: &str| {
        let _ = sender.send(msg.into());
    };

    send(&format!("Finishing bugfix {}...", name));

    LogicPipeline::execute_pipeline(
        &[
            StepKind::Checkout {
                branch: branch.clone(),
            },
            StepKind::Pull,
            StepKind::Push,
            StepKind::Checkout {
                branch: "develop".to_string(),
            },
            StepKind::Pull,
            StepKind::Merge {
                branch: branch.clone(),
            },
            StepKind::Commit {
                message: format!("Merge after {} bugfix merge", name),
            },
            StepKind::Push,
            StepKind::DeleteBranch {
                branch: branch.clone(),
            },
        ],
        sender.clone(),
    )?;

    send("Bugfix finished succesfully");

    Ok(())
}
