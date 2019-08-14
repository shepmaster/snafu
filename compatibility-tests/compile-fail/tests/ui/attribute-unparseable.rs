use snafu::Snafu;

#[derive(Debug, Snafu)]
enum InnerError {
    InnerVariant,
}

#[derive(Debug, Snafu)]
enum Error {
    AVariant {
        // Invalid identifier, and not a boolean; should error
        #[snafu(source(5))]
        a: InnerError,
    },
}

fn main() {}
