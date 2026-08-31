use std::error::Error;
use std::time::{Duration, Instant};

use ratatui::{
    DefaultTerminal, Frame,
    crossterm::event::{self, Event, KeyCode, KeyEventKind},
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    widgets::{Block, Borders, Paragraph},
};

use crate::atmoweb::AtmoWeb;
use crate::widgets::control_widget::ControlWidget;
use crate::widgets::graph_widget::GraphWidget;

const REFRESH_INTERVAL: Duration = Duration::from_secs(2);

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
    graph: GraphWidget,
    focus: Focus,
    online: bool,
    editing: bool,
    last_refresh: Instant,
}

const MIN_WIDTH: u16 = 60;
const MIN_HEIGHT: u16 = 20;

impl App {
    pub fn new(oven_ip: String) -> Self {
        let mut temp = ControlWidget::new("Temperature [0]", 20.0, "°C", 0.5, 20.0, 300.0);
        temp.select();

        Self {
            oven: AtmoWeb::new(oven_ip.clone()),
            oven_ip,
            exit: false,
            temp,
            flap: ControlWidget::new("Flap [1]", 0.0, "%", 1.0, 0.0, 100.0),
            fan: ControlWidget::new("Fan [2]", 0.0, "%", 1.0, 0.0, 100.0),
            graph: GraphWidget::new("Temperature Curve", "°C", 0.0, 300.0),
            focus: Focus::Temp,
            online: false,
            editing: false,
            last_refresh: Instant::now() - REFRESH_INTERVAL,
        }
    }

    pub async fn run(&mut self, terminal: &mut DefaultTerminal) -> Result<(), Box<dyn Error>> {
        while !self.exit {
            if self.last_refresh.elapsed() >= REFRESH_INTERVAL {
                self.refresh().await;
                self.last_refresh = Instant::now();
            }
            
            if self.editing {
                if let Ok(value) = crate::widgets::float_input_widget::float_input_widget(
                    terminal,
                ) {
                    self.focused_widget_mut().set_target(value);
                }
                self.editing = false;
                
                self.send_current_value().await;
            } else {
                terminal.draw(|frame| self.draw(frame))?;
                self.handle_events().await?;
            }    
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

        self.graph
            .push_sample(self.temp.current, self.oven.read_set_temp().await.unwrap());
    }

    fn layout(&self, frame: &Frame) -> [Rect; 5] {
        let horizontal = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(30), Constraint::Percentage(70)].as_ref())
            .split(frame.area());

        let side_bar = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Fill(1), Constraint::Length(3)].as_ref())
            .split(horizontal[0]);

        let control_widgets = Layout::default()
            .direction(Direction::Vertical)
            .constraints(
                [
                    Constraint::Percentage(33),
                    Constraint::Percentage(33),
                    Constraint::Percentage(33),
                ]
                .as_ref(),
            )
            .split(side_bar[0]);

        [
            control_widgets[0],
            control_widgets[1],
            control_widgets[2],
            side_bar[1],
            horizontal[1],
        ]
    }

    fn draw_too_small_warning(&self, frame: &mut Frame, area: Rect) {
        let text = format!(
            "Terminal Window to small!\nMin {}x{} needed,\ncurrent {}x{}.",
            MIN_WIDTH, MIN_HEIGHT, area.width, area.height
        );

        let paragraph = Paragraph::new(text)
            .alignment(Alignment::Center)
            .style(Style::default().fg(Color::Red))
            .block(Block::default().borders(Borders::ALL));

        frame.render_widget(paragraph, area);
    }
    fn draw(&self, frame: &mut Frame) {
        let area = frame.area();

        if area.width < MIN_WIDTH || area.height < MIN_HEIGHT {
            self.draw_too_small_warning(frame, area);
            return;
        }

        let tiles = self.layout(frame);

        frame.render_widget(&self.temp, tiles[0]);
        frame.render_widget(&self.flap, tiles[1]);
        frame.render_widget(&self.fan, tiles[2]);

        let info = format!(
            "Ofen: {} ({})",
            self.oven_ip,
            if self.online { "online" } else { "offline" }
        );

        let block = Block::default().title("Status").borders(Borders::ALL);
        let paragraph =
            Paragraph::new(info)
                .block(block)
                .style(Style::default().fg(if self.online {
                    Color::Green
                } else {
                    Color::Red
                }));
        frame.render_widget(paragraph, tiles[3]);

        frame.render_widget(&self.graph, tiles[4]);
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
                    KeyCode::Right => self.focused_widget_mut().increase(),
                    KeyCode::Char('+') => self.focused_widget_mut().increase(),
                    KeyCode::Char('-') => self.focused_widget_mut().decrease(),
                    KeyCode::Left => self.focused_widget_mut().decrease(),
                    KeyCode::Down => self.focused_widget_mut().decrease(),
                    KeyCode::Enter => self.send_current_value().await,
                    KeyCode::Char('0') => self.change_focus(Focus::Temp),
                    KeyCode::Char('1') => self.change_focus(Focus::Flap),
                    KeyCode::Char('2') => self.change_focus(Focus::Fan),
                    KeyCode::Char('e') => self.editing = true,
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
        let _ = match self.focus {
            Focus::Temp => self.oven.set_temp(self.temp.value).await.map(f64::from),
            Focus::Flap => self.oven.set_flap(self.flap.value as f64).await,
            Focus::Fan => self.oven.set_fan(self.fan.value as f64).await,
        };

        self.refresh().await;
        self.last_refresh = Instant::now();
    }
}
