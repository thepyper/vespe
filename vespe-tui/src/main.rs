use anyhow::Result;
use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind},
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};
use ratatui::{prelude::*, widgets::*};
use std::io::stdout;
use tracing::{info, warn, error, debug};
use crate::pages::task_edit::InputFocus;
use vespe::{Project, ProjectError};
use tracing_subscriber::{fmt, prelude::*};
use std::sync::Arc;

// Color Constants
const TASKS_COLOR: Color = Color::LightBlue;
const TOOLS_COLOR: Color = Color::LightCyan;
const AGENTS_COLOR: Color = Color::LightMagenta;
const CHAT_COLOR: Color = Color::LightYellow;
const DEFAULT_FOOTER_BG_COLOR: Color = Color::Rgb(0x22, 0x22, 0x22);
const SELECTED_FOOTER_FG_COLOR: Color = Color::Black;
const DEFAULT_FOOTER_FG_COLOR: Color = Color::White;
const _QUIT_BUTTON_BG_COLOR: Color = Color::Red;
const TASK_EDIT_VIEW_COLOR: Color = Color::Rgb(0x44, 0x44, 0x44);
const TASK_EDIT_EDIT_COLOR: Color = Color::Rgb(0x66, 0x44, 0x44);

#[derive(Debug, Default, PartialEq, Eq, Clone, Copy)]
pub enum Page {
    #[default]
    Tasks,
    Tools,
    Agents,
    Chat,
    TaskEdit,
}

impl Page {
    fn title(&self) -> &str {
        match self {
            Page::Tasks => "Tasks",
            Page::Tools => "Tools",
            Page::Agents => "Agents",
            Page::Chat => "Chat",
            Page::TaskEdit => "Edit Task",
        }
    }

    fn color(&self, task_edit_state: &pages::task_edit::TaskEditState) -> Color {
        match self {
            Page::Tasks => TASKS_COLOR,
            Page::Tools => TOOLS_COLOR,
            Page::Agents => AGENTS_COLOR,
            Page::Chat => CHAT_COLOR,
            Page::TaskEdit => match task_edit_state.mode {
                pages::task_edit::TaskEditMode::ReadOnly => TASK_EDIT_VIEW_COLOR,
                pages::task_edit::TaskEditMode::Editing => TASK_EDIT_EDIT_COLOR,
            },
        }
    }

    fn page_footer_actions(&self, task_edit_state: &pages::task_edit::TaskEditState) -> Vec<(&str, KeyCode)> {
        match self {
            Page::Tasks => vec![
                ("New", KeyCode::F(5)),
                ("Inspect", KeyCode::F(6)),
                ("Delete", KeyCode::F(7)),
                ("View", KeyCode::F(8)),
            ],
            Page::Tools => vec![
                ("New", KeyCode::F(5)),
                ("Edit", KeyCode::F(6)),
                ("Delete", KeyCode::F(7)),
                ("Run", KeyCode::F(8)),
            ],
            Page::Agents => vec![
                ("New", KeyCode::F(5)),
                ("Edit", KeyCode::F(6)),
                ("Delete", KeyCode::F(7)),
                ("Interact", KeyCode::F(8)),
            ],
            Page::Chat => vec![
                ("Send", KeyCode::F(5)),
                ("History", KeyCode::F(6)),
                ("Clear", KeyCode::F(7)),
                ("Config", KeyCode::F(8)),
            ],
            Page::TaskEdit => match task_edit_state.mode {
                pages::task_edit::TaskEditMode::ReadOnly => vec![
                    ("Back", KeyCode::F(5)),
                    ("Edit", KeyCode::F(6)),
                ],
                pages::task_edit::TaskEditMode::Editing => vec![
                    ("Cancel", KeyCode::F(5)),
                    ("Save", KeyCode::F(6)),
                ],
            },
        }
    }

    pub fn entering(&self, app: &mut App) -> Result<()> {
        match self {
            Page::Tasks => {
                pages::tasks::load_tasks_into_state(app)?;
            }
            _ => {}
        }
        Ok(())
    }
}

