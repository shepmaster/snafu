use snafu::prelude::*;

#[derive(Debug, Snafu)]
struct DummyError;

#[derive(Debug, Snafu)]
enum EnumErrorWithGenericSourceAndContextSelector {
    TheVariant {
        #[snafu(source(from(generic)))]
        source: DummyError,
    },
}

#[derive(Debug, Snafu)]
struct StructErrorWithGenericSourceAndContextSelector {
    #[snafu(source(from(generic)))]
    source: DummyError,
}

fn main() {}
