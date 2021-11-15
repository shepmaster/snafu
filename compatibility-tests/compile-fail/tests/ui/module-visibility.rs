mod inside {
    use snafu::prelude::*;

    #[derive(Debug, Snafu)]
    #[snafu(module)]
    enum Error {
        Variant,
    }

    fn can_access_in_same_module() {
        let _ = error::VariantSnafu;
    }
}

fn cant_access_outside_of_module() {
    let _ = inside::error::VariantSnafu;
}

fn main() {}
