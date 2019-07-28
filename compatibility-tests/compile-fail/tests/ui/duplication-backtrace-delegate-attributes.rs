use snafu::Snafu;

#[derive(Debug, Snafu)]
enum EnumError {
    AVariant {
        // Should detect second attribute as duplicate
        #[snafu(backtrace(delegate))]
        #[snafu(backtrace(delegate))]
        source: String,
    },
}

fn main() {}
