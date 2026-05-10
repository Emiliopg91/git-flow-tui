use std::sync::mpsc::Sender;

use crate::{
    git::GitWrapper,
    logic::{
        errors::PipelineError,
        pipeline::{LogicPipeline, Precondition, Step},
    },
};

pub fn release_start(name: &str, sender: Sender<String>) -> Result<(), PipelineError> {
    let branch = format!("release/{name}");

    let send = |msg: &str| {
        let _ = sender.send(msg.into());
    };

    send(&format!("Starting creation of release {name}..."));

    LogicPipeline::execute_pipeline(
        &[Precondition::RequiresMissingBranch(branch.clone())],
        &[
            Step::Checkout("develop".into()),
            Step::Pull(),
            Step::CreateBranch(branch.clone()),
        ],
        &sender,
    )?;

    send("Release started successfully");

    Ok(())
}

pub fn release_finish(name: &str, sender: Sender<String>) -> Result<(), PipelineError> {
    let branch = format!("release/{name}");

    let send = |msg: &str| {
        let _ = sender.send(msg.into());
    };

    send(&format!("Finishing release {name}..."));

    let git = GitWrapper::global().lock().unwrap();
    let main_branch = git.main_branch.clone();
    drop(git);

    LogicPipeline::execute_pipeline(
        &[Precondition::RequiresExistingLocalBranch(branch.clone())],
        &[
            Step::Checkout(branch.clone()),
            Step::Pull(),
            Step::Push(),
            Step::Checkout(main_branch.clone()),
            Step::Pull(),
            Step::Merge(branch.clone()),
            Step::Tag(name.to_string()),
            Step::Push(),
            Step::PushTags(),
            Step::Checkout("develop".into()),
            Step::Merge(main_branch.clone()),
            Step::Push(),
            Step::DeleteBranch(branch.clone()),
        ],
        &sender,
    )?;

    send("Release finished successfully");

    Ok(())
}
