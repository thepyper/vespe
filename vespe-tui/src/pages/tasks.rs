use ratatui::{prelude::*, widgets::*};
use crossterm::event::KeyCode;
use crate::{App, Page, MessageType};

pub fn render_tasks_page(frame: &mut Frame, area: Rect) {
    frame.render_widget(
        Paragraph::new("Tasks Page Content")
            .white()
            .on_dark_gray(),
        area,
    );
}

pub fn handle_events(app: &mut App, key_code: KeyCode) -> Result<(), anyhow::Error> {
    match key_code {
        KeyCode::F(5) => {
            // New Task
            app.task_edit_state = super::task_edit::TaskEditState::default();
            app.current_page = Page::TaskEdit;
            app.message = None;
        }
        KeyCode::F(6) => {
            // Edit Task
            // TODO: Load selected task into task_edit_state
            app.message = Some("Edit functionality not yet implemented. Please select a task first.".to_string());
            app.message_type = MessageType::Info;
            // app.current_page = Page::TaskEdit;
        }
        _ => {},
    }
    Ok(())
}
