use std::collections::BTreeSet;

/// A change to one member of a set of held inputs.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HoldChange<K> {
    Press(K),
    Release(K),
    /// A release that also erases this window's press, so the input samples only
    /// if it is still held at the boundary.
    Cancel(K),
}

/// A change to an input that only carries a value while it is held.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ValueChange<T> {
    Hold(T),
    Release,
    /// A release that also erases this window's value, so the input samples only
    /// if it is still held at the boundary.
    Cancel,
}

/// A set of inputs held at the boundary, unioned with those pressed during the window.
#[derive(Clone, Debug)]
pub struct UnionSet<K: Ord> {
    held: BTreeSet<K>,
    transient: BTreeSet<K>,
}

impl<K: Ord> Default for UnionSet<K> {
    fn default() -> Self {
        Self {
            held: BTreeSet::new(),
            transient: BTreeSet::new(),
        }
    }
}

impl<K: Clone + Ord> UnionSet<K> {
    pub fn apply(&mut self, change: HoldChange<K>) {
        match change {
            HoldChange::Press(key) => {
                self.held.insert(key.clone());
                self.transient.insert(key);
            }
            HoldChange::Release(key) => {
                self.held.remove(&key);
            }
            HoldChange::Cancel(key) => {
                self.held.remove(&key);
                self.transient.remove(&key);
            }
        }
    }

    pub fn held(&self) -> impl Iterator<Item = &K> {
        self.held.iter()
    }

    pub fn sample(&mut self) -> BTreeSet<K> {
        let mut sampled = self.held.clone();
        sampled.append(&mut self.transient);
        sampled
    }

    pub fn clear(&mut self) {
        self.held.clear();
        self.transient.clear();
    }
}

/// An optional input held at the boundary, or the last value it carried before being released during the window.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct UnionValue<T> {
    held: Option<T>,
    transient: Option<T>,
}

impl<T> Default for UnionValue<T> {
    fn default() -> Self {
        Self {
            held: None,
            transient: None,
        }
    }
}

impl<T: Clone> UnionValue<T> {
    pub fn apply(&mut self, change: ValueChange<T>) {
        match change {
            ValueChange::Hold(value) => {
                self.held = Some(value.clone());
                self.transient = Some(value);
            }
            ValueChange::Release => {
                self.held = None;
            }
            ValueChange::Cancel => {
                self.held = None;
                self.transient = None;
            }
        }
    }

    pub fn held(&self) -> Option<&T> {
        self.held.as_ref()
    }

    pub fn sample(&mut self) -> Option<T> {
        let sampled = self.held.clone().or_else(|| self.transient.clone());
        self.transient = None;
        sampled
    }

    pub fn clear(&mut self) {
        self.apply(ValueChange::Cancel);
    }
}

/// A held input where the last write during the window wins. Intermediate values are not kept.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Latest<T> {
    held: T,
}

impl<T> Latest<T> {
    pub fn new(initial: T) -> Self {
        Self { held: initial }
    }

    pub fn set(&mut self, value: T) {
        self.held = value;
    }

    pub fn held(&self) -> &T {
        &self.held
    }
}

impl<T: Clone> Latest<T> {
    pub fn sample(&mut self) -> T {
        self.held.clone()
    }
}

/// Deduplicated one-shot actions that cannot be held across a boundary.
#[derive(Clone, Debug)]
pub struct Pending<A: Ord> {
    transient: BTreeSet<A>,
}

impl<A: Ord> Default for Pending<A> {
    fn default() -> Self {
        Self {
            transient: BTreeSet::new(),
        }
    }
}

impl<A: Ord> Pending<A> {
    pub fn request(&mut self, action: A) {
        self.transient.insert(action);
    }

    pub fn sample(&mut self) -> Vec<A> {
        std::mem::take(&mut self.transient).into_iter().collect()
    }

    pub fn clear(&mut self) {
        self.transient.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn union_set_includes_a_tap_that_ended_before_the_boundary() {
        let mut buttons = UnionSet::default();
        buttons.apply(HoldChange::Press("button"));
        buttons.apply(HoldChange::Release("button"));

        assert!(buttons.sample().contains("button"));
        assert!(buttons.sample().is_empty());
    }

    #[test]
    fn union_set_cancel_erases_only_its_own_press() {
        let mut buttons = UnionSet::default();
        buttons.apply(HoldChange::Press("cancelled"));
        buttons.apply(HoldChange::Press("held"));
        buttons.apply(HoldChange::Cancel("cancelled"));

        let sampled = buttons.sample();
        assert!(!sampled.contains("cancelled"));
        assert!(sampled.contains("held"));
    }

    #[test]
    fn latest_keeps_only_the_final_write_of_the_window() {
        let mut lid = Latest::new(false);
        lid.set(true);
        lid.set(false);

        assert!(!lid.sample());
    }

    #[test]
    fn union_value_keeps_the_last_value_of_a_completed_tap() {
        let mut touch = UnionValue::default();
        touch.apply(ValueChange::Hold((10, 20)));
        touch.apply(ValueChange::Hold((30, 40)));
        touch.apply(ValueChange::Release);

        assert_eq!(touch.sample(), Some((30, 40)));
        assert_eq!(touch.sample(), None);
    }

    #[test]
    fn union_value_holds_a_value_across_boundaries_until_released() {
        let mut touch = UnionValue::default();
        touch.apply(ValueChange::Hold((10, 20)));

        assert_eq!(touch.sample(), Some((10, 20)));
        assert_eq!(touch.sample(), Some((10, 20)));

        touch.apply(ValueChange::Release);
        assert_eq!(touch.sample(), None);
    }

    #[test]
    fn union_value_cancel_erases_a_completed_tap() {
        let mut touch = UnionValue::default();
        touch.apply(ValueChange::Hold((10, 20)));
        touch.apply(ValueChange::Cancel);

        assert_eq!(touch.sample(), None);
    }

    #[test]
    fn pending_requests_are_deduplicated_and_consumed() {
        let mut actions = Pending::default();
        actions.request("reset");
        actions.request("reset");

        assert_eq!(actions.sample(), vec!["reset"]);
        assert!(actions.sample().is_empty());
    }
}
