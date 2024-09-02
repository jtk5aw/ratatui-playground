use color_eyre::eyre::{bail, WrapErr};
use ratatui::{
    buffer::Buffer,
    crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind},
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Style, Stylize},
    symbols::border,
    text::{Line, Span, Text},
    widgets::{
        block::{Position, Title},
        Block, Borders, Paragraph, Widget,
    },
    Frame,
};

mod tui;

fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;
    let mut terminal = tui::init()?;
    let app_result = App::default().run(&mut terminal);
    if let Err(err) = tui::restore() {
        eprintln!(
            "failed to restore terminal. Run `reset` or restart your terminal to recover: {}",
            err
        );
    }
    app_result
}

#[derive(Debug)]
pub struct App {
    focus_on: usize,
    counters: Vec<Counter>,
    exit: bool,
}

impl Default for App {
    fn default() -> Self {
        Self {
            focus_on: 0,
            counters: vec![
                Counter::start_focused(),
                Counter::default(),
                Counter::default(),
            ],
            exit: false,
        }
    }
}

#[derive(Debug, Default)]
pub struct Counter {
    focused: bool,
    counter: u8,
}

impl Counter {
    fn start_focused() -> Self {
        Self {
            focused: true,
            counter: 0,
        }
    }
}

impl App {
    /// runs the application's main loop until the user quits
    pub fn run(&mut self, terminal: &mut tui::Tui) -> color_eyre::Result<()> {
        while !self.exit {
            terminal.draw(|frame| self.render_frame(frame))?;
            self.handle_events().wrap_err("handle events failed")?;
        }
        Ok(())
    }

    fn render_frame(&self, frame: &mut Frame) {
        let outer_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints(vec![Constraint::Percentage(5), Constraint::Percentage(99)])
            .split(frame.area());

        let title = Title::from(Line::from("Multi-Counter").bold().blue().on_white());

        let title_block = Block::default()
            .borders(Borders::ALL)
            .style(Style::default())
            .title(title.alignment(Alignment::Center).position(Position::Top))
            .title_style(Style::new().blue().on_white().bold().italic());
        let instructions = Paragraph::new(Line::from(vec![
            " Decrement ".into(),
            Span::styled("<j>", Style::new().blue().bold()),
            " Increment ".into(),
            Span::styled("<k>", Style::new().blue().bold()),
            " Left ".into(),
            Span::styled("<h>", Style::new().blue().bold()),
            " Right ".into(),
            Span::styled("<l>", Style::new().blue().bold()),
            " Quit ".into(),
            Span::styled("<Q> ", Style::new().blue().bold()),
        ]))
        .alignment(Alignment::Center)
        .block(title_block);

        frame.render_widget(instructions, outer_layout[0]);

        let counter_layout = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(vec![
                Constraint::Percentage(33),
                Constraint::Percentage(33),
                Constraint::Percentage(33),
            ])
            .split(outer_layout[1]);

        for (i, rect) in counter_layout.iter().enumerate() {
            frame.render_widget(&self.counters[i], *rect);
        }
    }

    fn handle_events(&mut self) -> color_eyre::Result<()> {
        match event::read()? {
            // it's important to check that the event is a key press event as
            // crossterm also emits key release and repeat events on Windows.
            Event::Key(key_event) if key_event.kind == KeyEventKind::Press => self
                .handle_key_event(key_event)
                .wrap_err_with(|| format!("handling key event failed:\n{key_event:#?}")),
            _ => Ok(()),
        }
    }

    fn handle_key_event(&mut self, key_event: KeyEvent) -> color_eyre::Result<()> {
        match key_event.code {
            KeyCode::Char('q') => self.exit(),
            KeyCode::Char('l') => self.next_counter()?,
            KeyCode::Char('h') => self.previous_counter()?,
            KeyCode::Char('j') => self.decrement_current_counter()?,
            KeyCode::Char('k') => self.increment_current_counter()?,
            _ => {}
        }
        Ok(())
    }

    fn exit(&mut self) {
        self.exit = true;
    }

