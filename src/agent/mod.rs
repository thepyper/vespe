pub struct ShellAgentCall {
    command: String,
}

impl ShellAgentCall {
    pub fn new(command: String) -> Self {
        ShellAgentCall { command }
    }
}
