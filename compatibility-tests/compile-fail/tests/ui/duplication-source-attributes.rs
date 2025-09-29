use snafu::prelude::*;

#[derive(Debug, Snafu)]
enum EnumError {
    AVariant {
        // First source, legitimate
        source: String,

        // Should mark this second source as a duplicate
        #[snafu(source)]
        my_source: String,
    },

    AnotherVariant {
        // First source, legitimate
        #[snafu(source)]
        my_source: String,

        // Should mark this second source as a duplicate
        source: String,
    },
}

fn main() {}
