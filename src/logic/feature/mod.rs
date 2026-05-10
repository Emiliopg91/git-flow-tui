use std::sync::mpsc::Sender;

use crate::logic::{
    errors::PipelineError,
    pipeline::{LogicPipeline, Precondition, Step},
};

pub fn feature_start(name: &str, sender: Sender<String>) -> Result<(), PipelineError> {
    let branch = format!("feature/{}", name);

    let send = |msg: &str| {
        let _ = sender.send(msg.into());
    };

    send(&format!("Starting creation of feature {}...", name));

    LogicPipeline::execute_pipeline(
        &[Precondition::RequiresMissingBranch(branch.clone())],
        &[
            Step::Checkout("develop".to_string()),
            Step::Pull(),
            Step::CreateBranch(branch),
        ],
        &sender,
    )?;

    send("Feature started succesfully");

    Ok(())
}

pub fn feature_finish(name: &str, sender: Sender<String>) -> Result<(), PipelineError> {
    let branch = format!("feature/{}", name);

    let send = |msg: &str| {
        let _ = sender.send(msg.into());
    };

    send(&format!("Finishing feature {}...", name));

    LogicPipeline::execute_pipeline(
        &[Precondition::RequiresExistingLocalBranch(branch.clone())],
        &[
            Step::Checkout(branch.clone()),
            Step::Pull(),
            Step::Push(),
            Step::Checkout("develop".to_string()),
            Step::Pull(),
            Step::Merge(branch.clone()),
            Step::Commit(format!("Merge after {} feature merge", name)),
            Step::Push(),
            Step::DeleteBranch(branch.clone()),
        ],
        &sender,
    )?;

    send("Feature finished succesfully");

    Ok(())
}
