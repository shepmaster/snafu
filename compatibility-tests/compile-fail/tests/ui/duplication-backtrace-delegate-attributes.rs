use snafu::prelude::*;

#[derive(Debug, Snafu)]
enum EnumError {
    AVariant {
        // Should detect second attribute as duplicate
        #[snafu(backtrace)]
        #[snafu(backtrace)]
        source: String,
    },
}

fn main() {}
