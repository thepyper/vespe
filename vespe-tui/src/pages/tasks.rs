use ratatui::{prelude::*, widgets::*};

pub fn render_tasks_page(frame: &mut Frame, area: Rect) {
    frame.render_widget(
        Paragraph::new("Tasks Page Content")
            .white()
            .on_dark_gray(),
        area,
    );
}
