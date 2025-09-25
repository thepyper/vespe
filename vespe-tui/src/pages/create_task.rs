use ratatui::{prelude::*, widgets::*};

pub const PRIORITIES: &[&str] = &["Low", "Medium", "High"];

pub struct CreateTaskState {
    pub title: String,
    pub description: String,
    pub priority: usize, // Index into a list of priorities
    pub due_date: String,
    pub tags: String,
    pub input_focus: InputFocus,
}

impl Default for CreateTaskState {
    fn default() -> Self {
        Self {
            title: String::new(),
            description: String::new(),
            priority: 0,
            due_date: String::new(),
            tags: String::new(),
            input_focus: InputFocus::Title,
        }
    }
}

pub enum InputFocus {
    Title,
    Description,
    Priority,
    DueDate,
    Tags,
}

impl InputFocus {
    pub fn next(&self) -> Self {
        match self {
            InputFocus::Title => InputFocus::Description,
            InputFocus::Description => InputFocus::Priority,
            InputFocus::Priority => InputFocus::DueDate,
            InputFocus::DueDate => InputFocus::Tags,
            InputFocus::Tags => InputFocus::Title,
        }
    }
}

pub fn render_create_task_page(frame: &mut Frame, area: Rect, state: &CreateTaskState) {
    let layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints(vec![
            Constraint::Length(3), // Title
            Constraint::Length(5), // Description
            Constraint::Length(3), // Priority
            Constraint::Length(3), // Due Date
            Constraint::Length(3), // Tags
            Constraint::Min(0),    // Spacer
        ])
        .split(area);

    let title_block = Block::default().borders(Borders::ALL).title("Title");
    let title_paragraph = Paragraph::new(state.title.as_str()).block(if state.input_focus == InputFocus::Title { title_block.border_style(Style::default().fg(Color::Yellow)) } else { title_block });
    frame.render_widget(title_paragraph, layout[0]);

    let description_block = Block::default().borders(Borders::ALL).title("Description");
    let description_paragraph = Paragraph::new(state.description.as_str()).block(if state.input_focus == InputFocus::Description { description_block.border_style(Style::default().fg(Color::Yellow)) } else { description_block });
    frame.render_widget(description_paragraph, layout[1]);

    let priority_block = Block::default().borders(Borders::ALL).title("Priority");
    let priority_paragraph = Paragraph::new(PRIORITIES[state.priority]).block(if state.input_focus == InputFocus::Priority { priority_block.border_style(Style::default().fg(Color::Yellow)) } else { priority_block });
    frame.render_widget(priority_paragraph, layout[2]);

    let due_date_block = Block::default().borders(Borders::ALL).title("Due Date");
    let due_date_paragraph = Paragraph::new(state.due_date.as_str()).block(if state.input_focus == InputFocus::DueDate { due_date_block.border_style(Style::default().fg(Color::Yellow)) } else { due_date_block });
    frame.render_widget(due_date_paragraph, layout[3]);

    let tags_block = Block::default().borders(Borders::ALL).title("Tags");
    let tags_paragraph = Paragraph::new(state.tags.as_str()).block(if state.input_focus == InputFocus::Tags { tags_block.border_style(Style::default().fg(Color::Yellow)) } else { tags_block });
    frame.render_widget(tags_paragraph, layout[4]);
}
