extern crate snafu;

use snafu::prelude::*;

type BoxError = Box<dyn std::error::Error>;

#[derive(Debug, Snafu)]
enum Error {
    #[snafu(transparent)]
    MissingSource {},

    #[snafu(transparent)]
    HasUserFields { source: BoxError, a: i32, b: i32 },

    #[snafu(transparent(false))]
    TransparentFalseDoesNothing { source: BoxError },

    #[snafu(transparent)]
    #[snafu(display("Oh snap!"))]
    NonsensicalDisplay { source: BoxError },
}

fn main() {}
