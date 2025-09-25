use anyhow::Result;
use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind},
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};
use ratatui::{prelude::*, widgets::*};
use std::io::stdout;
use crate::pages::task_edit::InputFocus;
use vespe::{Project, ProjectError};

// Color Constants
const TASKS_COLOR: Color = Color::LightBlue;
const TOOLS_COLOR: Color = Color::LightCyan;
const AGENTS_COLOR: Color = Color::LightMagenta;
const CHAT_COLOR: Color = Color::LightYellow;
const DEFAULT_FOOTER_BG_COLOR: Color = Color::Rgb(0x22, 0x22, 0x22);
const SELECTED_FOOTER_FG_COLOR: Color = Color::Black;
const DEFAULT_FOOTER_FG_COLOR: Color = Color::White;
const _QUIT_BUTTON_BG_COLOR: Color = Color::Red;

#[derive(Debug, Default, PartialEq, Eq, Clone, Copy)]
enum Page {
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

    fn color(&self) -> Color {
        match self {
            Page::Tasks => TASKS_COLOR,
            Page::Tools => TOOLS_COLOR,
            Page::Agents => AGENTS_COLOR,
            Page::Chat => CHAT_COLOR,
            Page::TaskEdit => Color::LightGreen,
        }
    }

    fn page_footer_actions(&self) -> Vec<(&str, KeyCode)> {
        match self {
            Page::Tasks => vec![
                ("New", KeyCode::F(5)),
                ("Edit", KeyCode::F(6)),
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
            Page::TaskEdit => vec![
                ("Save", KeyCode::F(5)),
                ("Cancel", KeyCode::F(6)),
            ],
        }
    }
}

#[derive(Debug)]
struct App {
    current_page: Page,
    task_edit_state: pages::task_edit::TaskEditState,
    tasks_page_state: pages::tasks::TasksPageState,
    project: Project,
    message: Option<String>,
    message_type: MessageType,
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
        Self {
            current_page: Page::default(),
            task_edit_state: pages::task_edit::TaskEditState::default(),
            tasks_page_state: pages::tasks::TasksPageState::default(),
            project: Project::load(std::path::Path::new("h:\\my\\github\\vespe")).expect("Failed to load project"), // TODO: Use a proper path
            message: None,
            message_type: MessageType::default(),
        }
    }
}

mod pages;

fn main() -> Result<()> {
    // setup terminal
    enable_raw_mode()?;
    stdout().execute(EnterAlternateScreen)?;
    let mut terminal = Terminal::new(CrosstermBackend::new(stdout()))?;

    let mut app = App::default();

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
                Page::Chat => pages::chat::render_chat_page(frame, layout[1]),
                Page::TaskEdit => pages::task_edit::render_task_edit_page(frame, layout[1], &app.task_edit_state),
            }

            // Render page-specific F5-F8 footer
            render_page_footer(frame, layout[1], &app.current_page);

            // Render global F1-F4 footer
            render_global_footer(frame, layout[2], &app.current_page);
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
                match key.code {
                    KeyCode::Char('q') => return Ok(true),
                    KeyCode::F(1) => {
                        app.current_page = Page::Tasks;
                        app.message = None;
                        match app.project.list_all_tasks() {
                            Ok(tasks) => {
                                app.tasks_page_state.tasks = tasks;
                                app.tasks_page_state.selected_task_index = 0;
                            }
                            Err(e) => {
                                app.message = Some(format!("Error loading tasks: {:?}", e));
                                app.message_type = MessageType::Error;
                            }
                        }
                    }
                    KeyCode::F(2) => {
                        app.current_page = Page::Tools;
                        app.message = None;
                    }
                    KeyCode::F(3) => {
                        app.current_page = Page::Agents;
                        app.message = None;
                    }
                    KeyCode::F(4) => {
                        app.current_page = Page::Chat;
                        app.message = None;
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
        }
    }
    Ok(false)
}

fn render_page_footer(frame: &mut Frame, area: Rect, current_page: &Page) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints(vec![
            Constraint::Percentage(25),
            Constraint::Percentage(25),
            Constraint::Percentage(25),
            Constraint::Percentage(25),
        ])
        .split(area);

    let actions = current_page.page_footer_actions();
    let background_color = current_page.color();

    for (i, (label, _key_code)) in actions.iter().enumerate() {
        let text = format!("F{} {}", i + 5, label);
        let paragraph = Paragraph::new(text)
            .style(Style::default().fg(SELECTED_FOOTER_FG_COLOR).bg(background_color))
            .alignment(Alignment::Center);
        frame.render_widget(paragraph, chunks[i]);
    }
}

fn render_global_footer(frame: &mut Frame, area: Rect, current_page: &Page) {
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
        let is_selected = page == current_page;
        let background_color = if is_selected { page.color() } else { DEFAULT_FOOTER_BG_COLOR };
        let foreground_color = if is_selected { SELECTED_FOOTER_FG_COLOR } else { DEFAULT_FOOTER_FG_COLOR };

        let text = format!("F{} {}", i + 1, page.title());
        let paragraph = Paragraph::new(text)
            .style(Style::default().fg(foreground_color).bg(background_color))
            .alignment(Alignment::Center);
        frame.render_widget(paragraph, chunks[i]);
    }
}