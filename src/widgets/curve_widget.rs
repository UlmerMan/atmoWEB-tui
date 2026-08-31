// ponytail: minimal heating curve widget with keyboard controls and non-blocking runner
use std::time::{Duration, Instant};

use ratatui::{
    buffer::Buffer,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    symbols,
    text::{Line, Span},
    widgets::{Axis, Block, Borders, Chart, Dataset, GraphType, Row, Table, Widget},
};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CurvePoint {
    pub time_min: f64,
    pub temp: f32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RunnerStatus {
    Idle,
    Running,
    Paused,
    Finished,
}

#[derive(Debug, Clone)]
pub struct CurveRunner {
    pub status: RunnerStatus,
    start_time: Option<Instant>,
    elapsed_before_pause: Duration,
}

impl Default for CurveRunner {
    fn default() -> Self {
        Self::new()
    }
}

impl CurveRunner {
    pub fn new() -> Self {
        Self {
            status: RunnerStatus::Idle,
            start_time: None,
            elapsed_before_pause: Duration::ZERO,
        }
    }

    pub fn start_or_resume(&mut self) {
        match self.status {
            RunnerStatus::Idle | RunnerStatus::Finished => {
                self.start_time = Some(Instant::now());
                self.elapsed_before_pause = Duration::ZERO;
                self.status = RunnerStatus::Running;
            }
            RunnerStatus::Paused => {
                self.start_time = Some(Instant::now());
                self.status = RunnerStatus::Running;
            }
            RunnerStatus::Running => {}
        }
    }

    pub fn pause(&mut self) {
        if self.status == RunnerStatus::Running {
            if let Some(start) = self.start_time {
                self.elapsed_before_pause += start.elapsed();
            }
            self.start_time = None;
            self.status = RunnerStatus::Paused;
        }
    }

    pub fn toggle_run_pause(&mut self) {
        match self.status {
            RunnerStatus::Running => self.pause(),
            RunnerStatus::Idle | RunnerStatus::Paused | RunnerStatus::Finished => {
                self.start_or_resume()
            }
        }
    }

    pub fn stop(&mut self) {
        self.status = RunnerStatus::Idle;
        self.start_time = None;
        self.elapsed_before_pause = Duration::ZERO;
    }

    pub fn elapsed(&self) -> Duration {
        match self.status {
            RunnerStatus::Running => {
                let current = self.start_time.map_or(Duration::ZERO, |s| s.elapsed());
                self.elapsed_before_pause + current
            }
            RunnerStatus::Paused => self.elapsed_before_pause,
            RunnerStatus::Idle | RunnerStatus::Finished => Duration::ZERO,
        }
    }

    pub fn mark_finished(&mut self) {
        self.status = RunnerStatus::Finished;
        self.start_time = None;
    }
}

pub struct CurveWidget {
    pub title: String,
    pub unit: String,
    pub min_temp: f32,
    pub max_temp: f32,
    pub points: Vec<CurvePoint>,
    pub selected: usize,
    pub selected_tile: bool,
    pub runner: CurveRunner,
    temp_step: f32,
    time_step_min: f64,
}

impl CurveWidget {
    pub fn new(title: &str, unit: &str, min_temp: f32, max_temp: f32) -> Self {
        // Default sensible heating curve (warm up, soak, ramp up, soak, cool)
        let default_points = vec![
            CurvePoint {
                time_min: 0.0,
                temp: 20.0,
            },
            CurvePoint {
                time_min: 15.0,
                temp: 100.0,
            },
            CurvePoint {
                time_min: 45.0,
                temp: 100.0,
            },
            CurvePoint {
                time_min: 65.0,
                temp: 200.0,
            },
            CurvePoint {
                time_min: 125.0,
                temp: 200.0,
            },
            CurvePoint {
                time_min: 155.0,
                temp: 30.0,
            },
        ];

        Self {
            title: title.to_string(),
            unit: unit.to_string(),
            min_temp,
            max_temp,
            points: default_points,
            selected: 1,
            selected_tile: false,
            runner: CurveRunner::new(),
            temp_step: 5.0,
            time_step_min: 5.0,
        }
    }

    pub fn select(&mut self) {
        self.selected_tile = true;
    }

    pub fn deselect(&mut self) {
        self.selected_tile = false;
    }

    pub fn next_point(&mut self) {
        if !self.points.is_empty() && self.selected + 1 < self.points.len() {
            self.selected += 1;
        }
    }

    pub fn prev_point(&mut self) {
        if self.selected > 0 {
            self.selected -= 1;
        }
    }

    pub fn increase_temp(&mut self) {
        if self.selected < self.points.len() {
            self.points[self.selected].temp =
                (self.points[self.selected].temp + self.temp_step).min(self.max_temp);
        }
    }

    pub fn decrease_temp(&mut self) {
        if self.selected < self.points.len() {
            self.points[self.selected].temp =
                (self.points[self.selected].temp - self.temp_step).max(self.min_temp);
        }
    }

    pub fn set_selected_temp(&mut self, temp: f32) {
        if self.selected < self.points.len() {
            self.points[self.selected].temp = temp.clamp(self.min_temp, self.max_temp);
        }
    }

    pub fn set_selected_time(&mut self, time_min: f64) {
        if self.selected == 0 {
            // First point always at 0 min
            return;
        }
        if self.selected < self.points.len() {
            let min_allowed = self.points[self.selected - 1].time_min + 1.0;
            let target_time = time_min.max(min_allowed);
            let delta = target_time - self.points[self.selected].time_min;
            // Shift this and all subsequent points to preserve segment durations
            for p in &mut self.points[self.selected..] {
                p.time_min += delta;
            }
        }
    }

    pub fn increase_time(&mut self) {
        if self.selected == 0 || self.selected >= self.points.len() {
            return;
        }
        let delta = self.time_step_min;
        for p in &mut self.points[self.selected..] {
            p.time_min += delta;
        }
    }

    pub fn decrease_time(&mut self) {
        if self.selected == 0 || self.selected >= self.points.len() {
            return;
        }
        let prev_time = self.points[self.selected - 1].time_min;
        let current_time = self.points[self.selected].time_min;
        let max_reduction = (current_time - (prev_time + 1.0)).max(0.0);
        let delta = self.time_step_min.min(max_reduction);
        if delta > 0.0 {
            for p in &mut self.points[self.selected..] {
                p.time_min -= delta;
            }
        }
    }

    pub fn add_point(&mut self) {
        if self.points.is_empty() {
            self.points.push(CurvePoint {
                time_min: 0.0,
                temp: 20.0,
            });
            self.selected = 0;
            return;
        }

        if self.selected + 1 >= self.points.len() {
            // Add after last
            let last = *self.points.last().unwrap();
            let new_point = CurvePoint {
                time_min: last.time_min + 15.0,
                temp: last.temp,
            };
            self.points.push(new_point);
            self.selected = self.points.len() - 1;
        } else {
            // Insert between selected and selected + 1
            let p0 = self.points[self.selected];
            let p1 = self.points[self.selected + 1];
            let mid_time = (p0.time_min + p1.time_min) / 2.0;
            let mid_temp = (p0.temp + p1.temp) / 2.0;
            self.points.insert(
                self.selected + 1,
                CurvePoint {
                    time_min: mid_time,
                    temp: mid_temp,
                },
            );
            self.selected += 1;
        }
    }

    pub fn delete_point(&mut self) {
        if self.points.len() <= 2 {
            // Keep at least 2 points for a valid curve
            return;
        }

        self.points.remove(self.selected);
        if self.selected >= self.points.len() {
            self.selected = self.points.len() - 1;
        }
        // Ensure point 0 always starts at 0 min
        if self.selected == 0 {
            self.points[0].time_min = 0.0;
        }
    }

    pub fn total_duration_min(&self) -> f64 {
        self.points.last().map_or(0.0, |p| p.time_min)
    }

    pub fn max_curve_temp(&self) -> f32 {
        self.points
            .iter()
            .map(|p| p.temp)
            .fold(self.min_temp, f32::max)
    }

    pub fn interpolate_temp(&self, time_min: f64) -> f32 {
        if self.points.is_empty() {
            return self.min_temp;
        }
        if time_min <= self.points[0].time_min {
            return self.points[0].temp;
        }
        if time_min >= self.points.last().unwrap().time_min {
            return self.points.last().unwrap().temp;
        }

        for window in self.points.windows(2) {
            let p0 = window[0];
            let p1 = window[1];
            if time_min >= p0.time_min && time_min <= p1.time_min {
                let dt = p1.time_min - p0.time_min;
                if dt.abs() < 1e-6 {
                    return p1.temp;
                }
                let ratio = (time_min - p0.time_min) / dt;
                return p0.temp + (p1.temp - p0.temp) * ratio as f32;
            }
        }

        self.points.last().unwrap().temp
    }

    /// Evaluates current target temperature if running, updating status if complete
    pub fn poll_runner(&mut self) -> Option<f32> {
        if self.runner.status != RunnerStatus::Running {
            return None;
        }

        let elapsed_min = self.runner.elapsed().as_secs_f64() / 60.0;
        let total_min = self.total_duration_min();

        if elapsed_min >= total_min {
            self.runner.mark_finished();
            return self.points.last().map(|p| p.temp);
        }

        Some(self.interpolate_temp(elapsed_min))
    }
}

impl Widget for &CurveWidget {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let block_style = if self.selected_tile {
            Style::default().fg(Color::Cyan)
        } else {
            Style::default()
        };

        let block = Block::default()
            .title(format!(" {} [3] ", self.title))
            .borders(Borders::ALL)
            .style(block_style);

        let inner = block.inner(area);
        block.render(area, buf);

        if inner.height < 6 || inner.width < 20 {
            return;
        }

        // Layout: Top chart (flexible), bottom info & table (fixed height)
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Min(8), Constraint::Length(7)])
            .split(inner);

        // Prepare chart datasets
        let max_time = self.total_duration_min().max(10.0);
        let x_bounds = [0.0, (max_time * 1.05).ceil()];
        let y_max = (self.max_curve_temp() + 20.0).min(self.max_temp);
        let y_bounds = [self.min_temp as f64, y_max as f64];

        // 1. Line interpolation points for curve
        let mut line_data: Vec<(f64, f64)> = Vec::new();
        for p in &self.points {
            line_data.push((p.time_min, p.temp as f64));
        }

        // 2. Node markers
        let node_data: Vec<(f64, f64)> = self
            .points
            .iter()
            .map(|p| (p.time_min, p.temp as f64))
            .collect();

        // 3. Selected point marker & vertical crosshair
        let mut selected_data: Vec<(f64, f64)> = Vec::new();
        let mut guide_data: Vec<(f64, f64)> = Vec::new();
        if let Some(sp) = self.points.get(self.selected) {
            selected_data.push((sp.time_min, sp.temp as f64));
            guide_data.push((sp.time_min, self.min_temp as f64));
            guide_data.push((sp.time_min, sp.temp as f64));
        }

        // 4. Runner current position marker if running or paused
        let mut runner_data: Vec<(f64, f64)> = Vec::new();
        if self.runner.status == RunnerStatus::Running || self.runner.status == RunnerStatus::Paused
        {
            let elapsed_min = self.runner.elapsed().as_secs_f64() / 60.0;
            let current_target = self.interpolate_temp(elapsed_min);
            runner_data.push((elapsed_min, current_target as f64));
        }

        let mut datasets = vec![
            Dataset::default()
                .name("Curve")
                .marker(symbols::Marker::Braille)
                .graph_type(GraphType::Line)
                .style(Style::default().fg(Color::Yellow))
                .data(&line_data),
            Dataset::default()
                .name("Nodes")
                .marker(symbols::Marker::Dot)
                .graph_type(GraphType::Scatter)
                .style(Style::default().fg(Color::LightYellow))
                .data(&node_data),
            Dataset::default()
                .name("Selected")
                .marker(symbols::Marker::Block)
                .graph_type(GraphType::Scatter)
                .style(
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD),
                )
                .data(&selected_data),
            Dataset::default()
                .name("Guide")
                .marker(symbols::Marker::Braille)
                .graph_type(GraphType::Line)
                .style(Style::default().fg(Color::DarkGray))
                .data(&guide_data),
        ];

        if !runner_data.is_empty() {
            datasets.push(
                Dataset::default()
                    .name("Live Target")
                    .marker(symbols::Marker::Block)
                    .graph_type(GraphType::Scatter)
                    .style(
                        Style::default()
                            .fg(Color::Green)
                            .add_modifier(Modifier::RAPID_BLINK),
                    )
                    .data(&runner_data),
            );
        }

        let y_mid = (self.min_temp + y_max) / 2.0;
        let chart = Chart::new(datasets)
            .x_axis(
                Axis::default()
                    .title("Time (min)")
                    .style(Style::default().fg(Color::DarkGray))
                    .bounds(x_bounds)
                    .labels(vec![
                        Span::raw("0m"),
                        Span::raw(format!("{:.0}m", max_time / 2.0)),
                        Span::raw(format!("{:.0}m", max_time)),
                    ]),
            )
            .y_axis(
                Axis::default()
                    .title(self.unit.clone())
                    .style(Style::default().fg(Color::DarkGray))
                    .bounds(y_bounds)
                    .labels(vec![
                        Span::raw(format!("{:.0}", self.min_temp)),
                        Span::raw(format!("{:.0}", y_mid)),
                        Span::raw(format!("{:.0}", y_max)),
                    ]),
            );

        chart.render(chunks[0], buf);

        // Bottom Pane: Status + Point Table + Keymap
        let bottom_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(1), // Status Bar
                Constraint::Length(4), // Selected Point Info / Table
                Constraint::Length(1), // Keybind Bar
            ])
            .split(chunks[1]);

        // 1. Status Bar
        let status_span = match self.runner.status {
            RunnerStatus::Idle => Span::styled("● IDLE", Style::default().fg(Color::DarkGray)),
            RunnerStatus::Running => {
                let elapsed = self.runner.elapsed();
                let mins = elapsed.as_secs() / 60;
                let secs = elapsed.as_secs() % 60;
                Span::styled(
                    format!(
                        "▶ RUNNING ({:02}:{:02} / {:.0}m) Target: {:.1}{}",
                        mins,
                        secs,
                        self.total_duration_min(),
                        self.interpolate_temp(elapsed.as_secs_f64() / 60.0),
                        self.unit
                    ),
                    Style::default()
                        .fg(Color::Green)
                        .add_modifier(Modifier::BOLD),
                )
            }
            RunnerStatus::Paused => {
                let elapsed = self.runner.elapsed();
                let mins = elapsed.as_secs() / 60;
                let secs = elapsed.as_secs() % 60;
                Span::styled(
                    format!("⏸ PAUSED ({:02}:{:02})", mins, secs),
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD),
                )
            }
            RunnerStatus::Finished => Span::styled(
                "✓ FINISHED",
                Style::default()
                    .fg(Color::LightCyan)
                    .add_modifier(Modifier::BOLD),
            ),
        };

        let total_time_span = Span::styled(
            format!(
                "Total: {:.0} min | Peak: {:.1} {}",
                self.total_duration_min(),
                self.max_curve_temp(),
                self.unit
            ),
            Style::default().fg(Color::White),
        );

        let status_line = Line::from(vec![
            status_span,
            Span::raw("   "),
            Span::styled("│", Style::default().fg(Color::DarkGray)),
            Span::raw("   "),
            total_time_span,
        ]);
        buf.set_line(bottom_layout[0].x, bottom_layout[0].y, &status_line, bottom_layout[0].width);

        // 2. Point Table / Selected Point Card
        let header = Row::new(vec!["#", "Time", "Duration", "Target Temp", "Ramp Rate"])
            .style(Style::default().fg(Color::DarkGray));

        let rows: Vec<Row> = self
            .points
            .iter()
            .enumerate()
            .map(|(i, p)| {
                let is_sel = i == self.selected;
                let duration_str = if i == 0 {
                    "-".to_string()
                } else {
                    format!("+{:.0}m", p.time_min - self.points[i - 1].time_min)
                };

                let ramp_str = if i == 0 {
                    "Start".to_string()
                } else {
                    let dt = p.time_min - self.points[i - 1].time_min;
                    let dtemp = p.temp - self.points[i - 1].temp;
                    if dt > 0.0 {
                        let rate = dtemp / dt as f32;
                        if rate.abs() < 0.01 {
                            "Hold".to_string()
                        } else {
                            format!("{:+.1} °C/m", rate)
                        }
                    } else {
                        "Hold".to_string()
                    }
                };

                let prefix = if is_sel { "▶ " } else { "  " };
                let row_style = if is_sel {
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(Color::White)
                };

                Row::new(vec![
                    format!("{}{}", prefix, i + 1),
                    format!("{:.0}m", p.time_min),
                    duration_str,
                    format!("{:.1}{}", p.temp, self.unit),
                    ramp_str,
                ])
                .style(row_style)
            })
            .collect();

        // Scroll table to keep selected row visible
        let visible_height = bottom_layout[1].height.saturating_sub(1) as usize;
        let start_row = if self.selected >= visible_height {
            self.selected.saturating_sub(visible_height - 1)
        } else {
            0
        };
        let visible_rows: Vec<Row> = rows.into_iter().skip(start_row).take(visible_height).collect();

        let table = Table::new(
            visible_rows,
            [
                Constraint::Length(6),
                Constraint::Length(8),
                Constraint::Length(10),
                Constraint::Length(14),
                Constraint::Length(12),
            ],
        )
        .header(header);

        table.render(bottom_layout[1], buf);

        // 3. Keybinds Help Bar
        let keybinds_line = Line::from(vec![
            Span::styled("[←/→]", Style::default().fg(Color::Yellow)),
            Span::raw(" Step "),
            Span::styled("[↑/↓]", Style::default().fg(Color::Yellow)),
            Span::raw(" Temp "),
            Span::styled("[+/-]", Style::default().fg(Color::Yellow)),
            Span::raw(" Time "),
            Span::styled("[a]", Style::default().fg(Color::Yellow)),
            Span::raw(" Add "),
            Span::styled("[d]", Style::default().fg(Color::Yellow)),
            Span::raw(" Del "),
            Span::styled("[Space]", Style::default().fg(Color::Yellow)),
            Span::raw(" Run/Pause "),
            Span::styled("[s]", Style::default().fg(Color::Yellow)),
            Span::raw(" Stop "),
            Span::styled("[e]", Style::default().fg(Color::Yellow)),
            Span::raw(" Edit"),
        ])
        .alignment(Alignment::Left);

        buf.set_line(bottom_layout[2].x, bottom_layout[2].y, &keybinds_line, bottom_layout[2].width);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_interpolation() {
        let mut widget = CurveWidget::new("Curve", "°C", 20.0, 300.0);
        widget.points = vec![
            CurvePoint {
                time_min: 0.0,
                temp: 20.0,
            },
            CurvePoint {
                time_min: 10.0,
                temp: 120.0,
            },
            CurvePoint {
                time_min: 20.0,
                temp: 120.0,
            },
        ];

        assert_eq!(widget.interpolate_temp(0.0), 20.0);
        assert_eq!(widget.interpolate_temp(5.0), 70.0);
        assert_eq!(widget.interpolate_temp(10.0), 120.0);
        assert_eq!(widget.interpolate_temp(15.0), 120.0);
        assert_eq!(widget.interpolate_temp(20.0), 120.0);
        assert_eq!(widget.interpolate_temp(30.0), 120.0);
    }

    #[test]
    fn test_add_delete_point() {
        let mut widget = CurveWidget::new("Curve", "°C", 20.0, 300.0);
        let initial_len = widget.points.len();

        widget.selected = 1;
        widget.add_point();
        assert_eq!(widget.points.len(), initial_len + 1);

        widget.delete_point();
        assert_eq!(widget.points.len(), initial_len);
    }

    #[test]
    fn test_time_adjust() {
        let mut widget = CurveWidget::new("Curve", "°C", 20.0, 300.0);
        widget.selected = 1;
        let t1_orig = widget.points[1].time_min;
        let t2_orig = widget.points[2].time_min;

        widget.increase_time();
        assert_eq!(widget.points[1].time_min, t1_orig + 5.0);
        assert_eq!(widget.points[2].time_min, t2_orig + 5.0);
    }

    #[test]
    fn test_runner_toggle() {
        let mut runner = CurveRunner::new();
        assert_eq!(runner.status, RunnerStatus::Idle);

        runner.toggle_run_pause();
        assert_eq!(runner.status, RunnerStatus::Running);

        runner.toggle_run_pause();
        assert_eq!(runner.status, RunnerStatus::Paused);

        runner.stop();
        assert_eq!(runner.status, RunnerStatus::Idle);
    }
}
