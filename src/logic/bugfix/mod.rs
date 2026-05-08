use std::sync::mpsc::Sender;

use crate::{
    git::errors::GitError,
    logic::pipeline::{LogicPipeline, Step},
};

pub fn bugfix_start(name: &str, sender: Sender<String>) -> Result<(), GitError> {
    let branch = format!("bugfix/{}", name);
    let send = |msg: &str| {
        let _ = sender.send(msg.into());
    };

    send(&format!("Starting creation of bugfix {}...", name));

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

    send("Bugfix started succesfully");

    Ok(())
}

pub fn bugfix_finish(name: &str, sender: Sender<String>) -> Result<(), GitError> {
    let branch = format!("bugfix/{}", name);
    let send = |msg: &str| {
        let _ = sender.send(msg.into());
    };

    send(&format!("Finishing bugfix {}...", name));

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
                message: format!("Merge after {} bugfix merge", name),
            },
            Step::Push,
            Step::DeleteBranch {
                branch: branch.clone(),
            },
        ],
        &sender,
    )?;

    send("Bugfix finished succesfully");

    Ok(())
}
