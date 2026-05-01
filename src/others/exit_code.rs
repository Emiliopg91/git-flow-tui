pub enum ExitCode {
    Ok = 0,
    NotAGitRepository = 1,
}
impl ExitCode {
    pub fn code(self) -> i32 {
        self as i32
    }
}
