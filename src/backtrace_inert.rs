use core::fmt;

/// A backtrace starting from the beginning of the thread.
///
/// Backtrace functionality is currently **disabled**. Please review
/// [the feature flags](crate::guide::feature_flags) to enable it.
#[derive(Debug, Default)]
pub struct Backtrace(());

impl Backtrace {
    /// Creates the backtrace.
    pub fn new() -> Self {
        Backtrace(())
    }
}

impl fmt::Display for Backtrace {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "disabled backtrace")
    }
}
