/// Trait that all nanachi-generated state structs implement.
///
/// The generator creates a concrete struct (e.g. `ParseState`) with fields
/// for each `let flag` / `let counter` declaration in the grammar, and
/// implements this trait on it.
pub trait State: Clone + Default {
    /// Return the original input bytes, used for `LINE_START` / `LINE_END`.
    fn original_input(&self) -> &[u8];

    /// Check whether position is at the start of a line.
    fn is_at_line_start(&self, position: usize) -> bool {
        position == 0 || self.original_input().get(position - 1) == Some(&b'\n')
    }

    /// Check whether position is at the end of a line.
    fn is_at_line_end(&self, position: usize) -> bool {
        let input = self.original_input();
        position >= input.len() || input.get(position) == Some(&b'\n')
    }

    // ── Flag operations ──

    fn get_flag(&self, name: &str) -> bool;
    fn set_flag(&mut self, name: &str, value: bool);

    // ── Counter operations ──

    fn get_counter(&self, name: &str) -> usize;
    fn set_counter(&mut self, name: &str, value: usize);

    fn increment_counter(&mut self, name: &str, amount: usize) {
        let current = self.get_counter(name);
        self.set_counter(name, current + amount);
    }

    fn decrement_counter(&mut self, name: &str, amount: usize) {
        let current = self.get_counter(name);
        self.set_counter(name, current.saturating_sub(amount));
    }
}

#[cfg(test)]
mod tests {
    use super::State;

    #[derive(Clone, Default)]
    struct DummyState {
        input: Vec<u8>,
        flag: bool,
        counter: usize,
    }

    impl DummyState {
        fn with_input(input: &str) -> Self {
            Self {
                input: input.as_bytes().to_vec(),
                ..Default::default()
            }
        }
    }

    impl State for DummyState {
        fn original_input(&self) -> &[u8] {
            &self.input
        }

        fn get_flag(&self, name: &str) -> bool {
            match name {
                "flag" => self.flag,
                _ => false,
            }
        }

        fn set_flag(&mut self, name: &str, value: bool) {
            if name == "flag" {
                self.flag = value;
            }
        }

        fn get_counter(&self, name: &str) -> usize {
            match name {
                "counter" => self.counter,
                _ => 0,
            }
        }

        fn set_counter(&mut self, name: &str, value: usize) {
            if name == "counter" {
                self.counter = value;
            }
        }
    }

    #[test]
    fn line_start_detects_beginning_and_after_newline() {
        let state = DummyState::with_input("ab\ncd");

        assert!(state.is_at_line_start(0));
        assert!(!state.is_at_line_start(1));
        assert!(state.is_at_line_start(3));
    }

    #[test]
    fn line_end_detects_newline_and_eof() {
        let state = DummyState::with_input("ab\ncd");

        assert!(!state.is_at_line_end(0));
        assert!(state.is_at_line_end(2));
        assert!(state.is_at_line_end(5));
        assert!(state.is_at_line_end(6));
    }

    #[test]
    fn increment_counter_adds_amount() {
        let mut state = DummyState::default();

        state.increment_counter("counter", 2);
        state.increment_counter("counter", 3);

        assert_eq!(state.get_counter("counter"), 5);
    }

    #[test]
    fn decrement_counter_saturates_at_zero() {
        let mut state = DummyState::default();
        state.set_counter("counter", 2);

        state.decrement_counter("counter", 1);
        assert_eq!(state.get_counter("counter"), 1);

        state.decrement_counter("counter", 5);
        assert_eq!(state.get_counter("counter"), 0);
    }
}
