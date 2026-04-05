pub mod runtime;

pub use runtime::input::{Input, LocatingSlice, Stateful};
pub use runtime::state::State;

/// Re-export winnow for generated code.
pub use winnow;
