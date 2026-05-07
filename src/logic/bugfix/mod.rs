use std::sync::mpsc::Sender;

use crate::git::{GitWrapper, errors::GitError};

pub fn bugfix_start(name: &String, sender: Sender<String>) -> Result<(), GitError> {
    let branch = format!("bugfix/{}", name);
    let git = GitWrapper::global().lock().unwrap();

    let send = |msg: &str| {
        let _ = sender.send(msg.into());
    };

    send(&format!("Starting creation of bugfix {}...", name));

    if git.get_branch()? != "develop" {
        send("  Checking out develop branch...");
        git.checkout("develop")?;
    }
    send("  Syncing changes from remote...");
    git.pull()?;

    send(&format!("  Creating branch {}...", branch));
    git.create_branch(&branch)?;

    send("Bugfix started succesfully");

    Ok(())
}

pub fn bugfix_finish(name: &String, sender: Sender<String>) -> Result<(), GitError> {
    let branch = format!("bugfix/{}", name);
    let git = GitWrapper::global().lock().unwrap();

    let send = |msg: &str| {
        let _ = sender.send(msg.into());
    };

    send(&format!("Finishing bugfix {}...", name));

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
    git.commit(&format!("Merge after {} bugfix merge", name))?;

    send("  Pushing develop to remote...");
    git.push()?;

    send(&format!("  Deleting local {} branch...", branch));
    git.delete_branch(&branch)?;

    send("Bugfix finished succesfully");

    Ok(())
}
