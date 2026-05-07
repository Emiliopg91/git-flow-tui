use std::sync::mpsc::Sender;

use crate::git::{GitWrapper, errors::GitError};

pub fn release_start(name: &String, sender: Sender<String>) -> Result<(), GitError> {
    let branch = format!("release/{name}");

    let send = |msg: &str| {
        let _ = sender.send(msg.into());
    };

    send(&format!("Starting creation of release {name}..."));

    let git = GitWrapper::global().lock().unwrap();

    if git.get_branch()? != "develop" {
        send("  Checking out develop branch...");
        git.checkout("develop")?;
    }

    send("  Syncing changes from remote...");
    git.pull()?;

    send(&format!("  Creating branch {branch}..."));
    git.create_branch(&branch)?;

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

    if git.get_branch()? != branch {
        send(&format!("  Checking out {branch} branch..."));
        git.checkout(&branch)?;
    }

    if git.get_remote_branches()?.contains(&branch) {
        send("  Syncing changes from remote...");
        git.pull()?;
    }

    send(&format!("  Pushing {branch} to remote..."));
    git.push()?;

    send("  Checking out main branch...");
    git.checkout(&git.main_branch)?;

    send("  Syncing main with remote...");
    git.pull()?;

    send(&format!("  Merging {branch} to main branch..."));
    git.merge(&branch)?;

    send("  Pushing main branch to remote...");
    git.push()?;

    send(&format!("  Creating tag {name}..."));
    git.tag(name)?;

    send("  Pushing tags...");
    git.push_tags()?;

    send("  Checking out develop branch...");
    git.checkout("develop")?;

    send("  Merging main branch to develop...");
    git.merge(&git.main_branch)?;

    send("  Pushing develop to remote...");
    git.push()?;

    send(&format!("  Deleting local {branch} branch..."));
    git.delete_branch(&branch)?;

    send("Release finished successfully");

    Ok(())
}
