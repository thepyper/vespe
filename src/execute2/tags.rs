

struct 


#[enum_dispatch]
pub trait CommandBehavior {
    fn from_json(json: &serde_json::Value) -> Self
    {
        serde_json::from_value(json)
    }
    fn load(&mut self, path: &Path) -> Result<Self>
    {
        let json = std::fs::read_file(path)?;
        serde_json::from_str(&json)
    }
    fn execute(&mut self);
    fn description(&self) -> String;
}

impl CommandBehavior for AnswerState {
    fn execute(&mut self, worker: &execute::Worker, collector: Collector) -> Result<(Collector, Option<) {
    }
}

#[enum_dispatch(CommandBehavior)]
#[derive(Debug)]
pub enum CommandState {
    DummyState(DummyState),
    Answer(AnswerState),
    Include(IncludeState),
    Derive(DeriveState),
}

pub fn create_command_state_from_command_kind(kind: CommandKind) -> CommandState {
    match kind {
        CommandKind::Answer => CommandState::Answer(AnswerState::default()),
         CommandKind::Include => CommandState::Include(IncludeState),
         CommandKind::Derive => CommandState::Derive(DeriveState::default()),
     }
}