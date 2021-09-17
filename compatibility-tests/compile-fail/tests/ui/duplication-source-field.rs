use snafu::prelude::*;

#[derive(Debug, Snafu)]
enum EnumError {
    AVariant {
        // Should mark second attribute as duplicate
        #[snafu(source)]
        #[snafu(source)]
        my_source: String,
    },
}

fn main() {}
