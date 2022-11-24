#![feature(yeet_expr)]
#![cfg(test)]

use snafu::prelude::*;

#[test]
fn can_yeet_context_selector() {
    #[derive(Debug, Snafu)]
    struct MyError {
        name: String,
    }

    fn usage() -> Result<(), MyError> {
        let name = "gronk";
        do yeet MySnafu { name };
    }

    let r = usage();
    assert!(r.is_err(), "{r:?} should have been an error");
}
