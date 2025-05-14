#![cfg(test)]
#![no_std]

extern crate alloc;

use snafu::prelude::*;

#[derive(Debug, Snafu)]
struct BaseError;

fn boxed_trait_object() -> Result<(), alloc::boxed::Box<dyn snafu::Error + Send + Sync>> {
    BaseSnafu.fail().boxed()
}

fn whatever() -> Result<(), snafu::Whatever> {
    whatever!("this allocates")
}

#[test]
fn implements_error() {
    fn check<T: core::error::Error>() {}
    check::<BaseError>();

    boxed_trait_object().unwrap_err();
    whatever().unwrap_err();
}

#[test]
fn cleaned_error_text() {
    let e = BaseSnafu.build();
    let cleaned = snafu::CleanedErrorText::new(&e);
    assert_eq!(1, cleaned.count());
}
