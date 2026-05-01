use crate::git::GitWrapper;

mod git;
mod others;

fn main() {
    let git = GitWrapper::global().lock().unwrap();
    println!("Git repository found: {}", git.path);
    println!("Has changes?: {}", git.has_changes().unwrap());
}