    fn next_counter(&mut self) -> color_eyre::Result<()> {
        self.counters[self.focus_on].focused = false;
        if self.focus_on == self.counters.len() - 1 {
            self.focus_on = 0;
        } else {
            self.focus_on = self.focus_on + 1;
        }
        self.counters[self.focus_on].focused = true;
        Ok(())
    }

    fn previous_counter(&mut self) -> color_eyre::Result<()> {
        self.counters[self.focus_on].focused = false;
        if self.focus_on == 0 {
            self.focus_on = self.counters.len() - 1;
        } else {
            self.focus_on = self.focus_on - 1;
        }
        self.counters[self.focus_on].focused = true;
        Ok(())
    }

    fn increment_current_counter(&mut self) -> color_eyre::Result<()> {
        let curr = &mut self.counters[self.focus_on];
        curr.counter += 1;
        if curr.counter > 2 {
            bail!("counter overflow");
        }
        Ok(())
    }

    fn decrement_current_counter(&mut self) -> color_eyre::Result<()> {
        let curr = &mut self.counters[self.focus_on];
        curr.counter -= 1;
        Ok(())
    }
}

impl Widget for &Counter {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let title = Title::from(" Counter ".bold());
        let border_style = match self.focused {
            true => Style::default().blue(),
            false => Style::default(),
        };

        let block = Block::bordered()
            .title(title.alignment(Alignment::Center))
            .style(border_style)
            .border_set(border::THICK);

        let counter_text = Text::from(vec![Line::from(vec![
            "Value: ".into(),
            self.counter.to_string().yellow(),
        ])]);

        Paragraph::new(counter_text)
            .centered()
            .block(block)
            .render(area, buf);
    }
}

#[cfg(test)]
mod tests {
    use ratatui::style::Style;

    use super::*;

    // Frankly, I think this is dumb but keeping it as an example
    #[test]
    fn render() {
        let app = App::default();
        let mut buf = Buffer::empty(Rect::new(0, 0, 50, 4));

        app.counters[0].render(buf.area, &mut buf);

        let mut expected = Buffer::with_lines(vec![
            "┏━━━━━━━━━━━━━━━━━━━ Counter ━━━━━━━━━━━━━━━━━━━━┓",
            "┃                    Value: 0                    ┃",
            "┃                                                ┃",
            "┗━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━┛",
        ]);
        let title_style = Style::new().blue();
        let title_text_style = Style::new().blue().bold();
        let counter_style = Style::new().yellow();
        expected.set_style(Rect::new(0, 0, 20, 1), title_style);
        expected.set_style(Rect::new(20, 0, 9, 1), title_text_style);
        expected.set_style(Rect::new(29, 0, 21, 1), title_style);
        expected.set_style(Rect::new(0, 1, 28, 1), title_style);
        expected.set_style(Rect::new(28, 1, 1, 1), counter_style);
        expected.set_style(Rect::new(29, 1, 21, 1), title_style);
        expected.set_style(Rect::new(0, 2, 50, 2), title_style);

        // note ratatui also has an assert_buffer_eq! macro that can be used to
        // compare buffers and display the differences in a more readable way
        assert_eq!(buf, expected);
    }

    #[test]
    fn handle_key_event() {
        let mut app = App::default();
        app.handle_key_event(KeyCode::Char('k').into()).unwrap();
        assert_eq!(app.counters[0].counter, 1);

        app.handle_key_event(KeyCode::Char('j').into()).unwrap();
        assert_eq!(app.counters[0].counter, 0);

        let mut app = App::default();
        app.handle_key_event(KeyCode::Char('q').into()).unwrap();
        assert!(app.exit);
    }

    #[test]
    #[should_panic(expected = "attempt to subtract with overflow")]
    fn handle_key_event_panic() {
        let mut app = App::default();
        let _ = app.handle_key_event(KeyCode::Char('j').into());
    }

    #[test]
    fn handle_key_event_overflow() {
        let mut app = App::default();
        println!("{:?}", app);
        assert!(app.handle_key_event(KeyCode::Char('k').into()).is_ok());
        println!("{:?}", app);
        assert!(app.handle_key_event(KeyCode::Char('k').into()).is_ok());
        assert_eq!(
            app.handle_key_event(KeyCode::Char('k').into())
                .unwrap_err()
                .to_string(),
            "counter overflow"
        );
    }
}
