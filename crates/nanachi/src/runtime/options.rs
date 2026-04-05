/// Per-parse configuration for generated nanachi parsers.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct ParseOptions {
    detailed_errors: bool,
}

impl ParseOptions {
    /// Create default options.
    pub const fn new() -> Self {
        Self {
            detailed_errors: false,
        }
    }

    /// Create options with detailed parse errors enabled.
    pub const fn detailed() -> Self {
        Self {
            detailed_errors: true,
        }
    }

    /// Enable or disable detailed parse errors.
    pub const fn with_detailed_errors(mut self, enabled: bool) -> Self {
        self.detailed_errors = enabled;
        self
    }

    /// Whether detailed parse errors are enabled.
    pub const fn detailed_errors(self) -> bool {
        self.detailed_errors
    }
}

#[cfg(test)]
mod tests {
    use super::ParseOptions;

    #[test]
    fn defaults_to_lightweight_errors() {
        assert!(!ParseOptions::new().detailed_errors());
        assert!(!ParseOptions::default().detailed_errors());
    }

    #[test]
    fn detailed_builder_enables_flag() {
        assert!(ParseOptions::detailed().detailed_errors());
        assert!(ParseOptions::new().with_detailed_errors(true).detailed_errors());
        assert!(!ParseOptions::detailed().with_detailed_errors(false).detailed_errors());
    }
}
