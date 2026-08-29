use std::collections::VecDeque;

use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Modifier, Style},
    symbols,
    text::Span,
    widgets::{Axis, Block, Borders, Chart, Dataset, GraphType, Widget},
};

/// Wie viele Messpunkte im Verlauf aufgehoben werden.
const HISTORY_LEN: usize = 60;

pub struct GraphWidget {
    pub title: String,
    pub unit: String,
    pub min: f32,
    pub max: f32,
    current_history: VecDeque<(f64, f64)>,
    target_history: VecDeque<(f64, f64)>,
    next_x: f64,
}

impl GraphWidget {
    pub fn new(title: &str, unit: &str, min: f32, max: f32) -> Self {
        Self {
            title: title.to_string(),
            unit: unit.to_string(),
            min,
            max,
            current_history: VecDeque::with_capacity(HISTORY_LEN),
            target_history: VecDeque::with_capacity(HISTORY_LEN),
            next_x: 0.0,
        }
    }

    pub fn push_sample(&mut self, current: f32, target: f32) {
        if self.current_history.len() >= HISTORY_LEN {
            self.current_history.pop_front();
        }
        if self.target_history.len() >= HISTORY_LEN {
            self.target_history.pop_front();
        }

        self.current_history
            .push_back((self.next_x, current as f64));
        self.target_history.push_back((self.next_x, target as f64));
        self.next_x += 1.0;
    }

    fn x_bounds(&self) -> [f64; 2] {
        let min_x = self.current_history.front().map(|(x, _)| *x).unwrap_or(0.0);
        let max_x = self.next_x.max(min_x + 1.0);
        [min_x, max_x]
    }
}

impl Widget for &GraphWidget {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let block = Block::default()
            .title(self.title.clone())
            .borders(Borders::ALL)
            .style(Style::default());

        let current_data: Vec<(f64, f64)> = self.current_history.iter().copied().collect();
        let target_data: Vec<(f64, f64)> = self.target_history.iter().copied().collect();

        let datasets = vec![
            Dataset::default()
                .name("current")
                .marker(symbols::Marker::Braille)
                .graph_type(GraphType::Line)
                .style(Style::default().fg(Color::Green))
                .data(&current_data),
            Dataset::default()
                .name("target")
                .marker(symbols::Marker::Braille)
                .graph_type(GraphType::Line)
                .style(
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD),
                )
                .data(&target_data),
        ];

        let x_bounds = self.x_bounds();
        let y_bounds = [self.min as f64, self.max as f64];
        let y_mid = (self.min + self.max) / 2.0;

        let chart = Chart::new(datasets)
            .block(block)
            .x_axis(
                Axis::default()
                    .title("time")
                    .style(Style::default().fg(Color::DarkGray))
                    .bounds(x_bounds)
                    .labels(Vec::<Span>::new()),
            )
            .y_axis(
                Axis::default()
                    .title(self.unit.clone())
                    .style(Style::default().fg(Color::DarkGray))
                    .bounds(y_bounds)
                    .labels(vec![
                        Span::raw(format!("{:.0}", self.min)),
                        Span::raw(format!("{:.0}", y_mid)),
                        Span::raw(format!("{:.0}", self.max)),
                    ]),
            );

        chart.render(area, buf);
    }
}
