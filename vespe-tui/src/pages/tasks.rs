use ratatui::{prelude::*, widgets::*};
use crossterm::event::KeyCode;
use crate::{App, Page, MessageType};
use vespe::Task;

#[derive(Debug, Default)]
pub struct TasksPageState {
    pub tasks: Vec<Task>,
    pub selected_task_index: usize,
}

pub fn render_tasks_page(frame: &mut Frame, area: Rect, state: &TasksPageState) {
    let tasks_items: Vec<ListItem> = state.tasks
        .iter()
        .enumerate()
        .map(|(i, task)| {
            let content = format!("{} - {}", task.config.uid, task.config.name);
            if i == state.selected_task_index {
                ListItem::new(content).style(Style::default().fg(Color::Black).bg(Color::LightBlue))
            } else {
                ListItem::new(content)
            }
        })
        .collect();

    let tasks_list = List::new(tasks_items)
        .block(Block::default().borders(Borders::ALL).title("Tasks"))
        .highlight_style(Style::default().add_modifier(Modifier::BOLD))
        .highlight_symbol("> ");

    frame.render_widget(tasks_list, area);
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
            if !app.tasks_page_state.tasks.is_empty() {
                let selected_task = &app.tasks_page_state.tasks[app.tasks_page_state.selected_task_index];
                app.task_edit_state.current_task_uid = Some(selected_task.uid.clone());
                app.task_edit_state.name = selected_task.config.name.clone();
                app.task_edit_state.objective = selected_task.objective.clone();
                app.task_edit_state.agent_uid = selected_task.config.created_by_agent_uid.clone();
                app.current_page = Page::TaskEdit;
                app.message = None;
            } else {
                app.message = Some("No task to edit.".to_string());
                app.message_type = MessageType::Info;
            }
        }
        KeyCode::Up => {
            if !app.tasks_page_state.tasks.is_empty() {
                if app.tasks_page_state.selected_task_index > 0 {
                    app.tasks_page_state.selected_task_index -= 1;
                }
            }
        }
        KeyCode::Down => {
            if !app.tasks_page_state.tasks.is_empty() {
                if app.tasks_page_state.selected_task_index < app.tasks_page_state.tasks.len() - 1 {
                    app.tasks_page_state.selected_task_index += 1;
                }
            }
        }
        _ => {},
    }
    Ok(())
}
