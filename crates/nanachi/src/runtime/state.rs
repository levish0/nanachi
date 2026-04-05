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
