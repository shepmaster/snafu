use snafu::prelude::*;

#[derive(Debug)]
struct ImplicitData;

impl snafu::GenerateImplicitData for ImplicitData {
    fn generate() -> Self {
        Self
    }
}

#[derive(Debug, Snafu)]
enum EnumError {
    AVariant {
        // Second attribute should be marked as duplicate
        #[snafu(implicit)]
        #[snafu(implicit)]
        my_data: ImplicitData,
    },
}

fn main() {}
