use ratatui::{
    buffer::Buffer,
    layout::{Alignment, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Widget},
};

pub struct ControlWidget {
    pub title: String,
    pub current: f32,
    pub value: f32,
    pub unit: String,
    pub step: f32,
    pub min: f32,
    pub max: f32,
    pub selected: bool,
}

impl ControlWidget {
    pub fn new(title: &str, start: f32, unit: &str, step: f32, min: f32, max: f32) -> Self {
        Self {
            title: title.to_string(),
            current: start,
            value: start,
            unit: unit.to_string(),
            step,
            min,
            max,
            selected: false,
        }
    }

    pub fn select(&mut self) {
        self.selected = true;
    }

    pub fn deselect(&mut self) {
        self.selected = false;
    }

    pub fn increase(&mut self) {
        self.value = (self.value + self.step).min(self.max);
    }

    pub fn decrease(&mut self) {
        self.value = (self.value - self.step).max(self.min);
    }

    pub fn set_current(&mut self, current: f32) {
        self.current = current;
    }
}

impl Widget for &ControlWidget {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let block = Block::default()
            .title(self.title.clone())
            .borders(Borders::ALL)
            .style(if self.selected {
                Style::default().fg(Color::Cyan)
            } else {
                Style::default()
            });

        let inner = block.inner(area);
        block.render(area, buf);

        let current_line = Line::from(vec![
            Span::styled("Ist:  ", Style::default().fg(Color::DarkGray)),
            Span::styled(
                format!("{:.1} {}", self.current, self.unit),
                Style::default(),
            ),
        ])
        .alignment(Alignment::Center);

        let target_line = Line::from(vec![
            Span::styled("Soll: ", Style::default().fg(Color::DarkGray)),
            Span::styled(
                format!("{:.1} {}", self.value, self.unit),
                Style::default().add_modifier(Modifier::BOLD).fg(
                    if self.selected { Color::Cyan } else { Color::White },
                ),
            ),
        ])
        .alignment(Alignment::Center);

        let ratio = (self.value - self.min) / (self.max - self.min);
        let bar_width = inner.width.saturating_sub(2) as usize;
        let filled = (bar_width as f32 * ratio).round() as usize;
        let bar = format!(
            "[{}{}]",
            "#".repeat(filled),
            "-".repeat(bar_width.saturating_sub(filled))
        );
        let bar_line = Line::from(Span::styled(bar, Style::default())).alignment(Alignment::Center);

        let hint_line = if self.selected {
            Line::from("↑/↓ ändern, ↵ senden")
        } else {
            Line::from("Tab zum Auswählen")
        }
        .alignment(Alignment::Center)
        .style(Style::default().fg(Color::DarkGray));

        let lines: [(&Line, bool); 4] = [
            (&current_line, true),
            (&target_line, true),
            (&bar_line, inner.height >= 3),
            (&hint_line, inner.height >= 4),
        ];

        let visible: Vec<&Line> = lines
            .iter()
            .filter(|(_, show)| *show)
            .map(|(line, _)| *line)
            .collect();

        let start_y = inner.y + (inner.height.saturating_sub(visible.len() as u16)) / 2;
        for (i, line) in visible.iter().enumerate() {
            let y = start_y + i as u16;
            if y < inner.y + inner.height {
                buf.set_line(inner.x, y, line, inner.width);
            }
        }
    }
}