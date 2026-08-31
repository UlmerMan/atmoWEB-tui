// ponytail: minimal app coordinator with single non-blocking event loop
use std::error::Error;
use std::time::{Duration, Instant};

use ratatui::{
    DefaultTerminal,
    crossterm::event::{self, Event},
};

use crate::atmoweb::AtmoWeb;
use crate::tui::handler;
use crate::tui::state::{AppMode, AppState, InputState, ManualFocus, RightView};
use crate::tui::ui;
use crate::widgets::control_widget::ControlWidget;
use crate::widgets::curve_widget::CurveWidget;
use crate::widgets::graph_widget::GraphWidget;

const REFRESH_INTERVAL: Duration = Duration::from_secs(2);
const PROBE_INTERVAL: Duration = Duration::from_millis(500);

pub struct App {
    pub oven: AtmoWeb,
    pub oven_ip: String,
    pub exit: bool,
    pub state: AppState,
    pub mode: AppMode,
    pub manual_focus: ManualFocus,
    pub input_state: InputState,
    pub right_view: RightView,
    pub online: bool,
    pub temp: ControlWidget,
    pub flap: ControlWidget,
    pub fan: ControlWidget,
    pub curve: CurveWidget,
    pub graph: GraphWidget,
    pub spinner_tick: u64,
    pub connect_attempts: u32,
    pub connect_status: String,
    last_refresh: Instant,
    last_probe: Instant,
}

impl App {
    pub fn new(oven_ip: String) -> Self {
        let mut temp = ControlWidget::new("Temperature [0]", 20.0, "°C", 0.5, 20.0, 300.0);
        let flap = ControlWidget::new("Flap [1]", 0.0, "%", 1.0, 0.0, 100.0);
        let fan = ControlWidget::new("Fan [2]", 0.0, "%", 1.0, 0.0, 100.0);
        temp.select();

        let initial_status = format!("Connecting to http://{}/atmoweb...", oven_ip);

        Self {
            oven: AtmoWeb::new(oven_ip.clone()),
            oven_ip,
            exit: false,
            state: AppState::Connecting,
            mode: AppMode::Manual,
            manual_focus: ManualFocus::Temp,
            input_state: InputState::Normal,
            right_view: RightView::LiveGraph,
            online: false,
            temp,
            flap,
            fan,
            curve: CurveWidget::new("Heating Curve", "°C", 20.0, 300.0),
            graph: GraphWidget::new("Live Temperature History", "°C", 0.0, 300.0),
            spinner_tick: 0,
            connect_attempts: 0,
            connect_status: initial_status,
            last_refresh: Instant::now(),
            last_probe: Instant::now() - PROBE_INTERVAL,
        }
    }

    pub async fn run(&mut self, terminal: &mut DefaultTerminal) -> Result<(), Box<dyn Error>> {
        while !self.exit {
            self.spinner_tick = self.spinner_tick.wrapping_add(1);

            match self.state {
                AppState::Connecting => {
                    if self.last_probe.elapsed() >= PROBE_INTERVAL {
                        self.last_probe = Instant::now();
                        self.connect_attempts += 1;
                        self.connect_status = format!(
                            "Probing oven at {} (attempt #{})...",
                            self.oven_ip, self.connect_attempts
                        );

                        if self.oven.is_online().await {
                            self.online = true;
                            self.connect_status = "Connected! Fetching initial state...".to_string();
                            self.initial_sync().await;
                            self.state = AppState::Running;
                            self.last_refresh = Instant::now();
                        } else {
                            self.connect_status = format!(
                                "Oven unreachable at {}. Waiting for connection...",
                                self.oven_ip
                            );
                        }
                    }
                }
                AppState::Running => {
                    if self.last_refresh.elapsed() >= REFRESH_INTERVAL {
                        self.refresh().await;
                        self.last_refresh = Instant::now();
                    }
                }
            }

            terminal.draw(|frame| ui::draw(self, frame))?;

            if event::poll(Duration::from_millis(40))?
                && let Event::Key(key) = event::read()?
            {
                handler::handle_key(self, key).await?;
            }
        }
        Ok(())
    }

    pub async fn initial_sync(&mut self) {
        if let Ok(v) = self.oven.read_temp1().await {
            self.temp.set_current(v as f32);
        }
        if let Ok(set_temp) = self.oven.read_set_temp().await {
            self.temp.sync_applied(set_temp);
            self.graph.push_sample(self.temp.current, set_temp);
        }
        if let Ok(v) = self.oven.read_flap().await {
            self.flap.set_current(v as f32);
            self.flap.sync_applied(v as f32);
        }
        if let Ok(v) = self.oven.read_fan().await {
            self.fan.set_current(v as f32);
            self.fan.sync_applied(v as f32);
        }
    }

    pub async fn refresh(&mut self) {
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

        // Auto Mode curve execution: step target temperature non-blocking
        if self.mode == AppMode::Auto
            && let Some(target_temp) = self.curve.poll_runner()
        {
            self.temp.set_target(target_temp);
            let _ = self.oven.set_temp(target_temp).await;
            self.temp.mark_applied();
        }

        if let Ok(set_temp) = self.oven.read_set_temp().await {
            self.graph.push_sample(self.temp.current, set_temp);
            if !self.temp.is_dirty() && self.mode == AppMode::Manual {
                self.temp.sync_applied(set_temp);
            }
        }
    }

    pub fn toggle_mode(&mut self) {
        self.set_mode(self.mode.toggle());
    }

    pub fn set_mode(&mut self, mode: AppMode) {
        self.mode = mode;
        match self.mode {
            AppMode::Manual => {
                self.temp.set_locked(false);
                self.flap.set_locked(false);
                self.fan.set_locked(false);
                self.curve.deselect();
                self.right_view = RightView::LiveGraph;
                self.apply_manual_focus();
            }
            AppMode::Auto => {
                self.temp.set_locked(true);
                self.flap.set_locked(true);
                self.fan.set_locked(true);
                self.curve.select();
                self.right_view = RightView::Curve;
            }
        }
    }

    pub fn toggle_right_view(&mut self) {
        self.right_view = match self.right_view {
            RightView::Curve => RightView::LiveGraph,
            RightView::LiveGraph => RightView::Curve,
        };
    }

    pub fn change_manual_focus(&mut self, new_focus: ManualFocus) {
        self.manual_focus = new_focus;
        self.apply_manual_focus();
    }

    fn apply_manual_focus(&mut self) {
        self.temp.deselect();
        self.flap.deselect();
        self.fan.deselect();
        match self.manual_focus {
            ManualFocus::Temp => self.temp.select(),
            ManualFocus::Flap => self.flap.select(),
            ManualFocus::Fan => self.fan.select(),
        }
    }

    pub fn focused_control_mut(&mut self) -> &mut ControlWidget {
        match self.manual_focus {
            ManualFocus::Temp => &mut self.temp,
            ManualFocus::Flap => &mut self.flap,
            ManualFocus::Fan => &mut self.fan,
        }
    }

    pub async fn send_current_value(&mut self) {
        if self.mode == AppMode::Manual {
            let res = match self.manual_focus {
                ManualFocus::Temp => self.oven.set_temp(self.temp.value).await.map(f64::from),
                ManualFocus::Flap => self.oven.set_flap(self.flap.value as f64).await,
                ManualFocus::Fan => self.oven.set_fan(self.fan.value as f64).await,
            };
            if res.is_ok() {
                self.focused_control_mut().mark_applied();
            }
            self.refresh().await;
            self.last_refresh = Instant::now();
        }
    }
}
