use snafu::prelude::*;

#[derive(Debug, Snafu)]
#[snafu(provide(chain, u8 => 0))]
struct Error;

fn main() {}
