use super::model::{
    BoundaryIndex, BoundaryInput, ButtonMask, ConsoleButton, ConsoleInputState, SystemAction,
    TouchPoint,
};
use super::primitives::{HoldChange, Latest, Pending, UnionSet, UnionValue, ValueChange};

/// A semantic input change received from host bindings.
///
/// Changes are an ingestion detail. Replays store sampled [`BoundaryInput`]
/// values instead.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum InputChange {
    Button(HoldChange<ConsoleButton>),
    Touch(ValueChange<TouchPoint>),
    LidClosed(bool),
    SystemAction(SystemAction),
}

/// NDS-specific composition of the generic temporal input primitives.
#[derive(Clone, Debug, Default)]
pub struct InputAccumulator {
    buttons: UnionSet<ConsoleButton>,
    touch: UnionValue<TouchPoint>,
    lid_closed: Latest<bool>,
    actions: Pending<SystemAction>,
}

impl InputAccumulator {
    pub fn new() -> Self {
        Self::default()
    }

    /// The state as of right now, excluding taps that have already ended.
    pub fn held_state(&self) -> ConsoleInputState {
        ConsoleInputState {
            buttons: button_mask(self.buttons.held().copied()),
            touch: self.touch.held().copied(),
            lid_closed: *self.lid_closed.held(),
        }
    }

    pub fn apply(&mut self, change: InputChange) {
        match change {
            InputChange::Button(change) => self.buttons.apply(change),
            InputChange::Touch(change) => self.touch.apply(change),
            InputChange::LidClosed(closed) => self.lid_closed.set(closed),
            InputChange::SystemAction(action) => self.actions.request(action),
        }
    }

    /// Forgets every press, touch, and pending action, for when the wrong input
    /// was given and the window should start over.
    ///
    /// The lid is absolute held state with no empty value, so it is left as it
    /// is; close or open it explicitly instead.
    pub fn clear(&mut self) {
        self.buttons.clear();
        self.touch.clear();
        self.actions.clear();
    }

    /// Closes the current window and selects the input for `boundary`.
    pub fn sample(&mut self, boundary: BoundaryIndex) -> BoundaryInput {
        BoundaryInput {
            boundary,
            state: ConsoleInputState {
                buttons: button_mask(self.buttons.sample()),
                touch: self.touch.sample(),
                lid_closed: self.lid_closed.sample(),
            },
            actions: self.actions.sample(),
        }
    }
}