#[derive(Debug, Default, PartialEq, Eq, Clone, Copy)]
enum Mode {
    #[default]
    Normal,
    Confirming,
}

struct Confirmation {
    message: String,
    action: Arc<dyn Fn(&mut App) -> Result<()>>,
}

impl std::fmt::Debug for Confirmation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Confirmation")
            .field("message", &self.message)
            .finish()
    }
}

#[derive(Debug)]
pub struct App {
    mode: Mode,
    current_page: Page,
    task_edit_state: pages::task_edit::TaskEditState,
    tasks_page_state: pages::tasks::TasksPageState,
    project: Project,
    message: Option<String>,
    message_type: MessageType,
    confirmation: Option<Confirmation>,
}

#[derive(Debug, Default, PartialEq, Eq, Clone, Copy)]
pub enum MessageType {
    #[default]
    Info,
    Success,
    Error,
}

impl Default for App {
    fn default() -> Self {
        let project = Project::find_root(&std::env::current_dir().unwrap()).expect("Failed to find project root");
        Self {
            mode: Mode::default(),
            current_page: Page::default(),
            task_edit_state: pages::task_edit::TaskEditState::default(),
            tasks_page_state: pages::tasks::TasksPageState::default(),
            project,
            message: None,
            message_type: MessageType::default(),
            confirmation: None,
        }
    }
}

mod pages;

fn main() -> Result<()> {
    let mut app = App::default();

    let log_dir = app.project.log_dir().join("tui");
    std::fs::create_dir_all(&log_dir)?;
    let log_file = tracing_appender::rolling::daily(log_dir, "tui.log");
    let (non_blocking_writer, _guard) = tracing_appender::non_blocking(log_file);

    tracing_subscriber::registry()
        .with(fmt::layer().with_writer(non_blocking_writer))
        .init();

    info!("Application started.");
    // setup terminal
    enable_raw_mode()?;
    stdout().execute(EnterAlternateScreen)?;
    let mut terminal = Terminal::new(CrosstermBackend::new(stdout()))?;

    pages::tasks::load_tasks_into_state(&mut app)?;

    // run app
    let mut should_quit = false;
    while !should_quit {
        terminal.draw(|frame| {
            let layout = Layout::default()
                .direction(Direction::Vertical)
                .constraints(vec![
                    Constraint::Min(1),      // Main content area
                    Constraint::Length(1),   // Page-specific F5-F8 footer
                    Constraint::Length(1),   // Global F1-F4 footer
                ])
                .split(frame.size());

            // Render main content based on current page
            match app.current_page {
                Page::Tasks => pages::tasks::render_tasks_page(frame, layout[0], &app.tasks_page_state),
                Page::Tools => pages::tools::render_tools_page(frame, layout[0]),
                Page::Agents => pages::agents::render_agents_page(frame, layout[0]),
                Page::Chat => pages::chat::render_chat_page(frame, layout[0]),
                Page::TaskEdit => pages::task_edit::render_task_edit_page(frame, layout[0], &app.task_edit_state),
            }

            // Render page-specific F5-F8 footer
            render_page_footer(frame, layout[1], &app);

            // Render global F1-F4 footer
            render_global_footer(frame, layout[2], &app);

            if app.mode == Mode::Confirming {
                if let Some(confirmation) = &app.confirmation {
                    render_confirmation_dialog(frame, &confirmation.message);
                }
            }
        })?;
        should_quit = handle_events(&mut app)?;
    }

    // restore terminal
    stdout().execute(LeaveAlternateScreen)?;
    disable_raw_mode()?;
    Ok(())
}

