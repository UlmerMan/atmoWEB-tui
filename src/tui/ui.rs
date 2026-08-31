use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
};

use crate::tui::app::App;
use crate::tui::state::{AppMode, InputState, RightView};

pub const MIN_WIDTH: u16 = 70;
pub const MIN_HEIGHT: u16 = 20;

pub fn draw(app: &App, frame: &mut Frame) {
    let area = frame.area();

    if area.width < MIN_WIDTH || area.height < MIN_HEIGHT {
        draw_too_small_warning(frame, area);
        return;
    }

    // Top Header (3 rows) + Body Area
    let main_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Min(15)])
        .split(area);

    draw_header(app, frame, main_chunks[0]);
    draw_body(app, frame, main_chunks[1]);

    if let InputState::Editing(ref widget) = app.input_state {
        let popup = layout_centered_box(frame.area(), 60, 3);
        frame.render_widget(ratatui::widgets::Clear, popup);
        frame.render_widget(widget, popup);
    }
}

fn draw_header(app: &App, frame: &mut Frame, area: Rect) {
    let header_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Fill(1), Constraint::Length(32)])
        .split(area);

    // Mode Banner
    let (mode_badge, mode_style, hint_text) = match app.mode {
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

    let mode_block = Block::default().borders(Borders::ALL).title(" System Mode ");
    let mode_p = Paragraph::new(mode_line).block(mode_block);
    frame.render_widget(mode_p, header_chunks[0]);

    // Oven Info / Status Banner
    let online_span = if app.online {
        Span::styled(
            "● ONLINE",
            Style::default()
                .fg(Color::Green)
                .add_modifier(Modifier::BOLD),
        )
    } else {
        Span::styled(
            "○ OFFLINE",
            Style::default()
                .fg(Color::Red)
                .add_modifier(Modifier::BOLD),
        )
    };

    let status_line = Line::from(vec![
        Span::raw("IP: "),
        Span::styled(&app.oven_ip, Style::default().fg(Color::Cyan)),
        Span::raw("  "),
        online_span,
    ]);

    let status_block = Block::default().borders(Borders::ALL).title(" Oven Status ");
    let status_p = Paragraph::new(status_line)
        .alignment(Alignment::Right)
        .block(status_block);
    frame.render_widget(status_p, header_chunks[1]);
}

fn draw_body(app: &App, frame: &mut Frame, area: Rect) {
    let body_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(28), Constraint::Percentage(72)])
        .split(area);

    // Left controls: 3 vertical tiles
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

    // Right panel: Curve widget or Live Graph
    match app.right_view {
        RightView::Curve => frame.render_widget(&app.curve, body_chunks[1]),
        RightView::LiveGraph => frame.render_widget(&app.graph, body_chunks[1]),
    }
}

fn layout_centered_box(area: Rect, width: u16, height: u16) -> Rect {
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
        .alignment(Alignment::Center)
        .style(Style::default().fg(Color::Red))
        .block(Block::default().borders(Borders::ALL));

    frame.render_widget(paragraph, area);
}
