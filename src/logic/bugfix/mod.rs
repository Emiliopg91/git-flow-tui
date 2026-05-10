use std::sync::mpsc::Sender;

use crate::logic::{
    errors::PipelineError,
    pipeline::{LogicPipeline, Precondition, Step},
};

pub fn bugfix_start(name: &str, sender: Sender<String>) -> Result<(), PipelineError> {
    let branch = format!("bugfix/{}", name);
    let send = |msg: &str| {
        let _ = sender.send(msg.into());
    };

    send(&format!("Starting creation of bugfix {}...", name));

    LogicPipeline::execute_pipeline(
        &[Precondition::RequiresMissingBranch(branch.clone())],
        &[
            Step::Checkout("develop".to_string()),
            Step::Pull(),
            Step::CreateBranch(branch),
        ],
        &sender,
    )?;

    send("Bugfix started succesfully");

    Ok(())
}

pub fn bugfix_finish(name: &str, sender: Sender<String>) -> Result<(), PipelineError> {
    let branch = format!("bugfix/{}", name);
    let send = |msg: &str| {
        let _ = sender.send(msg.into());
    };

    send(&format!("Finishing bugfix {}...", name));

    LogicPipeline::execute_pipeline(
        &[Precondition::RequiresExistingLocalBranch(branch.clone())],
        &[
            Step::Checkout(branch.clone()),
            Step::Pull(),
            Step::Push(),
            Step::Checkout("develop".to_string()),
            Step::Pull(),
            Step::Merge(branch.clone()),
            Step::Commit(format!("Merge after {} bugfix merge", name)),
            Step::Push(),
            Step::DeleteBranch(branch.clone()),
        ],
        &sender,
    )?;

    send("Bugfix finished succesfully");

    Ok(())
}
