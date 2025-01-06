use wry::keyboard::{KeyCode, ModifiersState};
use tracing::debug;

#[derive(Debug)]
pub enum KeyCommand {
    Back,
    Forward,
    Refresh,
    NewTab,
    CloseTab,
    SwitchTab(usize),
    FocusAddressBar,
    StopLoading,
}

pub fn handle_keyboard_input(key: KeyCode, modifiers: ModifiersState) -> Option<KeyCommand> {
    match (key, modifiers.alt(), modifiers.control()) {
        // Navigation
        (KeyCode::ArrowLeft, true, false) => Some(KeyCommand::Back),
        (KeyCode::ArrowRight, true, false) => Some(KeyCommand::Forward),
        
        // Tab Management
        (KeyCode::KeyT, false, true) => Some(KeyCommand::NewTab),
        (KeyCode::KeyW, false, true) => Some(KeyCommand::CloseTab),
        
        // Numbers 1-9 for tab switching
        (KeyCode::Digit1..=KeyCode::Digit9, false, true) => {
            let tab_index = match key {
                KeyCode::Digit1 => 0,
                KeyCode::Digit2 => 1,
                KeyCode::Digit3 => 2,
                KeyCode::Digit4 => 3,
                KeyCode::Digit5 => 4,
                KeyCode::Digit6 => 5,
                KeyCode::Digit7 => 6,
                KeyCode::Digit8 => 7,
                KeyCode::Digit9 => 8,
                _ => unreachable!(),
            };
            Some(KeyCommand::SwitchTab(tab_index))
        }
        
        // Page Controls
        (KeyCode::KeyR, false, true) => Some(KeyCommand::Refresh),
        (KeyCode::KeyL, false, true) => Some(KeyCommand::FocusAddressBar),
        (KeyCode::Escape, false, false) => Some(KeyCommand::StopLoading),
        
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
