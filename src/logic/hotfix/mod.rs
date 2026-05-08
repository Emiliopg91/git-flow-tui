use std::sync::mpsc::Sender;

use crate::{
    git::{GitWrapper, errors::GitError},
    logic::pipeline::{LogicPipeline, Step},
};

pub fn hotfix_start(name: &str, sender: Sender<String>) -> Result<(), GitError> {
    let branch = format!("hotfix/{}", name);
    let git = GitWrapper::global().lock().unwrap();
    let main_branch = git.main_branch.clone();
    drop(git);

    let send = |msg: &str| {
        let _ = sender.send(msg.into());
    };

    send(&format!("Starting creation of hotfix {}...", name));

    LogicPipeline::execute_pipeline(
        &[
            Step::Checkout {
                branch: main_branch.clone(),
            },
            Step::Pull,
            Step::CreateBranch { branch },
        ],
        &sender,
    )?;

    send("Hotfix started succesfully");

    Ok(())
}

pub fn hotfix_finish(name: &str, sender: Sender<String>) -> Result<(), GitError> {
    let branch = format!("hotfix/{}", name);
    let git = GitWrapper::global().lock().unwrap();
    let main_branch = git.main_branch.clone();
    drop(git);

    let send = |msg: &str| {
        let _ = sender.send(msg.into());
    };

    send(&format!("Finishing hotfix {}...", name));

    LogicPipeline::execute_pipeline(
        &[
            Step::Checkout {
                branch: branch.clone(),
            },
            Step::Pull,
            Step::Push,
            Step::Checkout {
                branch: main_branch.clone(),
            },
            Step::Pull,
            Step::Merge {
                branch: branch.clone(),
            },
            Step::Commit {
                message: format!("Merge after {} hotfix merge", name),
            },
            Step::Push,
            Step::Checkout {
                branch: "develop".to_string(),
            },
            Step::Pull,
            Step::Merge {
                branch: main_branch.clone(),
            },
            Step::Push,
            Step::DeleteBranch {
                branch: branch.clone(),
            },
        ],
        &sender,
    )?;

    send("Hotfix finished succesfully");

    Ok(())
}
