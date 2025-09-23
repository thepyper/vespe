use anyhow::Result;
use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind},
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};
use ratatui::{prelude::*, widgets::*};
use std::io::{stdout, Stdout};

#[derive(Debug, Default, PartialEq, Eq, Clone, Copy)]
enum Page {
    #[default]
    Tasks,
    Tools,
    Agents,
    Chat,
}

impl Page {
    fn title(&self) -> &str {
        match self {
            Page::Tasks => "Tasks",
            Page::Tools => "Tools",
            Page::Agents => "Agents",
            Page::Chat => "Chat",
        }
    }

    fn color(&self) -> Color {
        match self {
            Page::Tasks => Color::LightBlue,
            Page::Tools => Color::LightCyan,
            Page::Agents => Color::LightMagenta,
            Page::Chat => Color::LightYellow,
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
        }
    }
}

#[derive(Debug, Default)]
struct App {
    current_page: Page,
}

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
                Page::Tasks => render_tasks_page(frame, layout[0]),
                Page::Tools => render_tools_page(frame, layout[0]),
                Page::Agents => render_agents_page(frame, layout[0]),
                Page::Chat => render_chat_page(frame, layout[0]),
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
                    // F5-F8 will be handled by page-specific logic later
                    _ => {},
                }
            }
        }
    }
    Ok(false)
}

fn render_tasks_page(frame: &mut Frame, area: Rect) {
    frame.render_widget(
        Paragraph::new("Tasks Page Content")
            .white()
            .on_dark_gray(),
        area,
    );
}

fn render_tools_page(frame: &mut Frame, area: Rect) {
    frame.render_widget(
        Paragraph::new("Tools Page Content")
            .white()
            .on_dark_gray(),
        area,
    );
}

fn render_agents_page(frame: &mut Frame, area: Rect) {
    frame.render_widget(
        Paragraph::new("Agents Page Content")
            .white()
            .on_dark_gray(),
        area,
    );
}

fn render_chat_page(frame: &mut Frame, area: Rect) {
    frame.render_widget(
        Paragraph::new("Chat Page Content")
            .white()
            .on_dark_gray(),
        area,
    );
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

    for (i, (label, _key_code)) in actions.iter().enumerate() {
        let text = format!("F{} {}", i + 5, label);
        let paragraph = Paragraph::new(text)
            .style(Style::default().fg(Color::White).bg(Color::DarkGray))
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
        let background_color = if is_selected { page.color() } else { Color::DarkGray };
        let foreground_color = if is_selected { Color::Black } else { Color::White };

        let text = format!("F{} {}", i + 1, page.title());
        let paragraph = Paragraph::new(text)
            .style(Style::default().fg(foreground_color).bg(background_color))
            .alignment(Alignment::Center);
        frame.render_widget(paragraph, chunks[i]);
    }
}
