use std::sync::mpsc::Sender;

use crate::{
    git::errors::GitError,
    logic::pipeline::{LogicPipeline, Step},
};

pub fn feature_start(name: &str, sender: Sender<String>) -> Result<(), GitError> {
    let branch = format!("feature/{}", name);

    let send = |msg: &str| {
        let _ = sender.send(msg.into());
    };

    send(&format!("Starting creation of feature {}...", name));

    LogicPipeline::execute_pipeline(
        &[
            Step::Checkout {
                branch: "develop".to_string(),
            },
            Step::Pull,
            Step::CreateBranch { branch },
        ],
        &sender,
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
            Step::Checkout {
                branch: branch.clone(),
            },
            Step::Pull,
            Step::Push,
            Step::Checkout {
                branch: "develop".to_string(),
            },
            Step::Pull,
            Step::Merge {
                branch: branch.clone(),
            },
            Step::Commit {
                message: format!("Merge after {} feature merge", name),
            },
            Step::Push,
            Step::DeleteBranch {
                branch: branch.clone(),
            },
        ],
        &sender,
    )?;

    send("Feature finished succesfully");

    Ok(())
}
