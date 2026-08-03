mod accumulator;
mod bindings;
mod bridge;
mod model;
mod primitives;

pub use accumulator::{InputAccumulator, InputChange};
pub use bridge::InputBridge;
pub use bindings::{
    Binding, BindingOutcome, Bindings, ConsoleBinding, FrontendCommand, InputEvent, KeyCombination,
    Modifiers,
};
pub use model::{
    BoundaryIndex, BoundaryInput, ButtonMask, ConsoleButton, ConsoleInputState, SystemAction,
    TouchPoint,
};
pub use primitives::{HoldChange, Latest, Pending, UnionSet, UnionValue, ValueChange};
