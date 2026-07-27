use std::collections::HashMap;

use winit::event::{ModifiersState, VirtualKeyCode};

use super::accumulator::InputChange;
use super::model::{ConsoleButton, TouchPoint};
use super::primitives::{HoldChange, ValueChange};

/// A raw host event, before any binding has given it meaning.
#[derive(Debug, PartialEq, Clone)]
pub enum InputEvent {
    KeyDown(VirtualKeyCode),
    KeyUp(VirtualKeyCode),
    CursorMove(u8, u8),
    MouseDown,
    MouseUp,
    KeyModifierChange(ModifiersState),
}

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub struct KeyCombination {
    pub key_code: VirtualKeyCode,
    pub modifiers: ModifiersState,
}

/// What a host key is bound to.
///
/// Deliberately not serializable: the config file owns its own flat spelling of
/// this, so the file format can change without touching this module.
#[derive(Debug, PartialEq, Clone)]
pub enum Binding {
    Console(ConsoleBinding),
    Command(FrontendCommand),
}

/// A binding the emulated console sees.
#[derive(Debug, PartialEq, Clone, Copy)]
pub enum ConsoleBinding {
    Button(ConsoleButton),
    OpenLid,
    CloseLid,
}

/// A binding the emulator itself acts on, invisible to the console and absent
/// from replays.
#[derive(Debug, PartialEq, Clone)]
pub enum FrontendCommand {
    PlayPause,
    Step,
    WriteSavedata(String),
    ReadSavestate(String),
    WriteSavestate(String),
    WriteMainRam(String),
    ToggleReplayMode,
    SaveReplay,
}

/// What a host event turned out to mean.
#[derive(Debug, PartialEq, Clone)]
pub enum BindingOutcome {
    Console(InputChange),
    Command(FrontendCommand),
}

/// Translates host events into console input changes and frontend commands.
///
/// This is the only layer that knows about physical keys and mouse buttons, and
/// the only one that can tell an intentional press from OS key repeat or a drag
/// from idle cursor motion.
#[derive(Debug, Default)]
pub struct Bindings {
    key_map: HashMap<KeyCombination, Binding>,
    modifiers: ModifiersState,
    held: HashMap<VirtualKeyCode, Binding>,
    mouse_held: bool,
    cursor: Option<TouchPoint>,
}

impl Bindings {
    pub fn new(key_map: HashMap<KeyCombination, Binding>) -> Self {
        Self {
            key_map,
            modifiers: ModifiersState::empty(),
            held: HashMap::new(),
            mouse_held: false,
            cursor: None,
        }
    }

    pub fn handle(&mut self, event: InputEvent) -> Option<BindingOutcome> {
        match event {
            InputEvent::KeyDown(key_code) => self.press(key_code),
            InputEvent::KeyUp(key_code) => self.release(key_code),
            InputEvent::CursorMove(x, y) => {
                self.cursor = TouchPoint::new(x, y);
                self.drag()
            }
            InputEvent::MouseDown => {
                self.mouse_held = true;
                self.drag()
            }
            InputEvent::MouseUp => {
                self.mouse_held = false;
                Some(BindingOutcome::Console(InputChange::Touch(
                    ValueChange::Release,
                )))
            }
            InputEvent::KeyModifierChange(modifiers) => {
                self.modifiers = modifiers;
                None
            }
        }
    }

    fn press(&mut self, key_code: VirtualKeyCode) -> Option<BindingOutcome> {
        let binding = self
            .key_map
            .get(&KeyCombination {
                key_code,
                modifiers: self.modifiers,
            })?
            .clone();

        // A key already held is OS repeat, not a new press. Commands in
        // particular must not fire again until the key comes back up.
        if self.held.insert(key_code, binding.clone()).is_some() {
            return None;
        }

        Some(match binding {
            Binding::Console(ConsoleBinding::Button(button)) => {
                BindingOutcome::Console(InputChange::Button(HoldChange::Press(button)))
            }
            Binding::Console(ConsoleBinding::OpenLid) => {
                BindingOutcome::Console(InputChange::LidClosed(false))
            }
            Binding::Console(ConsoleBinding::CloseLid) => {
                BindingOutcome::Console(InputChange::LidClosed(true))
            }
            Binding::Command(command) => BindingOutcome::Command(command),
        })
    }

    /// Releases by key code rather than by combination, so a key still counts as
    /// released when the modifiers changed while it was down.
    fn release(&mut self, key_code: VirtualKeyCode) -> Option<BindingOutcome> {
        match self.held.remove(&key_code)? {
            Binding::Console(ConsoleBinding::Button(button)) => Some(BindingOutcome::Console(
                InputChange::Button(HoldChange::Release(button)),
            )),
            // The lid is absolute state and commands fire on press, so neither
            // has anything to do when the key comes up.
            Binding::Console(_) | Binding::Command(_) => None,
        }
    }

