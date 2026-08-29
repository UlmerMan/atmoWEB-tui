use std::error::Error;
use std::time::{Duration, Instant};

use ratatui::{
    DefaultTerminal, Frame,
    crossterm::event::{self, Event, KeyCode, KeyEventKind},
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    widgets::{Block, Borders, Paragraph},
};

use crate::atmoweb::AtmoWeb;
use crate::widgets::control_widget::ControlWidget;

const REFRESH_INTERVAL: Duration = Duration::from_millis(50);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Focus {
    Temp,
    Flap,
    Fan,
}

impl Focus {
    fn next(self) -> Self {
        match self {
            Focus::Temp => Focus::Flap,
            Focus::Flap => Focus::Fan,
            Focus::Fan => Focus::Temp,
        }
    }
    fn prev(self) -> Self {
        match self {
            Focus::Temp => Focus::Fan,
            Focus::Flap => Focus::Temp,
            Focus::Fan => Focus::Flap,
        }
    }
}

pub struct App {
    oven: AtmoWeb,
    oven_ip: String,
    exit: bool,
    temp: ControlWidget,
    flap: ControlWidget,
    fan: ControlWidget,
    focus: Focus,
    status: String,
    online: bool,
    last_refresh: Instant,
}

impl App {
    pub fn new(oven_ip: String) -> Self {
        let mut temp = ControlWidget::new("Solltemperatur", 21.0, "°C", 0.5, 5.0, 35.0);
        temp.select(); // Fokus startet auf der Temperatur-Kachel

        Self {
            oven: AtmoWeb::new(oven_ip.clone()),
            oven_ip,
            exit: false,
            temp,
            flap: ControlWidget::new("Klappe", 0.0, "%", 5.0, 0.0, 100.0),
            fan: ControlWidget::new("Lüfter", 0.0, "%", 5.0, 0.0, 100.0),
            focus: Focus::Temp,
            status: "Bereit".to_string(),
            online: false,
            last_refresh: Instant::now() - REFRESH_INTERVAL,
        }
    }

    pub async fn run(&mut self, terminal: &mut DefaultTerminal) -> Result<(), Box<dyn Error>> {
        while !self.exit {
            if self.last_refresh.elapsed() >= REFRESH_INTERVAL {
                self.refresh().await;
                self.last_refresh = Instant::now();
            }

            terminal.draw(|frame| self.draw(frame))?;
            self.handle_events().await?;
        }
        Ok(())
    }

    async fn refresh(&mut self) {
        self.online = self.oven.is_online().await;

        if let Ok(v) = self.oven.read_temp1().await {
            self.temp.set_current(v as f32);
        }
        if let Ok(v) = self.oven.read_flap().await {
            self.flap.set_current(v as f32);
        }
        if let Ok(v) = self.oven.read_fan().await {
            self.fan.set_current(v as f32);
        }
    }

    fn layout(&self, frame: &Frame) -> [Rect; 4] {
        let vertical = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(frame.area());

        let top = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(vertical[0]);

        let bottom = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(vertical[1]);

        [top[0], top[1], bottom[0], bottom[1]]
    }

    fn draw(&self, frame: &mut Frame) {
        let tiles = self.layout(frame);

        frame.render_widget(&self.temp, tiles[0]);
        frame.render_widget(&self.flap, tiles[1]);
        frame.render_widget(&self.fan, tiles[2]);

        let info = format!(
            "Ofen: {} ({})\n\n{}\n\nTab: Kachel wechseln\n↵: Soll-Wert senden\n(Auto-Refresh alle {}s)",
            self.oven_ip,
            if self.online { "online" } else { "offline" },
            self.status,
            REFRESH_INTERVAL.as_secs(),
        );

        let block = Block::default().title("Status").borders(Borders::ALL);
        let paragraph = Paragraph::new(info)
            .block(block)
            .style(Style::default().fg(if self.online { Color::Green } else { Color::Red }));
        frame.render_widget(paragraph, tiles[3]);
    }

    async fn handle_events(&mut self) -> Result<(), Box<dyn Error>> {
        if event::poll(Duration::from_millis(200))? {
            if let Event::Key(key) = event::read()? {
                if key.kind != KeyEventKind::Press {
                    return Ok(());
                }
                match key.code {
                    KeyCode::Char('q') => self.exit = true,
                    KeyCode::Tab => self.change_focus(self.focus.next()),
                    KeyCode::BackTab => self.change_focus(self.focus.prev()),
                    KeyCode::Up => self.focused_widget_mut().increase(),
                    KeyCode::Down => self.focused_widget_mut().decrease(),
                    KeyCode::Enter => self.send_current_value().await,
                    _ => {}
                }
            }
        }
        Ok(())
    }

    fn change_focus(&mut self, new_focus: Focus) {
        self.focused_widget_mut().deselect();
        self.focus = new_focus;
        self.focused_widget_mut().select();
    }

    fn focused_widget_mut(&mut self) -> &mut ControlWidget {
        match self.focus {
            Focus::Temp => &mut self.temp,
            Focus::Flap => &mut self.flap,
            Focus::Fan => &mut self.fan,
        }
    }

    async fn send_current_value(&mut self) {
        let result = match self.focus {
            Focus::Temp => self.oven.set_temp(self.temp.value).await.map(f64::from),
            Focus::Flap => self.oven.set_flap(self.flap.value as f64).await,
            Focus::Fan => self.oven.set_fan(self.fan.value as f64).await,
        };

        self.status = match result {
            Ok(v) => format!("Übernommen: {v:.1}"),
            Err(e) => format!("Fehler: {e}"),
        };

        self.refresh().await;
        self.last_refresh = Instant::now();
    }
}