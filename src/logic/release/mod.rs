use std::sync::mpsc::Sender;

use crate::{
    git::{GitWrapper, errors::GitError},
    logic::pipeline::{LogicPipeline, StepKind},
};

pub fn release_start(name: &str, sender: Sender<String>) -> Result<(), GitError> {
    let branch = format!("release/{name}");

    let send = |msg: &str| {
        let _ = sender.send(msg.into());
    };

    send(&format!("Starting creation of release {name}..."));

    LogicPipeline::execute_pipeline(
        &[
            StepKind::Checkout {
                branch: "develop".into(),
            },
            StepKind::Pull,
            StepKind::CreateBranch { branch },
        ],
        sender.clone(),
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

    LogicPipeline::execute_pipeline(
        &[
            StepKind::Checkout {
                branch: branch.clone(),
            },
            StepKind::Pull,
            StepKind::Push,
            StepKind::Checkout {
                branch: git.main_branch.clone(),
            },
            StepKind::Pull,
            StepKind::Merge {
                branch: branch.clone(),
            },
            StepKind::Push,
            StepKind::Tag {
                tag: name.to_string(),
            },
            StepKind::PushTags,
            StepKind::Checkout {
                branch: "develop".into(),
            },
            StepKind::Merge {
                branch: git.main_branch.clone(),
            },
            StepKind::Push,
            StepKind::DeleteBranch {
                branch: branch.clone(),
            },
        ],
        sender.clone(),
    )?;

    send("Release finished successfully");

    Ok(())
}
