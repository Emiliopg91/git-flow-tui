use std::sync::mpsc::Sender;

use crate::{
    git::GitWrapper,
    logic::{
        errors::PipelineError,
        pipeline::{Pipeline, Precondition, Step},
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

    Pipeline::new(sender.clone())
        .step(Step::Checkout(main_branch.clone()))
        .step(Step::Pull())
        .step(Step::CreateBranch(branch.clone()))
        .run()?;

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

    Pipeline::new(sender.clone())
        .requires(Precondition::RequiresExistingLocalBranch(branch.clone()))
        .step(Step::Checkout(branch.clone()))
        .step(Step::Pull())
        .step(Step::Push())
        .step(Step::Checkout(main_branch.clone()))
        .step(Step::Pull())
        .step(Step::Merge(branch.clone()))
        .step(Step::Commit(format!("Merge after {} hotfix merge", name)))
        .step(Step::Push())
        .step(Step::Checkout("develop".to_string()))
        .step(Step::Pull())
        .step(Step::Merge(main_branch.clone()))
        .step(Step::Push())
        .step(Step::DeleteBranch(branch.clone()))
        .run()?;

    send("Hotfix finished succesfully");

    Ok(())
}
