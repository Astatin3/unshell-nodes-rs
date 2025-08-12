use ratatui::{Frame, layout::Rect, style::palette::tailwind, widgets::Paragraph};

pub const fn color() -> tailwind::Palette {
    tailwind::INDIGO
}

pub fn render(area: Rect, frame: &mut Frame) {
    frame.render_widget(Paragraph::new("Hello, World!"), area);
}
