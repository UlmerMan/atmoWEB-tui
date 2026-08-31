use std::io::Error;

use ratatui::{
    DefaultTerminal,
    layout::Constraint,
    layout::Layout,
    style::{Color, Style},
    widgets::{Block, Borders},
};

use ratatui_textarea::{Input, Key, TextArea};

fn validate(textarea: &mut TextArea) -> bool {
    if let Err(err) = textarea.lines()[0].parse::<f64>() {
        textarea.set_style(Style::default().fg(Color::LightRed));
        textarea.set_block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Color::LightRed)
                .title(format!("ERROR: {}", err)),
        );
        false
    } else {
        textarea.set_style(Style::default().fg(Color::LightGreen));
        textarea.set_block(
            Block::default()
                .border_style(Color::LightGreen)
                .borders(Borders::ALL)
                .title("OK"),
        );
        true
    }
}

pub fn float_input_widget(
    terminal: &mut DefaultTerminal,
) -> Result<f32, Box<dyn std::error::Error>> {
    let mut textarea = TextArea::default();
    textarea.set_cursor_line_style(Style::default());
    textarea.set_placeholder_text("Enter a valid float (e.g. 1.56)");
    let layout = Layout::default().constraints([Constraint::Length(3), Constraint::Min(1)]);
    let mut is_valid = validate(&mut textarea);

    loop {
        terminal.draw(|f| {
            let chunks = layout.split(f.area());
            f.render_widget(&textarea, chunks[0]);
        })?;

        match crossterm::event::read()?.into() {
            Input { key: Key::Esc, .. } => {
                Err(Error::new(std::io::ErrorKind::InvalidInput, "User cancelled input"))?
            }
            Input {
                key: Key::Enter, ..
            } if is_valid => return Ok(textarea.lines()[0].parse::<f32>().unwrap()),
            Input {
                key: Key::Char('m'),
                ctrl: true,
                ..
            }
            | Input {
                key: Key::Enter, ..
            } => {}
            input => {
                // TextArea::input returns if the input modified its text
                if textarea.input(input) {
                    is_valid = validate(&mut textarea);
                }
            }
        }
    }
}
