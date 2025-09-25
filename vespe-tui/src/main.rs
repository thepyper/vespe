use anyhow::Result;
use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind},
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};
use ratatui::{prelude::*, widgets::*};
use crate::pages::create_task::{InputFocus, PRIORITIES};
use vespe::{Project, VespeError};
use std::io::stdout;

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
    CreateTask,
}

impl Page {
    fn title(&self) -> &str {
        match self {
            Page::Tasks => "Tasks",
            Page::Tools => "Tools",
            Page::Agents => "Agents",
            Page::Chat => "Chat",
            Page::CreateTask => "Create Task",
        }
    }

    fn color(&self) -> Color {
        match self {
            Page::Tasks => TASKS_COLOR,
            Page::Tools => TOOLS_COLOR,
            Page::Agents => AGENTS_COLOR,
            Page::Chat => CHAT_COLOR,
            Page::CreateTask => Color::LightGreen,
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
            Page::CreateTask => vec![
                ("Save", KeyCode::F(5)),
                ("Cancel", KeyCode::F(6)),
            ],
        }
    }
}

#[derive(Debug, Default)]
struct App {
    current_page: Page,
    create_task_state: pages::create_task::CreateTaskState,
    project: Project,
}

impl Default for App {
    fn default() -> Self {
        Self {
            current_page: Page::default(),
            create_task_state: pages::create_task::CreateTaskState::default(),
            project: Project::new("h:\\my\\github\\vespe\").expect("Failed to create project"), // TODO: Use a proper path
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
                Page::Tasks => pages::tasks::render_tasks_page(frame, layout[0]),
                Page::Tools => pages::tools::render_tools_page(frame, layout[0]),
                Page::Agents => pages::agents::render_agents_page(frame, layout[0]),
                Page::Chat => pages::chat::render_chat_page(frame, layout[0]),
                Page::CreateTask => pages::create_task::render_create_task_page(frame, layout[0], &app.create_task_state),
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
                    KeyCode::F(1) => app.current_page = Page::Tasks,
                    KeyCode::F(2) => app.current_page = Page::Tools,
                    KeyCode::F(3) => app.current_page = Page::Agents,
                    KeyCode::F(4) => app.current_page = Page::Chat,
                    KeyCode::F(5) => {
                        match app.current_page {
                            Page::Tasks => app.current_page = Page::CreateTask,
                            Page::CreateTask => {
                                // Handle Save action for CreateTask page
                                let title = app.create_task_state.title.clone();
                                let description = if app.create_task_state.description.is_empty() { None } else { Some(app.create_task_state.description.clone()) };
                                let priority = Some(vespe::TaskPriority::from_usize(app.create_task_state.priority));
                                // TODO: Parse due_date and tags
                                let due_date = None;
                                let tags = None;

                                match app.project.create_task(title, description, priority, due_date, tags) {
                                    Ok(task) => {
                                        // TODO: Show success message in TUI
                                        println!("Task created: {:?}", task);
                                        app.current_page = Page::Tasks;
                                        app.create_task_state = pages::create_task::CreateTaskState::default(); // Reset form
                                    }
                                    Err(e) => {
                                        // TODO: Show error message in TUI
                                        eprintln!("Error creating task: {:?}", e);
                                    }
                                }
                            }
                            _ => {},
                        }
                    }
                    KeyCode::F(6) => {
                        match app.current_page {
                            Page::CreateTask => {
                                // Handle Cancel action for CreateTask page
                                // For now, just go back to Tasks page
                                app.current_page = Page::Tasks;
                            }
                            _ => {},
                        }
                    KeyCode::Char(c) => {
                        if app.current_page == Page::CreateTask {
                            match app.create_task_state.input_focus {
                                pages::create_task::InputFocus::Title => app.create_task_state.title.push(c),
                                pages::create_task::InputFocus::Description => app.create_task_state.description.push(c),
                                pages::create_task::InputFocus::DueDate => app.create_task_state.due_date.push(c),
                                pages::create_task::InputFocus::Tags => app.create_task_state.tags.push(c),
                                _ => {},
                            }
                        }
                    }
                    KeyCode::Backspace => {
                        if app.current_page == Page::CreateTask {
                            match app.create_task_state.input_focus {
                                pages::create_task::InputFocus::Title => {
                                    app.create_task_state.title.pop();
                                }
                                pages::create_task::InputFocus::Description => {
                                    app.create_task_state.description.pop();
                                }
                                pages::create_task::InputFocus::DueDate => {
                                    app.create_task_state.due_date.pop();
                                }
                                pages::create_task::InputFocus::Tags => {
                                    app.create_task_state.tags.pop();
                                }
                                _ => {},
                            }
                        }
                    }
                    KeyCode::Tab => {
                        if app.current_page == Page::CreateTask {
                            app.create_task_state.input_focus = app.create_task_state.input_focus.next();
                        }
                    }
                    KeyCode::Up => {
                        if app.current_page == Page::CreateTask {
                            if let pages::create_task::InputFocus::Priority = app.create_task_state.input_focus {
                                if app.create_task_state.priority > 0 {
                                    app.create_task_state.priority -= 1;
                                }
                            }
                        }
                    }
                    KeyCode::Down => {
                        if app.current_page == Page::CreateTask {
                            if let pages::create_task::InputFocus::Priority = app.create_task_state.input_focus {
                                if app.create_task_state.priority < pages::create_task::PRIORITIES.len() - 1 {
                                    app.create_task_state.priority += 1;
                                }
                            }
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