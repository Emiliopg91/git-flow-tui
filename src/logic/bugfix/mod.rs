use std::sync::mpsc::Sender;

use crate::git::{errors::GitError, GitWrapper};

pub fn bugfix_start(name: &String, sender: Sender<String>) -> Result<(), GitError> {
    let branch = format!("bugfix/{}", name);
    let git = GitWrapper::global().lock().unwrap();

    sender
        .send(format!("Starting creation of bugfix {}...", name))
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
        .send("Bugfix started succesfully".to_string())
        .unwrap();

    Ok(())
}

pub fn bugfix_finish(name: &String, sender: Sender<String>) -> Result<(), GitError> {
    let branch = format!("bugfix/{}", name);
    let git = GitWrapper::global().lock().unwrap();

    sender
        .send(format!("Finishing bugfix {}...", name))
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
    git.commit(&format!("Merge after {} bugfix merge", name))
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
        .send("Bugfix finished succesfully".to_string())
        .unwrap();

    Ok(())
}
