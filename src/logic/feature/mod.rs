use std::sync::mpsc::Sender;

use crate::{
    git::errors::GitError,
    logic::pipeline::{LogicPipeline, StepKind},
};

pub fn feature_start(name: &str, sender: Sender<String>) -> Result<(), GitError> {
    let branch = format!("feature/{}", name);

    let send = |msg: &str| {
        let _ = sender.send(msg.into());
    };

    send(&format!("Starting creation of feature {}...", name));

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

    send("Feature started succesfully");

    Ok(())
}

pub fn feature_finish(name: &str, sender: Sender<String>) -> Result<(), GitError> {
    let branch = format!("feature/{}", name);

    let send = |msg: &str| {
        let _ = sender.send(msg.into());
    };

    send(&format!("Finishing feature {}...", name));

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

    send("Feature finished succesfully");

    Ok(())
}
