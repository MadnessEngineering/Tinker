use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum KeyCode {
    ArrowLeft,
    ArrowRight,
    KeyT,
    KeyW,
    Digit1,
    Digit9,
}

impl fmt::Display for KeyCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            KeyCode::ArrowLeft => write!(f, "ArrowLeft"),
            KeyCode::ArrowRight => write!(f, "ArrowRight"),
            KeyCode::KeyT => write!(f, "KeyT"),
            KeyCode::KeyW => write!(f, "KeyW"),
            KeyCode::Digit1 => write!(f, "Digit1"),
            KeyCode::Digit9 => write!(f, "Digit9"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ModifiersState {
    pub alt: bool,
    pub ctrl: bool,
    pub shift: bool,
    pub meta: bool,
}

impl ModifiersState {
    pub const ALT: Self = Self {
        alt: true,
        ctrl: false,
        shift: false,
        meta: false,
    };

    pub const CONTROL: Self = Self {
        alt: false,
        ctrl: true,
        shift: false,
        meta: false,
    };
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum KeyCommand {
    Back,
    Forward,
    NewTab,
    CloseTab,
    SwitchTab(usize),
}

pub fn handle_keyboard_input(key: KeyCode, modifiers: ModifiersState) -> Option<KeyCommand> {
    match (key, modifiers) {
        (KeyCode::ArrowLeft, m) if m.alt => Some(KeyCommand::Back),
        (KeyCode::ArrowRight, m) if m.alt => Some(KeyCommand::Forward),
        (KeyCode::KeyT, m) if m.ctrl => Some(KeyCommand::NewTab),
        (KeyCode::KeyW, m) if m.ctrl => Some(KeyCommand::CloseTab),
        (KeyCode::Digit1, m) if m.ctrl => Some(KeyCommand::SwitchTab(0)),
        (KeyCode::Digit9, m) if m.ctrl => Some(KeyCommand::SwitchTab(8)),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_navigation_shortcuts() {
        let alt = ModifiersState::ALT;
        assert!(matches!(
            handle_keyboard_input(KeyCode::ArrowLeft, alt),
            Some(KeyCommand::Back)
        ));
        assert!(matches!(
            handle_keyboard_input(KeyCode::ArrowRight, alt),
            Some(KeyCommand::Forward)
        ));
    }

    #[test]
    fn test_tab_shortcuts() {
        let ctrl = ModifiersState::CONTROL;
        assert!(matches!(
            handle_keyboard_input(KeyCode::KeyT, ctrl),
            Some(KeyCommand::NewTab)
        ));
        assert!(matches!(
            handle_keyboard_input(KeyCode::KeyW, ctrl),
            Some(KeyCommand::CloseTab)
        ));
    }

    #[test]
    fn test_tab_switching() {
        let ctrl = ModifiersState::CONTROL;
        assert!(matches!(
            handle_keyboard_input(KeyCode::Digit1, ctrl),
            Some(KeyCommand::SwitchTab(0))
        ));
        assert!(matches!(
            handle_keyboard_input(KeyCode::Digit9, ctrl),
            Some(KeyCommand::SwitchTab(8))
        ));
    }
} 
