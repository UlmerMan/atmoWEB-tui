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
    pub applied_value: f32,
    pub unit: String,
    pub step: f32,
    pub min: f32,
    pub max: f32,
    pub selected: bool,
    pub locked: bool,
}

impl ControlWidget {
    pub fn new(title: &str, start: f32, unit: &str, step: f32, min: f32, max: f32) -> Self {
        Self {
            title: title.to_string(),
            current: start,
            value: start,
            applied_value: start,
            unit: unit.to_string(),
            step,
            min,
            max,
            selected: false,
            locked: false,
        }
    }

    pub fn select(&mut self) {
        self.selected = true;
    }

    pub fn deselect(&mut self) {
        self.selected = false;
    }

    pub fn set_locked(&mut self, locked: bool) {
        self.locked = locked;
        if locked {
            self.selected = false;
        }
    }

    pub fn is_dirty(&self) -> bool {
        (self.value - self.applied_value).abs() > 0.001
    }

    pub fn mark_applied(&mut self) {
        self.applied_value = self.value;
    }

    pub fn sync_applied(&mut self, val: f32) {
        let clean = !self.is_dirty();
        self.applied_value = val.clamp(self.min, self.max);
        if clean {
            self.value = self.applied_value;
        }
    }

    pub fn increase(&mut self) {
        if !self.locked {
            self.value = (self.value + self.step).min(self.max);
        }
    }

    pub fn decrease(&mut self) {
        if !self.locked {
            self.value = (self.value - self.step).max(self.min);
        }
    }

    pub fn set_current(&mut self, current: f32) {
        self.current = current;
    }

    pub fn set_target(&mut self, target: f32) {
        self.value = target.clamp(self.min, self.max);
    }
}

impl Widget for &ControlWidget {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let title_text = if self.locked {
            format!(" {} [LOCKED] ", self.title)
        } else {
            format!(" {} ", self.title)
        };

        let is_dirty = self.is_dirty();

        let block_style = if self.locked {
            Style::default().fg(Color::DarkGray)
        } else if is_dirty {
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD)
        } else if self.selected {
            Style::default().fg(Color::Cyan)
        } else {
            Style::default()
        };

        let mut block = Block::default()
            .title(title_text)
            .borders(Borders::ALL)
            .style(block_style);

        if is_dirty && !self.locked {
            block = block.title_top(
                Line::from(Span::styled(
                    " [ ENTER ] to apply ",
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD),
                ))
                .alignment(Alignment::Right),
            );
        }

        let inner = block.inner(area);
        block.render(area, buf);

        let current_line = Line::from(vec![
            Span::styled("current:  ", Style::default().fg(Color::DarkGray)),
            Span::styled(
                format!("{:.1} {}", self.current, self.unit),
                if self.locked {
                    Style::default().fg(Color::DarkGray)
                } else {
                    Style::default()
                },
            ),
        ])
        .alignment(Alignment::Center);

        let target_style = if self.locked {
            Style::default().fg(Color::DarkGray)
        } else if is_dirty {
            Style::default()
                .add_modifier(Modifier::BOLD)
                .fg(Color::Yellow)
        } else if self.selected {
            Style::default()
                .add_modifier(Modifier::BOLD)
                .fg(Color::Cyan)
        } else {
            Style::default()
                .add_modifier(Modifier::BOLD)
                .fg(Color::White)
        };

        let target_line = Line::from(vec![
            Span::styled("target:  ", Style::default().fg(Color::DarkGray)),
            Span::styled(format!("{:.1} {}", self.value, self.unit), target_style),
        ])
        .alignment(Alignment::Center);

        let ratio = ((self.value - self.min) / (self.max - self.min)).clamp(0.0, 1.0);

        let bar_width = inner.width.saturating_sub(4) as usize;
        let filled = (bar_width as f32 * ratio).round() as usize;
        let bar = format!("[{}{}]", "#".repeat(filled), "-".repeat(bar_width - filled));
        let bar_style = if self.locked {
            Style::default().fg(Color::DarkGray)
        } else if is_dirty {
            Style::default().fg(Color::Yellow)
        } else {
            Style::default()
        };

        let bar_line = Line::from(vec![
            Span::styled("-", bar_style),
            Span::styled(bar, bar_style),
            Span::styled("+", bar_style),
        ]);

        let lines: [(&Line, bool); 3] = [
            (&current_line, true),
            (&target_line, true),
            (&bar_line, inner.height >= 3),
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lock_prevents_modification() {
        let mut widget = ControlWidget::new("Temp", 100.0, "°C", 5.0, 20.0, 300.0);
        assert_eq!(widget.value, 100.0);

        widget.increase();
        assert_eq!(widget.value, 105.0);

        widget.set_locked(true);
        assert!(widget.locked);
        assert!(!widget.selected);

        // Increase and decrease should be no-ops when locked
        widget.increase();
        assert_eq!(widget.value, 105.0);
        widget.decrease();
        assert_eq!(widget.value, 105.0);

        widget.set_locked(false);
        widget.decrease();
        assert_eq!(widget.value, 100.0);
    }

    #[test]
    fn test_dirty_state_and_apply() {
        let mut widget = ControlWidget::new("Temp", 100.0, "°C", 5.0, 20.0, 300.0);
        assert!(!widget.is_dirty());

        widget.increase();
        assert_eq!(widget.value, 105.0);
        assert!(widget.is_dirty());

        widget.decrease();
        assert_eq!(widget.value, 100.0);
        assert!(!widget.is_dirty());

        widget.increase();
        assert!(widget.is_dirty());
        widget.mark_applied();
        assert!(!widget.is_dirty());
        assert_eq!(widget.applied_value, 105.0);

        // sync_applied when clean should update both
        widget.sync_applied(120.0);
        assert_eq!(widget.value, 120.0);
        assert_eq!(widget.applied_value, 120.0);
        assert!(!widget.is_dirty());

        // sync_applied when dirty should not overwrite pending user edit
        widget.increase();
        assert_eq!(widget.value, 125.0);
        assert!(widget.is_dirty());
        widget.sync_applied(130.0);
        assert_eq!(widget.value, 125.0);
        assert_eq!(widget.applied_value, 130.0);
        assert!(widget.is_dirty());
    }
}
