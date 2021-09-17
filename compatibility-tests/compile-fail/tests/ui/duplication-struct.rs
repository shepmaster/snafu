use snafu::prelude::*;

#[derive(Debug, Snafu)]
enum UsableError {}

#[derive(Debug, Snafu)]
#[snafu(source(from(UsableError, Box::new)))]
#[snafu(source(from(UsableError, Box::new)))]
struct StructError(Box<UsableError>);

fn main() {}
