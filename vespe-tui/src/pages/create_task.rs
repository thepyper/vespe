use ratatui::{prelude::*, widgets::*};

pub fn render_create_task_page(frame: &mut Frame, area: Rect) {
    frame.render_widget(
        Paragraph::new("Create Task Page Content").white().on_blue(),
        area,
    );
}