fn handle_events(app: &mut App) -> Result<bool> {
    if event::poll(std::time::Duration::from_millis(50))? {
        if let Event::Key(key) = event::read()? {
            if key.kind == KeyEventKind::Press {
                match app.mode {
                    Mode::Normal => {
                        match key.code {
                            KeyCode::Char('q') => return Ok(true),
                            KeyCode::F(1) => {
                                let next_page = Page::Tasks;
                                app.current_page = next_page;
                                next_page.entering(app)?;
                            }
                            KeyCode::F(2) => {
                                let next_page = Page::Tools;
                                app.current_page = next_page;
                                next_page.entering(app)?;
                            }
                            KeyCode::F(3) => {
                                let next_page = Page::Agents;
                                app.current_page = next_page;
                                next_page.entering(app)?;
                            }
                            KeyCode::F(4) => {
                                let next_page = Page::Chat;
                                app.current_page = next_page;
                                next_page.entering(app)?;
                            }
                            _ => {
                                // Delegate page-specific events
                                match app.current_page {
                                    Page::Tasks => pages::tasks::handle_events(app, key.code)?,
                                    Page::TaskEdit => pages::task_edit::handle_events(app, key.code)?,
                                    _ => {},
                                }
                            }
                        }
                    }
                    Mode::Confirming => {
                        match key.code {
                            KeyCode::Char('y') | KeyCode::Char('Y') => {
                                if let Some(confirmation) = app.confirmation.take() {
                                    (confirmation.action)(app)?;
                                }
                                app.mode = Mode::Normal;
                            }
                            KeyCode::Char('n') | KeyCode::Char('N') => {
                                app.mode = Mode::Normal;
                                app.confirmation = None;
                            }
                            _ => {}
                        }
                    }
                }
            }
        }
    }
    Ok(false)
}

fn render_page_footer(frame: &mut Frame, area: Rect, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints(vec![
            Constraint::Percentage(25),
            Constraint::Percentage(25),
            Constraint::Percentage(25),
            Constraint::Percentage(25),
        ])
        .split(area);

    let actions = app.current_page.page_footer_actions(&app.task_edit_state);
    let background_color = app.current_page.color(&app.task_edit_state);

    for (i, (label, _key_code)) in actions.iter().enumerate() {
        let text = format!("F{} {}", i + 5, label);
        let paragraph = Paragraph::new(text)
            .style(Style::default().fg(SELECTED_FOOTER_FG_COLOR).bg(background_color))
            .alignment(Alignment::Center);
        frame.render_widget(paragraph, chunks[i]);
    }
}

fn render_global_footer(frame: &mut Frame, area: Rect, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints(vec![
            Constraint::Percentage(25),
            Constraint::Percentage(25),
            Constraint::Percentage(25),
            Constraint::Percentage(25),
        ])
        .split(area);

    let pages = [
        (Page::Tasks, KeyCode::F(1)),
        (Page::Tools, KeyCode::F(2)),
        (Page::Agents, KeyCode::F(3)),
        (Page::Chat, KeyCode::F(4)),
    ];

    for (i, (page, _key_code)) in pages.iter().enumerate() {
        let is_selected = page == &app.current_page;
        let background_color = if is_selected { page.color(&app.task_edit_state) } else { DEFAULT_FOOTER_BG_COLOR };
        let foreground_color = if is_selected { SELECTED_FOOTER_FG_COLOR } else { DEFAULT_FOOTER_FG_COLOR };

        let text = format!("F{} {}", i + 1, page.title());
        let paragraph = Paragraph::new(text)
            .style(Style::default().fg(foreground_color).bg(background_color))
            .alignment(Alignment::Center);
        frame.render_widget(paragraph, chunks[i]);
    }
}

fn render_confirmation_dialog(frame: &mut Frame, message: &str) {
    let area = centered_rect(60, 20, frame.size());
    let block = Block::default()
        .title("Confirm")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Yellow));

    let text = format!("{} (Y/n)", message);
    let paragraph = Paragraph::new(text)
        .wrap(Wrap { trim: true })
        .block(block);
    
    frame.render_widget(Clear, area); //this clears the background
    frame.render_widget(paragraph, area);
}

/// helper function to create a centered rect using up certain percentage of the available rect `r`
fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}

pub fn request_confirmation<F>(app: &mut App, message: String, action: F)
where
    F: Fn(&mut App) -> Result<()> + 'static,
{
    app.confirmation = Some(Confirmation {
        message,
        action: Arc::new(action),
    });
    app.mode = Mode::Confirming;
}