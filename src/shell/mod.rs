use std::process::Command;

use crate::shell::errors::ShellError;

pub mod errors;

pub struct ShellResult {
    pub status: i32,
    pub stdout: String,
    #[allow(dead_code)]
    pub stderr: String,
}

pub fn run_shell_command<I, S>(
    executable: &str,
    args: I,
    check: bool,
) -> Result<ShellResult, ShellError>
where
    I: IntoIterator<Item = S>,
    S: AsRef<str>,
{
    let args_vec: Vec<String> = args.into_iter().map(|s| s.as_ref().to_string()).collect();

    let res = Command::new(executable)
        .args(&args_vec)
        .env("LANG", "C")
        .output()?;

    let status = res
        .status
        .code()
        .ok_or_else(|| ShellError::UnexpectedExecutionEnd())?;

    let res = ShellResult {
        status,
        stdout: String::from_utf8_lossy(&res.stdout).trim().to_string(),
        stderr: String::from_utf8_lossy(&res.stderr).trim().to_string(),
    };

    if check && status != 0 {
        return Err(ShellError::BadExitStatus(status, res.stderr));
    }

    Ok(res)
}
