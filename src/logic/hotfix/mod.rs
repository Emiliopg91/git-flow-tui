use std::sync::mpsc::Sender;

use crate::{
    git::GitWrapper,
    logic::{
        errors::PipelineError,
        pipeline::{LogicPipeline, Precondition, Step},
    },
};

pub fn hotfix_start(name: &str, sender: Sender<String>) -> Result<(), PipelineError> {
    let branch = format!("hotfix/{}", name);
    let git = GitWrapper::global().lock().unwrap();
    let main_branch = git.main_branch.clone();
    drop(git);

    let send = |msg: &str| {
        let _ = sender.send(msg.into());
    };

    send(&format!("Starting creation of hotfix {}...", name));

    LogicPipeline::execute_pipeline(
        &[Precondition::RequiresMissingBranch(branch.clone())],
        &[
            Step::Checkout(main_branch.clone()),
            Step::Pull(),
            Step::CreateBranch(branch.clone()),
        ],
        &sender,
    )?;

    send("Hotfix started succesfully");

    Ok(())
}

pub fn hotfix_finish(name: &str, sender: Sender<String>) -> Result<(), PipelineError> {
    let branch = format!("hotfix/{}", name);
    let git = GitWrapper::global().lock().unwrap();
    let main_branch = git.main_branch.clone();
    drop(git);

    let send = |msg: &str| {
        let _ = sender.send(msg.into());
    };

    send(&format!("Finishing hotfix {}...", name));

    LogicPipeline::execute_pipeline(
        &[Precondition::RequiresExistingLocalBranch(branch.clone())],
        &[
            Step::Checkout(branch.clone()),
            Step::Pull(),
            Step::Push(),
            Step::Checkout(main_branch.clone()),
            Step::Pull(),
            Step::Merge(branch.clone()),
            Step::Commit(format!("Merge after {} hotfix merge", name)),
            Step::Push(),
            Step::Checkout("develop".to_string()),
            Step::Pull(),
            Step::Merge(main_branch.clone()),
            Step::Push(),
            Step::DeleteBranch(branch.clone()),
        ],
        &sender,
    )?;

    send("Hotfix finished succesfully");

    Ok(())
}
