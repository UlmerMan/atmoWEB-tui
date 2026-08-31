use ratatui::{
    style::{Color, Style},
    widgets::{Block, Borders, Widget},
};

use ratatui_textarea::{Input, Key, TextArea};

pub enum FloatInput<T> {
    Some(T),
    None,
    Abort,
}

pub struct FloatInputWidget<'a> {
    textarea: TextArea<'a>,
    is_valid: bool,
}

impl<'a> Default for FloatInputWidget<'a> {
    fn default() -> Self {
        Self::new()
    }
}

impl<'a> FloatInputWidget<'a> {
    pub fn new() -> Self {
        let mut textarea = TextArea::default();
        textarea.set_cursor_line_style(Style::default());
        textarea.set_placeholder_text("Enter a valid float (e.g. 1.56)");
        let is_valid = false;

        let mut float_widget = Self { textarea, is_valid };
        let _ = float_widget.validate();
        float_widget
    }

    pub fn handle_input(&mut self, input: Input) -> FloatInput<f32> {
        match input {
            Input { key: Key::Esc, .. } => FloatInput::Abort,
            Input {
                key: Key::Enter, ..
            } if self.is_valid => {
                let value = self.textarea.lines()[0].parse::<f32>().unwrap();
                self.textarea.clear();
                FloatInput::Some(value)
            }
            Input {
                key: Key::Char('m'),
                ctrl: true,
                ..
            }
            | Input {
                key: Key::Enter, ..
            } => FloatInput::None,
            _ => {
                if self.textarea.input(input) {
                    self.is_valid = self.validate();
                }
                FloatInput::None
            }
        }
    }

    fn validate(&mut self) -> bool {
        if let Err(err) = self.textarea.lines()[0].parse::<f64>() {
            self.textarea
                .set_style(Style::default().fg(Color::LightRed));
            self.textarea.set_block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Color::LightRed)
                    .title(format!("ERROR: {}", err)),
            );
            false
        } else {
            self.textarea
                .set_style(Style::default().fg(Color::LightGreen));
            self.textarea.set_block(
                Block::default()
                    .border_style(Color::LightGreen)
                    .borders(Borders::ALL)
                    .title("OK"),
            );
            true
        }
    }
}

impl Widget for &FloatInputWidget<'_> {
    fn render(self, area: ratatui::layout::Rect, buf: &mut ratatui::buffer::Buffer) {
        self.textarea.render(area, buf);
    }
}
