use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    widgets::{Block, Borders, Clear, Paragraph},
};

use crate::tui::app::App;
use crate::tui::state::{AppState, InputState, RightView};
use crate::widgets::connecting_widget::ConnectingWidget;
use crate::widgets::footer_widget::FooterWidget;
use crate::widgets::header_widget::HeaderWidget;

pub const MIN_WIDTH: u16 = 70;
pub const MIN_HEIGHT: u16 = 20;

pub fn draw(app: &App, frame: &mut Frame) {
    let area = frame.area();

    if area.width < MIN_WIDTH || area.height < MIN_HEIGHT {
        draw_too_small_warning(frame, area);
        return;
    }

    if app.state == AppState::Connecting {
        let popup = layout_centered_box(area, 62.min(area.width.saturating_sub(4)), 11);
        frame.render_widget(Clear, popup);
        let widget = ConnectingWidget::new(&app.oven_ip, &app.connect_status, app.spinner_tick);
        frame.render_widget(&widget, popup);
        return;
    }

    // Top Header (3 rows) + Body Area (flexible) + Footer (1 row)
    let main_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(14),
            Constraint::Length(1),
        ])
        .split(area);

    let header = HeaderWidget::new(app.mode, app.online, &app.oven_ip);
    frame.render_widget(&header, main_chunks[0]);

    draw_body(app, frame, main_chunks[1]);

    let footer = FooterWidget::new(app.mode, &app.input_state);
    frame.render_widget(&footer, main_chunks[2]);

    if let InputState::Editing(ref widget) = app.input_state {
        let popup = layout_centered_box(frame.area(), 60, 3);
        frame.render_widget(Clear, popup);
        frame.render_widget(&**widget, popup);
    }
}

fn draw_body(app: &App, frame: &mut Frame, area: Rect) {
    let body_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(28), Constraint::Percentage(72)])
        .split(area);

    let control_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage(33),
            Constraint::Percentage(33),
            Constraint::Percentage(34),
        ])
        .split(body_chunks[0]);

    frame.render_widget(&app.temp, control_chunks[0]);
    frame.render_widget(&app.flap, control_chunks[1]);
    frame.render_widget(&app.fan, control_chunks[2]);

    match app.right_view {
        RightView::Curve => frame.render_widget(&app.curve, body_chunks[1]),
        RightView::LiveGraph => frame.render_widget(&app.graph, body_chunks[1]),
    }
}

pub fn layout_centered_box(area: Rect, width: u16, height: u16) -> Rect {
    let vertical = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Fill(1),
            Constraint::Length(height),
            Constraint::Fill(1),
        ])
        .split(area);

    let horizontal = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Fill(1),
            Constraint::Length(width),
            Constraint::Fill(1),
        ])
        .split(vertical[1]);

    horizontal[1]
}

fn draw_too_small_warning(frame: &mut Frame, area: Rect) {
    let text = format!(
        "Terminal Window too small!\nMin {}x{} needed,\ncurrent {}x{}.",
        MIN_WIDTH, MIN_HEIGHT, area.width, area.height
    );

    let paragraph = Paragraph::new(text)
        .alignment(ratatui::layout::Alignment::Center)
        .style(Style::default().fg(Color::Red))
        .block(Block::default().borders(Borders::ALL));

    frame.render_widget(paragraph, area);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tui::state::AppMode;
    use ratatui::Terminal;
    use ratatui::backend::TestBackend;

    #[test]
    fn test_ui_renders_connecting_and_running() {
        let backend = TestBackend::new(80, 24);
        let mut terminal = Terminal::new(backend).unwrap();

        let mut app = App::new("127.0.0.1:8080".to_string());
        assert_eq!(app.state, AppState::Connecting);

        // Render connecting screen
        terminal.draw(|f| draw(&app, f)).unwrap();

        // Switch to running and render
        app.state = AppState::Running;
        terminal.draw(|f| draw(&app, f)).unwrap();

        // Switch to auto mode and render
        app.set_mode(AppMode::Auto);
        terminal.draw(|f| draw(&app, f)).unwrap();
    }

    #[test]
    fn test_ui_renders_too_small() {
        let backend = TestBackend::new(40, 10);
        let mut terminal = Terminal::new(backend).unwrap();

        let app = App::new("127.0.0.1:8080".to_string());
        terminal.draw(|f| draw(&app, f)).unwrap();
    }
}
