use ratatui::{
    Frame,
    crossterm::event::KeyCode,
    layout::{Constraint, Layout, Margin, Rect},
    style::{Modifier, Style, palette::tailwind},
    text::Text,
    widgets::{
        Block, BorderType, Cell, HighlightSpacing, Paragraph, Row, Scrollbar, ScrollbarOrientation,
        ScrollbarState, Table, TableState,
    },
};
pub const fn color() -> tailwind::Palette {
    tailwind::BLUE
}

#[derive(Clone)]
struct EnvVar {
    name: String,
    data: String,
    // email: String,
}

impl EnvVar {
    const fn ref_array(&self) -> [&String; 2] {
        [&self.name, &self.data]
    }
}

#[derive(Clone)]
pub struct State {
    state: TableState,
    items: Vec<EnvVar>,
    longest_item_name: u16,
    scroll_state: ScrollbarState,
    color_index: usize,
}

// const ITEM_HEIGHT: usize = 2;

impl State {
    pub fn render(&mut self, area: Rect, frame: &mut Frame) {
        let vertical = &Layout::vertical([Constraint::Min(5), Constraint::Length(4)]);
        let rects = vertical.split(area);

        // self.set_colors();

        self.render_table(frame, rects[0]);
        self.render_scrollbar(frame, rects[0]);
        // self.render_footer(frame, rects[1]);
    }

    fn render_table(&mut self, frame: &mut Frame, area: Rect) {
        // let header_style = Style::default()
        //     .fg(tailwind::SLATE.c200)
        //     .bg(tailwind::SLATE.c900);
        // let selected_row_style = Style::default()
        //     .add_modifier(Modifier::REVERSED)
        //     .fg(tailwind::SLATE.c400);
        // let selected_col_style = Style::default().fg(tailwind::SLATE.c400);
        // let selected_cell_style = Style::default()
        //     .add_modifier(Modifier::REVERSED)
        //     .fg(tailwind::SLATE.c600);

        let header = ["Name", "Address", "Email"]
            .into_iter()
            .map(Cell::from)
            .collect::<Row>()
            // .style(header_style)
            .height(1);

        let rows = self.items.iter().enumerate().map(|(i, data)| {
            let color = match i % 2 {
                0 => tailwind::SLATE.c950,
                _ => tailwind::SLATE.c900,
            };
            let item = data.ref_array();
            item.into_iter()
                .map(|content| Cell::from(Text::from(format!("{content}"))))
                .collect::<Row>()
                .style(Style::new().fg(tailwind::SLATE.c200).bg(color))
            // .height(ITEM_HEIGHT as u16)
        });
        let t = Table::new(
            rows,
            [
                Constraint::Length(self.longest_item_name),
                Constraint::Min(10),
            ],
        )
        .header(header)
        .highlight_symbol(Text::from(" █ "))
        .highlight_spacing(HighlightSpacing::Always);
        // .set_w
        frame.render_stateful_widget(t, area, &mut self.state);
    }

    fn render_scrollbar(&mut self, frame: &mut Frame, area: Rect) {
        frame.render_stateful_widget(
            Scrollbar::default()
                .orientation(ScrollbarOrientation::VerticalRight)
                .begin_symbol(None)
                .end_symbol(None),
            area,
            &mut self.scroll_state,
        );
    }

    // fn render_footer(&self, frame: &mut Frame, area: Rect) {
    //     let info_footer = Paragraph::new(Text::from_iter(INFO_TEXT))
    //         // .style(
    //         //     Style::new()
    //         //         .fg(self.colors.row_fg)
    //         //         .bg(self.colors.buffer_bg),
    //         // )
    //         .centered()
    //         .block(
    //             Block::bordered()
    //                 .border_type(BorderType::Double)
    //                 .border_style(Style::new().fg(self.colors.footer_border_color)),
    //         );
    //     frame.render_widget(info_footer, area);
    // }

    pub fn key(&mut self, keycode: KeyCode) -> bool {
        match keycode {
            KeyCode::Char('j') | KeyCode::Down => {
                self.next_row();
                false
            }
            KeyCode::Char('k') | KeyCode::Up => {
                self.previous_row();
                false
            }
            // KeyCode::Char('l') | KeyCode::Right if shift_pressed => self.next_color(),
            // KeyCode::Char('h') | KeyCode::Left if shift_pressed => {
            //     self.previous_color();
            // }
            // KeyCode::Char('l') | KeyCode::Right => {
            //     self.next_column();
            //     true
            // }
            // KeyCode::Char('h') | KeyCode::Left => {
            //     self.previous_column();
            //     true
            // }
            _ => false,
        }
    }
}

impl State {
    pub fn next_row(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i >= self.items.len() - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
        self.scroll_state = self.scroll_state.position(i);
    }

    pub fn previous_row(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i == 0 {
                    self.items.len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
        self.scroll_state = self.scroll_state.position(i);
    }

    pub fn next_column(&mut self) {
        self.state.select_next_column();
    }

    pub fn previous_column(&mut self) {
        self.state.select_previous_column();
    }

    // pub fn next_color(&mut self) {
    //     self.color_index = (self.color_index + 1) % PALETTES.len();
    // }

    // pub fn previous_color(&mut self) {
    //     let count = PALETTES.len();
    //     self.color_index = (self.color_index + count - 1) % count;
    // }

    // pub fn set_colors(&mut self) {
    //     self.colors = TableColors::new(&PALETTES[self.color_index]);
    // }
}

impl Default for State {
    fn default() -> Self {
        let data_vec: Vec<EnvVar> = std::env::vars()
            .map(|(k, v)| EnvVar { name: k, data: v })
            .collect();

        Self {
            state: TableState::default().with_selected(0),
            longest_item_name: data_vec.iter().map(|var| var.name.len()).max().unwrap_or(0) as u16,
            scroll_state: ScrollbarState::new((data_vec.len() - 1)),
            // colors: TableColors::new(&PALETTES[0]),
            color_index: 0,
            items: data_vec,
        }
    }
}

// fn constraint_len_calculator(items: &[Data]) -> (u16, u16, u16) {
//     let name_len = items
//         .iter()
//         .map(Data::name)
//         .map(UnicodeWidthStr::width)
//         .max()
//         .unwrap_or(0);
//     let address_len = items
//         .iter()
//         .map(Data::address)
//         .flat_map(str::lines)
//         .map(UnicodeWidthStr::width)
//         .max()
//         .unwrap_or(0);
//     let email_len = items
//         .iter()
//         .map(Data::email)
//         .map(UnicodeWidthStr::width)
//         .max()
//         .unwrap_or(0);

//     #[allow(clippy::cast_possible_truncation)]
//     (name_len as u16, address_len as u16, email_len as u16)
// }