    fn drag(&mut self) -> Option<BindingOutcome> {
        let point = self.cursor.filter(|_| self.mouse_held)?;
        Some(BindingOutcome::Console(InputChange::Touch(
            ValueChange::Hold(point),
        )))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn bindings() -> Bindings {
        Bindings::new(HashMap::from([
            (
                KeyCombination {
                    key_code: VirtualKeyCode::L,
                    modifiers: ModifiersState::empty(),
                },
                Binding::Console(ConsoleBinding::Button(ConsoleButton::A)),
            ),
            (
                KeyCombination {
                    key_code: VirtualKeyCode::Comma,
                    modifiers: ModifiersState::empty(),
                },
                Binding::Command(FrontendCommand::PlayPause),
            ),
        ]))
    }

    fn button_press(button: ConsoleButton) -> Option<BindingOutcome> {
        Some(BindingOutcome::Console(InputChange::Button(
            HoldChange::Press(button),
        )))
    }

    #[test]
    fn a_bound_key_presses_and_releases_its_button() {
        let mut bindings = bindings();

        assert_eq!(
            bindings.handle(InputEvent::KeyDown(VirtualKeyCode::L)),
            button_press(ConsoleButton::A)
        );
        assert_eq!(
            bindings.handle(InputEvent::KeyUp(VirtualKeyCode::L)),
            Some(BindingOutcome::Console(InputChange::Button(
                HoldChange::Release(ConsoleButton::A)
            )))
        );
    }

    #[test]
    fn an_unbound_key_means_nothing() {
        let mut bindings = bindings();

        assert_eq!(bindings.handle(InputEvent::KeyDown(VirtualKeyCode::Z)), None);
        assert_eq!(bindings.handle(InputEvent::KeyUp(VirtualKeyCode::Z)), None);
    }

    #[test]
    fn key_repeat_does_not_fire_a_command_twice() {
        let mut bindings = bindings();
        let play_pause = Some(BindingOutcome::Command(FrontendCommand::PlayPause));

        assert_eq!(
            bindings.handle(InputEvent::KeyDown(VirtualKeyCode::Comma)),
            play_pause
        );
        assert_eq!(
            bindings.handle(InputEvent::KeyDown(VirtualKeyCode::Comma)),
            None
        );

        bindings.handle(InputEvent::KeyUp(VirtualKeyCode::Comma));
        assert_eq!(
            bindings.handle(InputEvent::KeyDown(VirtualKeyCode::Comma)),
            play_pause
        );
    }

    #[test]
    fn a_key_released_under_different_modifiers_still_releases() {
        let mut bindings = bindings();
        bindings.handle(InputEvent::KeyDown(VirtualKeyCode::L));
        bindings.handle(InputEvent::KeyModifierChange(ModifiersState::CTRL));

        assert_eq!(
            bindings.handle(InputEvent::KeyUp(VirtualKeyCode::L)),
            Some(BindingOutcome::Console(InputChange::Button(
                HoldChange::Release(ConsoleButton::A)
            )))
        );
    }

    #[test]
    fn cursor_motion_only_touches_while_the_mouse_is_held() {
        let mut bindings = bindings();

        assert_eq!(bindings.handle(InputEvent::CursorMove(10, 20)), None);

        assert_eq!(
            bindings.handle(InputEvent::MouseDown),
            Some(BindingOutcome::Console(InputChange::Touch(
                ValueChange::Hold(TouchPoint::new(10, 20).unwrap())
            )))
        );
        assert_eq!(
            bindings.handle(InputEvent::CursorMove(30, 40)),
            Some(BindingOutcome::Console(InputChange::Touch(
                ValueChange::Hold(TouchPoint::new(30, 40).unwrap())
            )))
        );

        assert_eq!(
            bindings.handle(InputEvent::MouseUp),
            Some(BindingOutcome::Console(InputChange::Touch(
                ValueChange::Release
            )))
        );
        assert_eq!(bindings.handle(InputEvent::CursorMove(50, 60)), None);
    }

    #[test]
    fn clicking_before_the_cursor_is_known_touches_nothing() {
        let mut bindings = bindings();

        assert_eq!(bindings.handle(InputEvent::MouseDown), None);
        assert_eq!(
            bindings.handle(InputEvent::CursorMove(10, 20)),
            Some(BindingOutcome::Console(InputChange::Touch(
                ValueChange::Hold(TouchPoint::new(10, 20).unwrap())
            )))
        );
    }
}
