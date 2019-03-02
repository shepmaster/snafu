extern crate snafu;

use snafu::{Backtrace, ResultExt, Snafu};

type BoxError = Box<std::error::Error>;

#[derive(Debug, Snafu)]
enum Error<'a, 'x, A, Y> {
    Everything {
        source: BoxError,
        name: &'a str,
        length: A,
        backtrace: Backtrace,
    },
    Lifetime {
        key: &'x i32,
    },
    Type {
        value: Y,
    },
}

fn cause_error() -> Result<(), BoxError> {
    Ok(())
}

fn example<'s, 'k, V>(
    name: &'s str,
    key: &'k i32,
    value: V,
) -> Result<(), Error<'s, 'k, usize, V>> {
    let length = name.len();

    cause_error().context(Everything { name, length })?;

    if name == "alice" {
        return Lifetime { key }.fail();
    }

    if name == "bob" {
        return Type { value }.fail();
    }

    Ok(())
}

#[test]
fn implements_error() {
    let name = String::from("hello");
    let key = Box::new(42);
    let value = vec![false];

    example(&name, &key, value).unwrap();
}
