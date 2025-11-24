use snafu::prelude::*;

#[derive(Debug, Snafu)]
enum EnumError {
    AVariant {
        // First source, legitimate
        source: String,

        // source(from) implies source, so should be a duplicate
        #[snafu(source(from(EnumError, Box::new)))]
        source2: Box<EnumError>,
    },

    AnotherVariant {
        // source(from) implies source, legitimate
        #[snafu(source(from(EnumError, Box::new)))]
        source2: Box<EnumError>,

        // Should be a duplicate
        source: String,
    },
}

fn main() {}
