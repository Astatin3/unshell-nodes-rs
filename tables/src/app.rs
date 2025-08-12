use color_eyre::Result;
use ratatui::{
    DefaultTerminal, Frame,
    buffer::Buffer,
    crossterm::event::{self, Event, KeyCode, KeyEventKind},
    layout::{Constraint, Layout, Rect},
    style::{Color, Stylize, palette::tailwind},
    symbols,
    text::Line,
    widgets::{Block, Padding, Paragraph, Tabs, Widget},
};
use strum::IntoEnumIterator;

use crate::tabs::{Tab, TabController};

#[derive(Default)]
pub struct App {
    state: AppState,
    tab_controller: TabController,
}

#[derive(Default, Clone, Copy, PartialEq, Eq)]
enum AppState {
    #[default]
    Running,
    Quitting,
}

impl App {
    pub fn run(mut self, mut terminal: DefaultTerminal) -> Result<()> {
        while self.state == AppState::Running {
            terminal.draw(|frame| self.render(frame))?;
            self.handle_events()?;
        }
        Ok(())
    }

    fn handle_events(&mut self) -> std::io::Result<()> {
        if let Event::Key(key) = event::read()? {
            if key.kind == KeyEventKind::Press {
                if !self.tab_controller.key(key.code) {
                    match key.code {
                        KeyCode::Char('l') | KeyCode::Right => self.tab_controller.next(),
                        KeyCode::Char('h') | KeyCode::Left => self.tab_controller.previous(),
                        KeyCode::Char('q') | KeyCode::Esc => self.quit(),
                        _ => {}
                    }
                }
            }
        }
        Ok(())
    }

    pub fn quit(&mut self) {
        self.state = AppState::Quitting;
    }
}

impl App {
    fn render(&mut self, frame: &mut Frame) {
        let area = frame.area();

        use Constraint::{Length, Min};
        let vertical = Layout::vertical([Length(1), Min(0), Length(1)]);
        let [header_area, inner_area, footer_area] = vertical.areas(area);

        let horizontal = Layout::horizontal([Min(0), Length(20)]);
        let [tabs_area, title_area] = horizontal.areas(header_area);

        render_title(title_area, frame);
        self.tab_controller.render_tab_titles(tabs_area, frame);
        self.tab_controller.render(inner_area, frame);
        render_footer(footer_area, frame);
    }
}

fn render_title(area: Rect, frame: &mut Frame) {
    frame.render_widget("Ratatui Tabs Example".bold(), area);
}

fn render_footer(area: Rect, frame: &mut Frame) {
    frame.render_widget(
        Line::raw("◄ ► to change tab | Press q to quit").centered(),
        area,
    );
}
