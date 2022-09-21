#![cfg(test)]
#![feature(try_trait_v2)]

use snafu::{prelude::*, Report};

#[test]
fn can_be_used_with_the_try_operator() {
    #[derive(Debug, Snafu)]
    struct ExampleError;

    fn mainlike() -> Report<ExampleError> {
        ExampleSnafu.fail()?;

        Report::ok()
    }

    let _: Report<ExampleError> = mainlike();
}
