use std::sync::mpsc::Sender;

use crate::git::{GitWrapper, errors::GitError};

pub fn feature_start(name: &String, sender: Sender<String>) -> Result<(), GitError> {
    let branch = format!("feature/{}", name);
    let git = GitWrapper::global().lock().unwrap();

    sender
        .send(format!("Starting creation of feature {}...", name))
        .unwrap();

    if git.get_branch().unwrap() != "develop" {
        sender
            .send("  Checking out develop branch...".to_string())
            .unwrap();
        git.checkout(&"develop".to_string())?;
    }
    sender
        .send("  Syncing changes from remote...".to_string())
        .unwrap();
    git.pull().unwrap();

    sender
        .send(format!("  Creating branch {}...", branch))
        .unwrap();
    git.create_branch(&branch)?;

    sender
        .send("Feature started succesfully".to_string())
        .unwrap();

    Ok(())
}

pub fn feature_finish(name: &String, sender: Sender<String>) -> Result<(), GitError> {
    let branch = format!("feature/{}", name);
    let git = GitWrapper::global().lock().unwrap();

    sender
        .send(format!("Finishing feature {}...", name))
        .unwrap();

    if git.get_branch().unwrap() != branch {
        sender
            .send(format!("  Checking out {} branch...", branch))
            .unwrap();
        git.checkout(&branch).unwrap();
    }

    if git.get_remote_branches().unwrap().contains(&branch) {
        sender
            .send("  Syncing changes from remote...".to_string())
            .unwrap();
        git.pull().unwrap();
    }

    sender
        .send(format!("  Pushing {} to remote...", branch))
        .unwrap();
    git.push().unwrap();

    sender
        .send("  Checking out develop branch...".to_string())
        .unwrap();
    git.checkout(&"develop".to_string()).unwrap();

    sender
        .send(format!("  Merging {} to develop...", branch))
        .unwrap();
    git.merge(&branch).unwrap();

    sender
        .send("  Creating commit for merge".to_string())
        .unwrap();
    git.commit_empty(&format!("Merge for feature merge: {}", name))
        .unwrap();

    sender
        .send("  Pushing develop to remote...".to_string())
        .unwrap();
    git.push().unwrap();

    sender
        .send(format!("  Deleting local {} branch...", branch))
        .unwrap();
    git.delete_branch(&branch).unwrap();

    sender
        .send("Feature finished succesfully".to_string())
        .unwrap();

    Ok(())
}
