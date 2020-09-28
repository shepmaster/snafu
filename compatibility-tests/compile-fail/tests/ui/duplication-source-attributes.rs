use snafu::Snafu;

#[derive(Debug, Snafu)]
enum EnumError {
    AVariant {
        // First source, legitimate
        source: String,

        // Should mark this second source as a duplicate
        #[snafu(source)]
        my_source: String,
    },
}

fn main() {}
