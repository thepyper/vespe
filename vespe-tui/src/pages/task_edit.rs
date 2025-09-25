use ratatui::{prelude::*, widgets::*};
use crossterm::event::KeyCode;
use crate::{App, MessageType};
use tracing::{info, warn, error, debug};

#[derive(Debug, Default, PartialEq, Eq, Clone, Copy)]
pub enum InputFocus {
    #[default]
    Name,
    Objective,
    AgentUid,
}

impl InputFocus {
    pub fn next(&self) -> Self {
        match self {
            InputFocus::Name => InputFocus::Objective,
            InputFocus::Objective => InputFocus::AgentUid,
            InputFocus::AgentUid => InputFocus::Name,
        }
    }
}

#[derive(Debug, Default)]
pub struct TaskEditState {
    pub current_task_uid: Option<String>,
    pub name: String,
    pub objective: String,
    pub agent_uid: String,
    pub input_focus: InputFocus,
}

pub fn render_task_edit_page(frame: &mut Frame, area: Rect, state: &TaskEditState) {
    let layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints(vec![
            Constraint::Length(3), // Name
            Constraint::Min(5),    // Objective
            Constraint::Length(3), // Agent UID
        ])
        .split(area);

    let name_block = Block::default().borders(Borders::ALL).title("Name");
    let name_paragraph = Paragraph::new(state.name.as_str()).block(if state.input_focus == InputFocus::Name { name_block.border_style(Style::default().fg(Color::Yellow)) } else { name_block });
    frame.render_widget(name_paragraph, layout[0]);

    let objective_block = Block::default().borders(Borders::ALL).title("Objective");
    let objective_paragraph = Paragraph::new(state.objective.as_str()).block(if state.input_focus == InputFocus::Objective { objective_block.border_style(Style::default().fg(Color::Yellow)) } else { objective_block });
    frame.render_widget(objective_paragraph, layout[1]);

    let agent_uid_block = Block::default().borders(Borders::ALL).title("Agent UID");
    let agent_uid_paragraph = Paragraph::new(state.agent_uid.as_str()).block(if state.input_focus == InputFocus::AgentUid { agent_uid_block.border_style(Style::default().fg(Color::Yellow)) } else { agent_uid_block });
    frame.render_widget(agent_uid_paragraph, layout[2]);
}

pub fn handle_events(app: &mut App, key_code: KeyCode) -> Result<(), anyhow::Error> {
    match key_code {
        KeyCode::Char(c) => {
            info!("TaskEdit: KeyCode::Char({}) pressed.", c);
            match app.task_edit_state.input_focus {
                InputFocus::Name => app.task_edit_state.name.push(c),
                InputFocus::Objective => app.task_edit_state.objective.push(c),
                InputFocus::AgentUid => app.task_edit_state.agent_uid.push(c),
            }
        }
        KeyCode::Backspace => {
            info!("TaskEdit: KeyCode::Backspace pressed.");
            match app.task_edit_state.input_focus {
                InputFocus::Name => {
                    app.task_edit_state.name.pop();
                }
                InputFocus::Objective => {
                    app.task_edit_state.objective.pop();
                }
                InputFocus::AgentUid => {
                    app.task_edit_state.agent_uid.pop();
                }
            }
        }
        KeyCode::Tab => {
            info!("TaskEdit: KeyCode::Tab pressed. Changing focus.");
            app.task_edit_state.input_focus = app.task_edit_state.input_focus.next();
        }
        KeyCode::F(5) => {
            info!("TaskEdit: KeyCode::F(5) pressed. Saving task.");
            if let Some(uid) = &app.task_edit_state.current_task_uid {
                match app.project.update_task(
                    uid,
                    app.task_edit_state.name.clone(),
                    app.task_edit_state.objective.clone(),
                ) {
                    Ok(_) => {
                        app.message = Some("Task saved successfully.".to_string());
                        app.message_type = MessageType::Success;
                        app.current_page = crate::Page::Tasks;
                        crate::pages::tasks::load_tasks_into_state(app)?;
                    }
                    Err(e) => {
                        app.message = Some(format!("Failed to save task: {}", e));
                        app.message_type = MessageType::Error;
                    }
                }
            }
        }
        KeyCode::F(6) => {
            info!("TaskEdit: KeyCode::F(6) pressed. Cancelling edit.");
            app.current_page = crate::Page::Tasks;
        }
        _ => {},
    }
    Ok(())
}
