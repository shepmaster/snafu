use snafu::prelude::*;

#[derive(Debug, Snafu)]
struct InnerError;

#[derive(Debug, Snafu)]
enum Error {
    AVariant {
        // Invalid identifier, and not a boolean; should error
        #[snafu(source(5))]
        a: InnerError,
    },
}

#[derive(Debug, Snafu)]
struct SourceFromTransformInvalidType {
    #[snafu(source(from(Cow*, ?)))]
    source: InnerError,
}

fn main() {}
