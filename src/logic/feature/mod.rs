use std::sync::mpsc::Sender;

use crate::git::{GitWrapper, errors::GitError};

pub fn feature_start(name: &String, sender: Sender<String>) -> Result<(), GitError> {
    let branch = format!("feature/{}", name);
    let git = GitWrapper::global().lock().unwrap();

    let send = |msg: &str| {
        let _ = sender.send(msg.into());
    };

    send(&format!("Starting creation of feature {}...", name));

    if git.get_branch()? != "develop" {
        send("  Checking out develop branch...");
        git.checkout("develop")?;
    }
    send("  Syncing changes from remote...");
    git.pull()?;

    send(&format!("  Creating branch {}...", branch));
    git.create_branch(&branch)?;

    send("Feature started succesfully");

    Ok(())
}

pub fn feature_finish(name: &String, sender: Sender<String>) -> Result<(), GitError> {
    let branch = format!("feature/{}", name);
    let git = GitWrapper::global().lock().unwrap();

    let send = |msg: &str| {
        let _ = sender.send(msg.into());
    };

    send(&format!("Finishing feature {}...", name));

    if git.get_branch()? != branch {
        send(&format!("  Checking out {} branch...", branch));
        git.checkout(&branch)?;
    }

    if git.get_remote_branches()?.contains(&branch) {
        send("  Syncing changes from remote...");
        git.pull()?;
    }

    send(&format!("  Pushing {} to remote...", branch));
    git.push()?;

    send("  Checking out develop branch...");
    git.checkout("develop")?;

    send(&format!("  Merging {} to develop...", branch));
    git.merge(&branch)?;

    send("  Creating commit for merge");
    git.commit(&format!("Merge after {} feature merge", name))?;

    send("  Pushing develop to remote...");
    git.push()?;

    send(&format!("  Deleting local {} branch...", branch));
    git.delete_branch(&branch)?;

    send("Feature finished succesfully");

    Ok(())
}
