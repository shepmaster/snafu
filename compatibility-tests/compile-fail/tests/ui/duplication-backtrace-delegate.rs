use snafu::{Backtrace, Snafu};

#[derive(Debug, Snafu)]
enum EnumError {
    AVariant {
        // First backtrace, legitimate
        backtrace: Backtrace,

        // Second backtrace, this time a delegate, can't have both
        #[snafu(backtrace)]
        source: String,
    },
}

fn main() {}
