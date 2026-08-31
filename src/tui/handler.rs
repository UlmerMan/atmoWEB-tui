use std::error::Error;

use crossterm::event::{KeyCode, KeyEvent, KeyEventKind};

use crate::tui::app::App;
use crate::tui::state::{AppMode, InputState, ManualFocus};
use crate::widgets::float_input_widget::{FloatInput, FloatInputWidget};

pub async fn handle_key(app: &mut App, key: KeyEvent) -> Result<(), Box<dyn Error>> {
    if key.kind != KeyEventKind::Press {
        return Ok(());
    }

    // 1. Handle text input if editing
    if let InputState::Editing(ref mut input_widget) = app.input_state {
        match input_widget.handle_input(key.into()) {
            FloatInput::Some(val) => {
                match app.mode {
                    AppMode::Manual => {
                        app.focused_control_mut().set_target(val);
                        app.send_current_value().await;
                    }
                    AppMode::Auto => {
                        app.curve.set_selected_temp(val);
                    }
                }
                app.input_state = InputState::Normal;
            }
            FloatInput::Abort => {
                app.input_state = InputState::Normal;
            }
            FloatInput::None => {}
        }
        return Ok(());
    }

    // 2. Global Hotkeys
    match key.code {
        KeyCode::Char('q') => {
            app.exit = true;
            return Ok(());
        }
        KeyCode::Char('m') => {
            app.toggle_mode();
            return Ok(());
        }
        KeyCode::Char('g') => {
            app.toggle_right_view();
            return Ok(());
        }
        KeyCode::Char('e') => {
            app.input_state = InputState::Editing(FloatInputWidget::new());
            return Ok(());
        }
        _ => {}
    }

    // 3. Mode-Specific Key Dispatch
    match app.mode {
        AppMode::Manual => match key.code {
            KeyCode::Tab => app.change_manual_focus(app.manual_focus.next()),
            KeyCode::BackTab => app.change_manual_focus(app.manual_focus.prev()),
            KeyCode::Char('0') => app.change_manual_focus(ManualFocus::Temp),
            KeyCode::Char('1') => app.change_manual_focus(ManualFocus::Flap),
            KeyCode::Char('2') => app.change_manual_focus(ManualFocus::Fan),
            KeyCode::Up | KeyCode::Right | KeyCode::Char('+') => {
                app.focused_control_mut().increase();
            }
            KeyCode::Down | KeyCode::Left | KeyCode::Char('-') => {
                app.focused_control_mut().decrease();
            }
            KeyCode::Enter => {
                app.send_current_value().await;
            }
            _ => {}
        },
        AppMode::Auto => match key.code {
            KeyCode::Left | KeyCode::Char('h') => {
                app.curve.prev_point();
            }
            KeyCode::Right | KeyCode::Char('l') => {
                app.curve.next_point();
            }
            KeyCode::Up | KeyCode::Char('k') => {
                app.curve.increase_temp();
            }
            KeyCode::Down | KeyCode::Char('j') => {
                app.curve.decrease_temp();
            }
            KeyCode::Char('+') | KeyCode::Char(']') => {
                app.curve.increase_time();
            }
            KeyCode::Char('-') | KeyCode::Char('[') => {
                app.curve.decrease_time();
            }
            KeyCode::Char('a') | KeyCode::Char('n') | KeyCode::Insert => {
                app.curve.add_point();
            }
            KeyCode::Char('d') | KeyCode::Char('x') | KeyCode::Delete => {
                app.curve.delete_point();
            }
            KeyCode::Char(' ') | KeyCode::Char('r') => {
                app.curve.runner.toggle_run_pause();
            }
            KeyCode::Char('s') => {
                app.curve.runner.stop();
            }
            _ => {}
        },
    }

    Ok(())
}
