use std::sync::mpsc::Sender;

use crate::git::{errors::GitError, GitWrapper};

pub fn release_start(name: &String, sender: Sender<String>) -> Result<(), GitError> {
    let branch = format!("release/{name}");

    let send = |msg: &str| {
        let _ = sender.send(msg.into());
    };

    send(&format!("Starting creation of release {name}..."));

    let git = GitWrapper::global().lock().unwrap();

    if git.get_branch()? != "develop" {
        send("  Checking out develop branch...");
        git.checkout(&"develop".to_string())?;
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

    send("  Checking out master branch...");
    git.checkout(&"master".to_string())?;

    send(&format!("  Merging {branch} to master..."));
    git.merge(&branch)?;

    send(&format!("  Creating tag {name}..."));
    git.tag(&name.to_string())?;

    send("  Pushing tags...");
    git.push_tags()?;

    send("  Creating commit for merge");
    git.commit(&format!("Merge after {name} release merge"))?;

    send("  Pushing master to remote...");
    git.push()?;

    send("  Checking out develop branch...");
    git.checkout(&"develop".to_string())?;

    send("  Merging master to develop...");
    git.merge(&"master".to_string())?;

    send("  Pushing develop to remote...");
    git.push()?;

    // Limpieza
    send(&format!("  Deleting local {branch} branch..."));
    git.delete_branch(&branch)?;

    send("Release finished successfully");

    Ok(())
}
