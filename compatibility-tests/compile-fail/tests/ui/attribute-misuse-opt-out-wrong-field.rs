mod source {
    use snafu::prelude::*;

    #[derive(Debug, Snafu)]
    enum EnumError {
        AVariant {
            #[snafu(source(false))]
            not_source: u8,
        },
    }
}

mod backtrace {
    use snafu::prelude::*;

    #[derive(Debug, Snafu)]
    enum EnumError {
        AVariant {
            #[snafu(backtrace(false))]
            not_backtrace: u8,
        },
    }
}

fn main() {}
