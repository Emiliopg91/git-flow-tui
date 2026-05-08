use std::sync::mpsc::Sender;

use crate::{
    git::{GitWrapper, errors::GitError},
    logic::pipeline::{LogicPipeline, Step},
};

pub fn release_start(name: &str, sender: Sender<String>) -> Result<(), GitError> {
    let branch = format!("release/{name}");

    let send = |msg: &str| {
        let _ = sender.send(msg.into());
    };

    send(&format!("Starting creation of release {name}..."));

    LogicPipeline::execute_pipeline(
        &[
            Step::Checkout {
                branch: "develop".into(),
            },
            Step::Pull,
            Step::CreateBranch { branch },
        ],
        &sender,
    )?;

    send("Release started successfully");

    Ok(())
}

pub fn release_finish(name: &str, sender: Sender<String>) -> Result<(), GitError> {
    let branch = format!("release/{name}");

    let send = |msg: &str| {
        let _ = sender.send(msg.into());
    };

    send(&format!("Finishing release {name}..."));

    let git = GitWrapper::global().lock().unwrap();
    let main_branch = git.main_branch.clone();
    drop(git);

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
            Step::Push,
            Step::Tag {
                tag: name.to_string(),
            },
            Step::PushTags,
            Step::Checkout {
                branch: "develop".into(),
            },
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

    send("Release finished successfully");

    Ok(())
}
