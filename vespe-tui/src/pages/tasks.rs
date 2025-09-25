use ratatui::{prelude::*, widgets::*};
use crossterm::event::KeyCode;
use crate::{App, Page};
use crate::MessageType;
use vespe::Task;
use tracing::{info, warn, error, debug};

#[derive(Debug)]
pub struct TasksPageState {
    pub tasks: Vec<Task>,
    pub selected_task_index: usize,
    pub last_selected_task_uid: Option<String>,
}

impl Default for TasksPageState {
    fn default() -> Self {
        Self {
            tasks: Vec::new(),
            selected_task_index: 0,
            last_selected_task_uid: None,
        }
    }
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
            info!("Tasks: KeyCode::F(5) (New Task) pressed.");
            // New Task
            app.task_edit_state = super::task_edit::TaskEditState::default();
            app.task_edit_state.mode = super::task_edit::TaskEditMode::Editing;
            let next_page = Page::TaskEdit;
            app.current_page = next_page;
            next_page.entering(app)?;
            app.message = None;
        }
        KeyCode::F(6) => {
            info!("Tasks: KeyCode::F(6) (Edit Task) pressed.");
            // Edit Task
            if !app.tasks_page_state.tasks.is_empty() {
                let selected_task = &app.tasks_page_state.tasks[app.tasks_page_state.selected_task_index];
                let selected_uid = selected_task.uid.clone();
                let selected_name = selected_task.config.name.clone();
                let selected_objective = selected_task.objective.clone();
                let selected_agent_uid = selected_task.config.created_by_agent_uid.clone();

                app.task_edit_state.current_task_uid = Some(selected_uid.clone());
                app.task_edit_state.name = selected_name;
                app.task_edit_state.objective = selected_objective;
                app.task_edit_state.agent_uid = selected_agent_uid;
                app.task_edit_state.mode = super::task_edit::TaskEditMode::ReadOnly;
                let next_page = Page::TaskEdit;
                app.current_page = next_page;
                next_page.entering(app)?;
                app.message = None;
                info!("Tasks: Loaded task {} for editing.", selected_uid);
            } else {
                app.message = Some("No task to edit.".to_string());
                app.message_type = MessageType::Info;
                warn!("Tasks: Attempted to edit with no tasks available.");
            }
        }
        KeyCode::Up => {
            info!("Tasks: KeyCode::Up pressed.");
            if !app.tasks_page_state.tasks.is_empty() {
                if app.tasks_page_state.selected_task_index > 0 {
                    app.tasks_page_state.selected_task_index -= 1;
                    app.tasks_page_state.last_selected_task_uid = Some(app.tasks_page_state.tasks[app.tasks_page_state.selected_task_index].uid.clone());
                    info!("Tasks: Selected task index: {}.", app.tasks_page_state.selected_task_index);
                }
            }
        }
        KeyCode::Down => {
            info!("Tasks: KeyCode::Down pressed.");
            if !app.tasks_page_state.tasks.is_empty() {
                if app.tasks_page_state.selected_task_index < app.tasks_page_state.tasks.len() - 1 {
                    app.tasks_page_state.selected_task_index += 1;
                    app.tasks_page_state.last_selected_task_uid = Some(app.tasks_page_state.tasks[app.tasks_page_state.selected_task_index].uid.clone());
                    info!("Tasks: Selected task index: {}.", app.tasks_page_state.selected_task_index);
                }
            }
        }
        _ => {},
    }
    Ok(())
}

pub fn load_tasks_into_state(app: &mut App) -> Result<(), anyhow::Error> {
    match app.project.list_all_tasks() {
        Ok(tasks) => {
            app.tasks_page_state.tasks = tasks;

            // Try to restore the last selected task
            if let Some(last_uid) = &app.tasks_page_state.last_selected_task_uid {
                if let Some(index) = app.tasks_page_state.tasks.iter().position(|t| &t.uid == last_uid) {
                    app.tasks_page_state.selected_task_index = index;
                } else if !app.tasks_page_state.tasks.is_empty() {
                    // If the last selected task is no longer available, select the first one
                    app.tasks_page_state.selected_task_index = 0;
                } else {
                    // No tasks available
                    app.tasks_page_state.selected_task_index = 0;
                }
            } else if !app.tasks_page_state.tasks.is_empty() {
                // If no last selected UID, but tasks are available, select the first one
                app.tasks_page_state.selected_task_index = 0;
            } else {
                // No tasks available
                app.tasks_page_state.selected_task_index = 0;
            }
            info!("Tasks loaded successfully.");
        }
        Err(e) => {
            app.message = Some(format!("Error loading tasks: {:?}", e));
            app.message_type = MessageType::Error;
            error!("Error loading tasks: {:?}", e);
        }
    }
    Ok(())
}
