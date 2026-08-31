use ratatui::{
    buffer::Buffer,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Widget},
};

use crate::tui::state::AppMode;

pub struct HeaderWidget<'a> {
    pub mode: AppMode,
    pub online: bool,
    pub oven_ip: &'a str,
}

impl<'a> HeaderWidget<'a> {
    pub fn new(mode: AppMode, online: bool, oven_ip: &'a str) -> Self {
        Self {
            mode,
            online,
            oven_ip,
        }
    }
}

impl Widget for &HeaderWidget<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Fill(1), Constraint::Length(32)])
            .split(area);

        // Mode Banner
        let (mode_badge, mode_style, hint_text) = match self.mode {
            AppMode::Manual => (
                "  [ MANUAL MODE ]  ",
                Style::default()
                    .bg(Color::Blue)
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD),
                "Direct manual control (Press 'm' to switch to Auto Mode)",
            ),
            AppMode::Auto => (
                "  [ AUTO MODE - HEATING CURVE ]  ",
                Style::default()
                    .bg(Color::Green)
                    .fg(Color::Black)
                    .add_modifier(Modifier::BOLD),
                "Heating curve engine (Manual locked - Press 'm' to switch to Manual Mode)",
            ),
        };

        let mode_line = Line::from(vec![
            Span::styled(mode_badge, mode_style),
            Span::raw("  "),
            Span::styled(hint_text, Style::default().fg(Color::DarkGray)),
        ]);

        let mode_block = Block::default()
            .borders(Borders::ALL)
            .title(" System Mode ");
        let mode_p = Paragraph::new(mode_line).block(mode_block);
        mode_p.render(chunks[0], buf);

        // Oven Info / Status Banner
        let online_span = if self.online {
            Span::styled(
                "● ONLINE",
                Style::default()
                    .fg(Color::Green)
                    .add_modifier(Modifier::BOLD),
            )
        } else {
            Span::styled(
                "○ OFFLINE",
                Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
            )
        };

        let status_line = Line::from(vec![
            Span::raw("IP: "),
            Span::styled(self.oven_ip, Style::default().fg(Color::Cyan)),
            Span::raw("  "),
            online_span,
        ]);

        let status_block = Block::default()
            .borders(Borders::ALL)
            .title(" Oven Status ");
        let status_p = Paragraph::new(status_line)
            .alignment(Alignment::Right)
            .block(status_block);
        status_p.render(chunks[1], buf);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_header_widget_renders() {
        let widget = HeaderWidget::new(AppMode::Manual, true, "192.168.1.22");
        let area = Rect::new(0, 0, 80, 3);
        let mut buf = Buffer::empty(area);
        (&widget).render(area, &mut buf);

        let auto_widget = HeaderWidget::new(AppMode::Auto, false, "192.168.1.22");
        (&auto_widget).render(area, &mut buf);
    }
}
