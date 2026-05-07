use std::sync::mpsc::Sender;

use crate::git::{GitWrapper, errors::GitError};

pub fn hotfix_start(name: &str, sender: Sender<String>) -> Result<(), GitError> {
    let branch = format!("hotfix/{}", name);
    let git = GitWrapper::global().lock().unwrap();

    let send = |msg: &str| {
        let _ = sender.send(msg.into());
    };

    send(&format!("Starting creation of hotfix {}...", name));

    if git.get_branch()? != git.main_branch {
        send("  Checking out main branch...");
        git.checkout(&git.main_branch)?;
    }
    send("  Syncing changes from remote...");
    git.pull()?;

    send(&format!("  Creating branch {}...", branch));
    git.create_branch(&branch)?;

    send("Hotfix started succesfully");

    Ok(())
}

pub fn hotfix_finish(name: &str, sender: Sender<String>) -> Result<(), GitError> {
    let branch = format!("hotfix/{}", name);
    let git = GitWrapper::global().lock().unwrap();

    let send = |msg: &str| {
        let _ = sender.send(msg.into());
    };

    send(&format!("Finishing hotfix {}...", name));

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

    send("  Checking out main branch...");
    git.checkout(&git.main_branch)?;

    send("  Syncing changes from remote...");
    git.pull()?;

    send(&format!("  Merging {} to main branch...", branch));
    git.merge(&branch)?;

    send("  Creating commit for merge");
    git.commit(&format!("Merge after {} hotfix merge", name))?;

    send("  Pushing main branch to remote...");
    git.push()?;

    send("  Checking out develop branch...");
    git.checkout("develop")?;

    send("  Syncing changes from remote...");
    git.pull()?;

    send("  Merging main branch to develop branch...");
    git.merge(&git.main_branch)?;

    send("  Pushing develop branch to remote...");
    git.push()?;

    send(&format!("  Deleting local {} branch...", branch));
    git.delete_branch(&branch)?;

    send("Hotfix finished succesfully");

    Ok(())
}
