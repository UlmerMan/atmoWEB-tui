use ratatui::{
    buffer::Buffer,
    layout::{Alignment, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Paragraph, Widget},
};

use crate::tui::state::{AppMode, InputState};

pub struct FooterWidget<'a> {
    pub mode: AppMode,
    pub input_state: &'a InputState,
}

impl<'a> FooterWidget<'a> {
    pub fn new(mode: AppMode, input_state: &'a InputState) -> Self {
        Self { mode, input_state }
    }
}

impl Widget for &FooterWidget<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let spans = match self.input_state {
            InputState::Editing(_) => vec![
                Span::styled(
                    " [Enter] ",
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled("Confirm", Style::default().fg(Color::White)),
                Span::styled("  │  ", Style::default().fg(Color::DarkGray)),
                Span::styled(
                    " [Esc] ",
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled("Cancel", Style::default().fg(Color::White)),
            ],
            InputState::Normal => match self.mode {
                AppMode::Manual => vec![
                    Span::styled(
                        " [q] ",
                        Style::default()
                            .fg(Color::Cyan)
                            .add_modifier(Modifier::BOLD),
                    ),
                    Span::styled("Quit", Style::default().fg(Color::White)),
                    Span::styled(" │ ", Style::default().fg(Color::DarkGray)),
                    Span::styled(
                        " [m] ",
                        Style::default()
                            .fg(Color::Cyan)
                            .add_modifier(Modifier::BOLD),
                    ),
                    Span::styled("Auto", Style::default().fg(Color::White)),
                    Span::styled(" │ ", Style::default().fg(Color::DarkGray)),
                    Span::styled(
                        " [g] ",
                        Style::default()
                            .fg(Color::Cyan)
                            .add_modifier(Modifier::BOLD),
                    ),
                    Span::styled("View", Style::default().fg(Color::White)),
                    Span::styled(" │ ", Style::default().fg(Color::DarkGray)),
                    Span::styled(
                        " [0..2/Tab] ",
                        Style::default()
                            .fg(Color::Cyan)
                            .add_modifier(Modifier::BOLD),
                    ),
                    Span::styled("Focus", Style::default().fg(Color::White)),
                    Span::styled(" │ ", Style::default().fg(Color::DarkGray)),
                    Span::styled(
                        " [↑↓/+-] ",
                        Style::default()
                            .fg(Color::Cyan)
                            .add_modifier(Modifier::BOLD),
                    ),
                    Span::styled("Adjust", Style::default().fg(Color::White)),
                    Span::styled(" │ ", Style::default().fg(Color::DarkGray)),
                    Span::styled(
                        " [Enter] ",
                        Style::default()
                            .fg(Color::Yellow)
                            .add_modifier(Modifier::BOLD),
                    ),
                    Span::styled(
                        "Apply",
                        Style::default()
                            .fg(Color::Yellow)
                            .add_modifier(Modifier::BOLD),
                    ),
                    Span::styled(" │ ", Style::default().fg(Color::DarkGray)),
                    Span::styled(
                        " [e] ",
                        Style::default()
                            .fg(Color::Cyan)
                            .add_modifier(Modifier::BOLD),
                    ),
                    Span::styled("Input", Style::default().fg(Color::White)),
                ],
                AppMode::Auto => vec![
                    Span::styled(
                        " [q] ",
                        Style::default()
                            .fg(Color::Cyan)
                            .add_modifier(Modifier::BOLD),
                    ),
                    Span::styled("Quit", Style::default().fg(Color::White)),
                    Span::styled(" │ ", Style::default().fg(Color::DarkGray)),
                    Span::styled(
                        " [m] ",
                        Style::default()
                            .fg(Color::Cyan)
                            .add_modifier(Modifier::BOLD),
                    ),
                    Span::styled("Manual", Style::default().fg(Color::White)),
                    Span::styled(" │ ", Style::default().fg(Color::DarkGray)),
                    Span::styled(
                        " [g] ",
                        Style::default()
                            .fg(Color::Cyan)
                            .add_modifier(Modifier::BOLD),
                    ),
                    Span::styled("View", Style::default().fg(Color::White)),
                    Span::styled(" │ ", Style::default().fg(Color::DarkGray)),
                    Span::styled(
                        " [h/l] ",
                        Style::default()
                            .fg(Color::Cyan)
                            .add_modifier(Modifier::BOLD),
                    ),
                    Span::styled("Point", Style::default().fg(Color::White)),
                    Span::styled(" │ ", Style::default().fg(Color::DarkGray)),
                    Span::styled(
                        " [j/k] ",
                        Style::default()
                            .fg(Color::Cyan)
                            .add_modifier(Modifier::BOLD),
                    ),
                    Span::styled("Temp", Style::default().fg(Color::White)),
                    Span::styled(" │ ", Style::default().fg(Color::DarkGray)),
                    Span::styled(
                        " [[/]] ",
                        Style::default()
                            .fg(Color::Cyan)
                            .add_modifier(Modifier::BOLD),
                    ),
                    Span::styled("Time", Style::default().fg(Color::White)),
                    Span::styled(" │ ", Style::default().fg(Color::DarkGray)),
                    Span::styled(
                        " [a] ",
                        Style::default()
                            .fg(Color::Cyan)
                            .add_modifier(Modifier::BOLD),
                    ),
                    Span::styled("Add", Style::default().fg(Color::White)),
                    Span::styled(" │ ", Style::default().fg(Color::DarkGray)),
                    Span::styled(
                        " [d] ",
                        Style::default()
                            .fg(Color::Cyan)
                            .add_modifier(Modifier::BOLD),
                    ),
                    Span::styled("Del", Style::default().fg(Color::White)),
                    Span::styled(" │ ", Style::default().fg(Color::DarkGray)),
                    Span::styled(
                        " [Space] ",
                        Style::default()
                            .fg(Color::Cyan)
                            .add_modifier(Modifier::BOLD),
                    ),
                    Span::styled("Run/Pause", Style::default().fg(Color::White)),
                    Span::styled(" │ ", Style::default().fg(Color::DarkGray)),
                    Span::styled(
                        " [s] ",
                        Style::default()
                            .fg(Color::Cyan)
                            .add_modifier(Modifier::BOLD),
                    ),
                    Span::styled("Stop", Style::default().fg(Color::White)),
                ],
            },
        };

        let p = Paragraph::new(Line::from(spans)).alignment(Alignment::Center);
        p.render(area, buf);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_footer_widget_renders() {
        let input_state = InputState::Normal;
        let widget = FooterWidget::new(AppMode::Manual, &input_state);
        let area = Rect::new(0, 0, 80, 1);
        let mut buf = Buffer::empty(area);
        (&widget).render(area, &mut buf);

        let auto_widget = FooterWidget::new(AppMode::Auto, &input_state);
        (&auto_widget).render(area, &mut buf);
    }
}