fn button_mask(buttons: impl IntoIterator<Item = ConsoleButton>) -> ButtonMask {
    buttons
        .into_iter()
        .fold(ButtonMask::empty(), |mask, button| mask | button.into())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_button_tap_is_visible_for_one_window() {
        let mut inputs = InputAccumulator::new();
        inputs.apply(InputChange::Button(HoldChange::Press(ConsoleButton::A)));
        inputs.apply(InputChange::Button(HoldChange::Release(ConsoleButton::A)));

        let during_tap = inputs.sample(BoundaryIndex(0));
        let after_tap = inputs.sample(BoundaryIndex(1));

        assert!(during_tap.state.buttons.contains(ButtonMask::A));
        assert!(!after_tap.state.buttons.contains(ButtonMask::A));
    }

    #[test]
    fn a_held_button_survives_multiple_boundaries() {
        let mut inputs = InputAccumulator::new();
        inputs.apply(InputChange::Button(HoldChange::Press(ConsoleButton::B)));

        assert!(inputs
            .sample(BoundaryIndex(0))
            .state
            .buttons
            .contains(ButtonMask::B));
        assert!(inputs
            .sample(BoundaryIndex(1))
            .state
            .buttons
            .contains(ButtonMask::B));
    }

    #[test]
    fn duplicate_action_requests_only_fire_once_per_boundary() {
        let mut inputs = InputAccumulator::new();
        inputs.apply(InputChange::SystemAction(SystemAction::Reset));
        inputs.apply(InputChange::SystemAction(SystemAction::Reset));

        assert_eq!(
            inputs.sample(BoundaryIndex(0)).actions,
            vec![SystemAction::Reset]
        );
        assert!(inputs.sample(BoundaryIndex(1)).actions.is_empty());
    }

    #[test]
    fn a_later_action_request_fires_again() {
        let mut inputs = InputAccumulator::new();
        inputs.apply(InputChange::SystemAction(SystemAction::Reset));
        inputs.sample(BoundaryIndex(0));
        inputs.apply(InputChange::SystemAction(SystemAction::Reset));

        assert_eq!(
            inputs.sample(BoundaryIndex(1)).actions,
            vec![SystemAction::Reset]
        );
    }

    #[test]
    fn a_touch_tap_reports_its_last_position() {
        let mut inputs = InputAccumulator::new();
        let begin = TouchPoint::new(10, 20).unwrap();
        let end = TouchPoint::new(30, 40).unwrap();
        inputs.apply(InputChange::Touch(ValueChange::Hold(begin)));
        inputs.apply(InputChange::Touch(ValueChange::Hold(end)));
        inputs.apply(InputChange::Touch(ValueChange::Release));

        assert_eq!(inputs.sample(BoundaryIndex(0)).state.touch, Some(end));
        assert_eq!(inputs.sample(BoundaryIndex(1)).state.touch, None);
    }

    #[test]
    fn lid_changes_set_absolute_state() {
        let mut inputs = InputAccumulator::new();
        inputs.apply(InputChange::LidClosed(true));
        assert!(inputs.sample(BoundaryIndex(0)).state.lid_closed);

        inputs.apply(InputChange::LidClosed(false));
        assert!(!inputs.sample(BoundaryIndex(1)).state.lid_closed);
    }

    #[test]
    fn a_cancelled_press_never_reaches_the_boundary() {
        let mut inputs = InputAccumulator::new();
        inputs.apply(InputChange::Button(HoldChange::Press(ConsoleButton::A)));
        inputs.apply(InputChange::Button(HoldChange::Cancel(ConsoleButton::A)));
        inputs.apply(InputChange::Button(HoldChange::Press(ConsoleButton::B)));

        let sampled = inputs.sample(BoundaryIndex(0));
        assert!(!sampled.state.buttons.contains(ButtonMask::A));
        assert!(sampled.state.buttons.contains(ButtonMask::B));
    }

    #[test]
    fn clearing_drops_inputs_and_actions_but_leaves_the_lid() {
        let mut inputs = InputAccumulator::new();
        inputs.apply(InputChange::Button(HoldChange::Press(ConsoleButton::A)));
        inputs.apply(InputChange::Button(HoldChange::Release(ConsoleButton::A)));
        inputs.apply(InputChange::Button(HoldChange::Press(ConsoleButton::B)));
        inputs.apply(InputChange::Touch(ValueChange::Hold(
            TouchPoint::new(10, 20).unwrap(),
        )));
        inputs.apply(InputChange::LidClosed(true));
        inputs.apply(InputChange::SystemAction(SystemAction::Reset));

        inputs.clear();

        let sampled = inputs.sample(BoundaryIndex(0));
        assert!(sampled.state.buttons.is_empty());
        assert_eq!(sampled.state.touch, None);
        assert!(sampled.actions.is_empty());
        assert!(sampled.state.lid_closed);
    }

    #[test]
    fn separate_taps_in_one_window_share_a_boundary() {
        let mut inputs = InputAccumulator::new();
        inputs.apply(InputChange::Button(HoldChange::Press(ConsoleButton::A)));
        inputs.apply(InputChange::Button(HoldChange::Release(ConsoleButton::A)));
        inputs.apply(InputChange::Button(HoldChange::Press(ConsoleButton::B)));
        inputs.apply(InputChange::Button(HoldChange::Release(ConsoleButton::B)));

        let state = inputs.sample(BoundaryIndex(0)).state;
        assert!(state.buttons.contains(ButtonMask::A | ButtonMask::B));
    }
}
