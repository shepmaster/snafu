// There are also happy-path tests

pub mod inner {
    use snafu::prelude::*;

    #[derive(Debug, Snafu)]
    pub(crate) struct Error;
}

fn private_is_default() {
    let _ = inner::Snafu.build();
}

fn main() {}
