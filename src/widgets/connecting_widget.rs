use ratatui::{
    buffer::Buffer,
    layout::{Alignment, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Paragraph, Widget, Wrap},
};

pub struct ConnectingWidget<'a> {
    pub oven_ip: &'a str,
    pub status: &'a str,
    pub spinner_tick: u64,
}

impl<'a> ConnectingWidget<'a> {
    pub fn new(oven_ip: &'a str, status: &'a str, spinner_tick: u64) -> Self {
        Self {
            oven_ip,
            status,
            spinner_tick,
        }
    }
}

impl Widget for &ConnectingWidget<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        const SPINNER: &[&str] = &["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"];
        let spinner_glyph = SPINNER[(self.spinner_tick as usize / 2) % SPINNER.len()];

        let lines = vec![
            Line::from(""),
            Line::from(vec![
                Span::styled(
                    format!("  {} ", spinner_glyph),
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(
                    "Connecting to Memmert atmoWEB...",
                    Style::default()
                        .fg(Color::White)
                        .add_modifier(Modifier::BOLD),
                ),
            ]),
            Line::from(""),
            Line::from(vec![
                Span::styled("  Target : ", Style::default().fg(Color::DarkGray)),
                Span::styled(
                    format!("http://{}/atmoweb", self.oven_ip),
                    Style::default().fg(Color::Cyan),
                ),
            ]),
            Line::from(vec![
                Span::styled("  Status : ", Style::default().fg(Color::DarkGray)),
                Span::styled(self.status, Style::default().fg(Color::Yellow)),
            ]),
            Line::from(""),
            Line::from(vec![
                Span::styled(
                    "[q] / [Esc]",
                    Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
                ),
                Span::styled(" Abort and Exit", Style::default().fg(Color::DarkGray)),
            ])
            .alignment(Alignment::Center),
        ];

        let block = Block::default()
            .title(" Connecting ")
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .style(Style::default().fg(Color::Cyan));

        let paragraph = Paragraph::new(lines).block(block).wrap(Wrap { trim: true });

        paragraph.render(area, buf);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_connecting_widget_renders_with_long_status() {
        let widget = ConnectingWidget::new(
            "192.168.1.22",
            "Oven unreachable at 192.168.1.22. Waiting for connection...",
            4,
        );
        let area = Rect::new(0, 0, 60, 11);
        let mut buf = Buffer::empty(area);
        (&widget).render(area, &mut buf);
    }
}
