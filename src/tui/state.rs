// ponytail: minimal state enums for modes and input
use crate::widgets::float_input_widget::FloatInputWidget;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AppState {
    Connecting,
    Running,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AppMode {
    Manual,
    Auto,
}

impl AppMode {
    pub fn toggle(self) -> Self {
        match self {
            AppMode::Manual => AppMode::Auto,
            AppMode::Auto => AppMode::Manual,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ManualFocus {
    Temp,
    Flap,
    Fan,
}

impl ManualFocus {
    pub fn next(self) -> Self {
        match self {
            ManualFocus::Temp => ManualFocus::Flap,
            ManualFocus::Flap => ManualFocus::Fan,
            ManualFocus::Fan => ManualFocus::Temp,
        }
    }

    pub fn prev(self) -> Self {
        match self {
            ManualFocus::Temp => ManualFocus::Fan,
            ManualFocus::Flap => ManualFocus::Temp,
            ManualFocus::Fan => ManualFocus::Flap,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RightView {
    Curve,
    LiveGraph,
}

pub enum InputState {
    Normal,
    Editing(Box<FloatInputWidget<'static>>),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mode_toggle() {
        assert_eq!(AppMode::Manual.toggle(), AppMode::Auto);
        assert_eq!(AppMode::Auto.toggle(), AppMode::Manual);
    }

    #[test]
    fn test_focus_cycle() {
        assert_eq!(ManualFocus::Temp.next(), ManualFocus::Flap);
        assert_eq!(ManualFocus::Flap.next(), ManualFocus::Fan);
        assert_eq!(ManualFocus::Fan.next(), ManualFocus::Temp);

        assert_eq!(ManualFocus::Temp.prev(), ManualFocus::Fan);
        assert_eq!(ManualFocus::Fan.prev(), ManualFocus::Flap);
        assert_eq!(ManualFocus::Flap.prev(), ManualFocus::Temp);
    }

    #[test]
    fn test_app_state() {
        let state = AppState::Connecting;
        assert_eq!(state, AppState::Connecting);
        assert_ne!(state, AppState::Running);
    }
}
